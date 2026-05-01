<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-04-18 | Updated: 2026-04-18 -->

# docs (项目文档)

## Purpose
存放项目的所有文档、架构设计、决策记录和功能说明。

## Subdirectories
| Directory | Purpose |
|-----------|---------|
| `ai-chat/` | AI聊天功能相关文档 (see `ai-chat/AGENTS.md`) |
| `superpowers/` | 超级能力体系文档 (see `superpowers/AGENTS.md`) |

## For AI Agents

### Working In This Directory
- 文档使用Markdown格式，图片放在对应目录的images子目录
- 架构决策记录(ADR)放在对应功能目录的adr子目录
- 中文文档优先，重要文档提供中英文版本
- 文档需要和代码同步更新，避免过时

## Dependencies

### Internal
- 文档内容对应 `src/` 和 `src-tauri/` 中的代码实现

<!-- MANUAL: -->
