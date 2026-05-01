<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-04-18 | Updated: 2026-04-18 -->

# components (Vue组件库)

## Purpose
存放所有可复用的Vue组件，按功能领域分类组织。

## Key Files
| File | Description |
|------|-------------|
| `CommandPalette.vue` | 命令面板组件，支持快捷键触发搜索功能 |
| `AttachmentMenu.vue` | 附件菜单组件，支持文件上传、技能调用等 |
| `UpdateDialog.vue` | 应用更新提示对话框 |

## Subdirectories
| Directory | Purpose |
|-----------|---------|
| `agent/` | AI代理相关组件 (see `agent/AGENTS.md`) |
| `common/` | 通用基础组件 (see `common/AGENTS.md`) |
| `dialogs/` | 对话框组件 (see `dialogs/AGENTS.md`) |
| `hosts/` | Hosts管理功能组件 (see `hosts/AGENTS.md`) |
| `layout/` | 布局组件 (see `layout/AGENTS.md`) |
| `podcast/` | 播客功能组件 (see `podcast/AGENTS.md`) |

## For AI Agents

### Working In This Directory
- 组件名使用PascalCase，文件名与组件名一致
- 通用组件放在 `common/`，业务组件放在对应功能目录
- 组件 props 需要定义TypeScript类型
- 组件事件使用 `defineEmits` 显式声明
- 可复用组件需要支持主题切换适配

### Common Patterns
- 对话框组件使用 `v-model` 控制显示状态
- 列表组件使用插槽自定义渲染内容
- 表单组件支持 `v-model` 双向绑定
- 组件样式使用 scoped 避免污染全局

<!-- MANUAL: -->
