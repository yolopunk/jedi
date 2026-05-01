<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-04-18 | Updated: 2026-04-18 -->

# src (前端源代码)

## Purpose
Jedi应用的前端Vue代码，包含所有UI界面、业务逻辑、状态管理和前端工具功能实现。

## Key Files
| File | Description |
|------|-------------|
| `App.vue` | 根组件，包含主布局结构 |
| `main.ts` | 应用入口，初始化Vue、插件和全局配置 |
| `vue-router` | 路由配置，管理页面导航 |

## Subdirectories
| Directory | Purpose |
|-----------|---------|
| `agent/` | AI代理相关逻辑 (see `agent/AGENTS.md`) |
| `api/` | 前后端通信API封装 |
| `assets/` | 样式、字体、图片等静态资源 |
| `components/` | 可复用Vue组件 (see `components/AGENTS.md`) |
| `composables/` | Vue组合式函数，逻辑复用 |
| `i18n/` | 多语言国际化配置 |
| `mcp/` | Model Context Protocol相关实现 |
| `plugins/` | Vue插件初始化 |
| `router/` | 路由配置 |
| `skills/` | 可执行技能实现 |
| `stores/` | 状态管理store |
| `types/` | TypeScript类型定义 |
| `utils/` | 通用工具函数 |
| `views/` | 页面级组件 (see `views/AGENTS.md`) |

## For AI Agents

### Working In This Directory
- 组件命名使用PascalCase，文件命名使用kebab-case
- 优先使用Composition API，避免Options API
- 所有API调用需要通过 `@/api/` 目录下的封装，不要直接调用invoke
- 多语言文本需要在 `i18n/locales/` 中配置，不要硬编码中文/英文

### Testing Requirements
- 组件开发需要测试主题切换适配（深色/浅色模式）
- 需要测试响应式布局，支持不同窗口大小
- 功能修改需要验证快捷键、上下文菜单等交互正常

### Common Patterns
- 页面组件放在 `views/`，可复用组件放在 `components/`
- 通用逻辑抽离为composables，放在 `composables/` 目录
- 状态使用Pinia stores，定义在 `stores/` 目录
- API层统一处理错误和loading状态

## Dependencies

### Internal
- `@/components/` - 可复用UI组件
- `@/composables/` - 共享逻辑
- `@/api/` - 后端接口调用
- `@/stores/` - 全局状态

### External
- Vuetify 3.x - UI组件库
- Vue Router 4.x - 路由管理
- Pinia - 状态管理
- `@tauri-apps/api` - Tauri前端API

<!-- MANUAL: -->
