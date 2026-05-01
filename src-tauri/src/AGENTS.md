<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-04-18 | Updated: 2026-04-18 -->

# src (Rust后端源码)

## Purpose
Tauri后端的所有Rust源代码，实现所有原生功能和业务逻辑。

## Key Files
| File | Description |
|------|-------------|
| `main.rs` | 应用入口，初始化Tauri、注册命令和插件 |

## Subdirectories
| Directory | Purpose |
|-----------|---------|
| `agent/` | AI代理后端逻辑 |
| `api/` | 向前端暴露的API实现 (see `api/AGENTS.md`) |
| `config/` | 应用配置管理 |
| `mcp/` | Model Context Protocol服务端实现 |
| `utils/` | 通用工具函数 (see `utils/AGENTS.md`) |

## For AI Agents

### Working In This Directory
- 遵循Rust官方代码风格，使用`cargo fmt`格式化
- 所有对外暴露的Tauri命令需要做参数校验
- 错误处理需要友好，返回给前端的错误信息需要本地化
- 性能敏感的操作需要做异步处理，避免阻塞UI
- 跨平台代码使用`#[cfg(target_os = "xxx")]`条件编译

### Common Patterns
- API命令使用`#[tauri::command]`宏标记
- 错误使用`thiserror`定义错误类型，`anyhow`处理上下文
- 配置使用`serde`序列化/反序列化
- 异步任务使用`tokio::spawn`执行

## Dependencies

### External
- tauri - 桌面应用框架
- serde - 序列化框架
- tokio - 异步运行时
- anyhow/thiserror - 错误处理
- sysinfo - 系统信息采集

<!-- MANUAL: -->
