# 统一工具架构设计（Unified Tool Architecture）

**日期**: 2026-07-01
**状态**: 设计中
**版本**: 1.0
**关联**: `docs/superpowers/specs/2026-04-06-agent-architecture-design.md`、`src-tauri/src/api/ai_chat/agent.rs`、`src-tauri/src/mcp/`

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
- 工具风险分级，驱动统一的人机确认

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

/// 工具声明（喂给 LLM，也用于 UI 展示）
pub struct ToolDeclaration {
    /// 完全限定名，带命名空间，如 "hosts.add" / "mcp:github.create_issue"
    pub name: String,
    pub description: String,
    /// 标准 JSON Schema（object）
    pub input_schema: Value,
    pub risk: RiskLevel,
    /// 来源：native / mcp:<server_id>
    pub source: ToolSource,
}

/// 工具执行结果
pub struct ToolOutcome {
    pub content: String,       // 回填给 LLM 的文本
    pub is_error: bool,
    /// 可选：结构化改动预览（用于确认 UI / 回滚）
    pub preview: Option<ToolPreview>,
    /// 可选：回滚句柄（快照 id 等）
    pub undo_token: Option<String>,
}
```

### 3.2 AgentTool trait

```rust
#[async_trait]
pub trait AgentTool: Send + Sync {
    fn declaration(&self) -> ToolDeclaration;

    /// 干跑：只计算改动预览，不落地（用于确认 UI）。Read 类工具可不实现。
    async fn dry_run(&self, args: &Value) -> Result<Option<ToolPreview>, String> { Ok(None) }

    /// 真正执行
    async fn call(&self, args: Value) -> ToolOutcome;
}
```

### 3.3 ToolRegistry

```rust
pub struct ToolRegistry {
    tools: RwLock<HashMap<String, Arc<dyn AgentTool>>>,   // key = 完全限定名
}

impl ToolRegistry {
    pub fn register(&self, tool: Arc<dyn AgentTool>);
    pub fn unregister(&self, name: &str);                 // 断开某 MCP server 时批量移除
    pub fn declarations(&self, enabled: &[String]) -> Vec<ToolDeclaration>;
    pub async fn call(&self, name: &str, args: Value) -> ToolOutcome;
}
```

`ToolRegistry` 作为 Tauri 托管状态（`.manage(...)`），供 `agent_chat` 与配置命令共享。

---

## 4. 三层来源设计

### 4.1 主力：内置工具（Native）

直接**复用现有 API 函数**，包一层 trait 即可，不碰 MCP。

| 工具（命名空间） | 底层复用 | 风险 |
|------|------|------|
| `hosts.read` / `hosts.list` | `api::hosts::read_system_hosts` | Read |
| `hosts.add` / `hosts.remove` / `hosts.toggle` | `update_hosts_with_groups` | Write |
| `hosts.revert` | `api::hosts::revert_hosts` | Write |
| `wallpaper.list` / `wallpaper.current` | `get_wallpapers` / `get_current_wallpaper` | Read |
| `wallpaper.set` | `set_desktop_wallpaper` | Write |
| `podcast.subscriptions` / `podcast.episodes` | `get_subscriptions` / `fetch_episodes` | Read |
| `podcast.subscribe` / `podcast.unsubscribe` / `podcast.import_opml` | `save_subscription` 等 | Write |
| `system.info` | `api::os::get_os_info` | Read |

示例（把现有 hosts 逻辑包进 trait）：

```rust
pub struct HostsAddTool;

#[async_trait]
impl AgentTool for HostsAddTool {
    fn declaration(&self) -> ToolDeclaration {
        ToolDeclaration {
            name: "hosts.add".into(),
            description: "添加一条 hosts 记录到指定分组".into(),
            input_schema: json!({ "type":"object",
                "properties": { "ip":{"type":"string"}, "domain":{"type":"string"}, "group":{"type":"string"} },
                "required": ["ip","domain","group"] }),
            risk: RiskLevel::Write,
            source: ToolSource::Native,
        }
    }
    async fn dry_run(&self, args: &Value) -> Result<Option<ToolPreview>, String> {
        // 读当前 hosts，计算 diff，不写入
    }
    async fn call(&self, args: Value) -> ToolOutcome {
        // 复用 api::hosts 的写入逻辑，返回 undo_token（写前快照）
    }
}
```

> `agent.rs` 现有的 `HostsMcpServer` 派发切换为 `ToolRegistry`。`src-tauri/src/mcp/servers/hosts.rs` 的工具定义逻辑可迁移进 `HostsTool`，MCP 的 `types/transport/protocol` 保留给第 4.2/4.3 节使用。

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
- **SSE / HTTP**（新增）：连接远程 MCP server。需新增 `SseTransport`。

#### 连接生命周期

```
用户启用 server
  → 按 transport 启动/连接
  → initialize 握手（协议版本 2024-11-05，交换 capabilities）
  → tools/list 拉取工具清单
  → 每个远程工具包装为 McpClientTool，注册进 ToolRegistry
     （命名空间前缀 mcp:<server_id>.<tool>，避免重名冲突）
  → 运行期：tools/call 派发；监听 notifications/tools/list_changed 动态刷新
  → 断线：指数退避重连；期间对应工具从 registry 注销并在 UI 标灰
  → 用户禁用/退出：发送关闭，注销工具，回收进程
```

```rust
/// 把一个远程 MCP 工具适配为 AgentTool
pub struct McpClientTool {
    server_id: String,
    remote_name: String,           // 远端原始工具名
    decl: ToolDeclaration,         // source = Mcp(server_id), risk 默认 Write（见 §7）
    client: Arc<McpClient>,        // 复用 mcp::protocol::McpClient
}

#[async_trait]
impl AgentTool for McpClientTool {
    fn declaration(&self) -> ToolDeclaration { self.decl.clone() }
    async fn call(&self, args: Value) -> ToolOutcome {
        // client.call_tool(remote_name, args) → 映射 CallToolResult → ToolOutcome
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
- 对外暴露的写操作仍受 Jedi 的确认/审计约束（无头模式下按配置的默认策略）。

这层可最后做，但抽象上从一开始就用 `exportable` 标志预留。

---

## 5. 命名空间与冲突

- 内置：`hosts.add`、`wallpaper.set`、`system.info`
- 第三方：`mcp:<server_id>.<tool>`，如 `mcp:github.create_issue`
- Trace 面板已按 `server.name` 展示，沿用即可。
- `ToolRegistry.register` 遇同名拒绝并记审计；UI 提示冲突。

---

## 6. Provider 适配

工具声明 → 各家 function calling 格式的转换**已在 `agent.rs` 实现**（`tool_to_openai` / `tool_to_anthropic`），改为吃 `ToolDeclaration` 即可复用：

- OpenAI/兼容：`{type:"function", function:{name, description, parameters:input_schema}}`
- Anthropic：`{name, description, input_schema}`
- 模型能力探测：`fetch_models_dev` 返回的 `tool_call` 标志已可用——**若所选模型不支持 function calling，则禁用工具注入并在 UI 提示**（降级为纯聊天），避免对小模型硬塞工具。

---

## 7. 风险分级与人机确认

统一由 `RiskLevel` 驱动（对齐已确认的"分级确认"策略）：

| 风险 | 默认行为 |
|------|---------|
| `Read` | 自动执行，无需确认 |
| `Write` | 先 `dry_run` 生成 **diff 预览**，用户确认后 `call` |
| `System` | 强制二次确认 + 显著警示 |

- 确认在 **Agent 回路中挂起**：回路检测到待执行工具为 Write/System 时，发 `agent-event` 请求确认，前端弹确认卡片（含 diff），用户批准/拒绝/编辑参数后回路继续。
- 每个 server/工具支持 `autoApprove` 白名单（用户显式免确认）。
- 全局确认模式 `auto | always | dangerous`（对齐设计文档 `AgentConfig.confirmationMode`）。
- **第三方 MCP 工具默认风险 = Write（至少需确认）**，因为无法信任其自报的安全性；用户可在配置里手动降级为 Read。

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
- 内置工具启用状态、确认模式、autoApprove：并入现有 chat 设置。
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
统一列出 `ToolRegistry` 全部工具（内置 + 各 MCP server），带来源与风险徽标；供用户了解 Agent 能做什么、按需启用/禁用。

### 10.3 确认卡片（新）
Write/System 工具执行前，在对话流中插入确认卡片：工具名 + 参数 + **diff 预览**（hosts 改动尤其重要）+ [批准]/[拒绝]/[编辑参数]/[本会话免确认]。

### 10.4 Trace 面板增强（已存在，扩展）
`AgentTrace.vue` 增加：工具来源徽标（native / server 名）、风险色、确认状态（已确认/被拒）、回滚入口（若有 undo_token）。

---

## 11. Tauri 命令接口清单（新增）

```
# 工具注册表
tool_list_all() -> Vec<ToolDeclaration>            # 所有已注册工具
agent_chat(...)                                    # 改为从 ToolRegistry 取工具（替换现 servers 参数语义）
tool_confirm(request_id, call_id, approved, edited_args?)  # 确认回路挂起的工具调用
tool_undo(undo_token) -> Result                    # 回滚

# MCP server 管理
mcp_server_list() -> Vec<McpServerStatus>
mcp_server_upsert(config) -> Result
mcp_server_remove(id) -> Result
mcp_server_connect(id) -> Result                   # 手动连接/重连
mcp_server_disconnect(id) -> Result
mcp_server_test(config) -> Vec<ToolDeclaration>    # 测试连接并预览工具

# MCP server（对外，战略层，后置）
# 通过 `jedi --mcp-server` 子命令启动，无需前端命令
```

---

## 12. 与现有代码的迁移路径

1. 新增 `src-tauri/src/tools/`：`mod.rs`(trait+registry)、`native/`(hosts/wallpaper/podcast/system)、`mcp_client.rs`(适配器)。
2. `agent.rs`：`collect_tools` / `call_server_tool` 改为走 `ToolRegistry`；工具声明转换函数改吃 `ToolDeclaration`。保留 `agent-event` 事件协议，新增确认事件类型。
3. `mcp/`：`types`/`protocol`/`transport` 保留；新增 `SseTransport`；`servers/hosts.rs` 的定义迁移进 `tools/native/hosts.rs` 后可删或转为"对外导出"用。
4. `main.rs`：`.manage(ToolRegistry)`；注册新命令；启动时加载 MCP 配置。
5. 前端：`api/ai-chat.ts` 增补命令封装；`stores/aiChat` 增 MCP server 管理状态；新增配置 UI 与确认卡片。

现有单测（`agent::tests`）随派发层调整更新；新增 registry / native tool / mcp client 适配的单测。

---

## 13. 分阶段实施

| 阶段 | 内容 | 产出 |
|------|------|------|
| **P1 抽象落地** | `AgentTool` + `ToolRegistry`；hosts 迁移为内置工具；`agent.rs` 切换派发 | Agent 走统一抽象，行为不变、更干净 |
| **P2 全产品工具化 + 确认** | 壁纸/播客/系统内置工具；风险分级 + diff 预览 + 确认卡片 + 回滚 | Agent 能**安全操作整个产品**（AI Native 内核）|
| **P3 第三方 MCP 客户端** | stdio+SSE 传输、server 配置 UI、连接生命周期、工具注入、安全边界 | 接入社区 MCP 生态，能力可扩展 |
| **P4 对外 MCP 服务端** | `--mcp-server` 导出内置工具 | Jedi 成为 MCP 生态节点 |

---

## 14. 开放问题

1. **对外 server 的无头确认**：Jedi 作 MCP server 被外部调用时，写操作如何确认？（选项：全部要求 autoApprove 白名单 / 弹系统通知 / 只读导出）
2. **SSE/HTTP MCP 的鉴权**：OAuth 流程是否要在 P3 覆盖，还是先只支持静态 token。
3. **工具过多时的选择**：当第三方 server 引入几十个工具，是否需要工具检索/分组注入（上下文工程，属 B3，暂记）。
4. **回滚的通用性**：并非所有工具可逆（如已发出的网络请求）；`undo_token` 仅对可逆工具提供。
