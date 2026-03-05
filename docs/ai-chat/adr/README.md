# 架构决策记录 (ADR)

本目录包含 Jedi AI Chat 模块的架构决策记录。

## 什么是 ADR？

架构决策记录（Architecture Decision Record，ADR）是一种记录软件架构决策的方法。每个 ADR 描述一个单一的架构决策，包括：
- 决策的上下文和背景
- 考虑的备选方案
- 最终决策及其理由
- 决策的后果（正面和负面）

## ADR 列表

| 编号 | 标题 | 状态 | 日期 |
|------|------|------|------|
| [ADR-001](ADR-001-api-key-storage.md) | API Key 存储方案 | 已批准 | 2026-03-05 |
| [ADR-002](ADR-002-mcp-sandbox.md) | MCP 沙箱方案 | 已批准 | 2026-03-05 |
| [ADR-003](ADR-003-chat-history-encryption.md) | 会话历史加密方案 | 已批准 | 2026-03-05 |
| [ADR-004](ADR-004-network-security.md) | 网络通信安全方案 | 已批准 | 2026-03-05 |
| [ADR-005](ADR-005-error-handling.md) | 错误处理方案 | 已批准 | 2026-03-05 |

## 如何使用 ADR

### 阅读 ADR
- 新加入团队的成员可以通过阅读 ADR 快速了解技术选型的理由
- 当需要修改架构时，先查看相关 ADR 了解历史决策

### 新增 ADR
当需要做出新的架构决策时：
1. 复制 `TEMPLATE.md` 为新文件（如果有模板）
2. 编号为下一个可用数字
3. 填写所有必要信息
4. 更新此索引文件
5. 提交到版本控制

### 更新 ADR
- ADR 状态可以变化（草案 → 已批准 → 已弃用）
- 不要删除已做出的决策，即使后来改变了
- 如果决策变更，新增一个 ADR 记录新决策，并在旧 ADR 中引用新 ADR

## ADR 模板（建议）

```markdown
# ADR-XXX: [决策标题]

**日期**：YYYY-MM-DD  
**状态**：[草案 | 已批准 | 已弃用 | 已替换]  
**架构师**：[姓名]

## 上下文

[描述做出此决策的背景和原因]

## 决策

[明确陈述最终决策]

## 备选方案

### 方案 1：[名称]
- **优点**：
- **缺点**：

### 方案 2：[名称]
- **优点**：
- **缺点**：

## 理由

[解释为什么选择这个决策，而不是其他备选方案]

## 后果

### 正面
- [正面后果 1]
- [正面后果 2]

### 负面
- [负面后果 1]
- [负面后果 2]

## 实现细节

[可选：技术实现细节、代码结构、依赖等]

## 相关决策

- [ADR-XXX：相关决策标题](ADR-XXX.md)

## 参考资料

- [参考链接 1](URL)
- [参考链接 2](URL)
```

## 相关资源

- [Markdown ADR](https://github.com/joelparkerhenderson/architecture-decision-record)
- [Documenting Architecture Decisions](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions.html)
- [ADR GitHub](https://adr.github.io/)
