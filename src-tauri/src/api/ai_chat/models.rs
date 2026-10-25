// AI Chat 模型提供商抽象层
// Phase 2: 多模型提供商支持（OpenAI、Anthropic、Ollama）

use reqwest::Client;
use serde::{Deserialize, Serialize};

// ========== 统一的消息结构体 ==========

/// 消息角色
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
  /// 系统消息
  System,
  /// 用户消息
  User,
  /// 助手消息
  Assistant,
}

impl ToString for MessageRole {
  fn to_string(&self) -> String {
    match self {
      MessageRole::System => "system".to_string(),
      MessageRole::User => "user".to_string(),
      MessageRole::Assistant => "assistant".to_string(),
    }
  }
}

/// 统一的消息结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
  /// 消息角色
  pub role: MessageRole,
  /// 消息内容
  pub content: String,
}

#[allow(dead_code)]
impl Message {
  /// 创建系统消息
  pub fn system(content: impl Into<String>) -> Self {
    Self {
      role: MessageRole::System,
      content: content.into(),
    }
  }

  /// 创建用户消息
  pub fn user(content: impl Into<String>) -> Self {
    Self {
      role: MessageRole::User,
      content: content.into(),
    }
  }

  /// 创建助手消息
  pub fn assistant(content: impl Into<String>) -> Self {
    Self {
      role: MessageRole::Assistant,
      content: content.into(),
    }
  }
}

// ========== 模型提供商 trait ==========

/// 聊天请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
  /// 模型名称
  pub model: String,
  /// 消息历史
  pub messages: Vec<Message>,
  /// 温度参数（0-2）
  pub temperature: Option<f32>,
  /// 最大 token 数
  pub max_tokens: Option<u32>,
  /// 是否使用流式响应
  pub stream: Option<bool>,
}

/// 非流式聊天响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
  /// 响应内容
  pub content: String,
  /// 使用的 token 数量（可选）
  pub usage: Option<Usage>,
}

/// Token 使用统计
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Usage {
  /// 输入 token 数
  pub prompt_tokens: u32,
  /// 输出 token 数
  pub completion_tokens: u32,
  /// 总 token 数
  pub total_tokens: u32,
}

/// 模型提供商 trait
#[async_trait::async_trait]
pub trait ModelProvider: Send + Sync {
  /// 获取提供商名称
  #[allow(dead_code)]
  fn name(&self) -> &str;

  /// 发送聊天请求（非流式）
  async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, String>;

}

// ========== OpenAI 提供商 ==========

/// OpenAI 提供商
pub struct OpenAiProvider {
  client: Client,
  api_key: String,
  base_url: String,
}

impl OpenAiProvider {
  /// 创建新的 OpenAI 提供商
  pub fn new(api_key: String, base_url: Option<String>) -> Self {
    Self {
      client: Client::new(),
      api_key,
      base_url: base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
    }
  }
}

#[async_trait::async_trait]
impl ModelProvider for OpenAiProvider {
  fn name(&self) -> &str {
    "openai"
  }

  async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, String> {
    // OpenAI API 请求体
    #[derive(Serialize)]
    struct OpenAiChatRequest {
      model: String,
      messages: Vec<OpenAiMessage>,
      #[serde(skip_serializing_if = "Option::is_none")]
      temperature: Option<f32>,
      #[serde(skip_serializing_if = "Option::is_none")]
      max_tokens: Option<u32>,
      stream: bool,
    }

    #[derive(Serialize, Deserialize)]
    struct OpenAiMessage {
      role: String,
      content: String,
    }

    // OpenAI API 响应
    #[derive(Deserialize)]
    struct OpenAiChatResponse {
      choices: Vec<OpenAiChoice>,
      #[serde(default)]
      usage: Option<OpenAiUsage>,
    }

    #[derive(Deserialize)]
    struct OpenAiChoice {
      message: OpenAiMessage,
    }

    #[derive(Deserialize)]
    struct OpenAiUsage {
      prompt_tokens: u32,
      completion_tokens: u32,
      total_tokens: u32,
    }

    let openai_request = OpenAiChatRequest {
      model: request.model,
      messages: request
        .messages
        .into_iter()
        .map(|m| OpenAiMessage {
          role: m.role.to_string(),
          content: m.content,
        })
        .collect(),
      temperature: request.temperature,
      max_tokens: request.max_tokens,
      stream: false,
    };

    let response = self
      .client
      .post(format!("{}/chat/completions", self.base_url))
      .header("Authorization", format!("Bearer {}", self.api_key))
      .header("Content-Type", "application/json")
      .json(&openai_request)
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

    let openai_response: OpenAiChatResponse = response
      .json()
      .await
      .map_err(|e| format!("Failed to parse OpenAI response: {}", e))?;

    let choice = openai_response
      .choices
      .into_iter()
      .next()
      .ok_or_else(|| "No choices in OpenAI response".to_string())?;

    Ok(ChatResponse {
      content: choice.message.content,
      usage: openai_response.usage.map(|u| Usage {
        prompt_tokens: u.prompt_tokens,
        completion_tokens: u.completion_tokens,
        total_tokens: u.total_tokens,
      }),
    })
  }
}

// ========== Anthropic 提供商 ==========

/// Anthropic 提供商
pub struct AnthropicProvider {
  client: Client,
  api_key: String,
  base_url: String,
}

impl AnthropicProvider {
  /// 创建新的 Anthropic 提供商
  pub fn new(api_key: String, base_url: Option<String>) -> Self {
    Self {
      client: Client::new(),
      api_key,
      base_url: base_url.unwrap_or_else(|| "https://api.anthropic.com/v1".to_string()),
    }
  }
}

#[async_trait::async_trait]
impl ModelProvider for AnthropicProvider {
  fn name(&self) -> &str {
    "anthropic"
  }

  async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, String> {
    // Anthropic API 请求体
    #[derive(Serialize)]
    struct AnthropicChatRequest {
      model: String,
      messages: Vec<AnthropicMessage>,
      #[serde(skip_serializing_if = "Option::is_none")]
      temperature: Option<f32>,
      #[serde(skip_serializing_if = "Option::is_none")]
      max_tokens: Option<u32>,
      #[serde(skip_serializing_if = "Option::is_none")]
      system: Option<String>,
      stream: bool,
    }

    #[derive(Serialize)]
    struct AnthropicMessage {
      role: String,
      content: String,
    }

    // Anthropic API 响应
    #[derive(Deserialize)]
    struct AnthropicChatResponse {
      content: Vec<AnthropicContent>,
      #[serde(default)]
      usage: Option<AnthropicUsage>,
    }

    #[derive(Deserialize)]
    struct AnthropicContent {
      text: String,
    }

    #[derive(Deserialize)]
    struct AnthropicUsage {
      input_tokens: u32,
      output_tokens: u32,
    }

    // 分离系统消息和其他消息
    let mut system_message = None;
    let mut anthropic_messages = Vec::new();

    for message in request.messages {
      match message.role {
        MessageRole::System => {
          system_message = Some(message.content);
        }
        MessageRole::User => {
          anthropic_messages.push(AnthropicMessage {
            role: "user".to_string(),
            content: message.content,
          });
        }
        MessageRole::Assistant => {
          anthropic_messages.push(AnthropicMessage {
            role: "assistant".to_string(),
            content: message.content,
          });
        }
      }
    }

    let anthropic_request = AnthropicChatRequest {
      model: request.model,
      messages: anthropic_messages,
      temperature: request.temperature,
      max_tokens: request.max_tokens,
      system: system_message,
      stream: false,
    };

    let response = self
      .client
      .post(format!("{}/messages", self.base_url))
      .header("x-api-key", &self.api_key)
      .header("anthropic-version", "2023-06-01")
      .header("Content-Type", "application/json")
      .json(&anthropic_request)
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

    let anthropic_response: AnthropicChatResponse = response
      .json()
      .await
      .map_err(|e| format!("Failed to parse Anthropic response: {}", e))?;

    let content = anthropic_response
      .content
      .into_iter()
      .next()
      .ok_or_else(|| "No content in Anthropic response".to_string())?;

    Ok(ChatResponse {
      content: content.text,
      usage: anthropic_response.usage.map(|u| Usage {
        prompt_tokens: u.input_tokens,
        completion_tokens: u.output_tokens,
        total_tokens: u.input_tokens + u.output_tokens,
      }),
    })
  }
}

// ========== Ollama 提供商 ==========

/// Ollama 提供商
pub struct OllamaProvider {
  client: Client,
  base_url: String,
}

impl OllamaProvider {
  /// 创建新的 Ollama 提供商
  pub fn new(base_url: Option<String>) -> Self {
    Self {
      client: Client::new(),
      base_url: base_url.unwrap_or_else(|| "http://localhost:11434/api".to_string()),
    }
  }
}

#[async_trait::async_trait]
impl ModelProvider for OllamaProvider {
  fn name(&self) -> &str {
    "ollama"
  }

  async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, String> {
    // Ollama API 请求体
    #[derive(Serialize)]
    struct OllamaChatRequest {
      model: String,
      messages: Vec<OllamaMessage>,
      #[serde(skip_serializing_if = "Option::is_none")]
      temperature: Option<f32>,
      #[serde(skip_serializing_if = "Option::is_none")]
      max_tokens: Option<u32>,
      stream: bool,
    }

    #[derive(Serialize, Deserialize)]
    struct OllamaMessage {
      role: String,
      content: String,
    }

    // Ollama API 响应
    #[derive(Deserialize)]
    #[allow(dead_code)]
    struct OllamaChatResponse {
      message: OllamaMessage,
      #[serde(default)]
      done: bool,
    }

    let ollama_request = OllamaChatRequest {
      model: request.model,
      messages: request
        .messages
        .into_iter()
        .map(|m| OllamaMessage {
          role: m.role.to_string(),
          content: m.content,
        })
        .collect(),
      temperature: request.temperature,
      max_tokens: request.max_tokens,
      stream: false,
    };

    let response = self
      .client
      .post(format!("{}/chat", self.base_url))
      .header("Content-Type", "application/json")
      .json(&ollama_request)
      .send()
      .await
      .map_err(|e| format!("Ollama API request failed: {}", e))?;

    if !response.status().is_success() {
      let error_text = response
        .text()
        .await
        .unwrap_or_else(|_| "Unknown error".to_string());
      return Err(format!("Ollama API error: {}", error_text));
    }

    let ollama_response: OllamaChatResponse = response
      .json()
      .await
      .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

    Ok(ChatResponse {
      content: ollama_response.message.content,
      usage: None,
    })
  }
}

// ========== Tauri commands ==========

use crate::utils::security::{KeyringManager, ModelProvider as KeyringModelProvider};
use secrecy::ExposeSecret;
use std::sync::Mutex;
use tauri::State;

/// 模型提供商管理器状态
pub struct ModelProviderManagerState {
  keyring_manager: Mutex<KeyringManager>,
}

impl ModelProviderManagerState {
  pub fn new(keyring_manager: KeyringManager) -> Self {
    Self {
      keyring_manager: Mutex::new(keyring_manager),
    }
  }
}

/// 发送聊天请求
#[tauri::command]
pub async fn send_chat_message(
  state: State<'_, ModelProviderManagerState>,
  provider: String,
  model: String,
  messages: Vec<Message>,
  temperature: Option<f32>,
  max_tokens: Option<u32>,
) -> Result<ChatResponse, String> {
  let request = ChatRequest {
    model,
    messages,
    temperature,
    max_tokens,
    stream: Some(false),
  };

  let provider_instance: Box<dyn ModelProvider> = match provider.to_lowercase().as_str() {
    "openai" => {
      let keyring = state.keyring_manager.lock().map_err(|e| e.to_string())?;
      let api_key = keyring
        .get_api_key(KeyringModelProvider::OpenAi)
        .map_err(|e| e.to_string())?;
      drop(keyring);
      let api_key = api_key.ok_or_else(|| "OpenAI API key not found".to_string())?;
      let key_str = api_key.key().expose_secret().to_string();
      let base_url = api_key.endpoint().map(|s| s.to_string());
      Box::new(OpenAiProvider::new(key_str, base_url))
    }
    "anthropic" => {
      let keyring = state.keyring_manager.lock().map_err(|e| e.to_string())?;
      let api_key = keyring
        .get_api_key(KeyringModelProvider::Anthropic)
        .map_err(|e| e.to_string())?;
      drop(keyring);
      let api_key = api_key.ok_or_else(|| "Anthropic API key not found".to_string())?;
      let key_str = api_key.key().expose_secret().to_string();
      let base_url = api_key.endpoint().map(|s| s.to_string());
      Box::new(AnthropicProvider::new(key_str, base_url))
    }
    "ollama" => Box::new(OllamaProvider::new(None)),
    _ => return Err(format!("Unknown provider type: {}", provider)),
  };

  provider_instance.chat(request).await
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_message_creation() {
    let system_msg = Message::system("You are a helpful assistant");
    assert_eq!(system_msg.role, MessageRole::System);
    assert_eq!(system_msg.content, "You are a helpful assistant");

    let user_msg = Message::user("Hello, how are you?");
    assert_eq!(user_msg.role, MessageRole::User);
    assert_eq!(user_msg.content, "Hello, how are you?");

    let assistant_msg = Message::assistant("I'm fine, thank you!");
    assert_eq!(assistant_msg.role, MessageRole::Assistant);
    assert_eq!(assistant_msg.content, "I'm fine, thank you!");
  }

  #[test]
  fn test_message_role_to_string() {
    assert_eq!(MessageRole::System.to_string(), "system");
    assert_eq!(MessageRole::User.to_string(), "user");
    assert_eq!(MessageRole::Assistant.to_string(), "assistant");
  }

  #[test]
  fn test_chat_request_creation() {
    let messages = vec![
      Message::system("You are a helpful assistant"),
      Message::user("Hello"),
    ];

    let request = ChatRequest {
      model: "gpt-4".to_string(),
      messages: messages.clone(),
      temperature: Some(0.7),
      max_tokens: Some(1000),
      stream: Some(false),
    };

    assert_eq!(request.model, "gpt-4");
    assert_eq!(request.messages.len(), 2);
    assert_eq!(request.temperature, Some(0.7));
    assert_eq!(request.max_tokens, Some(1000));
  }

  #[test]
  fn test_usage_default() {
    let usage = Usage::default();
    assert_eq!(usage.prompt_tokens, 0);
    assert_eq!(usage.completion_tokens, 0);
    assert_eq!(usage.total_tokens, 0);
  }

  #[test]
  fn test_openai_provider_creation() {
    let provider = OpenAiProvider::new("test-api-key".to_string(), None);
    assert_eq!(provider.api_key, "test-api-key");
    assert_eq!(provider.base_url, "https://api.openai.com/v1");
  }

  #[test]
  fn test_openai_provider_custom_base_url() {
    let provider = OpenAiProvider::new(
      "test-api-key".to_string(),
      Some("https://custom.openai.com/v1".to_string()),
    );
    assert_eq!(provider.api_key, "test-api-key");
    assert_eq!(provider.base_url, "https://custom.openai.com/v1");
  }

  #[test]
  fn test_anthropic_provider_creation() {
    let provider = AnthropicProvider::new("test-api-key".to_string(), None);
    assert_eq!(provider.api_key, "test-api-key");
    assert_eq!(provider.base_url, "https://api.anthropic.com/v1");
  }

  #[test]
  fn test_anthropic_provider_custom_base_url() {
    let provider = AnthropicProvider::new(
      "test-api-key".to_string(),
      Some("https://custom.anthropic.com".to_string()),
    );
    assert_eq!(provider.api_key, "test-api-key");
    assert_eq!(provider.base_url, "https://custom.anthropic.com");
  }

  #[test]
  fn test_ollama_provider_creation() {
    let provider = OllamaProvider::new(None);
    assert_eq!(provider.base_url, "http://localhost:11434/api");
  }

  #[test]
  fn test_ollama_provider_custom_base_url() {
    let provider = OllamaProvider::new(Some("http://custom.ollama.com:11434/api".to_string()));
    assert_eq!(provider.base_url, "http://custom.ollama.com:11434/api");
  }

  #[test]
  fn test_message_role_deserialization() {
    let system: MessageRole = serde_json::from_str("\"system\"").unwrap();
    assert_eq!(system, MessageRole::System);

    let user: MessageRole = serde_json::from_str("\"user\"").unwrap();
    assert_eq!(user, MessageRole::User);

    let assistant: MessageRole = serde_json::from_str("\"assistant\"").unwrap();
    assert_eq!(assistant, MessageRole::Assistant);
  }

  #[test]
  fn test_message_serialization() {
    let message = Message::user("Hello");
    let json = serde_json::to_string(&message).unwrap();
    assert!(json.contains("user"));
    assert!(json.contains("Hello"));

    let deserialized: Message = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.role, MessageRole::User);
    assert_eq!(deserialized.content, "Hello");
  }
}
