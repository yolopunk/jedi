# 确认白名单（Always-Allow Whitelist）实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development.
> Track progress with the checkboxes below — mark each `- [ ]` as `- [x]` when complete.

**Goal:** write/system 技能支持"始终允许"白名单，消除重复确认摩擦；并修复后台 agent 池无门禁执行危险技能的安全缺口。
**Architecture:** 白名单检查放在调用方传入的 `confirmTool` 层（`runAgent` 本体零改动，保持 agent/ 层不依赖 stores/）；后台池对非白名单写操作直接拒绝（模型可见 denial，可改道）；授权动作写入 `security-audit.log`。
**Tech Stack:** Vue 3 + Pinia + useStorage（settings.json key `skills-config`）；复用 `log_security_event` 审计命令。

---

## 设计决策

| 决策 | 理由 |
|---|---|
| 检查放 confirmTool 调用方层 | runAgent 保持纯净；前台（弹卡）与后台（从严拒绝）可各自定制策略 |
| 后台非白名单写操作直接拒绝 | 后台任务常无人值守，"排队等确认"等于全阻塞；denial 作为工具结果返回，模型可调整方案 |
| system 级技能允许入白名单但 UI 警示 | 保持灵活；警示文案降低误授权 |
| 白名单存 `skills-config` 第三标志位 | 与 enabled/autoCallable 同结构，无新存储 |
| 授权/移除写审计日志 | 复用现有 security-audit.log，安全动作可追溯 |

## 文件变更概览

| 文件 | 改动类型 | 职责 |
|---|---|---|
| `src/stores/skills.ts` | 修改 | `SkillFlags.alwaysAllow?`；`isAlwaysAllowed` / `setAlwaysAllowed`（持久化 + 审计） |
| `src/stores/aiChat.ts` | 修改 | `requestConfirmation` 先查白名单；`resolveConfirmation(approved, alwaysAllow?)` |
| `src/views/AiChat/ToolConfirmCard.vue` | 修改 | "批准并始终允许"按钮 + system 警示；现有硬编码文案迁 i18n |
| `src/stores/agentPool.ts` | 修改 | worker 的 runAgent 传 confirmTool（白名单放行/其余拒绝） |
| `src/views/AiChat/SkillsManagerDialog.vue` | 修改 | write/system 技能行第三开关"始终允许" |
| `src/i18n/locales/zh.ts`、`en.ts` | 修改 | `skills.alwaysAllow`、`skills.confirm.*` 文案 |

## 任务分解

### Task 1: store 层白名单

**文件:** `src/stores/skills.ts`、`src/stores/aiChat.ts`

- [x] **Step 1:** `SkillFlags` 加 `alwaysAllow?: boolean`；`isAlwaysAllowed(id): boolean`（flagMap 读，缺省 false）
- [x] **Step 2:** `setAlwaysAllowed(id, value)`：更新 flagMap、`setItem` 持久化、`logSecurityEvent`（event_type: `skill_whitelist_change`，metadata 含 skillId/value）
- [x] **Step 3:** aiChat：`requestConfirmation` 入口查 `useSkillsStore().isAlwaysAllowed(req.skillId)`，命中直接 `resolve(true)`（不弹卡）
- [x] **Step 4:** `resolveConfirmation(approved, alwaysAllow = false)`：批准且 alwaysAllow 时调用 `setAlwaysAllowed(skillId, true)`
- [x] 提交：`git commit -m "feat(skill): per-skill always-allow whitelist"`

### Task 2: UI 层

**文件:** `src/views/AiChat/ToolConfirmCard.vue`、`src/views/AiChat/SkillsManagerDialog.vue`、`src/i18n/locales/zh.ts`、`en.ts`

- [x] **Step 1:** ToolConfirmCard 三按钮（拒绝 / 批准 / 批准并始终允许）；system 风险时显示警示行；现有 5 处硬编码文案迁入 `skills.confirm.*`
- [x] **Step 2:** SkillsManagerDialog：risk !== 'read' 的技能行加"始终允许"开关（调 `setAlwaysAllowed`）
- [x] **Step 3:** zh/en 文案补齐
- [x] 提交：`git commit -m "feat(skill): always-allow UI in confirm card and manager"`

### Task 3: 后台池门禁

**文件:** `src/stores/agentPool.ts`

- [x] **Step 1:** `runWorker` 里给 runAgent 传 `confirmTool`：白名单内放行，否则 `Promise.resolve(false)`（注释说明从严策略与理由）
- [x] 提交：`git commit -m "fix(agent): gate background pool tools with whitelist"`

## 验证清单

- [x] `pnpm lint`、`pnpm exec vue-tsc --noEmit`、`pnpm build` 通过
- [ ] 对话中执行 write 技能 → 弹卡 → "批准并始终允许" → 同技能再执行不再弹卡；重启后仍不弹（待运行时人工验证）
- [ ] 管理对话框可查看/关闭"始终允许"（待运行时人工验证）
- [ ] 后台 worker 调非白名单 write 技能 → 结果可见拒绝信息而非静默执行（待运行时人工验证）
- [ ] `~/.jedi/security-audit.log` 出现 `skill_whitelist_change` 记录（待运行时人工验证）
