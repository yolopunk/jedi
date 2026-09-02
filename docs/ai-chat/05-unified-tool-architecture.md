# 统一工具架构设计（Unified Tool Architecture）

**日期**: 2026-07-01
**状态**: 设计中
**版本**: 1.2（已全量实现）
**关联**: `docs/superpowers/specs/2026-04-06-agent-architecture-design.md`、`src-tauri/src/api/ai_chat/agent.rs`、`src-tauri/src/mcp/`

> **v1.2 实现状态**：P1–P4 全部落地并合入分支 `claude/agent-capabilities-analysis-c8fyns`。
> 第三方 MCP 支持三种传输（stdio / HTTP+SSE / Streamable HTTP），三条链路均有真机端到端测试；
> 对外 server 已实现"只读默认 + 写白名单"；§6.1 相关性子集注入已实现。
> 唯一按设计**主动推迟**的是 §14-2 的 OAuth（见该条）。
>
> **v1.1 评审修订**：
> - R1 工具名改为下划线命名（function calling 强制 `^[a-zA-Z0-9_-]{1,64}$`，点号/冒号非法）
> - R2 新增 §7.1「可挂起回路」，确认机制采用后端挂起方案
> - Y1 新增 §7.2「快照一致性」，`dry_run` 产出快照 token，`call` 校验
> - Y2 `AgentTool` 增 `dynamic_risk` 钩子（按参数升级风险）
> - Y3 新增 §6.1「工具子集注入」，应对工具膨胀
> - Y4 回滚改为 per-turn undo 栈
> - §14 开放问题按评审推荐收敛

---

## 0. 背景与核心决策

### 0.1 问题

当前 Agent 工具调用（`agent.rs`）通过 `HostsMcpServer` 派发。但这层其实是**伪 MCP**——它没有走 JSON-RPC / stdio 传输，而是进程内直接函数调用，只是借用了 MCP 的数据结构（`Tool` / `CallToolResult`）。结果是既没享受 MCP 的跨进程能力，又背上了协议类型与 `initialize` 状态机的包袱。

同时，"让 Agent 能操作整个产品"这个 AI Native 目标要求把 Hosts / 壁纸 / 播客 / 系统信息都变成 Agent 可调的工具。如果全部套 MCP，等于给同进程内的函数调用中间架一个 JSON-RPC 交换机，是过度设计。

### 0.2 决策：一个抽象，三种来源

**Agent 只依赖一个统一的 `AgentTool` 抽象；MCP 只是这个抽象的一个来源/出口，不是唯一入口。**

| 层 | 定位 | 用途 | 是否走 MCP 协议 |
|----|------|------|:---:|
| **主力** | 内置工具（Native） | Jedi 自身功能：hosts/壁纸/播客/系统 | ❌ 进程内直调 |
| **可选** | MCP 客户端（Client） | 接入**第三方** MCP server，扩展能力边界 | ✅ |
| **战略** | MCP 服务端（Server） | 把 Jedi 能力**对外**暴露给 Claude Desktop / Cursor 等 | ✅ |

三层共享同一个工具声明格式与风险分级模型，Agent 回路对来源完全无感。

---

## 1. 目标与非目标

### 目标
- 定义统一的 `AgentTool` trait 与 `ToolRegistry`，作为 Agent 回路唯一的工具入口
- 把现有 Hosts/壁纸/播客/系统 API 以最小成本包装为内置工具
- 设计第三方 MCP server 的接入：配置、连接生命周期、工具注入、安全边界
- 设计 Jedi 作为 MCP server 对外暴露的机制
- 工具风险分级 + 可挂起的人机确认 + 可回滚

### 非目标（本文档不覆盖）
- Agent Loop 的规划/反思升级（见后续 Agent Loop 设计）
- 记忆系统
- 全局唤起入口（Cmd+K）

---

## 2. 总体架构

```
                    ┌─────────────────────────────┐
                    │        Agent Loop           │
                    │   (agent.rs 工具调用回路)    │
                    └──────────────┬──────────────┘
                                   │ 只认识 AgentTool 抽象
                    ┌──────────────▼──────────────┐
                    │        ToolRegistry          │  统一注册表
                    │  declarations() / call()     │  命名空间 / 风险分级
                    └──┬───────────┬────────────┬──┘
          ┌────────────▼──┐  ┌─────▼──────┐  ┌──▼──────────────────┐
          │ NativeTool     │  │ McpClient  │  │  McpServer(对外)     │
          │ (主力/进程内)  │  │ Tool(可选) │  │  把 registry 中      │
          │                │  │            │  │  exportable 的工具   │
          │ HostsTool      │  │ 远程 server │  │  通过 MCP 暴露       │
          │ WallpaperTool  │  │ 的工具翻译  │  └─────────────────────┘
          │ PodcastTool    │  │ 为 AgentTool│         │
          │ SystemTool     │  └─────┬──────┘         │ stdio / SSE
          └───────┬────────┘        │                ▼
                  │ 直接复用         │ StdioTransport / SseTransport
                  ▼                  ▼
          api/hosts.rs 等      第三方 MCP server 进程
```

---

## 3. 核心抽象

### 3.1 工具声明与结果

```rust
/// 工具的风险等级 → 驱动确认策略
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// 只读：读取 hosts、列播客、查系统信息 — 默认自动执行
    Read,
    /// 写入：改 hosts、设壁纸、增删订阅 — 默认 diff 预览后确认
    Write,
    /// 系统级/危险：需要提权、不可逆 — 强制二次确认
    System,
}

/// 工具来源
#[derive(Clone, Serialize, Deserialize)]
pub enum ToolSource {
    Native,
    /// 第三方 MCP server（携带 server_id 与远端原始工具名，供 UI/审计）
    Mcp { server_id: String, remote_name: String },
}

/// 工具声明（喂给 LLM，也用于 UI 展示）
pub struct ToolDeclaration {
    /// 【R1】喂给 LLM 的名字，必须匹配 ^[a-zA-Z0-9_-]{1,64}$
    /// 内置：hosts_add / wallpaper_set / system_info
    /// 第三方：mcp_<server_id>_<tool>，如 mcp_github_create_issue
    pub name: String,
    pub description: String,
    /// 标准 JSON Schema（object）
    pub input_schema: Value,
    /// 静态风险等级（可被 dynamic_risk 按参数升级，见 §3.2）
    pub risk: RiskLevel,
    pub source: ToolSource,
    /// UI 分组标签（如 "Hosts"/"Filesystem"），仅展示用，不进 LLM
    pub group: String,
}

/// 改动预览（用于确认 UI，并锁定一致性快照）
pub struct ToolPreview {
    /// 人类可读 diff（如 hosts 变更前后）
    pub diff: String,
    /// 【Y1】一致性快照 token：dry_run 时对目标资源打快照，
    /// call 执行前校验资源未被外部改动，变了则要求重新确认
    pub snapshot_token: String,
}

/// 工具执行结果
pub struct ToolOutcome {
    pub content: String,       // 回填给 LLM 的文本
    pub is_error: bool,
    /// 可选：回滚句柄（写前快照 id）。仅可逆工具提供（见 §7.3）
    pub undo_token: Option<String>,
}
```

### 3.2 AgentTool trait

```rust
#[async_trait]
pub trait AgentTool: Send + Sync {
    fn declaration(&self) -> ToolDeclaration;

    /// 【Y2】按实际参数动态升级风险。默认返回声明里的静态风险。
    /// 例：hosts_add 命中系统关键域名(update.microsoft.com 等) → 升级为 System
    fn dynamic_risk(&self, _args: &Value) -> RiskLevel { self.declaration().risk }

    /// 【Y1】干跑：只计算改动预览 + 打一致性快照，不落地。
    /// Read 类工具无需实现（返回 None）。
    async fn dry_run(&self, _args: &Value) -> Result<Option<ToolPreview>, String> { Ok(None) }

    /// 真正执行。expected_snapshot 为确认时锁定的快照 token；
    /// 实现需校验资源未变（不一致则返回错误，触发重新确认）。
    async fn call(&self, args: Value, expected_snapshot: Option<String>) -> ToolOutcome;

    /// 回滚（仅可逆工具实现）
    async fn undo(&self, _undo_token: &str) -> Result<(), String> { Err("不支持回滚".into()) }
}
```

### 3.3 ToolRegistry

```rust
pub struct ToolRegistry {
    tools: RwLock<HashMap<String, Arc<dyn AgentTool>>>,   // key = LLM 工具名
}

impl ToolRegistry {
    pub fn register(&self, tool: Arc<dyn AgentTool>);     // 重名拒绝并记审计
    pub fn unregister(&self, name: &str);                 // 断开某 MCP server 时批量移除
    pub fn declarations(&self, filter: &ToolFilter) -> Vec<ToolDeclaration>;  // 见 §6.1
    pub async fn call(&self, name: &str, args: Value, snapshot: Option<String>) -> ToolOutcome;
    pub fn get(&self, name: &str) -> Option<Arc<dyn AgentTool>>;
}
```

`ToolRegistry` 作为 Tauri 托管状态（`.manage(...)`），供 `agent_chat` 与配置命令共享。

---

## 4. 三层来源设计

### 4.1 主力：内置工具（Native）

直接**复用现有 API 函数**，包一层 trait 即可，不碰 MCP。

| 工具名（LLM 名） | 底层复用 | 风险 |
|------|------|------|
| `hosts_read` / `hosts_list` | `api::hosts::read_system_hosts` | Read |
| `hosts_add` / `hosts_remove` / `hosts_toggle` | `update_hosts_with_groups` | Write（命中关键域名→System）|
| `hosts_revert` | `api::hosts::revert_hosts` | Write |
| `wallpaper_list` / `wallpaper_current` | `get_wallpapers` / `get_current_wallpaper` | Read |
| `wallpaper_set` | `set_desktop_wallpaper` | Write |
| `podcast_subscriptions` / `podcast_episodes` | `get_subscriptions` / `fetch_episodes` | Read |
| `podcast_subscribe` / `podcast_unsubscribe` / `podcast_import_opml` | `save_subscription` 等 | Write |
| `system_info` | `api::os::get_os_info` | Read |

示例（把现有 hosts 逻辑包进 trait）：

```rust
pub struct HostsAddTool;

#[async_trait]
impl AgentTool for HostsAddTool {
    fn declaration(&self) -> ToolDeclaration {
        ToolDeclaration {
            name: "hosts_add".into(),
            description: "添加一条 hosts 记录到指定分组".into(),
            input_schema: json!({ "type":"object",
                "properties": { "ip":{"type":"string"}, "domain":{"type":"string"}, "group":{"type":"string"} },
                "required": ["ip","domain","group"] }),
            risk: RiskLevel::Write,
            source: ToolSource::Native,
            group: "Hosts".into(),
        }
    }
    fn dynamic_risk(&self, args: &Value) -> RiskLevel {
        // 命中系统关键域名列表 → RiskLevel::System，否则 Write
    }
    async fn dry_run(&self, args: &Value) -> Result<Option<ToolPreview>, String> {
        // 读当前 hosts → 计算 diff → snapshot_token = 内容哈希，不写入
    }
    async fn call(&self, args: Value, expected: Option<String>) -> ToolOutcome {
        // 若 expected 与当前 hosts 哈希不一致 → 报错要求重新确认
        // 否则复用 api::hosts 写入逻辑，返回 undo_token（写前快照）
    }
    async fn undo(&self, token: &str) -> Result<(), String> { /* 还原快照 */ }
}
```

> `agent.rs` 现有的 `HostsMcpServer` 派发切换为 `ToolRegistry`。`src-tauri/src/mcp/servers/hosts.rs` 的工具定义逻辑迁移进 `HostsTool`，MCP 的 `types/transport/protocol` 保留给第 4.2/4.3 节使用。

### 4.2 可选：MCP 客户端（接入第三方 MCP server）★ 重点

让 Jedi 能像 Claude Desktop 一样接入社区/自建的 MCP server（filesystem、git、browser、数据库…），把它们的工具注入 `ToolRegistry`。

#### 配置格式（对齐 Claude Desktop 习惯）

```jsonc
// 存储于 tauri-plugin-store，例如 mcp-servers.json
{
  "servers": [
    {
      "id": "filesystem",
      "name": "Filesystem",
      "transport": "stdio",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/Users/me/projects"],
      "env": { "FOO": "bar" },
      "enabled": true,
      "autoApprove": []          // 免确认的工具白名单（默认空）
    },
    {
      "id": "remote-tools",
      "transport": "sse",
      "url": "https://example.com/mcp",
      "headers": { "Authorization": "Bearer ..." },
      "enabled": false
    }
  ]
}
```

#### 传输层
- **stdio**：已有 `StdioTransport`（`command`/`args`/`startup_timeout`），启动子进程，行分隔 JSON-RPC。
- **SSE / HTTP**（新增）：连接远程 MCP server，需新增 `SseTransport`。见 §14-G1 的协议版本说明。

#### 连接生命周期

```
用户启用 server
  → 按 transport 启动/连接
  → initialize 握手（协议版本 2024-11-05，交换 capabilities）
  → tools/list 拉取工具清单
  → 每个远程工具包装为 McpClientTool，注册进 ToolRegistry
     （LLM 名 = mcp_<server_id>_<tool>，避免重名；原始名存 ToolSource）
  → 运行期：tools/call 派发；监听 notifications/tools/list_changed 动态刷新
  → 断线：指数退避重连；期间对应工具从 registry 注销并在 UI 标灰
  → 用户禁用/退出：发送关闭，注销工具，回收进程
```

```rust
/// 把一个远程 MCP 工具适配为 AgentTool
pub struct McpClientTool {
    decl: ToolDeclaration,         // name=mcp_<id>_<tool>, source=Mcp{..}, risk 默认 Write（见 §7）
    client: Arc<McpClient>,        // 复用 mcp::protocol::McpClient
}

#[async_trait]
impl AgentTool for McpClientTool {
    fn declaration(&self) -> ToolDeclaration { self.decl.clone() }
    async fn call(&self, args: Value, _snapshot: Option<String>) -> ToolOutcome {
        // client.call_tool(remote_name, args) → 映射 CallToolResult → ToolOutcome
        // 第三方工具无 dry_run/快照，一律靠 §7 确认兜底
    }
}
```

#### 风险与信任
第三方 MCP server = **运行任意代码 / 连接任意网络**，是重大信任边界。默认策略见 §7、§8。

### 4.3 战略：MCP 服务端（Jedi 对外暴露）

把 `ToolRegistry` 中标记 `exportable=true` 的**内置**工具，通过 MCP 协议对外提供，让 Claude Desktop / Cursor 等能调用 Jedi 管理 hosts、设壁纸。

- 复用 `mcp::protocol` / `types`，实现 server 侧 `initialize` / `tools/list` / `tools/call`。
- 传输：stdio（Jedi 以 `--mcp-server` 子命令启动一个纯 server 进程），后续可选本地 SSE 端口。
- 仅导出内置工具（不转发第三方工具，避免代理放大信任问题）。
- 对外暴露的写操作在无头模式下的确认策略见 §14-1（已决：只读导出 + 写操作须在 autoApprove 白名单）。

这层可最后做，但抽象上从一开始就用 `exportable` 标志预留。

---

## 5. 命名空间与冲突

- 【R1】喂给 LLM 的工具名**必须**匹配 `^[a-zA-Z0-9_-]{1,64}$`（OpenAI/Anthropic 共同约束），因此一律用下划线，不能用点号/冒号：
  - 内置：`hosts_add`、`wallpaper_set`、`system_info`
  - 第三方：`mcp_<server_id>_<tool>`，如 `mcp_github_create_issue`
- 结构化来源（server_id、远端原始名、UI 分组）存在 `ToolSource` / `ToolDeclaration.group`，只用于展示与审计，不进 LLM 请求。
- Trace 面板按 `group + 原始名` 展示。
- `ToolRegistry.register` 遇同名拒绝并记审计；UI 提示冲突（第三方重名时可加 `_2` 后缀消歧）。

---

## 6. Provider 适配

工具声明 → 各家 function calling 格式的转换**已在 `agent.rs` 实现**（`tool_to_openai` / `tool_to_anthropic`），改为吃 `ToolDeclaration` 即可复用：

- OpenAI/兼容：`{type:"function", function:{name, description, parameters:input_schema}}`
- Anthropic：`{name, description, input_schema}`
- 模型能力探测：`fetch_models_dev` 返回的 `tool_call` 标志已可用——**若所选模型不支持 function calling，则禁用工具注入并在 UI 提示**（降级为纯聊天），避免对小模型硬塞工具。

### 6.1 工具子集注入【Y3】

接入多个第三方 server 后，工具可能达几十上百个。**全量注入会推高 token 成本、并让模型更容易选错工具**——这是 AI Native 的真实瓶颈，必须处理。分阶段策略：

- **P2 基础版（先做）**：`ToolFilter` 按"已启用的来源"过滤——只注入用户在该会话启用的内置分组 + 启用的 MCP server 的工具。默认关闭全部第三方工具，用户按需开。
- **P3 进阶（工具多时）**：按功能页上下文预选（在 Hosts 页默认只注入 hosts_* 组）、或对工具描述做轻量语义检索，每轮只注入 top-K 相关工具。
- 无论哪种，**注入了哪些工具**都通过 `agent-event` 上报，Trace 面板可见，避免"静默裁剪"。

```rust
pub struct ToolFilter {
    pub enabled_groups: Vec<String>,     // 内置分组
    pub enabled_servers: Vec<String>,    // MCP server id
    pub max_tools: Option<usize>,        // P3 语义检索上限
    pub query: Option<String>,           // P3 相关性检索
}
```

---

## 7. 风险分级与人机确认

统一由 `RiskLevel`（经 `dynamic_risk` 按参数升级后）驱动，对齐"分级确认"策略：

| 风险 | 默认行为 |
|------|---------|
| `Read` | 自动执行，无需确认 |
| `Write` | 先 `dry_run` 生成 **diff 预览 + 快照**，用户确认后 `call`（带快照校验）|
| `System` | 强制二次确认 + 显著警示 |

- 每个 server/工具支持 `autoApprove` 白名单（用户显式免确认）。
- 全局确认模式 `auto | always | dangerous`（对齐设计文档 `AgentConfig.confirmationMode`）。
- **第三方 MCP 工具默认风险 = Write（至少需确认）**，因为无法信任其自报的安全性。用户可手动降级为 Read，但 UI 须二次警告（§14-G2）。

### 7.1 可挂起回路（确认机制的执行模型）【R2，方案 A】

确认要求 Agent 回路能"跑到一半停下、等前端确认、再恢复"。现有 `agent_chat` 是一次性 async command，跑完 `MAX_ITERATIONS` 才返回，不支持中途挂起。**采用后端挂起方案**：

- `agent_chat` 从"跑完即返回"改为**长驻任务**：回路在后端持续运行，通过 `agent-event-{request_id}` 与前端交互，最终以事件通知完成，命令本身可立即返回一个 `request_id`。
- 托管一张挂起表：

```rust
pub struct PendingConfirmations {
    map: Mutex<HashMap<String, oneshot::Sender<ConfirmDecision>>>,  // key = call_id
}
pub enum ConfirmDecision { Approve { edited_args: Option<Value> }, Reject }
```

- 回路遇到 Write/System 工具时：
  1. 调 `dry_run` 拿 `ToolPreview`
  2. 发 `agent-event` = `ConfirmRequest { call_id, tool, args, diff }`
  3. 注册一个 `oneshot`，`.await` 它（带**超时**，默认 120s，超时按 Reject 处理）
  4. 前端弹确认卡片，用户选择后调 `tool_confirm(request_id, call_id, approve, edited_args?)` 命令 → 后端 `send` 唤醒 oneshot
  5. Approve → 用 `edited_args`（若有）+ 锁定快照执行 `call`；Reject → 把"用户拒绝"作为工具结果回填给 LLM，回路继续
- **取消**：新增 `agent_cancel(request_id)`，丢弃挂起表项并中断回路。
- **清理**：回路结束/出错/取消时清空该 `request_id` 的所有挂起项，避免泄漏。

### 7.2 快照一致性【Y1】

`dry_run` 与 `call` 是两次独立访问资源，之间资源可能被外部改动（如用户手动编辑了 hosts）。因此：

- `dry_run` 对目标资源打**快照 token**（如 hosts 文件内容哈希），随 `ToolPreview` 一并送确认 UI。
- 用户确认后，`call(args, expected_snapshot)` 执行前**校验当前资源快照 == expected**：
  - 一致 → 正常写入
  - 不一致 → 返回特定错误，回路重新 `dry_run` + 重新确认（并提示"目标已被外部修改"）

### 7.3 回滚（per-turn undo 栈）【Y4】

- 每个可逆工具 `call` 成功后返回 `undo_token`（写前快照）。
- 回路为**本回合**维护一个 undo 栈（按执行顺序压栈）。
- UI 提供两级撤销：单步（撤某一次工具调用）与整回合（逆序弹栈全撤）。
- 不可逆工具（已发网络请求、已删除的外部资源等）不提供 `undo_token`，UI 标注"不可撤销"。

---

## 8. 第三方 MCP 的安全模型

第三方 server 是不可信代码，必须有边界：

1. **显式授权**：添加/启用 server 时，明确告知"将运行外部程序/连接外部服务"，需用户确认。
2. **工具级确认**：默认所有第三方工具走确认（§7），除非加入 autoApprove。
3. **审计全链路**：连接、initialize、每次 tools/call（工具名+参数摘要+结果状态）写入现有 `audit_log`。
4. **凭证隔离**：server 的 env/headers（可能含 token）经 keyring 存储，不落明文配置。
5. **资源约束**：启动超时、调用超时、输出大小上限、并发上限；子进程崩溃隔离不影响主应用。
6. **（可选/后续）沙箱**：stdio 子进程可加系统级沙箱（macOS sandbox-exec / Linux namespaces）。

---

## 9. 配置与持久化

- MCP server 配置：`tauri-plugin-store`（`mcp-servers.json`）+ 敏感字段走 keyring。
- 内置工具启用状态、确认模式、autoApprove、工具子集偏好：并入现有 chat 设置。
- `ToolRegistry` 为运行期状态，应用启动时：注册全部内置工具 → 读取 MCP 配置 → 连接 enabled 的 server。

---

## 10. 前端设计

### 10.1 MCP Server 配置界面（新）
`settings/ChatSettingsTab` 现有 MCP 占位开关升级为完整管理：
- server 列表：名称 / 传输方式 / 连接状态（绿=已连/黄=连接中/红=失败/灰=禁用）/ 工具数
- 添加/编辑：transport（stdio/sse）、command/args/env 或 url/headers、enabled、autoApprove
- "测试连接"按钮：initialize + tools/list 预览
- 每个 server 可展开查看其工具（名称/描述/风险徽标）

### 10.2 工具浏览器（新，轻量）
统一列出 `ToolRegistry` 全部工具（内置 + 各 MCP server），带来源、分组与风险徽标；供用户了解 Agent 能做什么、按需启用/禁用（驱动 §6.1 的 `ToolFilter`）。

### 10.3 确认卡片（新）
Write/System 工具执行前，在对话流中插入确认卡片：工具名 + 参数（可编辑）+ **diff 预览** + 风险徽标 + [批准]/[拒绝]/[编辑参数]/[本会话免确认]；超时倒计时可见。执行后若可撤销，卡片提供 [撤销] 入口（对接 §7.3 undo 栈）。

### 10.4 Trace 面板增强（已存在，扩展）
`AgentTrace.vue` 增加：工具来源徽标（native / server 名）、风险色、确认状态（待确认/已批准/被拒/超时）、本回合已注入工具数、回滚入口。

---

## 11. Tauri 命令接口清单（新增）

```
# 工具注册表
tool_list_all() -> Vec<ToolDeclaration>            # 所有已注册工具（供工具浏览器）
agent_chat(...) -> request_id                       # 改为长驻任务，从 ToolRegistry 按 ToolFilter 取工具
tool_confirm(request_id, call_id, approve, edited_args?)  # 唤醒挂起的工具调用（§7.1）
agent_cancel(request_id)                            # 取消回路，清理挂起项
tool_undo(undo_token)                               # 单步回滚
turn_undo(request_id)                               # 整回合逆序回滚（§7.3）

# MCP server 管理
mcp_server_list() -> Vec<McpServerStatus>
mcp_server_upsert(config)
mcp_server_remove(id)
mcp_server_connect(id)                              # 手动连接/重连
mcp_server_disconnect(id)
mcp_server_test(config) -> Vec<ToolDeclaration>     # 测试连接并预览工具

# MCP server（对外，战略层，后置）
# 通过 `jedi --mcp-server` 子命令启动，无需前端命令
```

---

## 12. 与现有代码的迁移路径

1. 新增 `src-tauri/src/tools/`：`mod.rs`(trait+registry)、`native/`(hosts/wallpaper/podcast/system)、`mcp_client.rs`(适配器)、`confirm.rs`(PendingConfirmations)。
2. `agent.rs`：`collect_tools` / `call_server_tool` 改为走 `ToolRegistry`；工具声明转换函数改吃 `ToolDeclaration`；回路改为长驻任务 + 确认挂起；扩展 `agent-event` 增确认/回滚事件。
3. `mcp/`：`types`/`protocol`/`transport` 保留；新增 `SseTransport`；`servers/hosts.rs` 的定义迁移进 `tools/native/hosts.rs` 后可删或转为"对外导出"用。
4. `main.rs`：`.manage(ToolRegistry)`、`.manage(PendingConfirmations)`；注册新命令；启动时加载 MCP 配置。
5. 前端：`api/ai-chat.ts` 增补命令封装；`stores/aiChat` 增 MCP server 管理 + 确认状态；新增配置 UI、工具浏览器、确认卡片。

现有单测（`agent::tests`）随派发层调整更新；新增 registry / native tool / mcp client 适配 / 快照校验 / 确认唤醒的单测。

---

## 13. 分阶段实施

| 阶段 | 内容 | 产出 |
|------|------|------|
| **P1 抽象落地** ✅ | `AgentTool` + `ToolRegistry`；hosts 迁移为内置工具；`agent.rs` 切换派发（命名改下划线）| Agent 走统一抽象，行为不变、更干净 |
| **P2 全产品工具化 + 确认** ✅ | 壁纸/播客/系统/记忆内置工具；动态风险 + dry_run/快照 + 可挂起回路 + 确认卡片 + per-turn 回滚；§6.1 工具过滤 | Agent 能**安全操作整个产品**（AI Native 内核）|
| **P3 第三方 MCP 客户端** ✅ | stdio + HTTP+SSE + Streamable HTTP 三种传输、server 配置 UI、连接生命周期、工具注入、安全边界、§6.1 相关性子集注入 | 接入社区 MCP 生态，能力可扩展 |
| **P4 对外 MCP 服务端** ✅ | `--mcp-server` 导出内置工具（只读默认 + `--allow-write` 白名单）| Jedi 成为 MCP 生态节点 |

> 另有超出原计划的增强：模型能力探测降级、全流式（OpenAI/Anthropic）、Agent 系统提示、
> 跨会话记忆工具、全局命令台（应用内 Cmd/Ctrl+J + 系统级 Cmd/Ctrl+Shift+J）。

---

## 14. 开放问题（v1.1 收敛 / v1.2 实现结果）

1. **对外 server 的无头确认** → ✅ 已决：**只导出只读工具；写操作必须在 autoApprove 白名单，否则拒绝**。避免无人值守时被外部 Agent 改系统。
2. **SSE/HTTP MCP 的鉴权** → ✅ 已决：**P3 先只支持静态 token/header，OAuth 后置**。
3. **工具过多时的选择** → ✅ 已提升为正文 §6.1（P2 基础过滤 + P3 上下文/语义预选）。
4. **回滚的通用性** → ✅ 已决：**仅可逆工具提供 `undo_token`，并按回合成栈**（§7.3）；不可逆工具 UI 明确标注。
5. **G1 SSE 协议版本** → ✅ **已双协议兼容**：同时实现 `2024-11-05` 的 HTTP+SSE 与 `2025-03-26` 的
   Streamable HTTP（单端点，兼容 `application/json` 与 `text/event-stream` 两种应答），各自有 mock server 端到端测试。
6. **（保留）G2 第三方工具降级为 Read 的误操作**：当前**尚未提供**把第三方工具风险降级为 Read 的入口，
   因此该误操作路径不存在；若日后开放该能力，需二次警告并记审计。

### 实现补充

- **§14-2 OAuth 仍按原决策推迟**：远程传输（SSE / Streamable HTTP）已支持自定义请求头，
  静态 token / `Authorization: Bearer` 可用，满足当前接入需求。完整 OAuth 2.1（授权服务器元数据发现、
  动态客户端注册、PKCE、浏览器回调）需要真实授权服务器才能验证，属于按需再做的独立议题。
- **Agent 回路的集成测试**：目前覆盖到确定性单元（流式累加器、工具选择、确认分级、回滚栈）与
  三条 MCP 传输的端到端测试。若要对 `run_openai_loop` / `run_anthropic_loop` 整体做集成测试，
  需要把这两个函数改为对 Tauri `Runtime` 泛型（以便用 `tauri::test::mock_app`），属于为可测性
  改动已上线代码，建议单独评估后再做。
