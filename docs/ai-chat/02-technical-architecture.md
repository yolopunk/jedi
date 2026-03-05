# Jedi AI Chat 模块技术方案

## 1. 概述

本文档详细描述了 Jedi 项目 AI Chat 模块的技术架构设计，基于产品经理的需求调研报告，结合现有代码架构进行设计。

### 1.1 项目背景
Jedi 是一个基于 Tauri v2 + Vue 3 + Vuetify 的跨平台桌面开发者工具箱，现有功能包括 Hosts 管理、知识壁纸、小宇宙播客。本次需要添加 AI Chat 模块，支持多模型提供商、MCP 协议集成和 Skills 技能系统。

### 1.2 设计原则
- **渐进式设计**：先实现 MVP 核心功能，再逐步添加高级特性
- **与现有架构对齐**：遵循项目现有的代码组织和设计模式
- **简洁实用**：避免过度设计，优先解决实际问题
- **可扩展性**：为未来的功能扩展预留合理的架构空间
- **安全优先**：将安全考虑融入架构设计的每个环节

---

## 2. 系统架构

### 2.1 整体架构图

```
┌─────────────────────────────────────────────────────────────────┐
│                        Jedi AI Chat                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌──────────────────────┐    ┌──────────────────────────────┐  │
│  │   前端层 (Vue 3)    │    │      后端层 (Rust/Tauri)    │  │
│  │                      │    │                              │  │
│  │  ┌────────────────┐  │    │  ┌────────────────────────┐  │
│  │  │ Chat UI        │  │    │  │ Tauri Commands         │  │
│  │  │ (Vuetify)      │  │    │  │ (API Gateway)          │  │
│  │  └────────────────┘  │    │  └────────────────────────┘  │
│  │           ↓            │    │              ↓               │  │
│  │  ┌────────────────┐  │    │  ┌────────────────────────┐  │
│  │  │ State Manager  │  │    │  │ AI Chat Service        │  │
│  │  │ (Pinia)        │  │    │  │ (Business Logic)       │  │
│  │  └────────────────┘  │    │  └────────────────────────┘  │
│  │           ↓            │    │              ↓               │  │
│  │  ┌────────────────┐  │    │  ┌────────────────────────┐  │
│  │  │ API Client     │  │◄──►│  │ Security Layer         │  │
│  │  │ (Tauri invoke) │  │    │  │ - Auth                 │  │
│  │  └────────────────┘  │    │  │ - Validation           │  │
│  │                        │    │  │ - Audit Logging        │  │
│  └──────────────────────┘    │  └────────────────────────┘  │
│                               │              ↓               │  │
│                               │  ┌────────────────────────┐  │
│                               │  │ Model Providers        │  │
│                               │  │ - OpenAI               │  │
│                               │  │ - Anthropic            │  │
│                               │  │ - Ollama               │  │
│                               │  └────────────────────────┘  │
│                               │              ↓               │  │
│                               │  ┌────────────────────────┐  │
│                               │  │ MCP Manager            │  │
│                               │  │ - Server Discovery     │  │
│                               │  │ - Sandbox Execution    │  │
│                               │  │ - Permission Control   │  │
│                               │  └────────────────────────┘  │
│                               │              ↓               │  │
│                               │  ┌────────────────────────┐  │
│                               │  │ Secure Storage         │  │
│                               │  │ - Keyring (API Keys)  │  │
│                               │  │ - Encrypted (History)  │  │
│                               │  └────────────────────────┘  │
│                               └──────────────────────────────┘  │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 安全架构

#### 安全分层设计

```
┌─────────────────────────────────────────────────────────┐
│                    安全架构分层                            │
├─────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────┐  │
│  │  用户界面安全层                                    │  │
│  │  - 输入验证                                        │  │
│  │  - 输出编码                                        │  │
│  │  - CSP 防护                                        │  │
│  │  - XSS 防护                                        │  │
│  └──────────────────────────────────────────────────┘  │
│                            ↓                              │
│  ┌──────────────────────────────────────────────────┐  │
│  │  通信安全层                                        │  │
│  │  - HTTPS 强制                                      │  │
│  │  - 证书固定                                        │  │
│  │  - 请求签名                                        │  │
│  │  - 响应验证                                        │  │
│  └──────────────────────────────────────────────────┘  │
│                            ↓                              │
│  ┌──────────────────────────────────────────────────┐  │
│  │  业务逻辑安全层                                    │  │
│  │  - 权限验证                                        │  │
│  │  - 审计日志                                        │  │
│  │  - 速率限制                                        │  │
│  │  - 错误处理                                        │  │
│  └──────────────────────────────────────────────────┘  │
│                            ↓                              │
│  ┌──────────────────────────────────────────────────┐  │
│  │  数据存储安全层                                    │  │
│  │  - API Key 密钥链存储                             │  │
│  │  - 历史记录加密存储                                │  │
│  │  - 内存安全处理                                    │  │
│  │  - 安全删除                                        │  │
│  └──────────────────────────────────────────────────┘  │
│                            ↓                              │
│  ┌──────────────────────────────────────────────────┐  │
│  │  MCP 沙箱层                                        │  │
│  │  - 容器隔离                                        │  │
│  │  - 资源限制                                        │  │
│  │  - 权限控制                                        │  │
│  │  - 操作审计                                        │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

---

## 3. 核心模块设计

### 3.1 API Key 安全管理模块

#### 设计目标
- 使用系统密钥链安全存储 API Key
- 内存安全处理，防止泄露
- 访问审计日志
- 脱敏显示

#### 技术实现

**存储架构**：
```rust
// 使用 keyring 存储 API Key
use keyring::Entry;
use secrecy::{Secret, ExposeSecret, Zeroize};
use zeroize::Zeroizing;

#[derive(Debug, Clone)]
pub struct ApiKey {
    inner: Secret<String>,
    provider: String,
}

impl ApiKey {
    pub fn new(provider: String, key: String) -> Self {
        ApiKey {
            inner: Secret::new(key),
            provider,
        }
    }

    pub fn expose(&self) -> &str {
        self.inner.expose_secret()
    }

    pub fn mask(&self) -> String {
        let key = self.inner.expose_secret();
        if key.len() > 8 {
            format!("{}...{}", &key[..4], &key[key.len()-4..])
        } else {
            "****".to_string()
        }
    }
}

pub struct KeyringManager {
    service_name: String,
}

impl KeyringManager {
    pub fn new() -> Self {
        KeyringManager {
            service_name: "jedi-ai-chat".to_string(),
        }
    }

    pub fn store_api_key(&self, provider: &str, api_key: &str) -> Result<(), Box<dyn Error>> {
        let entry = Entry::new(&self.service_name, &format!("api-key-{}", provider))?;
        entry.set_password(api_key)?;
        
        // 记录审计日志
        log_security_event(SecurityEvent::new(
            "API_KEY_STORE",
            &format!("provider/{}", provider),
            "create",
            "success"
        ));
        
        Ok(())
    }

    pub fn get_api_key(&self, provider: &str) -> Result<Secret<String>, Box<dyn Error>> {
        let entry = Entry::new(&self.service_name, &format!("api-key-{}", provider))?;
        let password = entry.get_password()?;
        
        // 记录审计日志
        log_security_event(SecurityEvent::new(
            "API_KEY_ACCESS",
            &format!("provider/{}", provider),
            "read",
            "success"
        ));
        
        Ok(Secret::new(password))
    }

    pub fn delete_api_key(&self, provider: &str) -> Result<(), Box<dyn Error>> {
        let entry = Entry::new(&self.service_name, &format!("api-key-{}", provider))?;
        entry.delete_credential()?;
        
        // 记录审计日志
        log_security_event(SecurityEvent::new(
            "API_KEY_DELETE",
            &format!("provider/{}", provider),
            "delete",
            "success"
        ));
        
        Ok(())
    }
}
```

### 3.2 MCP 安全沙箱模块

#### 设计目标
- 容器化隔离 MCP 服务器
- 细粒度权限控制
- 资源限制
- 操作审计

#### 技术实现

**沙箱架构**：
```
┌─────────────────────────────────────────────────────────┐
│                  MCP 安全沙箱架构                          │
├─────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────┐  │
│  │  MCP 客户端（Jedi）                               │  │
│  │  - 协议验证                                       │  │
│  │  - 权限检查                                       │  │
│  │  - 审计日志                                       │  │
│  └──────────────────────────────────────────────────┘  │
│                            ↓                              │
│  ┌──────────────────────────────────────────────────┐  │
│  │  沙箱管理层                                        │  │
│  │  - Docker 容器管理                                 │  │
│  │  - 资源配额（CPU/内存/网络）                       │  │
│  │  - 文件系统隔离                                   │  │
│  │  - 网络策略控制                                   │  │
│  └──────────────────────────────────────────────────┘  │
│                            ↓                              │
│  ┌──────────────────────────────────────────────────┐  │
│  │  沙箱容器                                          │  │
│  │  ┌────────────────────────────────────────────┐  │  │
│  │  │  MCP 服务器进程                              │  │  │
│  │  │  - 非特权用户运行                            │  │  │
│  │  │  - 只读文件系统                              │  │  │
│  │  │  - 受限系统调用                              │  │  │
│  │  └────────────────────────────────────────────┘  │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

**权限模型**：
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPermission {
    pub resource: String,
    pub action: String,
    pub allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub id: String,
    pub name: String,
    pub permissions: Vec<McpPermission>,
    pub resource_limits: ResourceLimits,
    pub sandbox_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub cpu_quota: Option<i64>,    // CPU 配额（微秒）
    pub memory_limit: Option<i64>,  // 内存限制（字节）
    pub network_access: bool,       // 网络访问权限
    pub fs_readonly: bool,          // 文件系统只读
}

pub struct McpSandboxManager {
    config: McpServerConfig,
}

impl McpSandboxManager {
    pub fn new(config: McpServerConfig) -> Self {
        McpSandboxManager { config }
    }

    pub fn check_permission(&self, resource: &str, action: &str) -> bool {
        for perm in &self.config.permissions {
            if perm.resource == resource && perm.action == action {
                return perm.allowed;
            }
        }
        false // 默认拒绝
    }

    pub async fn execute_in_sandbox(&self, command: McpCommand) -> Result<McpResponse, Box<dyn Error>> {
        // 权限检查
        if !self.check_permission(&command.resource, &command.action) {
            log_security_event(SecurityEvent::new(
                "MCP_PERMISSION_DENIED",
                &format!("mcp/{}/{}", self.config.id, command.resource),
                &command.action,
                "denied"
            ));
            return Err("Permission denied".into());
        }

        // 记录操作审计
        log_security_event(SecurityEvent::new(
            "MCP_EXECUTE",
            &format!("mcp/{}/{}", self.config.id, command.resource),
            &command.action,
            "started"
        ));

        // 在沙箱中执行
        let result = self.spawn_sandbox_container(command).await?;

        log_security_event(SecurityEvent::new(
            "MCP_EXECUTE",
            &format!("mcp/{}/{}", self.config.id, command.resource),
            &command.action,
            "success"
        ));

        Ok(result)
    }
}
```

### 3.3 输入验证和输出编码模块

#### 设计目标
- 严格的用户输入验证
- 模型输出 HTML 编码
- XSS 防护
- Prompt Injection 检测

#### 技术实现

**前端安全渲染**：
```typescript
import markdownIt from 'markdown-it';
import DOMPurify from 'dompurify';

// 安全的 Markdown 渲染配置
const md = markdownIt({
  html: false, // 禁用 HTML 标签
  xhtmlOut: true,
  breaks: true,
  linkify: true,
  typographer: true
});

// 允许的标签和属性
const ALLOWED_TAGS = [
  'p', 'br', 'strong', 'em', 'code', 'pre', 
  'ul', 'ol', 'li', 'a', 'h1', 'h2', 'h3',
  'h4', 'h5', 'h6', 'blockquote', 'table',
  'thead', 'tbody', 'tr', 'th', 'td'
];

const ALLOWED_ATTR = ['href', 'title', 'class'];

export function renderMarkdownSafe(content: string): string {
  // 首先用 markdown-it 渲染
  const rendered = md.render(content);
  
  // 然后用 DOMPurify 清理
  return DOMPurify.sanitize(rendered, {
    ALLOWED_TAGS,
    ALLOWED_ATTR,
    ALLOW_DATA_ATTR: false,
    FORBID_TAGS: ['script', 'style', 'iframe', 'form'],
    FORBID_ATTR: ['onerror', 'onload', 'onclick', 'onmouseover']
  });
}

// 输入验证
export function validateUserInput(input: string): ValidationResult {
  const issues: string[] = [];
  
  // 检查长度
  if (input.length > 10000) {
    issues.push('输入过长，最多10000字符');
  }
  
  // 检查特殊字符
  const suspiciousPatterns = [
    /<script/i,
    /javascript:/i,
    /on\w+=/i,
    /data:text\/html/i,
  ];
  
  for (const pattern of suspiciousPatterns) {
    if (pattern.test(input)) {
      issues.push('检测到可疑内容');
      break;
    }
  }
  
  return {
    valid: issues.length === 0,
    issues
  };
}

interface ValidationResult {
  valid: boolean;
  issues: string[];
}
```

### 3.4 网络通信安全模块

#### 设计目标
- 强制 HTTPS
- 证书固定
- 请求超时和重试
- 响应验证

#### 技术实现

**安全 HTTP 客户端**：
```rust
use reqwest::{Client, certificate::Certificate};
use std::fs;
use std::time::Duration;

pub struct SecureHttpClient {
    client: Client,
}

impl SecureHttpClient {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        // 配置 TLS
        let mut tls_config = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_webpki_roots()
            .with_no_client_auth();

        // 证书固定（示例）
        // tls_config.root_store.add(&cert)?;

        let client = Client::builder()
            .use_rustls_tls()
            .https_only(true)  // 强制 HTTPS
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .tcp_keepalive(Duration::from_secs(60))
            .no_gzip()  // 防止 CRIME/BREACH 攻击
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(5)
            .build()?;

        Ok(SecureHttpClient { client })
    }

    pub async fn post(&self, url: &str, body: serde_json::Value) -> Result<serde_json::Value, Box<dyn Error>> {
        let response = self.client
            .post(url)
            .json(&body)
            .send()
            .await?;

        // 检查状态码
        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()).into());
        }

        // 验证响应内容类型
        let content_type = response.headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if !content_type.contains("application/json") {
            return Err("Invalid content type".into());
        }

        let json: serde_json::Value = response.json().await?;

        Ok(json)
    }
}
```

### 3.5 会话历史加密存储模块

#### 设计目标
- 加密存储聊天历史
- 用户密码派生密钥
- 安全的加密算法
- 访问控制

#### 技术实现

**加密存储**：
```rust
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce
};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use base64::{engine::general_purpose, Engine as _};
use zeroize::Zeroizing;

pub struct EncryptedStorage {
    cipher: Aes256Gcm,
}

impl EncryptedStorage {
    pub fn new_from_password(password: &str, salt: &[u8]) -> Result<Self, Box<dyn Error>> {
        // 使用 Argon2 派生密钥
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), salt)?;
        
        // 提取密钥（简化示例，实际应使用 KDF）
        let mut key = [0u8; 32];
        key.copy_from_slice(&password_hash.hash.unwrap().as_bytes()[..32]);
        
        let cipher = Aes256Gcm::new(&key.into());
        
        Ok(EncryptedStorage { cipher })
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<String, Box<dyn Error>> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self.cipher.encrypt(&nonce, plaintext.as_bytes())?;
        
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);
        
        Ok(general_purpose::STANDARD.encode(result))
    }

    pub fn decrypt(&self, ciphertext: &str) -> Result<String, Box<dyn Error>> {
        let data = general_purpose::STANDARD.decode(ciphertext)?;
        
        let (nonce_bytes, ciphertext_bytes) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes)?;
        
        let plaintext = self.cipher.decrypt(nonce, ciphertext_bytes)?;
        
        Ok(String::from_utf8(plaintext)?)
    }
}

pub struct ChatHistoryManager {
    storage: EncryptedStorage,
    store_path: PathBuf,
}

impl ChatHistoryManager {
    pub fn new(password: &str, store_path: PathBuf) -> Result<Self, Box<dyn Error>> {
        // 生成或加载 salt
        let salt_path = store_path.with_extension("salt");
        let salt = if salt_path.exists() {
            fs::read(&salt_path)?
        } else {
            let mut s = [0u8; 16];
            OsRng.fill_bytes(&mut s);
            fs::write(&salt_path, &s)?;
            s.to_vec()
        };

        let storage = EncryptedStorage::new_from_password(password, &salt)?;

        Ok(ChatHistoryManager {
            storage,
            store_path,
        })
    }

    pub fn save_history(&self, history: &ChatHistory) -> Result<(), Box<dyn Error>> {
        let plaintext = serde_json::to_string(history)?;
        let ciphertext = self.storage.encrypt(&plaintext)?;
        fs::write(&self.store_path, ciphertext)?;
        
        log_security_event(SecurityEvent::new(
            "CHAT_HISTORY_SAVE",
            "chat/history",
            "write",
            "success"
        ));
        
        Ok(())
    }

    pub fn load_history(&self) -> Result<ChatHistory, Box<dyn Error>> {
        if !self.store_path.exists() {
            return Ok(ChatHistory::default());
        }

        let ciphertext = fs::read_to_string(&self.store_path)?;
        let plaintext = self.storage.decrypt(&ciphertext)?;
        let history: ChatHistory = serde_json::from_str(&plaintext)?;
        
        log_security_event(SecurityEvent::new(
            "CHAT_HISTORY_LOAD",
            "chat/history",
            "read",
            "success"
        ));
        
        Ok(history)
    }
}
```

### 3.6 安全审计日志模块

#### 设计目标
- 记录所有敏感操作
- 防篡改日志
- 结构化日志格式
- 日志保留策略

#### 技术实现

**审计日志系统**：
```rust
use serde::Serialize;
use chrono::Utc;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
pub struct SecurityEvent {
    pub timestamp: String,
    pub event_type: String,
    pub user_id: Option<String>,
    pub resource: String,
    pub action: String,
    pub result: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub session_id: Option<String>,
}

impl SecurityEvent {
    pub fn new(event_type: &str, resource: &str, action: &str, result: &str) -> Self {
        SecurityEvent {
            timestamp: Utc::now().to_rfc3339(),
            event_type: event_type.to_string(),
            user_id: None,
            resource: resource.to_string(),
            action: action.to_string(),
            result: result.to_string(),
            ip_address: None,
            user_agent: None,
            session_id: None,
        }
    }
}

pub struct AuditLogger {
    log_path: PathBuf,
}

impl AuditLogger {
    pub fn new(log_path: PathBuf) -> Self {
        AuditLogger { log_path }
    }

    pub fn log(&self, event: SecurityEvent) -> Result<(), Box<dyn Error>> {
        // 序列化为 JSON
        let mut json = serde_json::to_string(&event)?;
        json.push('\n');

        // 追加写入文件
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;

        file.write_all(json.as_bytes())?;
        file.flush()?;

        // 同时输出到 tracing
        tracing::info!(
            target: "security_audit",
            event_type = %event.event_type,
            resource = %event.resource,
            action = %event.action,
            result = %event.result,
        );

        Ok(())
    }

    pub fn query(&self, event_type: Option<&str>, start_time: Option<&str>) -> Result<Vec<SecurityEvent>, Box<dyn Error>> {
        // 简化的查询实现
        let content = std::fs::read_to_string(&self.log_path)?;
        let mut events = Vec::new();

        for line in content.lines() {
            let event: SecurityEvent = serde_json::from_str(line)?;
            
            let mut include = true;
            if let Some(et) = event_type {
                if event.event_type != et {
                    include = false;
                }
            }
            if let Some(st) = start_time {
                if event.timestamp < st {
                    include = false;
                }
            }

            if include {
                events.push(event);
            }
        }

        Ok(events)
    }
}

// 全局日志记录函数
pub fn log_security_event(event: SecurityEvent) {
    // 在实际应用中，使用单例或依赖注入获取 logger
    tracing::info!(
        target: "security_audit",
        event = %serde_json::to_string(&event).unwrap(),
    );
}
```

---

## 4. Tauri 安全配置

### 4.1 CSP 策略配置

```json
{
  "app": {
    "security": {
      "csp": {
        "default-src": "'self'",
        "script-src": "'self'",
        "style-src": "'self' 'unsafe-inline'",
        "img-src": "'self' data: https:",
        "connect-src": "'self' https://api.openai.com https://api.anthropic.com",
        "font-src": "'self'",
        "object-src": "'none'",
        "frame-src": "'none'",
        "form-action": "'none'",
        "base-uri": "'self'"
      }
    }
  }
}
```

### 4.2 权限最小化配置

```json
{
  "plugins": {
    "fs": {
      "scope": [
        "$DOCUMENT/jedi/ai-chat/*",
        "$CONFIG/jedi/*",
        "!$DOCUMENT/jedi/ai-chat/sensitive/*"
      ]
    },
    "shell": {
      "open": false,
      "scope": []
    },
    "process": {
      "scope": []
    },
    "clipboard": {
      "read": true,
      "write": true
    }
  }
}
```

---

## 5. 架构决策记录 (ADR)

### ADR-001: API Key 存储方案

**日期**：2026-03-05  
**状态**：已批准  
**上下文**：需要安全存储用户的 API Key，防止泄露  
**决策**：使用系统密钥链（Keychain/Keyring）存储 API Key  
**理由**：
- 系统密钥链提供了操作系统级别的安全保护
- 不需要管理加密密钥，降低复杂度
- 用户体验更好，不需要记住额外密码
- 符合安全最佳实践

**后果**：
- 需要跨平台适配不同系统的密钥链 API
- 依赖 `keyring` crate
- 备份和迁移需要系统特定的工具

---

### ADR-002: MCP 沙箱方案

**日期**：2026-03-05  
**状态**：已批准  
**上下文**：MCP 服务器可能执行敏感操作，需要隔离保护  
**决策**：使用 Docker 容器作为 MCP 服务器的沙箱环境  
**理由**：
- Docker 提供成熟的容器化隔离
- 支持资源限制（CPU、内存、网络）
- 广泛使用，社区支持好
- 可以配置只读文件系统和网络策略

**后果**：
- 用户需要安装 Docker
- 增加了系统资源消耗
- 需要管理容器生命周期
- Windows 和 macOS 上 Docker Desktop 需要额外安装

---

### ADR-003: 会话历史加密方案

**日期**：2026-03-05  
**状态**：已批准  
**上下文**：聊天历史可能包含敏感信息，需要加密存储  
**决策**：使用用户密码派生的密钥加密会话历史，使用 AES-256-GCM 算法  
**理由**：
- AES-256-GCM 提供认证加密，保证完整性
- Argon2 是推荐的密码哈希算法
- 不需要管理密钥存储
- 用户可以通过密码恢复数据

**后果**：
- 用户需要记住密码
- 密码丢失会导致数据无法恢复
- 需要安全的密码输入 UI
- 需要处理密码变更场景

---

### ADR-004: 网络通信安全方案

**日期**：2026-03-05  
**状态**：已批准  
**上下文**：需要确保与 AI 提供商 API 通信的安全性  
**决策**：强制 HTTPS，配置合理的超时和重试策略，暂不实施证书固定（MVP 阶段）  
**理由**：
- HTTPS 是基础安全要求
- 证书固定增加了维护成本（证书更新时需要发布新版本）
- MVP 阶段先实现基础安全
- 后续版本可以根据需要添加证书固定

**后果**：
- 依赖系统 CA 证书存储
- 存在理论上的中间人攻击风险（需要攻破系统 CA）
- 实现相对简单

---

### ADR-005: 错误处理方案

**日期**：2026-03-05  
**状态**：已批准  
**上下文**：需要防止错误信息泄露敏感数据  
**决策**：区分内部错误和用户可见错误，详细错误记录到日志，只向用户显示通用信息  
**理由**：
- 防止信息泄露
- 便于调试（详细日志）
- 用户体验友好（清晰的错误提示）

**后果**：
- 需要设计统一的错误类型
- 需要实现错误转换逻辑
- 日志需要包含足够的调试信息

---

## 6. 实施计划

### 6.1 阶段一：MVP 安全核心（发布前必须完成）

**时间**：2 周  
**目标**：实现所有高严重程度的安全修复

**任务清单**：
- [ ] 集成 `keyring` crate，实现 API Key 密钥链存储
- [ ] 集成 `secrecy` 和 `zeroize` crate，实现内存安全处理
- [ ] 实现 API Key 脱敏显示
- [ ] 实现安全审计日志系统
- [ ] 配置 Tauri CSP 策略
- [ ] 配置 Tauri 最小权限
- [ ] 实现前端输入验证
- [ ] 实现前端输出编码和 XSS 防护
- [ ] 实现安全的错误处理
- [ ] 配置强制 HTTPS 和超时设置
- [ ] 编写安全单元测试

**验收标准**：
- API Key 不再明文存储
- CSP 策略生效
- 输入验证正常工作
- 安全审计日志正常记录

### 6.2 阶段二：MCP 沙箱和高级安全（发布后 1 个月内）

**时间**：3 周  
**目标**：实现中严重程度的安全修复

**任务清单**：
- [ ] 实现 MCP Docker 沙箱基础架构
- [ ] 实现 MCP 权限控制模型
- [ ] 实现 MCP 操作审计
- [ ] 实现会话历史加密存储
- [ ] 集成 SCA 工具（cargo-audit, npm audit）
- [ ] 配置 CI/CD 安全扫描
- [ ] 实现证书固定（可选）
- [ ] 实现 API Key 内存安全处理（mlock 等）
- [ ] 编写 MCP 安全测试

**验收标准**：
- MCP 服务器在容器中运行
- 会话历史加密存储
- 依赖安全扫描通过 CI

### 6.3 阶段三：安全增强和完善（后续版本）

**时间**：持续  
**目标**：完善安全功能，提升安全 maturity

**任务清单**：
- [ ] 实现速率限制和使用量监控
- [ ] 实现可选的应用密码保护
- [ ] 实现敏感操作二次确认
- [ ] 实现安全配置向导
- [ ] 实现自动安全检查提醒
- [ ] 集成生物识别支持（Touch ID/Face ID）
- [ ] 进行渗透测试
- [ ] 实现日志防篡改
- [ ] 实现安全事件告警

---

## 7. 安全测试计划

### 7.1 测试范围

| 测试类别 | 测试项 | 验收标准 |
|---------|--------|----------|
| API Key 安全 | 存储安全 | 使用系统密钥链 |
| | 内存安全 | 内存中无明文残留 |
| | 显示安全 | 脱敏显示 |
| | 访问审计 | 完整日志记录 |
| MCP 安全 | 沙箱隔离 | 无法访问沙箱外资源 |
| | 权限控制 | 未授权操作被阻止 |
| | 输入验证 | 恶意输入被正确处理 |
| 网络通信 | HTTPS 强制 | 自动重定向到 HTTPS |
| | 证书验证 | 自签名证书被拒绝 |
| | 超时处理 | 请求正确超时 |
| 输入验证 | XSS 防护 | 脚本不执行 |
| | Prompt Injection | 检测或缓解成功 |
| | 输出编码 | 内容正确编码 |
| 权限控制 | 文件系统权限 | 未授权访问被拒绝 |
| | Tauri 权限 | 符合最小权限原则 |
| 依赖安全 | 已知漏洞 | 无高危漏洞 |
| | 依赖完整性 | 依赖版本一致 |

### 7.2 测试工具

- **SAST**：Clippy, rust-clippy, ESLint
- **SCA**：cargo-audit, npm audit, Trivy
- **渗透测试**：OWASP ZAP
- **模糊测试**：cargo-fuzz
- **内存分析**：Volatility, gdb

---

## 8. 总结

本文档详细描述了 Jedi AI Chat 模块的技术架构，特别强调了安全设计。通过分层的安全架构、密钥链存储、MCP 沙箱隔离、输入验证、加密存储等措施，全面应对安全评审报告中发现的风险。

实施将分为三个阶段进行，确保在发布前完成所有高严重程度的安全修复，为用户提供一个安全可靠的 AI Chat 体验。

---

**文档版本**：v2.0  
**编写日期**：2026-03-05  
**架构师**：架构师智能体  
**更新内容**：整合安全加固措施，新增 5 个 ADR，更新实施计划
