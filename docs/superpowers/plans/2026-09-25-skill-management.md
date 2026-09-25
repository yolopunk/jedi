# Skill 管理功能实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development.
> Track progress with the checkboxes below — mark each `- [ ]` as `- [x]` when complete.

**Goal:** 聊天页技能管理对话框（分组/详情/双开关）+ markdown 自定义技能（`~/.jedi/skills/`）+ 修复禁用状态持久化缺陷。
**Architecture:** SkillsManagerDialog → stores/skills.ts（useStorage 持久化 + CRUD 编排）→ skills/registry（source/setAutoCallable/custom 工厂）→ api/skills.ts → Rust skills.rs（照 memory.rs 骨架）。
**Tech Stack:** Vue 3 + Pinia + Vuetify（对话框手写 CSS 仿 McpServersDialog）；Rust + serde + std::fs。

---

## 文件变更概览

| 文件 | 改动类型 | 职责 |
|---|---|---|
| `src-tauri/src/api/skills.rs` | 新增 | `skills_list_custom` / `skills_save` / `skills_delete` 命令；frontmatter 解析；id 校验 |
| `src-tauri/src/api/mod.rs` | 修改 | 模块声明 |
| `src-tauri/src/main.rs` | 修改 | invoke_handler 注册 |
| `src/api/skills.ts` | 新增 | invoke 封装 |
| `src/api/index.ts` | 修改 | 导出 |
| `src/skills/types.ts` | 修改 | `SkillSource`、`Skill.source` |
| `src/skills/registry.ts` | 修改 | `setAutoCallable`、内置标 source |
| `src/skills/custom.ts` | 新增 | markdown 定义 → Skill 工厂（占位符→参数） |
| `src/skills/index.ts` | 修改 | 导出 custom |
| `src/stores/mcpClient.ts` | 修改 | 桥接标 `source: 'mcp'` |
| `src/stores/skills.ts` | 重构 | useStorage 迁移 + 旧 key 迁移 + 自定义技能加载/CRUD |
| `src/views/AiChat/SkillsManagerDialog.vue` | 新增 | 管理对话框 |
| `src/views/AiChat/index.vue` | 修改 | header 入口按钮 |
| `src/components/AttachmentMenu.vue` | 修改 | 底部"管理技能"项 |
| `src/i18n/locales/zh.ts`、`en.ts` | 修改 | `skills.*` 文案 |
| `src/views/AiChat/SkillPanel.vue` | 删除 | 死代码 |

## 任务分解

### Task 1: 后端自定义技能存储

**文件:** `src-tauri/src/api/skills.rs`、`src-tauri/src/api/mod.rs`、`src-tauri/src/main.rs`

- [x] **Step 1:** 照 memory.rs 骨架写 `skills.rs`：`CustomSkillDef` serde 结构（id/name/description/icon/risk/prompt）、`skills_dir()`、`parse_skill_file()`（frontmatter 解析，容错）、`skills_list_custom`、`skills_save`（id 正则校验 + 覆盖写）、`skills_delete`
- [x] **Step 2:** 单元测试：frontmatter 解析（正常/缺字段/无 frontmatter）、id 校验拒绝路径穿越
- [x] **Step 3:** mod.rs 声明 + main.rs invoke_handler 注册（分组注释 `// Custom skill management commands`）
- [x] **Step 4:** `cargo check && cargo test` 通过
- [x] 提交：`git commit -m "feat(skill): backend custom skill storage"`

### Task 2: 前端类型、注册表与 API 封装

**文件:** `src/skills/types.ts`、`src/skills/registry.ts`、`src/skills/custom.ts`、`src/skills/index.ts`、`src/api/skills.ts`、`src/api/index.ts`、`src/stores/mcpClient.ts`

- [x] **Step 1:** types.ts 加 `SkillSource` 与 `Skill.source?`
- [x] **Step 2:** registry.ts 加 `setAutoCallable(id, v)`，内置注册统一标 `source: 'builtin'`
- [x] **Step 3:** custom.ts：`createCustomSkill(def)` —— `{{param}}` 正则提取参数 schema（string/required）、`execute` 渲染返回 `{ type: 'prompt', prompt }`
- [x] **Step 4:** api/skills.ts 封装三个命令（中文 JSDoc），index.ts 导出
- [x] **Step 5:** mcpClient.ts `bridgeTools` 标 `source: 'mcp'`
- [x] 提交：`git commit -m "feat(skill): skill source and autoCallable support"`

### Task 3: skills store 重构

**文件:** `src/stores/skills.ts`、`src/views/AiChat/index.vue`（loadConfig 调用点）

- [x] **Step 1:** 持久化改 `useStorage` key `skills-config`（`Record<id, {enabled, autoCallable}>`），加载时**双向**应用（含禁用）
- [x] **Step 2:** 一次性迁移旧 `skills-enabled`（localStorage）→ 新格式后移除旧 key
- [x] **Step 3:** `loadCustomSkills()`（后端拉取 → createCustomSkill → registry 注册）；`saveCustomSkill` / `deleteCustomSkill` 动作（命令 + registry 同步）
- [x] **Step 4:** AiChat onMounted 改调 `loadConfig()`
- [x] 提交：`git commit -m "fix(skill): persist enable/autoCallable across restarts"`

### Task 4: 管理对话框与入口

**文件:** `src/views/AiChat/SkillsManagerDialog.vue`、`src/views/AiChat/index.vue`、`src/components/AttachmentMenu.vue`、`src/i18n/locales/zh.ts`、`en.ts`

- [x] **Step 1:** SkillsManagerDialog：三组分区、技能行（icon/名称/风险徽章/描述）、enabled 与 autoCallable 开关、参数展开表
- [x] **Step 2:** 自定义组新建/编辑表单（name/icon/description/risk/prompt textarea + `{{参数}}` 语法提示）与删除确认
- [x] **Step 3:** header 技能按钮（已启用数角标）+ AttachmentMenu"管理技能"项
- [x] **Step 4:** zh/en `skills.*` 文案，无硬编码
- [x] 提交：`git commit -m "feat(skill): skills manager dialog"`

### Task 5: 死代码清理

**文件:** `src/views/AiChat/SkillPanel.vue`、`src/stores/skills.ts`

- [x] **Step 1:** 删 SkillPanel.vue；移除 store 中 `skillPanelOpen/toggleSkillPanel/setSkillPanelOpen`
- [x] 提交：`git commit -m "chore(skill): remove dead SkillPanel"`

## 验证清单

- [x] `pnpm lint`、`pnpm build`（vue-tsc）通过
- [x] `cargo check`、`cargo test` 通过（93 passed，含 skills 模块 10 个）
- [ ] 对话框三组展示正确；MCP 连接/断开后分组实时增减（待运行时人工验证）
- [ ] 禁用内置技能 → 重启 → 仍禁用；旧数据已迁移（待运行时人工验证）
- [ ] 含 `{{参数}}` 的自定义技能可被 agent 调用且渲染正确；编辑/删除后 registry 同步（待运行时人工验证）
- [x] autoCallable 关闭的技能不进入 runAgent tools（`listAutoCallable` = enabled && autoCallable，代码路径保证）
- [x] 中英文文案齐全
