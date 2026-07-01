// AI Chat Agent 工具调用回路
// P1：统一 ToolRegistry 派发；P2：可挂起确认回路 + per-turn 回滚
//
// 回路：用户消息 → LLM（携带工具声明）→ 若返回 tool_calls → 经 ToolRegistry 执行
// （Write/System 工具先 dry_run 预览并挂起等待前端确认）→ 结果回填 → 再次调用 LLM
// → ... → 直到 LLM 给出最终回答。过程事件通过 `agent-event-{request_id}` 推送。

use crate::api::ai_chat::confirm::{
  should_confirm, ConfirmDecision, ConfirmMode, PendingConfirmations, UndoEntry, UndoStacks,
};
use crate::api::ai_chat::models::{Message, MessageRole, ModelProviderManagerState};
use crate::tools::{RiskLevel, ToolDeclaration, ToolFilter, ToolOutcome, ToolRegistry, ToolSource};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;
use tauri::{Emitter, State};

/// Agent 回路最大迭代次数（防止工具调用无限循环）
const MAX_ITERATIONS: usize = 8;
/// 单次确认等待超时
const CONFIRM_TIMEOUT_SECS: u64 = 120;

// ============================================================================
// 工具定义格式转换
// ============================================================================

fn tool_to_openai(decl: &ToolDeclaration) -> Value {
  json!({
    "type": "function",
    "function": {
      "name": decl.name,
      "description": decl.description,
      "parameters": decl.input_schema,
    }
  })
}

fn tool_to_anthropic(decl: &ToolDeclaration) -> Value {
  json!({
    "name": decl.name,
    "description": decl.description,
    "input_schema": decl.input_schema,
  })
}

fn tool_label(tools: &[ToolDeclaration], name: &str) -> String {
  tools
    .iter()
    .find(|d| d.name == name)
    .map(|d| match &d.source {
      ToolSource::Native => d.group.clone(),
      ToolSource::Mcp { server_id, .. } => server_id.clone(),
    })
    .unwrap_or_default()
}

fn risk_str(risk: RiskLevel) -> &'static str {
  match risk {
    RiskLevel::Read => "read",
    RiskLevel::Write => "write",
    RiskLevel::System => "system",
  }
}

/// 参数摘要（用于回滚项标签），截断避免过长
fn short_args(args: &Value) -> String {
  let s = args.to_string();
  if s.len() > 80 {
    format!("{}…", &s[..80])
  } else {
    s
  }
}

// ============================================================================
// Agent 事件（推送给前端 Trace / 确认卡片）
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentEvent {
  /// 模型的中间思考文本
  Thinking { text: String },
  /// 发起一次工具调用
  ToolCall {
    id: String,
    server: String,
    name: String,
    arguments: Value,
  },
  /// 需要用户确认（Write/System 工具在执行前发出，回路挂起等待 tool_confirm）
  ConfirmRequest {
    call_id: String,
    server: String,
    name: String,
    risk: String,
    arguments: Value,
    diff: String,
  },
  /// 工具调用返回结果
  ToolResult {
    id: String,
    name: String,
    content: String,
    is_error: bool,
    /// 可撤销时携带回滚句柄
    undo_token: Option<String>,
  },
  /// 最终回答内容
  Content { text: String },
  /// 回合结束
  Done,
  /// 出错
  Error { message: String },
}

fn emit_event(app: &tauri::AppHandle, request_id: &str, event: AgentEvent) {
  let _ = app.emit(&format!("agent-event-{}", request_id), &event);
}

// ============================================================================
// 执行上下文
// ============================================================================

/// 工具执行所需的共享上下文（贯穿回路）
struct ExecCtx<'a> {
  app: &'a tauri::AppHandle,
  request_id: &'a str,
  registry: &'a ToolRegistry,
  pending: &'a PendingConfirmations,
  undo: &'a UndoStacks,
  tools: &'a [ToolDeclaration],
  mode: ConfirmMode,
  auto_approve: &'a [String],
}

/// 执行单个工具调用（含确认挂起），返回 (回填给 LLM 的文本, 是否出错)
async fn execute_tool(ctx: &ExecCtx<'_>, id: &str, name: &str, args: &Value) -> (String, bool) {
  let server = tool_label(ctx.tools, name);

  emit_event(
    ctx.app,
    ctx.request_id,
    AgentEvent::ToolCall {
      id: id.to_string(),
      server: server.clone(),
      name: name.to_string(),
      arguments: args.clone(),
    },
  );

  if ctx.pending.is_cancelled(ctx.request_id) {
    return ("操作已被用户取消".to_string(), false);
  }

  let tool = match ctx.registry.get(name) {
    Some(t) => t,
    None => {
      let msg = format!("未知工具: {}", name);
      emit_event(
        ctx.app,
        ctx.request_id,
        AgentEvent::ToolResult {
          id: id.to_string(),
          name: name.to_string(),
          content: msg.clone(),
          is_error: true,
          undo_token: None,
        },
      );
      return (msg, true);
    }
  };

  let risk = tool.dynamic_risk(args);
  let mut exec_args = args.clone();
  let mut snapshot: Option<String> = None;

  // 需要确认 → dry_run 预览 + 挂起等待
  if should_confirm(risk, ctx.mode, ctx.auto_approve, name) {
    let preview = tool.dry_run(args).await.ok().flatten();
    let diff = preview
      .as_ref()
      .map(|p| p.diff.clone())
      .unwrap_or_default();
    snapshot = preview.map(|p| p.snapshot_token);

    emit_event(
      ctx.app,
      ctx.request_id,
      AgentEvent::ConfirmRequest {
        call_id: id.to_string(),
        server,
        name: name.to_string(),
        risk: risk_str(risk).to_string(),
        arguments: args.clone(),
        diff,
      },
    );

    let rx = ctx.pending.register(ctx.request_id, id);
    let decision = match tokio::time::timeout(Duration::from_secs(CONFIRM_TIMEOUT_SECS), rx).await {
      Ok(Ok(d)) => d,
      _ => ConfirmDecision::Reject, // 超时或通道关闭视为拒绝
    };

    match decision {
      ConfirmDecision::Approve { edited_args } => {
        if let Some(edited) = edited_args {
          exec_args = edited;
        }
      }
      ConfirmDecision::Reject => {
        emit_event(
          ctx.app,
          ctx.request_id,
          AgentEvent::ToolResult {
            id: id.to_string(),
            name: name.to_string(),
            content: "⛔ 用户拒绝执行该操作".to_string(),
            is_error: false,
            undo_token: None,
          },
        );
        return (
          "用户拒绝执行该操作。请不要重试，改为询问用户下一步该怎么做。".to_string(),
          false,
        );
      }
    }
  }

  // 执行
  let outcome = tool.call(exec_args, snapshot).await;
  if let Some(token) = &outcome.undo_token {
    ctx.undo.push(
      ctx.request_id,
      UndoEntry {
        tool: name.to_string(),
        token: token.clone(),
        label: format!("{} {}", name, short_args(args)),
      },
    );
  }

  emit_event(
    ctx.app,
    ctx.request_id,
    AgentEvent::ToolResult {
      id: id.to_string(),
      name: name.to_string(),
      content: outcome.content.clone(),
      is_error: outcome.is_error,
      undo_token: outcome.undo_token.clone(),
    },
  );

  (outcome.content, outcome.is_error)
}

// ============================================================================
// Provider 回路实现
// ============================================================================

fn to_openai_messages(messages: &[Message]) -> Vec<Value> {
  messages
    .iter()
    .map(|m| json!({ "role": m.role.to_string(), "content": m.content }))
    .collect()
}

/// OpenAI（及兼容协议）工具调用回路
#[allow(clippy::too_many_arguments)]
async fn run_openai_loop(
  ctx: &ExecCtx<'_>,
  client: &Client,
  base_url: &str,
  api_key: Option<&str>,
  model: &str,
  messages: &[Message],
  temperature: Option<f32>,
  max_tokens: Option<u32>,
) -> Result<String, String> {
  let mut native_messages = to_openai_messages(messages);
  let openai_tools: Vec<Value> = ctx.tools.iter().map(tool_to_openai).collect();

  for _ in 0..MAX_ITERATIONS {
    let mut body = json!({
      "model": model,
      "messages": native_messages,
      "stream": false,
    });
    if !openai_tools.is_empty() {
      body["tools"] = json!(openai_tools);
    }
    if let Some(t) = temperature {
      body["temperature"] = json!(t);
    }
    if let Some(m) = max_tokens {
      body["max_tokens"] = json!(m);
    }

    let mut req = client
      .post(format!("{}/chat/completions", base_url))
      .header("Content-Type", "application/json");
    if let Some(key) = api_key {
      req = req.header("Authorization", format!("Bearer {}", key));
    }

    let response = req
      .json(&body)
      .send()
      .await
      .map_err(|e| format!("OpenAI API request failed: {}", e))?;

    if !response.status().is_success() {
      let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
      return Err(format!("OpenAI API error: {}", error_text));
    }

    let data: Value = response
      .json()
      .await
      .map_err(|e| format!("Failed to parse OpenAI response: {}", e))?;

    let message = data
      .get("choices")
      .and_then(|c| c.get(0))
      .and_then(|c| c.get("message"))
      .ok_or_else(|| "No choices in OpenAI response".to_string())?;

    let tool_calls = message.get("tool_calls").and_then(|v| v.as_array());

    match tool_calls {
      Some(calls) if !calls.is_empty() => {
        if let Some(text) = message.get("content").and_then(|c| c.as_str()) {
          if !text.trim().is_empty() {
            emit_event(ctx.app, ctx.request_id, AgentEvent::Thinking { text: text.to_string() });
          }
        }
        native_messages.push(message.clone());

        for call in calls {
          let id = call.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
          let name = call
            .get("function")
            .and_then(|f| f.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
          let args_str = call
            .get("function")
            .and_then(|f| f.get("arguments"))
            .and_then(|v| v.as_str())
            .unwrap_or("{}");
          let args_value: Value = serde_json::from_str(args_str).unwrap_or_else(|_| json!({}));

          let (result_text, _is_error) = execute_tool(ctx, &id, &name, &args_value).await;

          native_messages.push(json!({
            "role": "tool",
            "tool_call_id": id,
            "content": result_text,
          }));
        }

        if ctx.pending.is_cancelled(ctx.request_id) {
          emit_event(ctx.app, ctx.request_id, AgentEvent::Done);
          return Ok("操作已被用户取消".to_string());
        }
      }
      _ => {
        let content = message.get("content").and_then(|c| c.as_str()).unwrap_or("").to_string();
        emit_event(ctx.app, ctx.request_id, AgentEvent::Content { text: content.clone() });
        emit_event(ctx.app, ctx.request_id, AgentEvent::Done);
        return Ok(content);
      }
    }
  }

  Err(format!("Agent 达到最大迭代次数 ({})，未能完成任务", MAX_ITERATIONS))
}

/// Anthropic 工具调用回路
#[allow(clippy::too_many_arguments)]
async fn run_anthropic_loop(
  ctx: &ExecCtx<'_>,
  client: &Client,
  base_url: &str,
  api_key: &str,
  model: &str,
  messages: &[Message],
  temperature: Option<f32>,
  max_tokens: Option<u32>,
) -> Result<String, String> {
  let system = messages
    .iter()
    .find(|m| m.role == MessageRole::System)
    .map(|m| m.content.clone());

  let mut native_messages: Vec<Value> = messages
    .iter()
    .filter(|m| m.role != MessageRole::System)
    .map(|m| json!({ "role": m.role.to_string(), "content": m.content }))
    .collect();

  let anthropic_tools: Vec<Value> = ctx.tools.iter().map(tool_to_anthropic).collect();

  for _ in 0..MAX_ITERATIONS {
    let mut body = json!({
      "model": model,
      "messages": native_messages,
      "max_tokens": max_tokens.unwrap_or(4096),
      "stream": false,
    });
    if let Some(sys) = &system {
      body["system"] = json!(sys);
    }
    if !anthropic_tools.is_empty() {
      body["tools"] = json!(anthropic_tools);
    }
    if let Some(t) = temperature {
      body["temperature"] = json!(t);
    }

    let response = client
      .post(format!("{}/messages", base_url))
      .header("x-api-key", api_key)
      .header("anthropic-version", "2023-06-01")
      .header("Content-Type", "application/json")
      .json(&body)
      .send()
      .await
      .map_err(|e| format!("Anthropic API request failed: {}", e))?;

    if !response.status().is_success() {
      let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
      return Err(format!("Anthropic API error: {}", error_text));
    }

    let data: Value = response
      .json()
      .await
      .map_err(|e| format!("Failed to parse Anthropic response: {}", e))?;

    let content_blocks = data
      .get("content")
      .and_then(|c| c.as_array())
      .cloned()
      .unwrap_or_default();

    let mut has_tool_use = false;
    let mut text_parts: Vec<String> = Vec::new();
    let mut tool_result_blocks: Vec<Value> = Vec::new();

    for block in &content_blocks {
      match block.get("type").and_then(|v| v.as_str()) {
        Some("text") => {
          if let Some(text) = block.get("text").and_then(|v| v.as_str()) {
            text_parts.push(text.to_string());
          }
        }
        Some("tool_use") => {
          has_tool_use = true;
          let id = block.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
          let name = block.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
          let input = block.get("input").cloned().unwrap_or_else(|| json!({}));

          let (result_text, is_error) = execute_tool(ctx, &id, &name, &input).await;

          tool_result_blocks.push(json!({
            "type": "tool_result",
            "tool_use_id": id,
            "content": result_text,
            "is_error": is_error,
          }));
        }
        _ => {}
      }
    }

    if has_tool_use {
      let thinking = text_parts.join("\n");
      if !thinking.trim().is_empty() {
        emit_event(ctx.app, ctx.request_id, AgentEvent::Thinking { text: thinking });
      }
      native_messages.push(json!({ "role": "assistant", "content": content_blocks }));
      native_messages.push(json!({ "role": "user", "content": tool_result_blocks }));

      if ctx.pending.is_cancelled(ctx.request_id) {
        emit_event(ctx.app, ctx.request_id, AgentEvent::Done);
        return Ok("操作已被用户取消".to_string());
      }
    } else {
      let content = text_parts.join("\n");
      emit_event(ctx.app, ctx.request_id, AgentEvent::Content { text: content.clone() });
      emit_event(ctx.app, ctx.request_id, AgentEvent::Done);
      return Ok(content);
    }
  }

  Err(format!("Agent 达到最大迭代次数 ({})，未能完成任务", MAX_ITERATIONS))
}

// ============================================================================
// Tauri commands
// ============================================================================

/// 列出全部已注册工具（供工具浏览器）
#[tauri::command]
pub fn tool_list_all(registry: State<'_, ToolRegistry>) -> Vec<ToolDeclaration> {
  registry.all_declarations()
}

/// 直接调用某个工具（手动/调试，不经确认）
#[tauri::command]
pub async fn tool_call(
  registry: State<'_, ToolRegistry>,
  name: String,
  args: Option<Value>,
) -> Result<ToolOutcome, String> {
  Ok(registry.call(&name, args.unwrap_or_else(|| json!({})), None).await)
}

/// Agent 聊天：携带启用的工具运行可挂起的工具调用回路，返回最终回答。
#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub async fn agent_chat(
  app: tauri::AppHandle,
  state: State<'_, ModelProviderManagerState>,
  registry: State<'_, ToolRegistry>,
  pending: State<'_, PendingConfirmations>,
  undo: State<'_, UndoStacks>,
  provider: String,
  model: String,
  messages: Vec<Message>,
  servers: Vec<String>,
  temperature: Option<f32>,
  max_tokens: Option<u32>,
  request_id: String,
  confirm_mode: Option<String>,
  auto_approve: Option<Vec<String>>,
) -> Result<String, String> {
  let filter = ToolFilter {
    enabled_groups: servers.clone(),
    enabled_servers: servers,
  };
  let tools = registry.declarations(&filter);
  let auto_approve = auto_approve.unwrap_or_default();

  let ctx = ExecCtx {
    app: &app,
    request_id: &request_id,
    registry: &registry,
    pending: &pending,
    undo: &undo,
    tools: &tools,
    mode: ConfirmMode::parse(confirm_mode.as_deref()),
    auto_approve: &auto_approve,
  };

  let (api_key, endpoint) = state.get_credentials(&provider)?;
  let client = Client::new();

  let result = match provider.to_lowercase().as_str() {
    "anthropic" => {
      let key = api_key.ok_or_else(|| "Anthropic API key not found".to_string())?;
      let base_url = endpoint.unwrap_or_else(|| "https://api.anthropic.com/v1".to_string());
      run_anthropic_loop(&ctx, &client, &base_url, &key, &model, &messages, temperature, max_tokens)
        .await
    }
    "ollama" => {
      let base_url = endpoint.unwrap_or_else(|| "http://localhost:11434/v1".to_string());
      run_openai_loop(&ctx, &client, &base_url, None, &model, &messages, temperature, max_tokens)
        .await
    }
    _ => {
      let base_url = endpoint.unwrap_or_else(|| "https://api.openai.com/v1".to_string());
      run_openai_loop(
        &ctx,
        &client,
        &base_url,
        api_key.as_deref(),
        &model,
        &messages,
        temperature,
        max_tokens,
      )
      .await
    }
  };

  // 清理该 request 的确认残留（保留 undo 栈供后续回滚）
  pending.clear(&request_id);

  if let Err(e) = &result {
    emit_event(&app, &request_id, AgentEvent::Error { message: e.clone() });
  }

  result
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
  use super::*;

  fn sample_decl() -> ToolDeclaration {
    ToolDeclaration {
      name: "hosts_add".into(),
      description: "add".into(),
      input_schema: json!({ "type":"object", "properties": { "ip": {"type":"string"} }, "required": ["ip"] }),
      risk: RiskLevel::Write,
      source: ToolSource::Native,
      group: "hosts".into(),
    }
  }

  #[test]
  fn test_tool_to_openai_shape() {
    let v = tool_to_openai(&sample_decl());
    assert_eq!(v["type"], "function");
    assert_eq!(v["function"]["name"], "hosts_add");
    assert_eq!(v["function"]["parameters"]["type"], "object");
  }

  #[test]
  fn test_tool_to_anthropic_shape() {
    let v = tool_to_anthropic(&sample_decl());
    assert_eq!(v["name"], "hosts_add");
    assert_eq!(v["input_schema"]["type"], "object");
  }

  #[test]
  fn test_tool_label_native_uses_group() {
    let tools = vec![sample_decl()];
    assert_eq!(tool_label(&tools, "hosts_add"), "hosts");
    assert_eq!(tool_label(&tools, "unknown"), "");
  }

  #[test]
  fn test_risk_str() {
    assert_eq!(risk_str(RiskLevel::Read), "read");
    assert_eq!(risk_str(RiskLevel::Write), "write");
    assert_eq!(risk_str(RiskLevel::System), "system");
  }

  #[test]
  fn test_short_args_truncates() {
    let long = json!({ "k": "x".repeat(200) });
    assert!(short_args(&long).ends_with('…'));
  }
}
