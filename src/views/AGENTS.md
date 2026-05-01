<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-04-18 | Updated: 2026-04-18 -->

# views (页面组件)

## Purpose
存放应用的所有页面级组件，对应路由配置中的各个页面。

## Subdirectories
| Directory | Purpose |
|-----------|---------|
| `AiChat/` | AI聊天功能页面 (see `AiChat/AGENTS.md`) |
| `hosts/` | Hosts管理功能页面 (see `hosts/AGENTS.md`) |
| `podcast/` | 播客客户端页面 (see `podcast/AGENTS.md`) |
| `wallpapers/` | 知识壁纸功能页面 (see `wallpapers/AGENTS.md`) |

## For AI Agents

### Working In This Directory
- 每个页面对应路由中的一个路径
- 页面组件负责组合业务逻辑和子组件
- 页面状态优先使用对应store管理，避免页面组件过于臃肿
- 页面间通信使用事件总线或者全局状态

### Common Patterns
- 页面组件使用 `definePageMeta` 定义路由元信息
- 页面加载状态统一管理
- 页面级的键盘快捷键在页面组件中注册
- 页面销毁时需要清理定时器、事件监听等资源

<!-- MANUAL: -->
