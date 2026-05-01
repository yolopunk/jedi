<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-04-18 | Updated: 2026-04-18 -->

# src-tauri (Tauri后端)

## Purpose
Jedi应用的Rust后端实现，处理系统级操作、原生API调用、性能敏感任务和跨平台功能。

## Key Files
| File | Description |
|------|-------------|
| `Cargo.toml` | Rust依赖管理和项目配置 |
| `Cargo.lock` | Rust依赖锁定文件 |
| `tauri.conf.json` | Tauri应用配置（权限、窗口、更新等） |
| `src/main.rs` | 后端主入口，初始化Tauri应用和注册命令 |
| `rustfmt.toml` | Rust代码格式化配置 |

## Subdirectories
| Directory | Purpose |
|-----------|---------|
| `capabilities/` | Tauri能力配置，定义前端可调用的权限 |
| `gen/` | 自动生成的代码和schema定义 |
| `icons/` | 应用图标资源，各平台格式 |
| `src/` | Rust源代码 (see `src/AGENTS.md`) |
| `target/` | Rust编译输出目录（自动生成） |

## For AI Agents

### Working In This Directory
- 使用 `cargo fmt` 格式化Rust代码
- 使用 `cargo clippy` 进行代码检查
- 添加新的Tauri命令需要在 `main.rs` 中注册
- 跨平台代码需要使用条件编译，避免平台特定API编译错误
- 敏感操作需要添加权限检查，遵循最小权限原则

### Testing Requirements
- 后端功能需要通过单元测试覆盖：`cargo test`
- 跨平台功能需要在Windows/macOS/Linux上分别测试
- 系统级操作（如Hosts文件修改）需要测试权限处理逻辑

### Common Patterns
- API命令定义在 `src/api/` 目录，按功能模块划分
- 错误处理使用 `anyhow` 和 `thiserror` 库
- 配置管理使用 `config` 库，支持多环境
- 异步任务使用 `tokio` 运行时

## Dependencies

### Internal
- 后端暴露的API通过Tauri invoke供前端调用
- 配置和状态通过Tauri Store持久化

### External
- Tauri 2.x - 桌面应用框架
- tokio - 异步运行时
- reqwest - HTTP客户端
- sysinfo - 系统信息采集
- anyhow/thiserror - 错误处理
- wallpaper - 系统壁纸设置
- tauri-plugin-store - 本地存储
- tauri-plugin-updater - 自动更新

<!-- MANUAL: -->
