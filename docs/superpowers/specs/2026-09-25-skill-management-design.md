# Skill 管理功能设计

> 日期：2026-09-25
> 状态：已批准

## 背景与问题

技能系统骨架已存在：`Skill` 接口（`src/skills/types.ts`）、`skillRegistry`（9 个内置技能）、MCP 远端工具动态桥接（`src/stores/mcpClient.ts`）。但管理能力几乎为零：

1. **无管理 UI**：唯一入口是 `AttachmentMenu` 的 ON/OFF 开关，无描述、无风险展示、无参数说明；`SkillPanel.vue` 是无引用死代码。
2. **禁用状态不持久**：`stores/skills.ts` 的 `loadFromStorage` 只对已保存 id 单向 `setEnabled(true)`，禁用操作重启后失效；且用裸 localStorage 而非项目惯例的 `useStorage`（Tauri Store）。
3. **autoCallable 形同虚设**：`SkillRegistry` 无 `setAutoCallable`，UI 无开关。
4. **无来源区分**：`Skill` 无 source 字段，MCP 工具仅靠 description 前缀区分。
5. **无自定义技能**：内置技能全部硬编码注册，用户无法扩展。

## 设计目标

- 聊天页新增技能管理对话框（仿 `McpServersDialog` 惯例：header 图标按钮 + 数量角标 + v-model 对话框）。
- 技能按来源分三组：内置 / 自定义 / MCP；展示描述、风险徽章、参数 schema；enabled 与 autoCallable 双开关。
- 支持 markdown 格式自定义技能，落盘 `~/.jedi/skills/`。
- 修复持久化缺陷，迁移到 `useStorage`（`settings.json` key `skills-config`）。

## 自定义技能格式

`~/.jedi/skills/<id>.md`，frontmatter + 提示词正文：

```markdown
---
name: DEPLOY_FRONTEND
description: 部署前端项目到测试环境
icon: 🚀
risk: read
---
请执行以下步骤部署项目：{{project}} ...
```

- `id`：文件名（不含扩展名），校验 `^[a-z0-9-_]+$`，防路径穿越。
- `risk`：`read | write | system`，缺省 `read`。
- 正文 `{{参数名}}` 占位符自动生成为 string 类型必填参数（沿用 `ParameterSchema` 每属性 `required` 的既有约定）。
- 执行语义：`execute()` 将 args 渲染进占位符，作为工具结果返回给模型——即"提示词技能"，模型据此调用真正的内置工具；底层工具自身的 risk 确认门仍然生效，自定义技能本身默认不额外加门。

## 架构

```
SkillsManagerDialog.vue (views/AiChat/)
   │  分组: builtin / custom / mcp（Skill.source 字段）
   ▼
stores/skills.ts
   │ useStorage('skills-config') 持久化 {id: {enabled, autoCallable}}
   │ 自定义技能 CRUD 编排（调后端命令 + registry 同步）
   ▼
skills/registry.ts（setAutoCallable / source 标记 / custom 工厂注册）
   │
   ▼
api/skills.ts → invoke → src-tauri/src/api/skills.rs → ~/.jedi/skills/*.md
```

## 决策记录

| 决策 | 理由 |
|---|---|
| 对话框而非独立 `/skills` 页 | 改动最小，与 MCP/Workers 管理入口一致；技能消费场景就在聊天页。后续可升级 |
| 自定义技能存 `~/.jedi/skills/` | `~/.jedi` 是既定 agent/用户级数据目录（memory、settings、审计日志）；markdown 便于手工编辑与版本管理 |
| 读写走后端命令而非前端 fs | memory.rs/podcast.rs 均为此模式；前端 fs 插件读写 home 目录需扩 scope，安全面更大 |
| 后端做 id 校验与解析 | 定义文件是文档型数据，后端解析校验更稳（serde + 单元测试） |
| 旧 `skills-enabled` localStorage 一次性迁移后清除 | 兼容既有用户数据，避免双写 |

## 后续迭代（本期不做）

执行统计（onToolStart/onToolEnd 聚合）、确认白名单（"始终允许" + 后台 agent 池补确认门）、`loop.ts` 与 `runAgent` 双执行路径清理、独立 `/skills` 管理页。
