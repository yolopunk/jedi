# 技能体系迭代路线图

> 日期：2026-09-25
> 顺序依据：先补安全缺口，再补可观测性，再还架构债，UI 升级按需触发。

## 迭代 1：确认白名单（已开工，见 `2026-09-25-confirmation-whitelist.md`）

消除 write/system 技能重复确认摩擦；修复后台 agent 池无门禁执行危险技能的安全缺口。白名单检查在 confirmTool 调用方层；后台非白名单写操作直接拒绝；授权写审计日志。

## 迭代 2：执行统计

- 埋点：不侵入 runAgent 本体，封装 `createStatsHooks()` 组合现有 hooks；aiChat / agentPool 两处统一接入
- 聚合：内存累积 + 防抖批量落盘；后端照 memory.rs 模式新增 `~/.jedi/skill_stats.json`（`skill_stats_list` / `skill_stats_record`（批量）/ `skill_stats_clear`）
- 数据结构：`{ [skillId]: { calls, failures, totalMs, lastUsedAt, recentErrors: [{ ts, msg }][≤10] } }`；错误只存截断 message（200 字符），**不存 args**（脱敏）
- UI：管理对话框技能行指标徽章（`N 次 · 成功率 · 均耗时`）、展开详情、按使用频率排序、清零入口
- 开工前需细化：防抖窗口大小、并发 worker 写入合并策略

## 迭代 3：`loop.ts` / `runAgent` 双执行路径清理

- 先 grep 确认 `AgentLoop` / `AgentStep` / `AgentEvent` 的真实消费方（`stores/agent.ts` 待查）
- 删除 `loop.ts` 与无人消费的类型/死状态；确认语义统一收敛到 runAgent 的 risk 门禁（废弃 `confirmationMode`）
- 已知陷阱：`AgentLoop.executeSkill` 的 `confirmationMode: 'always'` 路径 emit 后直接 return，无 resolve 通路会悬死——删除即修复
- 配一次全链路回归：对话 / 后台 worker / trace 面板

## 迭代 4：独立 `/skills` 管理页（按需触发）

- **触发条件**：迭代 2 的统计面板 + 自定义技能编辑器让 640px 对话框超载时再动，避免为升级而升级
- 做法：`/skills` 路由 + `AppSidebar.navItems` 入口 + i18n `sidebar.skills`（可选托盘菜单项）；技能行/编辑器先抽成 `components/skills/` 共享组件；页面按 HostsManager 编排模式，工具栏用 console-header-bar 风格；聊天页入口保留（仍可弹轻量版）

## 更远方向（暂不排期）

技能导入/导出与分享格式、每个 agent worker 独立技能集、技能市场（与 MCP 生态打通）、按会话粒度的技能开关。
