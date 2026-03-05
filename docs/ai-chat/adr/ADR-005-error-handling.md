# ADR-005: 错误处理方案

**日期**：2026-03-05  
**状态**：已批准  
**架构师**：架构师智能体

## 上下文

错误处理不当可能导致：
- 敏感信息泄露（文件路径、堆栈跟踪、API Key 部分内容）
- 用户体验差（模糊或技术性错误信息）
- 调试困难（缺少详细错误日志）

## 决策

区分内部错误和用户可见错误，详细错误记录到日志，只向用户显示通用信息。

## 备选方案

### 方案 1：详细错误显示给用户（否决）
- **优点**：调试方便
- **缺点**：信息泄露风险高

### 方案 2：所有错误都显示通用信息（否决）
- **优点**：安全
- **缺点**：用户体验差，不知道如何解决问题

### 方案 3：分离内部/外部错误（选中）
- **优点**：平衡安全和用户体验，便于调试
- **缺点**：需要额外的错误类型设计

## 理由

1. **安全**：防止敏感信息泄露给用户界面
2. **可调试性**：详细日志帮助开发和运维排查问题
3. **用户体验**：清晰、友好的错误提示
4. **可审计**：错误日志作为安全审计的一部分

## 后果

### 正面
- 良好的安全性
- 优秀的可调试性
- 友好的用户体验

### 负面
- 需要设计统一的错误类型
- 需要实现错误转换逻辑
- 日志需要妥善管理（防止日志过大）

## 实现细节

### 错误类型设计

```rust
// 内部错误（详细）
#[derive(Debug, thiserror::Error)]
pub enum InternalError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("API call failed: status={status}, body={body}")]
    ApiError {
        status: u16,
        body: String,
    },
    
    #[error("Keyring error: {0}")]
    Keyring(#[from] keyring::Error),
    
    // ... 更多内部错误
}

// 用户可见错误（通用）
#[derive(Debug, thiserror::Error)]
pub enum UserFacingError {
    #[error("配置加载失败，请检查设置")]
    ConfigLoadError,
    
    #[error("API 调用失败，请检查网络连接和 API Key")]
    ApiCallError,
    
    #[error("认证失败，请验证您的凭证")]
    AuthenticationError,
    
    #[error("内部错误，请稍后重试")]
    InternalError,
}

// 安全转换
impl From<InternalError> for UserFacingError {
    fn from(err: InternalError) -> Self {
        // 记录详细的内部错误
        tracing::error!(internal_error = ?err, "Internal error occurred");
        
        // 映射到用户可见错误
        match err {
            InternalError::Io(_) => UserFacingError::ConfigLoadError,
            InternalError::ApiError { .. } => UserFacingError::ApiCallError,
            InternalError::Keyring(_) => UserFacingError::AuthenticationError,
            _ => UserFacingError::InternalError,
        }
    }
}
```

### 日志策略

- **ERROR**：所有内部错误
- **WARN**：可恢复的问题
- **INFO**：安全审计事件
- **DEBUG**：详细的调试信息
- **TRACE**：最详细的跟踪

### 日志脱敏

```rust
// 自动脱敏 API Key
fn sanitize_log_message(msg: &str) -> String {
    // 替换 API Key 模式
    let re = regex::Regex::new(r"(sk-[a-zA-Z0-9]{20,})").unwrap();
    re.replace_all(msg, "sk-...").to_string()
}
```

## 相关决策

- ADR-001: API Key 存储方案
- ADR-006: 审计日志方案（后续）

## 参考资料

- OWASP Error Handling Cheat Sheet
- Rust Error Handling Best Practices
- Tracing crate 文档
