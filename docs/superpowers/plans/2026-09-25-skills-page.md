# 迭代 4：/skills 独立页（含设计债偿还）实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development.
> Track progress with the checkboxes below — mark each `- [ ]` as `- [x]` when complete.

**Goal:** 独立 `/skills` 管理页（搜索/筛选/排序/编辑器），同时偿还 UI/UX 审查发现的 P0/P1 设计债（主题令牌、i18n、z-index）并立 a11y 基线。
**Architecture:** 先还债 → 抽共享组件（SkillRow/SkillEditor，令牌化 + a11y 从诞生起）→ 页面复用组件；聊天页对话框保留为轻量入口。
**Tech Stack:** Vue 3 + Pinia + theme.css 语义令牌；Rust 托盘菜单扩展。

---

## 设计决策

| 决策 | 理由 |
|---|---|
| 顺序：还债 → 组件 → 页面 | 避免新页面复制旧债；共享组件从诞生起即令牌化 |
| 页面平铺 + 筛选 chips（不做分组） | 页面信息密度优先；对话框保留分组形态两者互补 |
| 两段式确认抽 `useTwoStepConfirm` | 删除/清零共用，3 秒自动复位（P3） |
| 确认卡"批准并始终允许"降为描边样式 | 主操作视觉权重不倒挂（P3） |
| 统计徽章 9px → 10.5px | 可读性下限（P2） |
| z-index 令牌化（scrim/panel/confirm/toast） | 结束三 overlay 同层裸奔（P1） |

## 任务分解（已完成）

### Task 1: 设计债偿还
- [x] theme.css 增加 `--z-scrim/panel/confirm/toast` 层级令牌
- [x] McpServersDialog / ToolConfirmCard 全量令牌迁移（硬编码色 → 语义令牌；`--mono-font` 幽灵变量统一为 `--jedi-font-mono`）
- [x] McpServersDialog + AttachmentMenu 标题 i18n（新增 `mcp.*` 节、`skills.menuTitle`）
- [x] P3：确认卡按钮权重对调、徽章字号、两段式确认自动复位
- [x] 提交：`fix(ui): migrate mcp/confirm dialogs to theme tokens, z-index scale, i18n`

### Task 2: 共享组件抽取（含 a11y）
- [x] `src/composables/useTwoStepConfirm.ts`
- [x] `src/components/skills/SkillRow.vue`（开关 role="switch"/aria-checked、编辑删除 aria-label）
- [x] `src/components/skills/SkillEditor.vue`
- [x] SkillsManagerDialog 瘦身为容器（样式全令牌化）
- [x] 提交：`refactor(skill): extract SkillRow and SkillEditor shared components`

### Task 3: 确认卡 a11y
- [x] Esc 拒绝 / Enter 批准 / 出现时焦点移入批准按钮；role="dialog" aria-modal
- [x] 提交：`feat(a11y): confirm-card keyboard support and dialog semantics`

### Task 4: /skills 页面
- [x] `src/views/skills/SkillsManager.vue`：console 工具栏（搜索 + 来源 chips + 排序 + 新建 + 清零）+ 平铺 SkillRow 列表 + 内嵌编辑器
- [x] 路由 `/skills`、侧边栏 navItems（mdiToolboxOutline）、托盘 `nav_skills`
- [x] i18n：`sidebar.skills`、`skills.page.*`
- [x] 提交：`feat(skill): standalone /skills manager page`

## 验证清单

- [x] `pnpm lint`（0 错误）、`pnpm exec vue-tsc --noEmit`、`pnpm build`、`cargo check` 通过
- [ ] 浅色/深色主题下三个对话框与 /skills 页视觉正确（待运行时人工验证）
- [ ] 确认卡 Esc/Enter/焦点行为（待运行时人工验证）
- [ ] /skills 页搜索/筛选/排序/新建/编辑/删除/清零（待运行时人工验证）
- [ ] 侧边栏与托盘菜单导航到 /skills（待运行时人工验证）
- [ ] 对话框与页面共享组件行为一致（待运行时人工验证）
