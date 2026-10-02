# 技能系统重构：对齐 Claude Code Agent Skills 范式

> 日期：2026-09-25
> 状态：已实施
> 取代：`2026-09-25-skill-management-design.md`（其中的"提示词技能"范式废弃）

## 背景与动机

原设计把三类不同的东西塞进同一个 `Skill` 接口：

1. **可执行工具**（terminal/filesystem/hosts/web/memory）——包装 Tauri 命令的函数；
2. **提示词模板**（自定义 markdown 技能）——`{{param}}` 渲染后作为伪工具结果返回；
3. **MCP 远端工具**——桥接进同一注册表。

这违背了 Claude Code 的核心洞察：**技能是知识（指令文档），工具是能力**。技能不应出现在 function-calling 工具列表里，而应通过"渐进式披露"按需加载——模型先看到元数据，需要时才读正文，再按正文指示使用工具完成任务。

## 新架构

```
┌─ 工具层 src/agent/tools/ ─────────────────────────────┐
│ toolRegistry（可执行，function-calling 直接暴露）        │
│ terminal / filesystem / hosts / web×2 / memory          │
│ browser / podcast / wallpaper（stub，默认禁用）          │
│ MCP 桥接（source: 'mcp'，确认门 risk: 'write'）          │
└────────────────────────────────────────────────────────┘
┌─ 技能层 src/skills/ ──────────────────────────────────┐
│ skillRegistry（SkillManifest，无执行体）                 │
│ builtin/：随应用分发的 SKILL.md（?raw 导入，只读）        │
│ user：~/.jedi/skills/<name>/SKILL.md（可编辑）           │
│ loader.ts：渐进披露第三层（正文按需加载 + 缓存）           │
└────────────────────────────────────────────────────────┘
```

### 三层渐进披露

1. **元数据**（name + description）：常驻系统提示（`## Skills` 清单）。
2. **正文**（SKILL.md body）：模型调用内置 `skill` 工具（参数 `{name}`）时加载；用户也可在输入框敲 `/技能名` 强制注入本轮上下文。
3. **附属文件**（references/ scripts/ assets/）：`skills_read` 返回文件清单与技能目录绝对路径，模型用 FILESYS/TERMINAL 按需读取执行。

### 双触发

- **自动**：description 是触发信号（写"何时该用"，措辞要外向），驱动模型主动调用 `skill` 工具。
- **手动**：聊天输入 `/名称 其余内容` → `stores/aiChat.ts` 的 `resolveSlashSkill` 把正文包装为 system 消息注入本轮，trace 留痕，`/` 命令面板列出全部已启用技能。

## 磁盘格式

```
~/.jedi/skills/
└── <name>/                  # name = 目录名，kebab-case，1-64 字符
    ├── SKILL.md             # frontmatter: name(可选,须与目录名一致)/description(必填)/icon(可选)
    └── (可选) references/ scripts/ assets/
```

- 旧版扁平 `<id>.md` 在 `skills_list` 时一次性迁移：id 归一化为 kebab-case 后写入 `<name>/SKILL.md` 并删除旧文件；目标目录已存在则跳过。
- `risk` 字段废除——风险属于工具层（确认门/白名单机制不变，仅 skillId→toolId 改名）。
- `{{param}}` 占位符机制废除——输入从对话上下文获取，正文里自行说明。

## 后端命令（src-tauri/src/api/skills.rs）

| 命令 | 说明 |
|---|---|
| `skills_list` | 扫描目录 + 旧格式迁移，返回 `Vec<SkillManifest>` |
| `skills_read {name}` | 正文 + 附属文件清单 + 技能目录绝对路径（`dir` 字段） |
| `skills_read_file {name, relPath}` | 读附属文件，canonicalize 防穿越，1MB 上限 |
| `skills_save {def}` | 目录制写入 |
| `skills_delete {name}` | 删目录 |
| `skill_stats_*` | 不变（统计键：工具 id；技能加载记 `skill:<name>`） |

## 持久化与迁移（前端）

| 数据 | 旧 | 新 |
|---|---|---|
| 工具开关+白名单 | `skills-config` 混合存放 | `tools-config` `{id: {enabled, alwaysAllow}}` |
| 技能开关 | 同上 | `skills-flags` `{name: {enabled}}` |
| 迁移 | — | `stores/skills.ts` 启动时读旧 `skills-config`，按 toolRegistry 注册项拆分，其余丢弃，删除旧 key |

`autoCallable` 开关废除：工具启用即暴露给模型；技能启用即进入触发候选。

## 内置技能包（示范）

- `hosts-troubleshoot`：hosts 排查流程（诊断重复/冲突/失效条目 → 逐条修复 → 验证）
- `rust-review`：Rust 代码审查（静态检查 + cargo/clippy 编译验证）
- `commit-message`：从 git diff 生成符合仓库习惯的提交信息

## 决策记录

| 决策 | 理由 |
|---|---|
| 工具 id 保持原值（terminal/filesystem/…） | 历史 `skill_stats.json` 统计不失效 |
| 内置技能用 Vite `?raw` 打包而非后端资源 | 无需后端参与发现；Tauri resource 路径跨平台配置成本更高 |
| skill 工具在 runAgent 内联注册（非 toolRegistry） | 它是技能机制的实现细节，不应出现在管理 UI 里 |
| 系统提示注入技能清单而非 RAG | 技能数量级小（个位数），全量元数据成本可忽略 |
| 斜杠技能正文注入为本轮 system 消息 | 无状态、随会话持久化兼容；不污染历史消息内容 |
| registry 非响应式，UI 侧以 flagMap 为响应式依赖 | 运行时读取（runAgent）无需订阅；UI 通过 `void flagMap.value` 建立依赖 |
