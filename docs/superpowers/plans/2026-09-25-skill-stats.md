# 执行统计（Skill Execution Stats）实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development.
> Track progress with the checkboxes below — mark each `- [ ]` as `- [x]` when complete.

**Goal:** 技能调用可观测——每技能调用次数、失败数、平均耗时、最近错误，聚合落盘 `~/.jedi/skill_stats.json` 并在管理对话框展示；支持按使用排序与一键清零。
**Architecture:** `withStatsHooks()` 组合包装调用方 hooks（不侵入 runAgent 本体）→ `stores/skillStats.ts` 内存聚合 + 3s 防抖整表批量写 → Rust 三命令。
**Tech Stack:** Vue 3 + Pinia；Rust serde + std::fs（照 memory.rs 骨架）。

---

## 设计决策

| 决策 | 理由 |
|---|---|
| `withStatsHooks()` 组合包装 | 调用方零语义变化；aiChat trace 记账、agentPool progress 更新原样保留 |
| 3s 防抖 + 整表批量写 | 数据有界（错误 ≤10 条/技能），整表覆盖最简单；最坏丢末 3s 计数 |
| 后端 save 强制截断（errors≤10、msg≤200 字符） | 前端聚合，后端最终防护 |
| 拒绝（denial）计入 failures | v1 简化，后续可加 `denied` 独立字段区分 |
| 错误不存 args | 脱敏，只存截断 message |

## 数据结构（前后端同构，camelCase）

```ts
{ [skillId]: {
  calls: number; failures: number; totalMs: number; lastUsedAt: number;
  recentErrors: { ts: number; msg: string }[]  // ≤10，msg ≤200 字符
} }
```

## 文件变更概览

| 文件 | 改动 | 职责 |
|---|---|---|
| `src-tauri/src/api/skills.rs` | 修改 | `SkillStatEntry`/`SkillStatError`、`stats_path()`、`skill_stats_list/save/clear` + 测试 |
| `src-tauri/src/main.rs` | 修改 | 注册三命令 |
| `src/api/skills.ts` | 修改 | `listSkillStats`/`saveSkillStats`/`clearSkillStats` |
| `src/stores/skillStats.ts` | 新增 | 聚合、防抖 flush、清零、查询 |
| `src/agent/statsHooks.ts` | 新增 | `withStatsHooks(hooks)` 包装器 |
| `src/stores/aiChat.ts` | 修改 | hooks 包装 + onMounted `loadStats()` |
| `src/stores/agentPool.ts` | 修改 | hooks 包装 |
| `src/views/AiChat/SkillsManagerDialog.vue` | 修改 | 徽章/详情/排序/清零 |
| `src/i18n/locales/zh.ts`、`en.ts` | 修改 | `skills.stats.*` |

## 任务分解

### Task 1: 后端统计存储

**文件:** `src-tauri/src/api/skills.rs`、`src-tauri/src/main.rs`

- [ ] **Step 1:** stats 结构与命令：`stats_path()`、`skill_stats_list`、`skill_stats_save`（截断防护：errors 保留最近 10 条、msg 200 字符）、`skill_stats_clear`
- [ ] **Step 2:** 单元测试：截断、环形上限、roundtrip
- [ ] **Step 3:** main.rs 注册
- [ ] 提交：`git commit -m "feat(skill): backend skill stats storage"`

### Task 2: 采集层

**文件:** `src/api/skills.ts`、`src/stores/skillStats.ts`、`src/agent/statsHooks.ts`、`src/stores/aiChat.ts`、`src/stores/agentPool.ts`

- [ ] **Step 1:** api 封装三命令
- [ ] **Step 2:** stats store：`loadStats`（幂等）、`recordToolEnd`（增量 + 防抖调度）、`flush`、`clearAll`、`statsFor` + 成功率/均耗时 computed
- [ ] **Step 3:** `withStatsHooks` 包装器（onToolStart/onToolEnd 透传原 hooks 再记账）
- [ ] **Step 4:** aiChat / agentPool 接线；AiChat onMounted 调 `loadStats()`
- [ ] 提交：`git commit -m "feat(skill): per-skill execution stats collection"`

### Task 3: UI 层

**文件:** `src/views/AiChat/SkillsManagerDialog.vue`、`src/i18n/locales/zh.ts`、`en.ts`

- [ ] **Step 1:** 技能行统计徽章（calls>0 才显示）；展开面板统计详情 + 最近错误列表
- [ ] **Step 2:** 头部"按使用排序"开关（组内 calls 降序）与"清零统计"按钮（两段式确认）
- [ ] **Step 3:** zh/en 文案
- [ ] 提交：`git commit -m "feat(skill): skill stats UI in manager"`

## 验证清单

- [ ] `pnpm lint`、`pnpm exec vue-tsc --noEmit`、`pnpm build` 通过
- [ ] `cargo test` 通过（含新增测试）
- [ ] 对话触发工具调用 → 3s 后 `~/.jedi/skill_stats.json` 出现计数（待运行时人工验证）
- [ ] 徽章/详情/排序/清零生效；后台 worker 调用计入（待运行时人工验证）
