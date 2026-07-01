// AI Chat Agent 工具调用回路
// Phase 4 / P1：Agent 通过统一的 ToolRegistry 调用工具（function calling）
//
// 回路：用户消息 → LLM（携带工具声明）→ 若返回 tool_calls → 经 ToolRegistry 执行 →
// 结果回填 → 再次调用 LLM → ... → 直到 LLM 给出最终回答。
// 每一步通过 `agent-event-{request_id}` 事件推送给前端 Trace 面板。

use crate::api::ai_chat::models::{Message, MessageRole, ModelProviderManagerState};
use crate::tools::{ToolDeclaration, ToolFilter, ToolOutcome, ToolRegistry, ToolSource};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::{Emitter, State};

/// Agent 回路最大迭代次数（防止工具调用无限循环）
const MAX_ITERATIONS: usize = 8;

// ============================================================================
// 工具定义格式转换
// ============================================================================

/// 转换为 OpenAI function calling 的 tool 定义
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

/// 转换为 Anthropic 的 tool 定义
fn tool_to_anthropic(decl: &ToolDeclaration) -> Value {
  json!({
    "name": decl.name,
    "description": decl.description,
    "input_schema": decl.input_schema,
  })
}

/// 工具的展示标签（内置=分组名，MCP=server_id）
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

// ============================================================================
// Agent 事件（推送给前端 Trace 面板）
// ============================================================================

/// Agent 执行过程中的事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentEvent {
  /// 模型的中间思考文本（伴随工具调用时）
  Thinking { text: String },
  /// 发起一次工具调用
  ToolCall {
    id: String,
    server: String,
    name: String,
    arguments: Value,
  },
  /// 工具调用返回结果
  ToolResult {
    id: String,
    name: String,
    content: String,
    is_error: bool,
  },
  /// 最终回答内容
  Content { text: String },
  /// 回合结束
  Done,
  /// 出错
  Error { message: String },
}

/// 发送 Agent 事件到前端
fn emit_event(app: &tauri::AppHandle, request_id: &str, event: AgentEvent) {
  let _ = app.emit(&format!("agent-event-{}", request_id), &event);
}

/// 执行单个工具调用并推送事件，返回 (结果文本, 是否出错)
async fn execute_tool(
  app: &tauri::AppHandle,
  request_id: &str,
  registry: &ToolRegistry,
  tools: &[ToolDeclaration],
  id: &str,
  name: &str,
  arguments: &Value,
) -> (String, bool) {
  let server = tool_label(tools, name);

  emit_event(
    app,
    request_id,
    AgentEvent::ToolCall {
      id: id.to_string(),
      server,
      name: name.to_string(),
      arguments: arguments.clone(),
    },
  );

  let ToolOutcome {
    content, is_error, ..
  } = registry.call(name, arguments.clone(), None).await;

  emit_event(
    app,
    request_id,
    AgentEvent::ToolResult {
      id: id.to_string(),
      name: name.to_string(),
      content: content.clone(),
      is_error,
    },
  );

  (content, is_error)
}

// ============================================================================
// Provider 回路实现
// ============================================================================

/// 把统一消息转换为 OpenAI 原生消息
fn to_openai_messages(messages: &[Message]) -> Vec<Value> {
  messages
    .iter()
    .map(|m| json!({ "role": m.role.to_string(), "content": m.content }))
    .collect()
}

/// OpenAI（及兼容协议）工具调用回路
#[allow(clippy::too_many_arguments)]
async fn run_openai_loop(
  app: &tauri::AppHandle,
  request_id: &str,
  registry: &ToolRegistry,
  client: &Client,
  base_url: &str,
  api_key: Option<&str>,
  model: &str,
  messages: &[Message],
  temperature: Option<f32>,
  max_tokens: Option<u32>,
  tools: &[ToolDeclaration],
) -> Result<String, String> {
  let mut native_messages = to_openai_messages(messages);
  let openai_tools: Vec<Value> = tools.iter().map(tool_to_openai).collect();

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
      let error_text = response
        .text()
        .await
        .unwrap_or_else(|_| "Unknown error".to_string());
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
            emit_event(app, request_id, AgentEvent::Thinking { text: text.to_string() });
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

          let (result_text, _is_error) =
            execute_tool(app, request_id, registry, tools, &id, &name, &args_value).await;

          native_messages.push(json!({
            "role": "tool",
            "tool_call_id": id,
            "content": result_text,
          }));
        }
      }
      _ => {
        let content = message
          .get("content")
          .and_then(|c| c.as_str())
          .unwrap_or("")
          .to_string();
        emit_event(app, request_id, AgentEvent::Content { text: content.clone() });
        emit_event(app, request_id, AgentEvent::Done);
        return Ok(content);
      }
    }
  }

  Err(format!("Agent 达到最大迭代次数 ({})，未能完成任务", MAX_ITERATIONS))
}

/// Anthropic 工具调用回路
#[allow(clippy::too_many_arguments)]
async fn run_anthropic_loop(
  app: &tauri::AppHandle,
  request_id: &str,
  registry: &ToolRegistry,
  client: &Client,
  base_url: &str,
  api_key: &str,
  model: &str,
  messages: &[Message],
  temperature: Option<f32>,
  max_tokens: Option<u32>,
  tools: &[ToolDeclaration],
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

  let anthropic_tools: Vec<Value> = tools.iter().map(tool_to_anthropic).collect();

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
      let error_text = response
        .text()
        .await
        .unwrap_or_else(|_| "Unknown error".to_string());
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

          let (result_text, is_error) =
            execute_tool(app, request_id, registry, tools, &id, &name, &input).await;

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
        emit_event(app, request_id, AgentEvent::Thinking { text: thinking });
      }
      native_messages.push(json!({ "role": "assistant", "content": content_blocks }));
      native_messages.push(json!({ "role": "user", "content": tool_result_blocks }));
    } else {
      let content = text_parts.join("\n");
      emit_event(app, request_id, AgentEvent::Content { text: content.clone() });
      emit_event(app, request_id, AgentEvent::Done);
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

/// 直接调用某个工具（手动/调试）
#[tauri::command]
pub async fn tool_call(
  registry: State<'_, ToolRegistry>,
  name: String,
  args: Option<Value>,
) -> Result<ToolOutcome, String> {
  Ok(registry.call(&name, args.unwrap_or_else(|| json!({})), None).await)
}

/// Agent 聊天：携带启用的工具运行工具调用回路，返回最终回答。
/// 过程事件通过 `agent-event-{request_id}` 推送给前端。
#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub async fn agent_chat(
  app: tauri::AppHandle,
  state: State<'_, ModelProviderManagerState>,
  registry: State<'_, ToolRegistry>,
  provider: String,
  model: String,
  messages: Vec<Message>,
  servers: Vec<String>,
  temperature: Option<f32>,
  max_tokens: Option<u32>,
  request_id: String,
) -> Result<String, String> {
  // servers 同时作为内置分组与 MCP server 的启用集合
  let filter = ToolFilter {
    enabled_groups: servers.clone(),
    enabled_servers: servers,
  };
  let tools = registry.declarations(&filter);

  let (api_key, endpoint) = state.get_credentials(&provider)?;
  let client = Client::new();

  let result = match provider.to_lowercase().as_str() {
    "anthropic" => {
      let key = api_key.ok_or_else(|| "Anthropic API key not found".to_string())?;
      let base_url = endpoint.unwrap_or_else(|| "https://api.anthropic.com/v1".to_string());
      run_anthropic_loop(
        &app, &request_id, &registry, &client, &base_url, &key, &model, &messages, temperature,
        max_tokens, &tools,
      )
      .await
    }
    "ollama" => {
      let base_url = endpoint.unwrap_or_else(|| "http://localhost:11434/v1".to_string());
      run_openai_loop(
        &app, &request_id, &registry, &client, &base_url, None, &model, &messages, temperature,
        max_tokens, &tools,
      )
      .await
    }
    _ => {
      let base_url = endpoint.unwrap_or_else(|| "https://api.openai.com/v1".to_string());
      run_openai_loop(
        &app,
        &request_id,
        &registry,
        &client,
        &base_url,
        api_key.as_deref(),
        &model,
        &messages,
        temperature,
        max_tokens,
        &tools,
      )
      .await
    }
  };

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
  use crate::tools::RiskLevel;

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
  fn test_registry_call_unknown_tool() {
    let reg = ToolRegistry::with_builtins();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let outcome = rt.block_on(reg.call("does_not_exist", json!({}), None));
    assert!(outcome.is_error);
  }
}
