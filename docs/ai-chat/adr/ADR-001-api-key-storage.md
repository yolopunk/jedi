# ADR-001: API Key 存储方案

**日期**：2026-03-05  
**状态**：已批准  
**架构师**：架构师智能体

## 上下文

Jedi AI Chat 模块需要安全存储用户的 API Key（OpenAI、Anthropic 等）。这些 API Key 是敏感信息，如果泄露可能导致：
- 经济损失（滥用 API 产生费用）
- 隐私泄露（通过 API 访问历史数据）
- 账户盗用

## 决策

使用系统密钥链（Keychain/Keyring）存储 API Key。

## 备选方案

### 方案 1：使用 tauri-plugin-store 加密存储
- **优点**：实现简单，与现有架构一致
- **缺点**：需要管理加密密钥，密钥存储本身又是一个问题

### 方案 2：使用用户密码派生密钥加密
- **优点**：用户控制密钥
- **缺点**：用户需要记住额外密码，忘记密码会导致数据丢失

### 方案 3：系统密钥链（选中）
- **优点**：操作系统级别的安全保护，不需要管理密钥，用户体验好
- **缺点**：跨平台实现需要适配，备份迁移需要系统工具

## 理由

1. **安全性**：系统密钥链由操作系统安全机制保护（macOS Keychain、Windows Credential Locker、Linux Secret Service）
2. **用户体验**：用户不需要记住额外密码
3. **最佳实践**：这是业界存储敏感凭证的推荐方式
4. **审计友好**：系统密钥链通常自带访问审计

## 后果

### 正面
- 高安全性
- 良好的用户体验
- 符合安全最佳实践

### 负面
- 需要跨平台适配（使用 `keyring` crate）
- 备份和迁移需要系统特定工具
- 依赖系统密钥链服务可用

## 实现细节

### 依赖
- `keyring` crate：跨平台密钥链访问
- `secrecy` crate：内存安全处理
- `zeroize` crate：内存清零

### 代码结构
```
src-tauri/src/security/
├── keyring.rs      # 密钥链管理
├── api_key.rs      # API Key 类型
└── mod.rs
```

## 相关决策

- ADR-003: 会话历史加密方案
- ADR-005: 错误处理方案

## 参考资料

- OWASP Password Storage Cheat Sheet
- Tauri Security Guide
- keyring crate 文档
