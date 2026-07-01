// AI Chat Agent 工具调用回路
// Phase 4: 将 MCP 工具接入聊天，实现 Agent 工具调用（function calling）
//
// 本模块把已实现的 MCP Server（当前为 Hosts）暴露给 LLM，
// 通过 OpenAI / Anthropic 的 function calling 能力实现"边思考边调工具"的 Agent 回路：
//
//   用户消息 → LLM（携带工具定义）→ 若返回 tool_calls → 执行 MCP 工具 →
//   把结果回填 → 再次调用 LLM → ... → 直到 LLM 给出最终回答
//
// 每一步都会通过 `agent-event-{request_id}` 事件推送给前端，用于 Agent Trace 面板。

use crate::api::ai_chat::models::{Message, MessageRole, ModelProviderManagerState};
use crate::mcp::servers::HostsMcpServer;
use crate::mcp::types::{CallToolResult, Content, Tool};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tauri::{Emitter, State};

/// Agent 回路最大迭代次数（防止工具调用无限循环）
const MAX_ITERATIONS: usize = 8;

// ============================================================================
// MCP 服务注册表（当前仅内置 Hosts，后续可扩展）
// ============================================================================

/// 列出指定 MCP 服务的工具
pub fn list_server_tools(server_id: &str) -> Result<Vec<Tool>, String> {
  match server_id {
    "hosts" => {
      let mut server = HostsMcpServer::new();
      server.initialize().map_err(|e| e.to_string())?;
      server.list_tools().map_err(|e| e.to_string())
    }
    other => Err(format!("Unknown MCP server: {}", other)),
  }
}

/// 调用指定 MCP 服务的工具
pub fn call_server_tool(
  server_id: &str,
  tool_name: &str,
  arguments: Option<HashMap<String, Value>>,
) -> Result<CallToolResult, String> {
  match server_id {
    "hosts" => {
      let mut server = HostsMcpServer::new();
      server.initialize().map_err(|e| e.to_string())?;
      server
        .call_tool(tool_name, arguments)
        .map_err(|e| e.to_string())
    }
    other => Err(format!("Unknown MCP server: {}", other)),
  }
}

/// 前端展示用的工具信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
  /// 所属 MCP 服务 ID
  pub server: String,
  /// 工具名称
  pub name: String,
  /// 工具描述
  pub description: Option<String>,
  /// 输入 JSON Schema
  pub input_schema: Value,
}

/// 收集一组已启用 MCP 服务的全部工具，并建立 工具名 → 服务ID 的映射
fn collect_tools(servers: &[String]) -> Result<(Vec<(String, Tool)>, HashMap<String, String>), String> {
  let mut tools = Vec::new();
  let mut tool_to_server = HashMap::new();

  for server_id in servers {
    let server_tools = list_server_tools(server_id)?;
    for tool in server_tools {
      tool_to_server.insert(tool.name.clone(), server_id.clone());
      tools.push((server_id.clone(), tool));
    }
  }

  Ok((tools, tool_to_server))
}

// ============================================================================
// 工具定义格式转换
// ============================================================================

/// 把 MCP 工具的输入 schema 转换为标准 JSON Schema 对象
fn input_schema_json(tool: &Tool) -> Value {
  let mut schema = json!({
    "type": "object",
    "properties": tool.input_schema.properties.clone().unwrap_or_default(),
  });
  if let Some(required) = &tool.input_schema.required {
    schema["required"] = json!(required);
  }
  schema
}

/// 转换为 OpenAI function calling 的 tool 定义
fn tool_to_openai(tool: &Tool) -> Value {
  json!({
    "type": "function",
    "function": {
      "name": tool.name,
      "description": tool.description.clone().unwrap_or_default(),
      "parameters": input_schema_json(tool),
    }
  })
}

/// 转换为 Anthropic 的 tool 定义
fn tool_to_anthropic(tool: &Tool) -> Value {
  json!({
    "name": tool.name,
    "description": tool.description.clone().unwrap_or_default(),
    "input_schema": input_schema_json(tool),
  })
}

/// 把工具调用结果拍平为纯文本（用于回填给 LLM）
fn flatten_result(result: &CallToolResult) -> String {
  result
    .content
    .iter()
    .map(|c| match c {
      Content::Text { text } => text.clone(),
      Content::Image { mime_type, .. } => format!("[image: {}]", mime_type),
      Content::Resource { resource } => resource
        .text
        .clone()
        .unwrap_or_else(|| format!("[resource: {}]", resource.uri)),
    })
    .collect::<Vec<_>>()
    .join("\n")
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
async fn run_openai_loop(
  app: &tauri::AppHandle,
  request_id: &str,
  client: &Client,
  base_url: &str,
  api_key: Option<&str>,
  model: &str,
  messages: &[Message],
  temperature: Option<f32>,
  max_tokens: Option<u32>,
  tools: &[(String, Tool)],
  tool_to_server: &HashMap<String, String>,
) -> Result<String, String> {
  let mut native_messages = to_openai_messages(messages);
  let openai_tools: Vec<Value> = tools.iter().map(|(_, t)| tool_to_openai(t)).collect();

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
        // 先展示可能的中间思考文本
        if let Some(text) = message.get("content").and_then(|c| c.as_str()) {
          if !text.trim().is_empty() {
            emit_event(app, request_id, AgentEvent::Thinking { text: text.to_string() });
          }
        }
        // 把 assistant（含 tool_calls）消息加入历史
        native_messages.push(message.clone());

        // 依次执行每个工具调用
        for call in calls {
          let id = call
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
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

          let (result_text, is_error) =
            execute_tool(app, request_id, tool_to_server, &id, &name, &args_value);

          native_messages.push(json!({
            "role": "tool",
            "tool_call_id": id,
            "content": result_text,
          }));
          let _ = is_error;
        }
      }
      _ => {
        // 没有工具调用 → 最终回答
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

  Err(format!(
    "Agent 达到最大迭代次数 ({})，未能完成任务",
    MAX_ITERATIONS
  ))
}

/// Anthropic 工具调用回路
async fn run_anthropic_loop(
  app: &tauri::AppHandle,
  request_id: &str,
  client: &Client,
  base_url: &str,
  api_key: &str,
  model: &str,
  messages: &[Message],
  temperature: Option<f32>,
  max_tokens: Option<u32>,
  tools: &[(String, Tool)],
  tool_to_server: &HashMap<String, String>,
) -> Result<String, String> {
  // 分离 system 消息
  let system = messages
    .iter()
    .find(|m| m.role == MessageRole::System)
    .map(|m| m.content.clone());

  let mut native_messages: Vec<Value> = messages
    .iter()
    .filter(|m| m.role != MessageRole::System)
    .map(|m| json!({ "role": m.role.to_string(), "content": m.content }))
    .collect();

  let anthropic_tools: Vec<Value> = tools.iter().map(|(_, t)| tool_to_anthropic(t)).collect();

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

    // 收集 tool_use 块与文本块
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
            execute_tool(app, request_id, tool_to_server, &id, &name, &input);

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
      // 展示中间思考文本
      let thinking = text_parts.join("\n");
      if !thinking.trim().is_empty() {
        emit_event(app, request_id, AgentEvent::Thinking { text: thinking });
      }
      // assistant 回合（原样回填 content 块）
      native_messages.push(json!({ "role": "assistant", "content": content_blocks }));
      // user 回合携带 tool_result
      native_messages.push(json!({ "role": "user", "content": tool_result_blocks }));
    } else {
      let content = text_parts.join("\n");
      emit_event(app, request_id, AgentEvent::Content { text: content.clone() });
      emit_event(app, request_id, AgentEvent::Done);
      return Ok(content);
    }
  }

  Err(format!(
    "Agent 达到最大迭代次数 ({})，未能完成任务",
    MAX_ITERATIONS
  ))
}

/// 执行单个工具调用并推送事件，返回 (结果文本, 是否出错)
fn execute_tool(
  app: &tauri::AppHandle,
  request_id: &str,
  tool_to_server: &HashMap<String, String>,
  id: &str,
  name: &str,
  arguments: &Value,
) -> (String, bool) {
  let server = tool_to_server.get(name).cloned().unwrap_or_default();

  emit_event(
    app,
    request_id,
    AgentEvent::ToolCall {
      id: id.to_string(),
      server: server.clone(),
      name: name.to_string(),
      arguments: arguments.clone(),
    },
  );

  // 把 JSON 对象参数转换为 HashMap
  let args_map: Option<HashMap<String, Value>> = arguments
    .as_object()
    .map(|obj| obj.clone().into_iter().collect());

  let (result_text, is_error) = if server.is_empty() {
    (format!("未知工具: {}", name), true)
  } else {
    match call_server_tool(&server, name, args_map) {
      Ok(result) => {
        let is_err = result.is_error.unwrap_or(false);
        (flatten_result(&result), is_err)
      }
      Err(e) => (format!("工具执行失败: {}", e), true),
    }
  };

  emit_event(
    app,
    request_id,
    AgentEvent::ToolResult {
      id: id.to_string(),
      name: name.to_string(),
      content: result_text.clone(),
      is_error,
    },
  );

  (result_text, is_error)
}

// ============================================================================
// Tauri commands
// ============================================================================

/// 列出已启用 MCP 服务的所有工具（供前端展示）
#[tauri::command]
pub fn mcp_list_tools(servers: Vec<String>) -> Result<Vec<ToolInfo>, String> {
  let mut infos = Vec::new();
  for server_id in &servers {
    for tool in list_server_tools(server_id)? {
      infos.push(ToolInfo {
        server: server_id.clone(),
        input_schema: input_schema_json(&tool),
        name: tool.name,
        description: tool.description,
      });
    }
  }
  Ok(infos)
}

/// 直接调用某个 MCP 工具（供手动/调试使用）
#[tauri::command]
pub fn mcp_call_tool(
  server: String,
  name: String,
  arguments: Option<HashMap<String, Value>>,
) -> Result<CallToolResult, String> {
  call_server_tool(&server, &name, arguments)
}

/// Agent 聊天：携带 MCP 工具运行工具调用回路，返回最终回答
///
/// 过程事件通过 `agent-event-{request_id}` 推送给前端。
#[tauri::command]
pub async fn agent_chat(
  app: tauri::AppHandle,
  state: State<'_, ModelProviderManagerState>,
  provider: String,
  model: String,
  messages: Vec<Message>,
  servers: Vec<String>,
  temperature: Option<f32>,
  max_tokens: Option<u32>,
  request_id: String,
) -> Result<String, String> {
  // 收集工具
  let (tools, tool_to_server) = match collect_tools(&servers) {
    Ok(v) => v,
    Err(e) => {
      emit_event(&app, &request_id, AgentEvent::Error { message: e.clone() });
      return Err(e);
    }
  };

  let (api_key, endpoint) = state.get_credentials(&provider)?;
  let client = Client::new();

  let result = match provider.to_lowercase().as_str() {
    "anthropic" => {
      let key = api_key.ok_or_else(|| "Anthropic API key not found".to_string())?;
      let base_url = endpoint.unwrap_or_else(|| "https://api.anthropic.com/v1".to_string());
      run_anthropic_loop(
        &app,
        &request_id,
        &client,
        &base_url,
        &key,
        &model,
        &messages,
        temperature,
        max_tokens,
        &tools,
        &tool_to_server,
      )
      .await
    }
    "ollama" => {
      let base_url = endpoint.unwrap_or_else(|| "http://localhost:11434/v1".to_string());
      run_openai_loop(
        &app,
        &request_id,
        &client,
        &base_url,
        None,
        &model,
        &messages,
        temperature,
        max_tokens,
        &tools,
        &tool_to_server,
      )
      .await
    }
    // openai 及其它 OpenAI 兼容协议（deepseek 等）
    _ => {
      let base_url = endpoint.unwrap_or_else(|| "https://api.openai.com/v1".to_string());
      run_openai_loop(
        &app,
        &request_id,
        &client,
        &base_url,
        api_key.as_deref(),
        &model,
        &messages,
        temperature,
        max_tokens,
        &tools,
        &tool_to_server,
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

  #[test]
  fn test_list_hosts_tools() {
    let tools = list_server_tools("hosts").unwrap();
    assert!(!tools.is_empty());
    let names: Vec<_> = tools.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"hosts_read"));
    assert!(names.contains(&"hosts_add"));
  }

  #[test]
  fn test_unknown_server() {
    assert!(list_server_tools("nope").is_err());
    assert!(call_server_tool("nope", "x", None).is_err());
  }

  #[test]
  fn test_collect_tools_builds_map() {
    let (tools, map) = collect_tools(&["hosts".to_string()]).unwrap();
    assert!(!tools.is_empty());
    assert_eq!(map.get("hosts_read").map(|s| s.as_str()), Some("hosts"));
  }

  #[test]
  fn test_tool_to_openai_shape() {
    let tools = list_server_tools("hosts").unwrap();
    let add = tools.iter().find(|t| t.name == "hosts_add").unwrap();
    let v = tool_to_openai(add);
    assert_eq!(v["type"], "function");
    assert_eq!(v["function"]["name"], "hosts_add");
    assert_eq!(v["function"]["parameters"]["type"], "object");
    // hosts_add 要求 ip/domain/group
    let required = v["function"]["parameters"]["required"].as_array().unwrap();
    assert!(required.iter().any(|x| x == "ip"));
  }

  #[test]
  fn test_tool_to_anthropic_shape() {
    let tools = list_server_tools("hosts").unwrap();
    let read = tools.iter().find(|t| t.name == "hosts_read").unwrap();
    let v = tool_to_anthropic(read);
    assert_eq!(v["name"], "hosts_read");
    assert_eq!(v["input_schema"]["type"], "object");
  }

  #[test]
  fn test_flatten_result_text() {
    let result = CallToolResult {
      content: vec![Content::text("line1"), Content::text("line2")],
      is_error: Some(false),
    };
    assert_eq!(flatten_result(&result), "line1\nline2");
  }

  #[test]
  fn test_input_schema_json_defaults_empty_properties() {
    let tools = list_server_tools("hosts").unwrap();
    let read = tools.iter().find(|t| t.name == "hosts_read").unwrap();
    let schema = input_schema_json(read);
    // hosts_read 无参数，properties 应为对象（可能为空）
    assert!(schema["properties"].is_object());
  }
}
