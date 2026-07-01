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
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
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

/// 默认的 Agent 系统提示：塑造角色、规划、确认与安全行为
fn build_agent_system_prompt() -> String {
  "你是 Jedi 工具箱的 AI Agent。你可以调用工具来帮助用户管理系统 hosts、设置知识壁纸、\
管理播客订阅、查询系统信息，并使用已连接的第三方 MCP 工具。\n\n\
工作方式：\n\
1. 对于多步任务，先用一两句话简要说明计划，再开始执行。\n\
2. 读操作可直接调用；写操作与系统级操作会请求用户确认——若用户拒绝，不要重试，改为询问用户下一步。\n\
3. 调用工具前确保参数完整、准确；缺少必要信息时先向用户询问。\n\
4. 如果有记忆工具，可用它记住用户的长期偏好与常用配置，并在需要时回忆。\n\
5. 完成后简洁地用中文总结你做了什么。不要臆造工具不存在的能力。"
    .to_string()
}

/// 若消息中没有 system，则在开头注入 Agent 系统提示
fn ensure_system_prompt(messages: &[Message], prompt: &str) -> Vec<Message> {
  if messages.iter().any(|m| m.role == MessageRole::System) {
    messages.to_vec()
  } else {
    let mut v = Vec::with_capacity(messages.len() + 1);
    v.push(Message::system(prompt));
    v.extend_from_slice(messages);
    v
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
  /// 提示信息（如降级为纯对话）
  Notice { text: String },
  /// 流式回答的增量文本片段
  ContentDelta { text: String },
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

/// OpenAI 流式增量的累加器（可单测）
#[derive(Default)]
struct OpenAiToolCall {
  id: String,
  name: String,
  args: String,
}

#[derive(Default)]
struct OpenAiStreamAcc {
  content: String,
  tool_calls: BTreeMap<u64, OpenAiToolCall>,
  done: bool,
}

impl OpenAiStreamAcc {
  /// 处理一条 SSE `data:` 负载，返回新增的内容片段（用于流式 delta）
  fn push(&mut self, data: &str) -> Option<String> {
    if data.trim() == "[DONE]" {
      self.done = true;
      return None;
    }
    let v: Value = serde_json::from_str(data).ok()?;
    let choice = v.get("choices").and_then(|c| c.get(0))?;
    let delta = choice.get("delta")?;

    let mut new_content = None;
    if let Some(c) = delta.get("content").and_then(|x| x.as_str()) {
      if !c.is_empty() {
        self.content.push_str(c);
        new_content = Some(c.to_string());
      }
    }
    if let Some(tcs) = delta.get("tool_calls").and_then(|x| x.as_array()) {
      for tc in tcs {
        let idx = tc.get("index").and_then(|x| x.as_u64()).unwrap_or(0);
        let entry = self.tool_calls.entry(idx).or_default();
        if let Some(id) = tc.get("id").and_then(|x| x.as_str()) {
          if !id.is_empty() {
            entry.id = id.to_string();
          }
        }
        if let Some(f) = tc.get("function") {
          if let Some(n) = f.get("name").and_then(|x| x.as_str()) {
            if !n.is_empty() {
              entry.name = n.to_string();
            }
          }
          if let Some(a) = f.get("arguments").and_then(|x| x.as_str()) {
            entry.args.push_str(a);
          }
        }
      }
    }
    if choice.get("finish_reason").and_then(|x| x.as_str()).is_some() {
      self.done = true;
    }
    new_content
  }

  fn has_tool_calls(&self) -> bool {
    !self.tool_calls.is_empty()
  }
}

/// OpenAI（及兼容协议）工具调用回路 — 流式
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
      "stream": true,
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

    // 逐块解析 SSE，实时推送内容片段
    let mut acc = OpenAiStreamAcc::default();
    let mut buffer = String::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
      let bytes = chunk.map_err(|e| format!("Stream error: {}", e))?;
      buffer.push_str(&String::from_utf8_lossy(&bytes));
      while let Some(pos) = buffer.find('\n') {
        let line: String = buffer.drain(..=pos).collect();
        let line = line.trim();
        if let Some(data) = line.strip_prefix("data: ") {
          if let Some(piece) = acc.push(data) {
            emit_event(ctx.app, ctx.request_id, AgentEvent::ContentDelta { text: piece });
          }
        }
      }
      if acc.done {
        break;
      }
    }

    if acc.has_tool_calls() {
      // 组装 assistant（含 tool_calls）消息
      let tool_calls_json: Vec<Value> = acc
        .tool_calls
        .values()
        .map(|t| {
          json!({
            "id": t.id,
            "type": "function",
            "function": { "name": t.name, "arguments": t.args }
          })
        })
        .collect();
      native_messages.push(json!({
        "role": "assistant",
        "content": acc.content,
        "tool_calls": tool_calls_json,
      }));

      for t in acc.tool_calls.values() {
        let args_value: Value = serde_json::from_str(&t.args).unwrap_or_else(|_| json!({}));
        let (result_text, _is_error) = execute_tool(ctx, &t.id, &t.name, &args_value).await;
        native_messages.push(json!({
          "role": "tool",
          "tool_call_id": t.id,
          "content": result_text,
        }));
      }

      if ctx.pending.is_cancelled(ctx.request_id) {
        emit_event(ctx.app, ctx.request_id, AgentEvent::Done);
        return Ok("操作已被用户取消".to_string());
      }
    } else {
      emit_event(ctx.app, ctx.request_id, AgentEvent::Content { text: acc.content.clone() });
      emit_event(ctx.app, ctx.request_id, AgentEvent::Done);
      return Ok(acc.content);
    }
  }

  // 达到步数上限：优雅收尾而非报错
  let msg = format!(
    "已达到最大工具调用步数（{}），任务可能未完全完成。请细化需求或分步进行。",
    MAX_ITERATIONS
  );
  emit_event(ctx.app, ctx.request_id, AgentEvent::Notice { text: msg.clone() });
  emit_event(ctx.app, ctx.request_id, AgentEvent::Done);
  Ok(msg)
}

/// Anthropic 流式块累加器（可单测）
#[derive(Default)]
struct AnthBlock {
  kind: String, // "text" | "tool_use"
  text: String, // 文本内容，或 tool_use 的 input JSON 累加
  id: String,
  name: String,
}

#[derive(Default)]
struct AnthropicStreamAcc {
  blocks: BTreeMap<u64, AnthBlock>,
  done: bool,
}

impl AnthropicStreamAcc {
  /// 处理一条 SSE `data:` 负载（按其内部 type 分派），返回新增文本片段
  fn push(&mut self, data: &str) -> Option<String> {
    let v: Value = serde_json::from_str(data).ok()?;
    match v.get("type").and_then(|t| t.as_str()) {
      Some("content_block_start") => {
        let idx = v.get("index").and_then(|x| x.as_u64()).unwrap_or(0);
        let cb = v.get("content_block");
        let block = AnthBlock {
          kind: cb
            .and_then(|c| c.get("type"))
            .and_then(|t| t.as_str())
            .unwrap_or("text")
            .to_string(),
          id: cb
            .and_then(|c| c.get("id"))
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string(),
          name: cb
            .and_then(|c| c.get("name"))
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string(),
          text: String::new(),
        };
        self.blocks.insert(idx, block);
        None
      }
      Some("content_block_delta") => {
        let idx = v.get("index").and_then(|x| x.as_u64()).unwrap_or(0);
        let delta = v.get("delta")?;
        let entry = self.blocks.entry(idx).or_default();
        match delta.get("type").and_then(|t| t.as_str()) {
          Some("text_delta") => {
            let text = delta.get("text").and_then(|s| s.as_str()).unwrap_or("");
            entry.text.push_str(text);
            if entry.kind.is_empty() {
              entry.kind = "text".to_string();
            }
            Some(text.to_string())
          }
          Some("input_json_delta") => {
            let pj = delta.get("partial_json").and_then(|s| s.as_str()).unwrap_or("");
            entry.text.push_str(pj);
            None
          }
          _ => None,
        }
      }
      Some("message_stop") => {
        self.done = true;
        None
      }
      _ => None,
    }
  }

  fn has_tool_use(&self) -> bool {
    self.blocks.values().any(|b| b.kind == "tool_use")
  }

  /// 还原 assistant content 块数组
  fn assistant_content(&self) -> Vec<Value> {
    self
      .blocks
      .values()
      .map(|b| {
        if b.kind == "tool_use" {
          let input: Value = serde_json::from_str(&b.text).unwrap_or_else(|_| json!({}));
          json!({ "type": "tool_use", "id": b.id, "name": b.name, "input": input })
        } else {
          json!({ "type": "text", "text": b.text })
        }
      })
      .collect()
  }

  fn text(&self) -> String {
    self
      .blocks
      .values()
      .filter(|b| b.kind != "tool_use")
      .map(|b| b.text.clone())
      .collect::<Vec<_>>()
      .join("")
  }
}

/// Anthropic 工具调用回路 — 流式
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
      "stream": true,
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

    let mut acc = AnthropicStreamAcc::default();
    let mut buffer = String::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
      let bytes = chunk.map_err(|e| format!("Stream error: {}", e))?;
      buffer.push_str(&String::from_utf8_lossy(&bytes));
      while let Some(pos) = buffer.find('\n') {
        let line: String = buffer.drain(..=pos).collect();
        let line = line.trim();
        if let Some(data) = line.strip_prefix("data: ") {
          if let Some(piece) = acc.push(data) {
            emit_event(ctx.app, ctx.request_id, AgentEvent::ContentDelta { text: piece });
          }
        }
      }
      if acc.done {
        break;
      }
    }

    if acc.has_tool_use() {
      native_messages.push(json!({ "role": "assistant", "content": acc.assistant_content() }));

      let mut tool_result_blocks: Vec<Value> = Vec::new();
      for b in acc.blocks.values().filter(|b| b.kind == "tool_use") {
        let input: Value = serde_json::from_str(&b.text).unwrap_or_else(|_| json!({}));
        let (result_text, is_error) = execute_tool(ctx, &b.id, &b.name, &input).await;
        tool_result_blocks.push(json!({
          "type": "tool_result",
          "tool_use_id": b.id,
          "content": result_text,
          "is_error": is_error,
        }));
      }
      native_messages.push(json!({ "role": "user", "content": tool_result_blocks }));

      if ctx.pending.is_cancelled(ctx.request_id) {
        emit_event(ctx.app, ctx.request_id, AgentEvent::Done);
        return Ok("操作已被用户取消".to_string());
      }
    } else {
      let content = acc.text();
      emit_event(ctx.app, ctx.request_id, AgentEvent::Content { text: content.clone() });
      emit_event(ctx.app, ctx.request_id, AgentEvent::Done);
      return Ok(content);
    }
  }

  // 达到步数上限：优雅收尾而非报错
  let msg = format!(
    "已达到最大工具调用步数（{}），任务可能未完全完成。请细化需求或分步进行。",
    MAX_ITERATIONS
  );
  emit_event(ctx.app, ctx.request_id, AgentEvent::Notice { text: msg.clone() });
  emit_event(ctx.app, ctx.request_id, AgentEvent::Done);
  Ok(msg)
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
  supports_tools: Option<bool>,
  system_prompt: Option<String>,
) -> Result<String, String> {
  let filter = ToolFilter {
    enabled_groups: servers.clone(),
    enabled_servers: servers,
  };
  // ②：所选模型明确不支持 function calling → 跳过工具注入，降级为纯对话
  let tools = if supports_tools == Some(false) {
    let all = registry.declarations(&filter);
    if !all.is_empty() {
      emit_event(
        &app,
        &request_id,
        AgentEvent::Notice {
          text: "所选模型不支持工具调用，已降级为纯对话。".to_string(),
        },
      );
    }
    Vec::new()
  } else {
    registry.declarations(&filter)
  };
  let auto_approve = auto_approve.unwrap_or_default();

  // Agent 模式（有工具）注入系统提示，塑造规划/确认/安全行为
  let effective_messages = if tools.is_empty() {
    messages
  } else {
    let prompt = system_prompt.unwrap_or_else(build_agent_system_prompt);
    ensure_system_prompt(&messages, &prompt)
  };

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
      run_anthropic_loop(&ctx, &client, &base_url, &key, &model, &effective_messages, temperature, max_tokens)
        .await
    }
    "ollama" => {
      let base_url = endpoint.unwrap_or_else(|| "http://localhost:11434/v1".to_string());
      run_openai_loop(&ctx, &client, &base_url, None, &model, &effective_messages, temperature, max_tokens)
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
        &effective_messages,
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

  #[test]
  fn test_stream_acc_content_pieces() {
    let mut acc = OpenAiStreamAcc::default();
    let p1 = acc.push(r#"{"choices":[{"delta":{"content":"Hel"}}]}"#);
    let p2 = acc.push(r#"{"choices":[{"delta":{"content":"lo"}}]}"#);
    assert_eq!(p1.as_deref(), Some("Hel"));
    assert_eq!(p2.as_deref(), Some("lo"));
    assert_eq!(acc.content, "Hello");
    assert!(!acc.has_tool_calls());
    acc.push("[DONE]");
    assert!(acc.done);
  }

  #[test]
  fn test_stream_acc_tool_calls_assembly() {
    let mut acc = OpenAiStreamAcc::default();
    // 分片到达的 tool_call
    acc.push(r#"{"choices":[{"delta":{"tool_calls":[{"index":0,"id":"call_1","function":{"name":"hosts_add","arguments":"{\"ip\":"}}]}}]}"#);
    acc.push(r#"{"choices":[{"delta":{"tool_calls":[{"index":0,"function":{"arguments":"\"1.2.3.4\"}"}}]}}]}"#);
    acc.push(r#"{"choices":[{"delta":{},"finish_reason":"tool_calls"}]}"#);
    assert!(acc.has_tool_calls());
    assert!(acc.done);
    let tc = acc.tool_calls.get(&0).unwrap();
    assert_eq!(tc.id, "call_1");
    assert_eq!(tc.name, "hosts_add");
    assert_eq!(tc.args, "{\"ip\":\"1.2.3.4\"}");
    // 累加的参数应为合法 JSON
    let v: Value = serde_json::from_str(&tc.args).unwrap();
    assert_eq!(v["ip"], "1.2.3.4");
  }

  #[test]
  fn test_stream_acc_ignores_garbage() {
    let mut acc = OpenAiStreamAcc::default();
    assert_eq!(acc.push("not json"), None);
    assert_eq!(acc.content, "");
  }

  #[test]
  fn test_anthropic_acc_text() {
    let mut acc = AnthropicStreamAcc::default();
    acc.push(r#"{"type":"content_block_start","index":0,"content_block":{"type":"text"}}"#);
    let p = acc.push(r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hi"}}"#);
    acc.push(r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":" there"}}"#);
    acc.push(r#"{"type":"message_stop"}"#);
    assert_eq!(p.as_deref(), Some("Hi"));
    assert_eq!(acc.text(), "Hi there");
    assert!(!acc.has_tool_use());
    assert!(acc.done);
  }

  #[test]
  fn test_agent_system_prompt_content() {
    let p = build_agent_system_prompt();
    assert!(p.contains("Jedi"));
    assert!(p.contains("确认"));
    assert!(p.contains("计划"));
  }

  #[test]
  fn test_ensure_system_prompt_adds_when_missing() {
    let msgs = vec![Message::user("hi")];
    let out = ensure_system_prompt(&msgs, "SYS");
    assert_eq!(out.len(), 2);
    assert_eq!(out[0].role, MessageRole::System);
    assert_eq!(out[0].content, "SYS");
    assert_eq!(out[1].role, MessageRole::User);
  }

  #[test]
  fn test_ensure_system_prompt_keeps_existing() {
    let msgs = vec![Message::system("orig"), Message::user("hi")];
    let out = ensure_system_prompt(&msgs, "SYS");
    assert_eq!(out.len(), 2);
    assert_eq!(out[0].content, "orig");
  }

  #[test]
  fn test_anthropic_acc_tool_use() {
    let mut acc = AnthropicStreamAcc::default();
    acc.push(r#"{"type":"content_block_start","index":0,"content_block":{"type":"tool_use","id":"toolu_1","name":"hosts_add"}}"#);
    acc.push(r#"{"type":"content_block_delta","index":0,"delta":{"type":"input_json_delta","partial_json":"{\"ip\":"}}"#);
    acc.push(r#"{"type":"content_block_delta","index":0,"delta":{"type":"input_json_delta","partial_json":"\"1.2.3.4\"}"}}"#);
    acc.push(r#"{"type":"message_stop"}"#);
    assert!(acc.has_tool_use());
    let content = acc.assistant_content();
    assert_eq!(content[0]["type"], "tool_use");
    assert_eq!(content[0]["name"], "hosts_add");
    assert_eq!(content[0]["input"]["ip"], "1.2.3.4");
  }
}
