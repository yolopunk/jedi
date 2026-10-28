# Jedi Chat 调用本机 Agent：适配与验收记录（2026-10-29）

对应需求：《Jedi Chat 调用本机 Agent：需求与验收说明》（P0）。

## 1. 本次改动范围

| 需求项 | 实现位置 |
|---|---|
| 检测状态 detecting / ready / needs_auth / invalid_path / unsupported，不静默回退 | `src-tauri/src/agent/detector.rs`、`src/agent/hub/types.ts`、`src/agent/hub/engineStatus.ts`、`src/views/AiChat/EngineSelector.vue` |
| 默认模型与实际模型分开；未知时显示“模型未确认”+原因 | `engineStatus.ts#defaultModelLabel`、各 driver 的 `hooks.onModel`、`opencode.ts#exportedModel` |
| 多轮上下文、流式、停止、失败恢复输入 | 各 driver 的原生 resume（`--resume` / `exec resume` / `-s`）、`src/stores/aiChat.ts`、`src/views/AiChat/index.vue` |
| 阶段 preparing → submitted → generating → done / failed / stopped，无无限转圈 | `aiChat.ts`（`AgentRunPhase`、5 分钟空闲看门狗 `AGENT_IDLE_TIMEOUT_MS`） |
| 明确的错误类别 + 提示 + 可展开详情 | `src/agent/hub/agentError.ts`（not_found / auth / model / network / quota / refused / crash / timeout），`index.vue` 中 `<details>` 技术详情 |
| 未适配引擎明确提示 | `aiChat.ts` 抛出“暂未适配”；选择器中置灰 |

安全约束：只读取模型名，不读取、不展示、不持久化任何凭据 / 认证文件 / API Key；错误详情仅保留 stderr 尾部 40 行、每行 ≤400 字符。

## 2. 实机验证结果

验证方式：直接以 driver 相同的参数调用各 CLI，并把真实报错文本喂给 `agentError.ts` 的分类规则。

| Agent | 宿主 | 版本 | 结果 | 结论 |
|---|---|---|---|---|
| Claude Code | WSL | 2.1.299 | “你好”正常回复；流内模型 `claude-opus-5-5`；`--resume` 两轮上下文（记住数字 → 回答 42） | ✅ 可用 |
| Codex | WSL | 0.164.0 | 正常回复；`exec resume <thread_id>` 两轮上下文回答 42 | ✅ 可用 |
| OpenCode | WSL | v2.0.29 | 正常回复；模型经 `session export` 得到 `opencode/flint-40b-free`；曾出现 “Model is unavailable” → 归类 model | ✅ 可用 |
| OpenCode | Windows | v2.0.28 | 服务端 400：该模型会使用请求数据训练，需在账号 Privacy settings 允许付费端点 | ⚠️ 账号设置问题；归类 refused，原文作为原因展示 |
| Gemini CLI | WSL | — | exit 41 “Please set an Auth method…” | ⚠️ 未登录；归类 auth（文本规则与 exit 41 双路径） |
| Qwen Code | WSL | — | result.is_error “No auth type is selected…” | ⚠️ 未登录；归类 auth（early，立即失败） |
| Pi | WSL | — | “No API key found for the selected model.” | ⚠️ 未配置 Key；归类 auth（early） |
| Antigravity | WSL | — | 需要 OAuth；driver 识别到 OAuth URL 后立即失败 | ⚠️ 未登录；归类 auth（early），不挂起 |
| Cursor CLI | — | — | 本机未安装 | 未验证（driver 复用 geminiCli 协议路径） |
| 程序缺失 | — | — | spawn 失败 | 归类 not_found ✅ |

`vue-tsc --noEmit` 通过；`detector.rs` 的 Rust 测试通过。

## 3. 未适配范围

Crush、Goose、Droid、Kimi Code、ZCode：会被检测到并显示“即将支持”，不可选择；若强行发送会得到明确的“暂未适配”错误，不会回退到其他引擎。本机实际检测到 Goose（WSL）与 ZCode（Windows），表现符合预期。

## 4. 未完成 / 待补

- **应用内截图缺失**：验证时 Windows 处于锁屏状态，computer-use 截图全黑，无法在 Jedi 窗口内完成 UI 级走查。经确认，本轮**以 CLI 级验证作为验收结论**；UI（阶段标签、耗时、错误提示、技术详情展开、失败恢复输入、徽标）未做实机走查，后续如需截图再补。
- Gemini CLI / Qwen Code / Pi / Antigravity 需各自登录后再验证“成功对话”路径；Cursor CLI 需安装后验证。
- OpenCode Windows 需在 opencode 账号中调整隐私设置或切换模型后复测。
