//! 输入验证模块
//! Phase 1: 输入验证和输出编码
//!
//! 提供服务端输入验证功能，作为前端验证的双重保障

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// ========== 验证结果 ==========

/// 验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
  /// 是否通过验证
  pub valid: bool,
  /// 清理后的内容
  pub sanitized: String,
  /// 警告信息
  pub warnings: Vec<String>,
}

#[allow(dead_code)]
impl ValidationResult {
  /// 创建通过的验证结果
  pub fn pass(sanitized: String) -> Self {
    Self {
      valid: true,
      sanitized,
      warnings: vec![],
    }
  }

  /// 创建失败的验证结果
  pub fn fail(warnings: Vec<String>) -> Self {
    Self {
      valid: false,
      sanitized: String::new(),
      warnings,
    }
  }

  /// 添加警告
  pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
    self.warnings.push(warning.into());
    self.valid = self.warnings.is_empty();
    self
  }
}

// ========== 验证选项 ==========

/// 验证选项
#[derive(Debug, Clone)]
pub struct ValidationOptions {
  /// 最大长度
  pub max_length: usize,
  /// 允许 URL
  pub allow_urls: bool,
  /// 允许的 URL 协议
  pub allowed_protocols: HashSet<String>,
  /// 阻止的模式列表
  pub blocklist: HashSet<String>,
  /// 最大 API Key 长度
  pub max_api_key_length: usize,
  /// 最小 API Key 长度
  pub min_api_key_length: usize,
}

impl Default for ValidationOptions {
  fn default() -> Self {
    Self {
      max_length: 100000,
      allow_urls: true,
      allowed_protocols: ["http".to_string(), "https".to_string()].into(),
      blocklist: [
        "javascript:".to_string(),
        "data:".to_string(),
        "vbscript:".to_string(),
        "livescript:".to_string(),
        "mocha:".to_string(),
        "eval(".to_string(),
        "expression(".to_string(),
      ]
      .into(),
      max_api_key_length: 512,
      min_api_key_length: 8,
    }
  }
}

// ========== 用户输入验证 ==========

/// 验证和清理用户输入
pub fn validate_user_input(input: &str, options: Option<&ValidationOptions>) -> ValidationResult {
  let default_opts = ValidationOptions::default();
  let opts = options.unwrap_or(&default_opts);
  let mut warnings = Vec::new();
  let mut sanitized = input.to_string();

  // 空值检查
  if input.trim().is_empty() {
    return ValidationResult::pass(String::new());
  }

  // 长度检查
  if sanitized.len() > opts.max_length {
    warnings.push(format!("输入长度超过限制（最大 {} 字符）", opts.max_length));
    sanitized.truncate(opts.max_length);
  }

  // 检查 blocklist
  for pattern in &opts.blocklist {
    let pattern_lower = pattern.to_lowercase();
    let input_lower = sanitized.to_lowercase();
    if input_lower.contains(&pattern_lower) {
      warnings.push(format!("检测到潜在危险内容: {}", pattern));
      // 简单替换，不做大小写敏感处理
      sanitized = sanitized.replace(pattern, "[REDACTED]");
    }
  }

  // 验证 URL
  if opts.allow_urls {
    if let Err(url_warnings) = validate_urls_in_text(&sanitized, opts) {
      warnings.extend(url_warnings);
    }
  }

  ValidationResult {
    valid: warnings.is_empty(),
    sanitized,
    warnings,
  }
}

// ========== API Key 验证 ==========

/// API Key 验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyValidation {
  pub valid: bool,
  pub warnings: Vec<String>,
  pub provider_hint: Option<String>,
}

/// 验证 API Key
pub fn validate_api_key(
  key: &str,
  provider: Option<&str>,
  options: Option<&ValidationOptions>,
) -> ApiKeyValidation {
  let default_opts = ValidationOptions::default();
  let opts = options.unwrap_or(&default_opts);
  let mut warnings = Vec::new();
  let trimmed = key.trim();

  // 基本检查
  if trimmed.is_empty() {
    return ApiKeyValidation {
      valid: false,
      warnings: vec!["API Key 不能为空".to_string()],
      provider_hint: None,
    };
  }

  if trimmed.len() < opts.min_api_key_length {
    warnings.push(format!(
      "API Key 长度太短（最少 {} 字符）",
      opts.min_api_key_length
    ));
  }

  if trimmed.len() > opts.max_api_key_length {
    warnings.push(format!(
      "API Key 长度超过限制（最大 {} 字符）",
      opts.max_api_key_length
    ));
  }

  // 提供商特定验证
  let mut provider_hint = None;

  if let Some(provider_name) = provider {
    match provider_name.to_lowercase().as_str() {
      "openai" => {
        if !trimmed.starts_with("sk-") {
          warnings.push("OpenAI API Key 应该以 'sk-' 开头".to_string());
        }
        if trimmed.len() < 50 {
          warnings.push("OpenAI API Key 长度看起来不正确".to_string());
        }
        provider_hint = Some("openai".to_string());
      }
      "anthropic" => {
        if !trimmed.starts_with("sk-ant-") {
          warnings.push("Anthropic API Key 应该以 'sk-ant-' 开头".to_string());
        }
        provider_hint = Some("anthropic".to_string());
      }
      _ => {}
    }
  } else {
    // 自动检测提供商
    if trimmed.starts_with("sk-") && trimmed.len() >= 50 {
      provider_hint = Some("openai".to_string());
    } else if trimmed.starts_with("sk-ant-") {
      provider_hint = Some("anthropic".to_string());
    }
  }

  ApiKeyValidation {
    valid: warnings.is_empty(),
    warnings,
    provider_hint,
  }
}

// ========== 端点 URL 验证 ==========

/// 验证端点 URL
pub fn validate_endpoint(url: &str, options: Option<&ValidationOptions>) -> ValidationResult {
  let default_opts = ValidationOptions::default();
  let opts = options.unwrap_or(&default_opts);
  let trimmed = url.trim();

  if trimmed.is_empty() {
    return ValidationResult::pass(String::new());
  }

  let mut warnings = Vec::new();

  // 解析 URL
  match url::Url::parse(trimmed) {
    Ok(parsed_url) => {
      // 检查协议
      let protocol = parsed_url.scheme().to_lowercase();
      if !opts.allowed_protocols.contains(&protocol) {
        warnings.push(format!(
          "不允许的协议: {}（只允许 {:?}）",
          protocol, opts.allowed_protocols
        ));
      }

      // 检查主机
      if let Some(host) = parsed_url.host_str() {
        let host_lower = host.to_lowercase();
        // 检查是否是内网地址（除了 localhost）
        if is_private_network(&host_lower) && !is_localhost(&host_lower) {
          warnings.push("不建议使用私有网络地址".to_string());
        }
      }
    }
    Err(_) => {
      warnings.push("无效的 URL 格式".to_string());
    }
  }

  ValidationResult {
    valid: warnings.is_empty(),
    sanitized: trimmed.to_string(),
    warnings,
  }
}

// ========== 聊天消息验证 ==========

/// 聊天消息验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageValidation {
  pub valid: bool,
  pub content: String,
  pub warnings: Vec<String>,
  pub metadata: MessageMetadata,
}

/// 消息元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetadata {
  pub word_count: usize,
  pub char_count: usize,
  pub has_urls: bool,
  pub has_code: bool,
}

/// 验证聊天消息
pub fn validate_chat_message(content: &str, is_user: bool) -> ChatMessageValidation {
  let mut warnings = Vec::new();
  let mut sanitized_content = content.to_string();

  // 用户消息需要更严格的验证
  if is_user {
    let validation = validate_user_input(content, None);
    sanitized_content = validation.sanitized;
    warnings.extend(validation.warnings);
  }

  // 分析元数据
  let words: Vec<&str> = sanitized_content
    .trim()
    .split_whitespace()
    .filter(|w| !w.is_empty())
    .collect();

  let url_regex = Regex::new(r"https?://[^\s]+").unwrap();
  let code_regex = Regex::new(r"```[\s\S]*?```|`[^`]+`").unwrap();

  let metadata = MessageMetadata {
    word_count: words.len(),
    char_count: sanitized_content.len(),
    has_urls: url_regex.is_match(&sanitized_content),
    has_code: code_regex.is_match(&sanitized_content),
  };

  // 额外检查
  if metadata.char_count == 0 {
    warnings.push("消息内容不能为空".to_string());
  }

  if is_user && metadata.char_count > 50000 {
    warnings.push("消息较长，可能会影响响应速度".to_string());
  }

  ChatMessageValidation {
    valid: warnings.is_empty() || (warnings.len() == 1 && metadata.char_count > 50000),
    content: sanitized_content,
    warnings,
    metadata,
  }
}

// ========== 辅助函数 ==========

/// 验证文本中的 URL
fn validate_urls_in_text(text: &str, opts: &ValidationOptions) -> Result<(), Vec<String>> {
  let mut warnings = Vec::new();
  let url_regex = Regex::new(r"(https?://[^\s]+)").unwrap();

  for cap in url_regex.captures_iter(text) {
    if let Some(url_match) = cap.get(1) {
      if let Ok(parsed_url) = url::Url::parse(url_match.as_str()) {
        let protocol = parsed_url.scheme().to_lowercase();
        if !opts.allowed_protocols.contains(&protocol) {
          warnings.push(format!("不允许的 URL 协议: {}", protocol));
        }
      }
    }
  }

  if warnings.is_empty() {
    Ok(())
  } else {
    Err(warnings)
  }
}

/// 检查是否是 localhost
fn is_localhost(host: &str) -> bool {
  matches!(
    host.to_lowercase().as_str(),
    "localhost" | "127.0.0.1" | "::1" | "0.0.0.0"
  )
}

/// 检查是否是私有网络地址
fn is_private_network(host: &str) -> bool {
  // 检查 IPv4 私有地址范围
  let private_ipv4_patterns = [
    Regex::new(r"^10\.").unwrap(),
    Regex::new(r"^172\.(1[6-9]|2[0-9]|3[01])\.").unwrap(),
    Regex::new(r"^192\.168\.").unwrap(),
  ];

  for pattern in &private_ipv4_patterns {
    if pattern.is_match(host) {
      return true;
    }
  }

  // 检查是否是内网域名
  if host.ends_with(".local") || host.ends_with(".internal") || host.ends_with(".lan") {
    return true;
  }

  false
}

// ========== HTML 清理（输出编码） ==========

/// 清理 HTML（基本清理，作为前端 DOMPurify 的双重保障）
pub fn sanitize_html(html: &str) -> String {
  // 移除危险标签
  let dangerous_tags = [
    "script", "iframe", "form", "input", "button", "textarea", "select", "option",
  ];

  let mut cleaned = html.to_string();

  for tag in &dangerous_tags {
    // 移除开始和结束标签及其内容
    let open_re = Regex::new(&format!(r"<{}\b[^>]*>", tag)).unwrap();
    let close_re = Regex::new(&format!(r"</{}>", tag)).unwrap();

    cleaned = open_re.replace_all(&cleaned, "").to_string();
    cleaned = close_re.replace_all(&cleaned, "").to_string();
  }

  // 移除危险属性
  let dangerous_attrs = [
    "onload",
    "onerror",
    "onclick",
    "onmouseover",
    "onfocus",
    "onblur",
    "onsubmit",
    "onmouseenter",
    "onmouseleave",
    "onkeydown",
    "onkeyup",
    "onkeypress",
  ];

  for attr in &dangerous_attrs {
    let attr_re = Regex::new(&format!(r#"\s+{}=["'][^"']*["']"#, attr)).unwrap();
    cleaned = attr_re.replace_all(&cleaned, "").to_string();
  }

  cleaned
}

/// HTML 实体编码
pub fn encode_html(text: &str) -> String {
  text
    .chars()
    .map(|c| match c {
      '<' => "&lt;".to_string(),
      '>' => "&gt;".to_string(),
      '&' => "&amp;".to_string(),
      '"' => "&quot;".to_string(),
      '\'' => "&#x27;".to_string(),
      _ => c.to_string(),
    })
    .collect()
}

// ========== 单元测试 ==========

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_validate_user_input_normal() {
    let result = validate_user_input("Hello, world!", None);
    assert!(result.valid);
    assert_eq!(result.sanitized, "Hello, world!");
    assert!(result.warnings.is_empty());
  }

  #[test]
  fn test_validate_user_input_empty() {
    let result = validate_user_input("", None);
    assert!(result.valid);
    assert_eq!(result.sanitized, "");
  }

  #[test]
  fn test_validate_user_input_javascript_url() {
    let result = validate_user_input("Click here: javascript:alert(1)", None);
    assert!(!result.valid);
    assert!(result.warnings.len() >= 1);
    assert!(result.sanitized.contains("[REDACTED]"));
  }

  #[test]
  fn test_validate_user_input_too_long() {
    let long_text: String = "a".repeat(200000);
    let opts = ValidationOptions {
      max_length: 100000,
      ..ValidationOptions::default()
    };
    let result = validate_user_input(&long_text, Some(&opts));
    assert!(!result.valid);
    assert_eq!(result.sanitized.len(), 100000);
  }

  #[test]
  fn test_validate_api_key_openai() {
    let valid_key = "sk-1234567890abcdefghijklmnopqrstuvwxyz1234567890abcd";
    let result = validate_api_key(valid_key, Some("openai"), None);
    assert!(result.valid);
    assert_eq!(result.provider_hint, Some("openai".to_string()));
  }

  #[test]
  fn test_validate_api_key_openai_invalid_prefix() {
    let invalid_key = "pk-1234567890abcdefghijklmnopqrstuvwxyz1234567890abcd";
    let result = validate_api_key(invalid_key, Some("openai"), None);
    assert!(!result.valid);
    assert!(!result.warnings.is_empty());
  }

  #[test]
  fn test_validate_api_key_anthropic() {
    let valid_key = "sk-ant-1234567890abcdefghijklmnopqrstuvwxyz";
    let result = validate_api_key(valid_key, Some("anthropic"), None);
    assert!(result.valid);
    assert_eq!(result.provider_hint, Some("anthropic".to_string()));
  }

  #[test]
  fn test_validate_api_key_empty() {
    let result = validate_api_key("", Some("openai"), None);
    assert!(!result.valid);
    assert!(result.warnings.contains(&"API Key 不能为空".to_string()));
  }

  #[test]
  fn test_validate_api_key_too_short() {
    let result = validate_api_key("sk-123", None, None);
    assert!(!result.valid);
  }

  #[test]
  fn test_validate_endpoint_http() {
    let result = validate_endpoint("http://localhost:11434", None);
    assert!(result.valid);
  }

  #[test]
  fn test_validate_endpoint_https() {
    let result = validate_endpoint("https://api.openai.com/v1", None);
    assert!(result.valid);
  }

  #[test]
  fn test_validate_endpoint_invalid_protocol() {
    let result = validate_endpoint("file:///etc/passwd", None);
    assert!(!result.valid);
    assert!(!result.warnings.is_empty());
  }

  #[test]
  fn test_validate_endpoint_invalid_url() {
    let result = validate_endpoint("not-a-url", None);
    assert!(!result.valid);
  }

  #[test]
  fn test_validate_chat_message_normal() {
    let result = validate_chat_message("Hello, how are you?", true);
    assert!(result.valid);
    assert_eq!(result.content, "Hello, how are you?");
    assert_eq!(result.metadata.word_count, 4);
  }

  #[test]
  fn test_validate_chat_message_with_code() {
    let result = validate_chat_message("Check this: `let x = 1;`", true);
    assert!(result.valid);
    assert!(result.metadata.has_code);
  }

  #[test]
  fn test_validate_chat_message_with_url() {
    let result = validate_chat_message("Visit https://example.com", true);
    assert!(result.valid);
    assert!(result.metadata.has_urls);
  }

  #[test]
  fn test_sanitize_html_script() {
    let input = "<script>alert(1)</script>Hello";
    let cleaned = sanitize_html(input);
    assert!(!cleaned.contains("<script"));
  }

  #[test]
  fn test_sanitize_html_onerror() {
    let input = r#"<img src="x.jpg" onerror="alert(1)">"#;
    let cleaned = sanitize_html(input);
    assert!(!cleaned.contains("onerror"));
  }

  #[test]
  fn test_encode_html() {
    let input = "<script>&\"'";
    let encoded = encode_html(input);
    assert_eq!(encoded, "&lt;script&gt;&amp;&quot;&#x27;");
  }

  #[test]
  fn test_is_localhost() {
    assert!(is_localhost("localhost"));
    assert!(is_localhost("127.0.0.1"));
    assert!(is_localhost("::1"));
    assert!(!is_localhost("example.com"));
  }

  #[test]
  fn test_is_private_network() {
    assert!(is_private_network("192.168.1.1"));
    assert!(is_private_network("10.0.0.1"));
    assert!(is_private_network("172.16.0.1"));
    assert!(is_private_network("server.local"));
    assert!(!is_private_network("example.com"));
    assert!(!is_private_network("1.1.1.1"));
  }
}
