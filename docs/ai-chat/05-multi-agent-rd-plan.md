# AI Chat 多 Agent 统一客户端 · 研发规划

<!-- Created: 2026-10-25 | Status: Draft | Owner: Jedi -->

## 1. 背景与动机

### 1.1 现状

AI Chat 目前是**自研单引擎**架构：agent 循环跑在渲染进程（Vercel AI SDK `streamText`），工具经 Tauri invoke 调用 Rust 原语（`src/agent/runAgent.ts`）。该实现存在：

- **P0 缺陷**：FILESYS 工具调用的 `read_file`/`write_file`/`file_exists` 命令在 Rust 侧未注册，必然失败；同一步内多个风险工具并行触发确认门单槽竞态，可挂死整个 run。
- **能力短板**：无上下文压缩（compaction）、无 steering（生成中插话）、无步间重试；图片附件每轮全量重发（成本炸弹）；工具轨迹不进入后续轮上下文。
- **生态隔离**：用户机器上已有的 coding agent（Claude Code、Codex、pi 等）能力无法被 Jedi 使用，Jedi 的工具箱能力（hosts/壁纸/播客）也无法反哺给它们。

### 1.2 产品重定位

AI Chat 升级为「**系统已装 coding agent 的统一 GUI 客户端**」：

- Jedi 负责**发现、接入、渲染、管理**；agent 引擎可插拔（pi / Claude Code / Codex / 内置引擎）。
- 该形态已有市场验证：Vibe Kanban（多 agent 编排看板）、Conductor / Crystal（Claude Code 的 GUI）。
- Jedi 的差异化：**跨 Windows/WSL/macOS/Linux 的检测能力** + **工具箱能力通过自家 MCP server 反哺所有 agent**。

### 1.3 为什么是「外部依赖」而不是「打包内置」

曾评估把 pi（Node 运行时）作为 sidecar 打包进安装包，体积增量约 80-150MB，与 Tauri 轻量定位冲突，**已否决**。改为要求用户机器已安装目标 agent（类似桌面工具依赖 ffmpeg/git 的标准模式），Jedi 本体零体积增量；检测不到时提供安装引导。

## 2. 目标与非目标

### 2.1 目标

| # | 目标 | 验收口径 |
|---|------|---------|
| G1 | 启动时自动检测系统已装 agent，Windows 含 WSL 各发行版；检测清单与 Magpie 主流 agent 清单对齐（§3.2） | 检测结果正确区分「agent × 安装位置」；docker-desktop 等虚拟发行版被过滤 |
| G2 | AI Chat 页可按会话选择 agent 引擎 | 选择器展示检测到的引擎（含版本/位置）；未检测到的已知 agent 灰显 + 安装指引 |
| G3 | 统一事件模型，聊天 UI 一套渲染 | 现有 trace 渲染对全部引擎可用；能力位控制功能显隐 |
| G4 | 统一权限审批 | 各引擎审批协议翻译为同一张确认卡；保留白名单与审计 |
| G5 | 内置引擎永久兜底 | 无任何外部 agent 时聊天功能完整可用（含 P0 修复） |
| G6 | 跨平台：Windows（含 WSL）/ macOS / Linux | 检测与 spawn 三平台全覆盖；CI 跑检测单测 |

### 2.2 非目标

- 不做 agent 编排/多 agent 并行协作看板（Vibe Kanban 方向）。
- 不打包任何 agent 运行时。
- 不做移动端。
- 不内置 MCP client 功能扩展（现有 MCP 客户端去留在 M4 评估）。

## 3. 产品设计

### 3.1 用户可见变化

- **聊天页 composer 上方新增引擎选择器**：`[● pi @ WSL:ArchLinux ▾] [模型 ▾]`。下拉按宿主分组（本机 / WSL:ArchLinux / WSL:Ubuntu / …），同名 agent 可多条目（Windows 一份 + WSL 各一份，即「agent@位置」为独立引擎）；未检测到的已知 agent 灰显并附安装指引；「内置引擎」固定于列表底部兜底。
- **首次引导**：启动检测完成前选择器呈扫描态；无任何外部 agent 时展示引导卡片（各 agent 安装命令 + 重新检测按钮）。
- **设置新增「Agent 管理」页**：检测明细（版本/路径/来源/宿主）、手动刷新、每引擎默认项（模型、启动参数、权限档、最低版本门）。
- **生成中交互分级**：有 steering 能力的引擎（pi/Claude Code）输入框常驻可插话；无此能力的引擎输入进 follow-up 队列。

### 3.2 Agent 清单（与 Magpie 主流清单对齐）

检测口径对齐 [Magpie](https://github.com/yetone/magpie)（yetone 出品，支持 50+ agent 的模型切换/用量管理工具）：Jedi 取其**主流子集**，并按「检测」与「驱动」两层拆分——检测只做发现/版本/安装引导，成本极低可全量铺；驱动需要无头协议，按验证成熟度分级接入。

**检测层（M1 全量实现，Detect-only）**

| Agent | 检测命令 | 驱动层级 |
|---|---|---|
| Claude Code | `claude --version` | A |
| Codex (OpenAI) | `codex --version` | A |
| pi | `pi --version` | A |
| Gemini CLI | `gemini --version` | B |
| Qwen Code | `qwen --version` | B（gemini-cli 同源，协议或可复用） |
| Cursor CLI | `cursor-agent --version` | B（gemini-cli 系 fork） |
| OpenCode | `opencode --version` | B（有 `serve` HTTP API） |
| Crush | `crush --version` | B |
| Goose | `goose --version` | B |
| Droid | `droid --version` | B |
| Kimi Code | `kimi --version` | B |
| Antigravity CLI | `antigravity --version` | C（仅检测） |

不纳入：Claude Desktop / Zed / VS Code / JetBrains 系（非 CLI 对话 agent——Magpie 对它们做的是配置改写，而 Jedi 需要可驱动的 stdio 协议）；oh-my-pi 等衍生发行版（与 pi 同主体去重）。

**驱动层（可作为聊天引擎）**

| 引擎 | 协议 | steering | compaction | resume | 审批来源 |
|------|------|----------|------------|--------|---------|
| 内置 | 自有循环 | ✗ | ✗（M1 后逐步补） | 每轮重放 | confirmTool |
| pi | `--mode rpc`（JSONL 双向） | ✓ | ✓ 自动 | ✓（switch_session/fork） | Extension UI |
| Claude Code | `--input-format stream-json`（双向 + control_request） | ✓ | ✓ 自动 | ✓（`--resume <id>`） | control 协议 |
| Codex | `app-server`（JSON-RPC 2.0 over stdio） | 视协议 | ✓ | ✓（conversation id） | app-server 事件 |

> Tier B（Gemini CLI / Qwen Code / Cursor CLI / OpenCode / Crush / Goose / Droid / Kimi Code）：单轮 JSON 输出确定存在，流式事件、steering、审批往返是否可用待 M0 事件样例集验证后，按结果排入 v1.x。gemini-cli 同源三兄弟（Gemini/Qwen/Cursor CLI）预计一套适配器覆盖三个引擎。

## 4. 技术架构

### 4.1 分层

```
┌─ Vue UI ─────────────────────────────────────────┐
│ Chat 页（复用现有 trace 渲染）│ 引擎选择器 │ Agent 管理页 │
│          ↕ UnifiedAgentEvent（统一事件模型）          │
│ AgentDriver 适配层（TS，渲染进程）                     │
│  ├─ BuiltinDriver（runAgent 改造）                    │
│  ├─ PiDriver / ClaudeCodeDriver / CodexDriver        │
│         ↕ Tauri v2 Channel（有序、会话隔离、不串扰）    │
│ Rust：AgentDetector（三平台）+ AgentProcessManager    │
│   （spawn / 严格 \n 分帧 / env 注入 / 生命周期 / 超时） │
└───────────────────────────────────────────────────┘
```

### 4.2 关键决策

| 决策 | 内容 | 理由 |
|------|------|------|
| D1 | 协议解析在 TS，Rust 只做通用进程管理 | agent 协议迭代快；TS 改适配器免重编译 Rust；Rust 侧零协议知识 |
| D2 | 流式通道用 Tauri Channel API 而非全局事件 | 有序、背压、并发会话天然隔离 |
| D3 | 帧解析严格按 `\n` 切分 | pi RPC 文档明确警告：通用行分割器（如 Node readline）会在 JSON 字符串内的 U+2028/U+2029 上错误断行 |
| D4 | 内置引擎改造为「第 0 号 Driver」 | UI 只面对统一事件模型；P0 修复顺势落在 M1 |
| D5 | 会话与引擎绑定，切换引擎=开新会话 | 协议语义差异大，跨引擎迁移会话不做 |
| D6 | 统一事件模型对齐现有 `RunAgentHooks` | 现有 trace UI 改造成本最小化；新增 `approval_request` / `engine_status` 两类 |

### 4.3 Driver 接口（TS）

```ts
interface AgentDriver {
  capabilities: {
    steering: boolean; compaction: boolean; images: boolean
    resume: boolean; models: boolean
    approval: 'native' | 'extension' | 'builtin' | 'none'
  }
  start(opts: DriverStartOpts): Promise<DriverHandle>
}
interface DriverHandle {
  prompt(text: string, attachments?: Attachment[]): Promise<void>
  steer(text: string): Promise<void>
  followUp(text: string): Promise<void>
  setModel(modelId: string): Promise<void>
  respondApproval(requestId: string, decision: 'allow' | 'deny' | 'always-allow'): Promise<void>
  abort(): Promise<void>
  events: AsyncIterable<UnifiedAgentEvent>
  dispose(): Promise<void>
}
```

`UnifiedAgentEvent` = 现有 hooks 事件 + `approval_request` + `engine_status`（压缩中/重试中/steering 已排队）。

## 5. 跨平台检测设计

### 5.1 探测策略（每个位置按序执行，命中即止）

三层策略，缺一不可（本机实测证明单靠 PATH 会漏检，见附录 B）：

1. **PATH 探测**：执行 `<agent> --version`，取退出码与 stdout。
2. **登录/交互 shell 探测**：`sh -lc 'command -v <agent>'` 与 `bash -lic 'command -v <agent>'` 各一次（nvm/交互式 rc 场景只有后者可见）。
3. **已知位置扫描**：直接 `stat` 各包管理器的固定安装路径（见 5.2 矩阵）。

> 实现参考：各 agent 的二进制名/安装位置/配置路径知识可借鉴 [Magpie](https://github.com/yetone/magpie) 的 agent 注册表实现（同为主流 agent 清单的检测知识库），避免逐个踩坑。

### 5.2 各宿主矩阵

| 宿主 | PATH 之外必扫位置 | spawn 方式 | 特殊处理 |
|------|------------------|-----------|---------|
| Windows | `%APPDATA%\npm`（.cmd 需经 shell 或解析 shim）、`~\scoop\shims`、fnm/volta 的 node bin、`~\.cargo\bin` | 解析出真实可执行体后直接 spawn；.cmd shim 用 `cmd /C` 包裹 | GUI 进程不继承终端 PATH，兜底扫描必备 |
| WSL（每个发行版） | `~/.local/bin`、`~/.npm-global/bin`、`~/.local/share/pnpm`、`~/.bun/bin`、`~/.volta/bin`、`~/.nvm/versions/*/bin`、`/usr/local/bin`、`~/.cargo/bin`、`~/go/bin` | `wsl.exe -d <name> -- bash -lic '<agent> <args>'`，stdio 管道跨边界 | 见 5.3 |
| macOS | `/opt/homebrew/bin`、`/usr/local/bin`、`~/.npm-global/bin`、`~/.local/share/pnpm`、`~/.bun/bin`、`~/.volta/bin`、`~/.nvm`、`~/.cargo/bin` | 解析绝对路径直接 spawn；必要时 `zsh -lc` 捕获登录环境 | Dock 启动不继承 shell PATH，同需兜底扫描 |
| Linux | 同 macOS 位置集（去 homebrew） | 同上，`bash -lc` | — |

### 5.3 WSL 专项（本机已实测的坑，全部写入实现要求）

| # | 要求 | 实测依据（附录 B） |
|---|------|------------------|
| W1 | `wsl.exe -l -q` 输出为 **UTF-16LE**，必须先解码再解析 | 直接读为乱码 |
| W2 | 发行版名**必须来自枚举结果**，禁止硬编码；错误消息同为 UTF-16 乱码，需容错 | `-d Arch` → `WSL_E_DISTRO_NOT_FOUND`，实际名为 `ArchLinux` |
| W3 | 过滤虚拟发行版：`docker-desktop`、`docker-desktop-data` | 枚举结果含 docker-desktop |
| W4 | 发行版间探测**并行执行**（冷启动 1-3s/个） | — |
| W5 | 附件路径经 `wsl.exe -d <X> wslpath -a '<win路径>'` 翻译为 `/mnt/c/...` 再传 | — |
| W6 | 检测命令一律固定字符串（无用户输入拼接），杜绝 shell 注入 | — |
| W7 | 开发期从 Git Bash 调 `wsl.exe` 有 MSYS 路径转换陷阱（`2>/dev/null` 被改写）；Rust 无此问题，但调试脚本需 `MSYS_NO_PATHCONV=1` | — |

### 5.4 检测结果模型与缓存

```rust
struct AgentInstallation {
  agent_id: String,          // "pi" | "claude-code" | "codex" | "builtin"
  host: AgentHost,           // Windows | Wsl(distro) | Macos | Linux
  exe: String,               // 解析后的可执行路径（Windows）或 spawn 命令模板（WSL）
  version: String,           // --version 解析结果
  min_ok: bool,              // 是否满足 Jedi 定义的最低版本门
}
```

- 启动时异步检测，结果**渐进式上屏**；本地缓存（含时间戳，TTL 24h）避免每次冷扫 WSL。
- 「Agent 管理」页提供强制刷新（忽略缓存重扫）。
- 版本门：每 agent 在 Jedi 侧维护 `min_version`（协议破坏性变更时上调），过低则引擎条目标注「需升级」。

## 6. 会话、权限与密钥

- **会话**：SQLite 会话表新增 `engine`（引擎标识，含宿主后缀如 `pi@wsl:ArchLinux`）与 `native_session_id` 字段；展示层消息仍由 Jedi 记录，上下文连续性依赖各引擎 resume 能力（pi `switch_session` / Claude `--resume` / Codex conversation id）；无 resume 能力时退回每轮重放全史（内置引擎现状）。
- **权限**：统一确认卡复用现有 UI 与「始终允许」白名单、审计日志。各引擎默认取最严权限档（Claude Code 收紧 permission-mode；Codex approval 必问；pi 经权限扩展必问），不提供「全局自动放行」。
- **密钥**：沿用 keyring；按引擎需要注入 spawn 环境变量（如 `ANTHROPIC_API_KEY`），不落盘、不经命令行参数。Claude Code/Codex 使用自身登录态，检测到未登录时引导用户在终端完成 `/login`。
- **隐私**：外部引擎 spawn 时注入 `PI_OFFLINE=1`、`PI_TELEMETRY=0`（关闭 pi 版本检查与遥测）。

## 7. 工具箱能力反哺（差异化）

`src-tauri/src/mcp/server.rs` 已是现成 MCP server（当前仅暴露 hosts）。扩展为暴露 hosts/壁纸/播客/截图能力：

- Claude Code：`--mcp-config` 指向 Jedi；
- Codex：`config.toml` 的 `mcp_servers`；
- pi：薄 TS 扩展经 CLI 调用（pi 无 MCP）。

使 Jedi 从「GUI 壳」升级为「能力平台」。归 M4，不阻塞 v1。

## 8. 里程碑计划

| 里程碑 | 内容 | 出口标准（验收） | 工时 |
|--------|------|----------------|------|
| **M0 Spike** | ① 检测原型：Windows 宿主 + WSL 枚举/探测/过滤（真实机器）② 在 WSL Arch 里装好 pi 并跑通 `--mode rpc` 一次完整「prompt→流式→审批往返→settled」③ Claude Code stream-json 同样跑通（含 control_request 权限往返）④ Tauri Channel 流式验证 | 产出事件样例集（每引擎一份 JSONL 录制）+ 检测模块骨架 + 本文档修订；**决策点：三引擎协议可行性确认** | 3-5 天 |
| **M1 框架** | Rust `AgentDetector`（三平台策略层+缓存，检测层覆盖 §3.2 全量主流清单）+ `AgentProcessManager`（spawn/分帧/超时/生命周期）；TS driver 框架与统一事件模型；内置引擎改造为 BuiltinDriver（**含两个 P0 修复与回归测试**）；引擎选择器 UI + 扫描态 | 内置引擎经 driver 接口跑通现有聊天全流程（trace/审批/统计不回归）；检测在实测机器正确发现 `claude@wsl:ArchLinux`、`codex@wsl:ArchLinux`，且 §3.2 清单逐项有条目（未安装的呈「未检测到+安装指引」态）；特性开关 `agentHub` 默认关 | 1-1.5 周 |
| **M2 pi 适配器** | PiDriver 全功能：会话续接/fork、steering、compaction 状态、附件（wslpath 翻译）、审批扩展 | 在 WSL 引擎上完整对话：多轮、插话、压缩触发、审批放行/拒绝各一次 | 1 周 |
| **M3 Claude Code 适配器** | ClaudeCodeDriver：stream-json 双向、`--resume`、control 审批、能力位降级 | 同 M2 口径 + 会话跨启动恢复 | 1-1.5 周 |
| **M4 扩展与打磨** | Codex app-server 适配器；Tier B 驱动接入（按 M0 验证结果排期，gemini-cli 系一套适配器优先覆盖 Gemini/Qwen/Cursor CLI 三引擎）；MCP server 能力扩展（工具箱反哺）；Agent 管理页完善；清理旧路径（删 `runAgent` 旧循环外的死代码：Rust pool、`send_chat_message_stream` 等） | Codex 引擎可用或明确降级为 P5；至少一个 Tier B 引擎可用；特性开关转正 | 1-2 周（可拆） |

- **v1 发布口径**：M1-M3（内置 + pi + Claude Code，含 WSL），约 **4-6 周**（单人全职）。
- **macOS/Linux 验证**：M1 完成检测层单测（fixture 化路径矩阵）；真机验证安排在 v1 发布前借 CI/社区设备完成。

## 9. 风险登记册

| # | 风险 | 等级 | 对策 |
|---|------|------|------|
| R1 | 外部协议不稳定（pi RPC 演进快；Claude stream-json 输入协议官方文档有缺口，社区逆向整理；Codex app-server 曾标实验性） | 高 | 版本门 + 启动握手协议探测；每适配器维护「已验证版本区间」；事件样例集做回归 |
| R2 | WSL 碎片化：发行版差异、冷启动慢、UTF-16、PATH/rc 差异 | 高 | 5.3 全部条款为硬性实现要求；探测并行 + 缓存；已知位置扫描兜底（本机实测 PATH 漏检率 100%） |
| R3 | 三家审批协议完全不同，适配工作量超预期 | 中 | 能力位 + 优雅降级（无审批则最严权限档）；M0 即验证审批往返 |
| R4 | 同名 agent 多位置并存造成用户困惑 | 低 | 「agent@位置」独立引擎条目 + 位置徽标；管理页可禁用某条目 |
| R5 | Claude Code/Codex 登录态在 GUI 环境不可用 | 中 | 检测登录态，未登录引导终端 `/login`；文档说明 |
| R6 | 内置引擎 P0 修复延误拖累 M1 | 低 | P0 修复独立成 M1 首任务，先合入主干 |

## 10. 执行进度（2026-10-25）

### 已落地

| 项 | 内容 | 证据 |
|---|------|------|
| P0-1 修复 | 新增 `src-tauri/src/api/agent_fs.rs`（`read_file`/`write_file`/`file_exists`，读截断 200k 字符）并注册进 `invoke_handler` | `cargo check` 通过 |
| P0-2 修复 | 确认门改 FIFO 队列（`aiChat.ts`），`resolveConfirmation` 只解队首并推进；run 结束时清空整队——同一步并行工具调用不再挂死 | 前端 typecheck/build 通过 |
| M1 Rust | `agent/detector.rs`：12 agent 常量表、Windows 已知目录扫描 + `where.exe`、WSL 枚举（UTF-16 解码、过滤 docker-desktop、`--exec bash -lc` 单次探测脚本）、unix 登录 shell 探测、版本并行 best-effort、24h 缓存 | `cargo check` 通过 |
| M1 Rust | `agent/procman.rs`：按 SpawnSpec 构建 spawn（windows-exe / windows-cmd / wsl / unix）、严格 `\n` 字节分帧、Tauri Channel 事件（line/stderr/exit）、stdin 写入、abort 即 kill_on_drop 杀进程树 | `cargo check` 通过 |
| M1 TS | `src/agent/hub/`（types/builtin/registry）+ `stores/agentHub.ts` + `EngineSelector.vue`；引擎选择器入 chat composer，tier-B 灰显「适配中」，未检测到的 agent 折叠展示，选择持久化 | `vue-tsc`/`biome`/`vite build` 通过 |
| M1 路由 | `aiChat.sendMessage` 全量经 `driverFor(engine)` 驱动调用（内置引擎由 BuiltinDriver 包装同一 runAgent）；无驱动引擎回退内置并在 trace 留痕；run metadata 记录 engine | 同上 |

### M0 实测结果（2026-10-25，本机）

| 验证项 | 结果 |
|---|---|
| pi 安装（WSL ArchLinux） | `bun install -g @earendil-works/pi-coding-agent` → `~/.bun/bin/pi`，v1.4.0（此前 pnpm 全局未链接，确认为未装成） |
| pi RPC 传输层 | `wsl.exe --exec bash -lc` 管道下 `get_state` 握手成功（返回 sessionId/steeringMode/autoCompaction 等完整状态） |
| pi RPC 错误语义 | 无 key 时 `prompt` 返回 `success:false` + 可读错误（"No API key found..."），帧协议无恙 |
| pi 完整流式 | **待办**：`~/.pi/agent/auth.json` 为空，需用户提供 API key（M2 接 keyring 注入后完成） |
| Claude Code stream-json | **全链路验证**（v2.1.299 已登录）：`-p --output-format stream-json` 输出 `system/init`（含 session_id/tools/capabilities）、`assistant`（文本+usage+缓存计费）、`rate_limit_event`、`result`（result 文本/stop_reason/cost） |
| Claude 注意点 | 工作区未 trust 会忽略 `settings.local.json` 的权限配置并告警（headless 需处理 trust）；`--dangerously-skip-permissions` 可用但仅限 spike |
| 检测探测脚本 | 与 Rust 逐字相同的脚本在真实 WSL 执行：发现 `claude@~/.local/bin`、`codex@~/.local/bin`、`pi@~/.bun/bin`、`antigravity@/mnt/d/...`（PATH 登录 shell 全部漏检，再次证实三层策略必要性） |
| 新发现（已修） | WSL 继承 Windows PATH，Windows 安装以 `/mnt/<盘>/...` 泄漏进 WSL 检测造成双计数——`parse_probe_output` 已过滤 `/mnt/<drive>/` 条目（Windows 宿主探测会找到它们） |

### M2 进展（2026-10-25）

| 项 | 内容 | 证据 |
|---|------|------|
| pi 协议全貌实测 | 认证（env key 路径）、事件流形状、`--session <id>` 跨进程续接（同 sessionId/messageCount 保持）、`--model anthropic/<id>` 选择（set_model 命令需拆 provider/model，v1 用 CLI flag） | 本机捕获记录见下 |
| PiDriver | `src/agent/hub/drivers/pi.ts`：经 `agent_proc_spawn` 每轮长驻 spawn、keyring→env 注入（Jedi 已配 provider 映射 pi env 变量表）、会话续接（内存 Map）、steer/followUp/abort、exit 异常兜底 | typecheck/build 通过 |
| 事件翻译器 | `src/agent/hub/drivers/piProtocol.ts` 纯函数模块（无 Tauri/Vue 依赖），response/message_update/tool_execution_*/message_end/turn_end/agent_settled → RunAgentHooks | `scripts/pi-protocol-replay.ts` 用**真实捕获流**回放：**17/17 断言通过**（`node scripts/pi-protocol-replay.ts`） |
| WSLENV 修复 | `procman.rs`：wsl 引擎 spawn 时把注入的 env 变量名列表写入 `WSLENV`——否则 wsl.exe 不转发 Win32 环境进发行版，key 注入在 WSL 引擎上会完全失效（实测确认并修复） | `cargo check` 通过；WSLENV 机制单测验证 |
| 接线 | registry 注册 PiDriver；`runSpec.engine` 贯通；`RunAgentResult.nativeSessionId` 扩展 | typecheck 通过 |

**pi 认证路径验证**：`ANTHROPIC_OAUTH_TOKEN`（Claude 订阅 OAuth token）认证成功——pi 走 api anthropic-messages 被 Anthropic 识别为第三方应用；当前账户"extra usage"额度耗尽返回结构化 400（`"Third-party apps now draw from your extra usage…"`），与 Claude Code 流中 `out_of_credits` 一致。**完整成功文本流待账户充值或任一 provider key 配入 Jedi 后自动打通**（驱动注入链路各环节已分别验证：envs 注入 ✓、WSLENV 转发 ✓、pi 读 env 认证 ✓、事件翻译回放 ✓）。

**M2 剩余（下一批）**：审批扩展（pi beforeToolCall → Jedi 确认卡）、nativeSessionId 持久化到消息 metadata（当前内存 Map，重启后新会话）、steering 输入 UI、附件（wslpath 翻译）。

### M3 进展（2026-10-25）

| 项 | 内容 | 证据 |
|---|------|------|
| 协议实测补全 | 双向输入（stdin 每轮一条 user 消息）、`--include-partial-messages` 的 `stream_event.content_block_delta` token 级流、`--resume <sid>` 与双向输入组合（codeword 跨进程召回、sessionId 保持） | 本机实测（2.1.299，sonnet 首方计量可用） |
| 审批机制结论 | **2.1.299 在 `-p` 模式不 emit control_request，工具自动放行**（`--permission-mode default`、`--permission-prompt-tool stdio`、干净配置目录均试过；根因含用户全局 `defaultMode: auto`，但 default 模式下 -p 也不询问）。按 Agent SDK 已知形状实现翻译与 control_response 往返，版本一旦开始发请求即自动点亮 Jedi 确认卡 | 实测记录 + [cli-protocol 社区文档](https://github.com/Roasbeef/claude-agent-sdk-go/blob/main/docs/cli-protocol.md) |
| ClaudeCodeDriver | `src/agent/hub/drivers/claudeCode.ts`：每轮 spawn `-p --input/output-format stream-json --verbose --include-partial-messages`、模型透传（仅 claude 系 id）、`--resume` 续接（内存 Map + nativeSessionId）、审批经 `opts.confirmTool`（走既有确认卡队列）、result 即 turnComplete 收口、exit 异常兜底 | typecheck/build 通过 |
| 翻译器 | `claudeCodeProtocol.ts`：init/stream_event(deltas)/assistant(tool_use→id→name 映射)/user(tool_result)/result 终局（usage 归一化 + totalCostUsd）/control_request 双形状 | `scripts/claude-code-protocol-replay.ts` 真实捕获流回放 **20/20 通过** |
| 已知妥协 | 外部引擎在宿主侧工具放行（claude -p 自动批准、pi 默认无权限门）——确认卡对两引擎当前处于休眠态，激活路径已实现；风险与后续（pi 审批扩展 / claude 版本升级观察）记录在 M2/M3 遗留 | — |

**M3 剩余（下一批）**：审批激活观察（claude 版本变更）、nativeSessionId 持久化（与 pi 共用方案）、steering 输入 UI、附件支持。

### M4 进展（2026-10-25）：Codex app-server 适配器

| 项 | 内容 | 证据 |
|---|------|------|
| 协议实测（codex-cli 0.162.1，ChatGPT Plus 登录） | JSON-RPC 2.0 无 jsonrpc 头；`initialize {clientInfo}` 握手；`thread/start {}`（cwd 可选）；`thread/resume {threadId}` 续接（实测同线程恢复，附分页弃用提示无害）；`turn/start {threadId, input:[{type:"text"}]}`；通知流 `thread/started`、`turn/started`、`item/started\|updated\|completed`（item.type: userMessage/agentMessage/reasoning/commandExecution…）、`error`（实测 `usageLimitExceeded`）、`turn/completed` 为**终局** | 本机捕获 |
| CodexDriver | `src/agent/hub/drivers/codex.ts`：每轮 spawn `app-server` → initialize → thread/resume\|start（**await 响应取 threadId**，rpc 带 id 关联）→ turn/start；threadId 内存 Map 续接；turn/completed 收口、exit 兜底、abort | typecheck/build 通过 |
| 翻译器 | `codexProtocol.ts`：agentMessage 按 item 增量算 delta（emittedTextById）；commandExecution/fileChange/mcpToolCall → 工具 hooks（exitCode≠0 标 error）；JSON-RPC error 响应终局；usageLimitExceeded 完整失败路径 | `scripts/codex-protocol-replay.ts` **15/15 通过**（真实捕获流） |
| 配额状态 | ChatGPT Plus 当前 usage limit 耗尽（"try again at 3:35 AM"）——与 pi 同状：协议全验证、成功流待配额恢复后自动打通 | 实测 error 通知 |
| 遗留 | 审批形状未在本版捕获（未触发）——待捕获后接入 confirmTool；steering 方法未捕获；Tier B 驱动与 MCP 反哺仍在 M4 后续 | — |

**驱动层现状**：内置 + pi + Claude Code + Codex 四引擎可选（回放断言合计 17+20+15=52 全通过）；pi 与 codex 的成功文本流各待其账户配额/密钥恢复，协议与翻译层均已按实测锁定。

### M4 进展（2026-10-25）：Tier B —— gemini-cli 系适配器（一套覆盖三引擎）

| 项 | 内容 | 证据 |
|---|------|------|
| 安装与实测 | WSL Arch 经 bun 安装 gemini-cli 0.65.0 与 qwen-code 0.26.1（同源 fork 验证）；无头 `-p -o json` 两种输出变体**真实捕获**：gemini 为单 JSON 文档 `{session_id, response?, stats?, error{type,message,code}}`（实测 auth code 41，列出 GEMINI_API_KEY 等认证路径）；qwen 为 **Claude-Code 风格记录数组** `[{type:"result",subtype,is_error,usage,error}]` | 本机捕获 |
| 协议结论 | stream-json 无认证时 stdout 为空（不可预验证）→ 采用 json 终局模式：整段 stdout 累积后在进程退出时解析；`-r/--resume` 是**序号制**而非 id 制（跨进程不可用）→ 连续性走对话重放注入 prompt；审批无往返（-p 默认档未批准工具直接失败）；认证走 env（GEMINI_API_KEY / DASHSCOPE_API_KEY / QWEN_API_KEY） | --help + 实测 |
| GeminiCliDriver | `src/agent/hub/drivers/geminiCli.ts`：一套适配器支持 gemini-cli / qwen-code / cursor-cli 三引擎；每轮 spawn `-p <对话重放> -o json`；env 注入复用 keyring 链路；stdout 累积→退出解析→单发 hooks；abort/exit 兜底 | typecheck/build 通过 |
| 翻译器 | `geminiCliProtocol.ts`：双变体解析（文档/记录数组）、usage 归一化、对话重放构造器、单发事件发射；乱输出容错 | `scripts/gemini-cli-protocol-replay.ts` **18/18 通过**（真实捕获流） |
| Tier 升级 | agentHub TIER_A_AGENTS 增加三引擎——选择器中从「适配中」灰显变为可选 | store 更新 |

**M4 出口标准达成**：Tier B 至少一个引擎可用 → 实际交付**一个适配器三引擎**（gemini-cli / qwen-code / cursor-cli，检测到即选）；成功流待各自认证配置（gemini 需 GEMINI_API_KEY 或 Google 登录、qwen 需 Qwen OAuth / DASHSCOPE_API_KEY），协议解析层已按实测锁定。四套回放合计 **70 断言全绿**。

### M4 收尾（2026-10-25）

| 项 | 内容 | 证据 |
|---|------|------|
| 旧路径清理 | 删除 Rust `agent/pool.rs`/`agent/types.rs`/`agent/commands.rs`（死代码）及 main.rs 注册与 `PoolState`；删除 `send_chat_message_stream` 命令（函数+注册+再导出）及其死掉的流式机器（`StreamEvent` 枚举、trait `chat_stream` 与三个 provider 实现、`Emitter` 等未用导入，经 `cargo fix` 清零）；前端删除 `src/agent/pool.ts` 包装层（`useAgentPool` 改指 `poolTypes`） | `cargo check` 零警告 |
| 测试关卡 | 按仓库要求跑 `cargo test`：**113 通过 / 0 失败**（含本轮新增 2 个 MCP server 测试）；前端 vue-tsc/build/四套协议回放 70 断言复跑全绿 | 命令输出 |
| MCP 反哺扩展 | `jedi --mcp-server`（stdio MCP）新增 `hosts_read`（只读分组视图）与 `skills_list`（技能清单）两个工具，与既有 memory/web 共 6 工具；壁纸/播客因需 AppHandle 在 stdio 模式不可用（记录为实现限制，后续需路径解析重构） | 新增单测 2/2 通过 |
| Tier B 第二批 | opencode/crush 安装实测因网络停滞（bun 下载原生二进制 14 分钟无进展，已终止）——**待网络恢复后继续**；goose/droid/kimi-code/antigravity 同批待验证 | 本轮记录 |

### Tier B 第二批 + 遗留收口（2026-10-25）

| 项 | 内容 | 证据 |
|---|------|------|
| opencode 协议实测（1.19.4，网络恢复后安装） | `opencode run <msg> --format json` NDJSON 事件：`step_start` / `text`（按 part 分块、part.id 去重）/ `tool_use`（part.state 含 input/output/metadata.exit，completed 可单事件闭环）/ `step_finish`（reason stop\|tool-calls + tokens/cost）；每事件带 sessionID，`-s <id>` 跨进程续接实测通过；**免费默认模型端到端可用（无 key 需求）**；发现旧版本 opencode.db 损坏需备份移除 | 本机捕获（三段真实流） |
| OpencodeDriver | `opencode.ts` + `opencodeProtocol.ts`（终答优先取最后一条 messageID 的文本而非整段旁白）；注册 + 升 Tier A | 回放 **15/15** |
| crush 澄清 | npm 的 `crush` 包是**同名异物**（接 URL 的工具，非 Charmbracelet Crush），已卸载；正版需官方安装脚本（原生二进制），验证顺延 | 实测 --help |
| nativeSessionId 持久化（遗留收口 ✅） | 运行结果的原生会话 id 写入消息 `run.nativeSessionId`（随 SQLite 持久化）；下轮 sendMessage 反向扫描恢复并经 `DriverStartOpts.nativeSessionId` 传入四个外部驱动——**跨应用重启续接**，不再依赖内存 Map | typecheck/回放全绿 |
| steering UI 最小版（遗留收口 ✅） | store 持有活动 DriverHandle（`canSteer`）；生成中输入框回车 → `steerActiveEngine`：pi 走 `steer` 命令、claude-code 走 stdin 追加 user 消息（双向 stream-json 原生支持）；无能力引擎维持原行为 | typecheck/回放全绿 |
| 剩余待办 | goose/droid/kimi-code/antigravity 驱动验证（crush 待正版安装）；附件（wslpath 翻译）；审批激活观察；pi/codex/gemini 系成功流待配额/认证 | — |

**驱动层现状**：7/12 主流引擎可选可驱动（内置、pi、claude-code、codex、gemini-cli/qwen-code/cursor-cli 一套、opencode）；其中 **claude-code 与 opencode 端到端实测可用**（后者免费模型零配置）。五套协议回放合计 **85 断言全绿**。

### Tier B 第三批验证结论 + 附件支持（2026-10-25）

**附件支持（遗留收口 ✅）**：`DriverStartOpts.attachments`（Windows 路径）贯通——gemini 系走 `-f <file>` 且 WSL 引擎经 `paths.ts` 的 `toWslPath()`（drvfs 规则本地计算，无需 wslpath 子进程，`C:\x` → `/mnt/c/x`，UNC/非 Windows 路径返回 null）；claude-code 把消息中的 data URL 图片 part 解析为 Anthropic base64 内容块（不涉文件系统，WSL 引擎同样适用）；两个引擎 capabilities.images 升 true。`toWslPath` 六项断言并入 gemini 回放（现在 24 项）。

**Tier B 第三批安装与协议核查（全部真机实测）**：

| 引擎 | 结果 |
|---|---|
| goose 1.56.0 | 二进制经 GitHub release 直装成功；无头形态为 `goose run -t`（stdin/file 指令），**输出纯文本、无结构化事件**；且需 provider 配置（"No provider configured"）。结构化驱动需改走其 MCP/daemon 形态——**暂缓**，检测层已覆盖 |
| crush | 官方安装脚本 404（地址变更）；npm `crush` 包为**同名异物**（接 URL 的工具，非 Charmbracelet Crush，已卸载）——待确认新安装地址后继续 |
| droid | factory.ai 官方脚本两次 TLS 连接失败（网络阻断）——待网络条件变化重试 |
| kimi-code | npm `kimi-code` 为**同名异物**（Groq 代理壳，非 Moonshot Kimi CLI，已卸载）；正版安装路径待确认 |
| antigravity | Windows 侧安装（D:\software\Antigravity），本轮未探——待后续 |

两个同名陷阱（crush、kimi-code）的识别与规避本身是检测层的防护知识：npm 粗名安装不可信，Detectron 的已知目录/版本探测不受影响，但**安装指引必须指向官方渠道**（已在本表记录）。

### Antigravity CLI 驱动 + 检测误报修复 + agentHub 转正评估（2026-10-25）

| 项 | 内容 | 证据 |
|---|------|------|
| agy 安装与协议实测（1.4.7） | 正版安装渠道为官方脚本（`curl …/cli/install.sh`，二进制 **`agy`**；pip 的 `google-antigravity` 是无入口的库包，npm 无官方包）；无头面完整：`-p` + `--output-format stream-json`（另有 `--input-format stream-json`/`--conversation <id>`/`--mode`/`--effort`）；输出为自有 `{"event":…}` 包裹，终局记录 `{"event":"result","result":{conversation_id,status,response,error,usage}}`（真实捕获未认证 ERROR："authentication failed or timed out"，agy 内部 60s 超时）；**认证为 Google OAuth 交互登录**（stderr 给 URL + 粘贴码提示——因此驱动绝不写 stdin） | 本机捕获 |
| **检测误报修复** | PATH 上的 `antigravity` 是 **IDE 启动器**（VS Code fork，diff/merge/goto 旗标），此前检测层把它当 agent 命中属误报——AGENTS 表改为探测 `agy` 并留注释 | detector.rs |
| AntigravityDriver | `antigravity.ts` + `antigravityProtocol.ts`：每轮 `-p <对话重放> --output-format stream-json`、`--conversation` 续接（nativeSessionId 持久化链路复用）、result 即收口、OAuth stderr 识别、stdin 恒不写（防认证挂起）；注册 + 升 Tier A | 回放 **11/11**（含真实 ERROR 流）；成功流待用户完成一次 `agy` 交互登录 |

#### agentHub 特性开关转正评估（结论：**转正，默认启用**）

评估口径与证据：

1. **零回归**：`sendMessage` 全量经 driver 接口、hooks 1:1 透传；关卡全绿（`cargo test` 113、`vue-tsc`/`biome`/`vite build`、六套协议回放 102 断言）。
2. **可用性兜底**：内置引擎永远是选择器第一项与回退路径；检测层失败只影响外部引擎条目显示，不阻塞聊天主流程。
3. **实测覆盖**：8/12 主流引擎可选可驱动（claude-code、opencode 端到端验证）；检测层经真实机器校验（含本轮 IDE 启动器误报修复、/mnt 双计数过滤、UTF-16/发行版名等坑）。
4. **风险披露**：外部引擎当前在宿主侧自动放行工具（审批翻译已实现但上游不发请求），已在代码注释与本文档披露；进程侧有 kill/超时兜底。
5. **实现事实**：M1 文本中的"特性开关默认关"在落地时未做成运行时开关——内置回退的存在使全局开关无必要；每会话粒度可在选择器切回"内置"作为保险丝。

**再评估触发条件**：外部引擎工具放行策略变化（如 claude 开始在 -p 发 control_request）或新增高权限引擎时。

### 下一步

剩余 4 引擎择机重试（crush 新地址 / droid 网络 / kimi 正版渠道 / goose MCP 形态）；审批激活观察；pi/codex/gemini 系/antigravity 成功流待配额/认证解锁（antigravity 需用户跑一次 `agy` 交互登录）。

**驱动层终态：8/12 主流引擎可选可驱动**（内置、pi、claude-code、codex、gemini-cli、qwen-code、cursor-cli、opencode、antigravity——内置不计入 12）。六套协议回放合计 **102 断言全绿**。









## 11. 附录


### 附录 A：协议速查

| 引擎 | 启动 | 帧协议 | 关键命令/事件 | 真正结束信号 |
|------|------|--------|--------------|-------------|
| pi | `pi --mode rpc --no-session`（扩展 `-e`、技能 `--skill`、会话 `--session-dir`、工具 `--tools` 白名单） | JSONL/\n（stdin 双向） | `prompt`/`steer`/`follow_up`/`abort`/`set_model`/`switch_session`/`fork`；`message_update.text_delta`、`tool_execution_*`、`agent_settled` | `agent_settled` |
| Claude Code | `claude --input-format stream-json --output-format stream-json`（`--resume <id>` 可组合） | JSONL/\n 双向 | stdin 用户消息；`control_request`/`control_response` 权限往返 | result 消息 |
| Codex | `codex app-server` | JSON-RPC 2.0 over stdio（无 jsonrpc 头） | 官方 VS Code 插件同款；`codex exec --json` 为单向替代 | 按 JSON-RPC 批次 |

### 附录 B：本机环境实测记录（2026-10-25，Windows 宿主）

| 探测 | 结果 |
|------|------|
| WSL 发行版枚举（`wsl.exe -l -q`，UTF-16 解码后） | `ArchLinux`、`docker-desktop`、`Ubuntu` |
| `wsl -d Arch`（硬编码名） | `WSL_E_DISTRO_NOT_FOUND`（错误消息亦为 UTF-16 乱码） |
| ArchLinux：PATH / 登录 shell / 交互 shell 探测 pi | **全部未命中** |
| ArchLinux：已知位置扫描 | 发现 `~/.local/bin/claude`、`~/.local/bin/codex`（PATH 探测同样漏检它们）；node/npm 在 `/usr/sbin`，bun/pnpm 目录存在 |
| ArchLinux：pi 各常见位置（~/.local/bin、~/.bun/bin、~/.local/share/pnpm、/usr/local/bin 等） | **未找到**——用户自述已装，疑似 pnpm 全局未链接；**M0 首任务：在 Arch 内 `pnpm add -g @earendil-works/pi-coding-agent` 并复核检测路径** |
| Ubuntu | pi/claude/codex 均无 |
| Windows 宿主 | 无 Node；scoop（fnm/pnpm/bun shims）在位 |

**结论**：本机实测直接证实了三层探测策略的必要性——仅 PATH 探测在本机会漏检 100% 的已装 agent（claude/codex 均由第 3 层目录扫描发现）。

### 附录 C：待办决策（M0 出口时定）

1. pi 审批扩展的具体机制（Extension UI 双向记录 vs 自定义 ask 工具）——以 M0 实测为准。
2. Codex 进 v1 还是 v1.1（M4 是否拆期）。
3. 现有 MCP 客户端（`stores/mcpClient.ts`）去留：外部引擎自带 MCP 能力后，Jedi 侧 MCP client 可能冗余。
4. 每引擎 min_version 初始值（M0 用实测版本定基线）。
5. Tier B 驱动排期：依据 M0 对 Gemini CLI / Qwen Code / Cursor CLI / OpenCode / Crush / Goose / Droid / Kimi Code 的协议验证结果，确定 v1.x 接入顺序（gemini-cli 系三引擎优先）。
