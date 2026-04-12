# AiChat 输入框交互改版实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 重构 AiChat 输入框交互，新会话时上移居中，聊天时底部一行 Glassmorphism 展示

**Architecture:** 修改 `index.vue` 中的输入框模板结构，添加状态判断逻辑；重写 `chat.css` 中 `.input-console` 相关样式，实现两种状态的差异化视觉

**Tech Stack:** Vue 3 + TypeScript + CSS（无新依赖）

---

## 文件变更概览

| 文件 | 改动类型 | 职责 |
|------|----------|------|
| `src/views/AiChat/index.vue` | 修改 | 输入框模板结构调整、状态切换逻辑 |
| `src/views/AiChat/chat.css` | 修改 | 全部输入框相关样式重写 |

---

## 任务分解

---

### Task 1: 重写输入框容器样式（chat.css）

**文件:** `src/views/AiChat/chat.css`（替换现有 `.input-console` 相关样式，位置约 1504-1705 行）

- [ ] **Step 1: 注释掉现有 `.input-console` 样式**

找到 chat.css 中从 `.input-console`（约 1505 行）到 `.send-btn.disabled`（约 1705 行）的全部样式，用 `/* --- DEPRECATED: new input styles below --- */` 注释包围

- [ ] **Step 2: 写入新样式**

```css
/* =========================================
   Input Console - Glassmorphism Style
   ========================================= */

.input-console {
  padding: 16px 20px;
  background: transparent;
  border-top: none;
  position: relative;
  z-index: 2;
  transition: all 0.3s ease;
}

/* 状态 A: 新会话 - 居中悬浮 */
.input-console.state-new-session {
  position: absolute;
  bottom: auto;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 100%;
  max-width: 700px;
  padding: 0 20px;
}

/* 输入框容器 - 胶囊形状 */
.input-wrapper {
  position: relative;
  width: 100%;
}

/* 输入框主体 */
.console-input {
  width: 100%;
  min-height: 48px;
  max-height: 160px;
  padding: 12px 20px;
  background: rgba(255, 255, 255, 0.08);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 24px;
  color: var(--hologram-text);
  font-size: 14px;
  font-family: inherit;
  resize: none;
  outline: none;
  transition: all 0.2s ease;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
}

.console-input:focus {
  background: rgba(255, 255, 255, 0.12);
  border-color: rgba(255, 255, 255, 0.2);
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.4);
}

.console-input::placeholder {
  color: rgba(255, 255, 255, 0.35);
}

/* 底部操作栏 - 一行布局 */
.input-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 12px;
}

.input-toolbar {
  display: flex;
  gap: 8px;
}

.input-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

/* 工具栏按钮 - 圆形 */
.toolbar-btn {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 50%;
  color: rgba(255, 255, 255, 0.6);
  font-size: 16px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.toolbar-btn:hover {
  background: rgba(255, 255, 255, 0.15);
  border-color: rgba(255, 255, 255, 0.2);
  color: rgba(255, 255, 255, 0.9);
  transform: scale(1.05);
}

/* Model 选择器 - 文字下拉 */
.model-selector {
  position: relative;
}

.model-dropdown-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  background: transparent;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 20px;
  color: rgba(255, 255, 255, 0.7);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
}

.model-dropdown-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  border-color: rgba(255, 255, 255, 0.2);
  color: rgba(255, 255, 255, 0.9);
}

.model-dropdown-btn svg {
  opacity: 0.6;
}

/* Model 下拉菜单 */
.model-dropdown-menu {
  position: absolute;
  bottom: 100%;
  right: 0;
  margin-bottom: 8px;
  min-width: 220px;
  max-height: 280px;
  overflow-y: auto;
  background: rgba(10, 15, 25, 0.9);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  padding: 6px;
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.5);
  z-index: 100;
}

.model-dropdown-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.15s;
  box-sizing: border-box;
}

.model-dropdown-item:hover {
  background: rgba(255, 255, 255, 0.08);
}

.model-dropdown-item.selected {
  background: rgba(59, 130, 246, 0.15);
  border: 1px solid rgba(59, 130, 246, 0.3);
}

.model-item-name {
  font-size: 13px;
  color: var(--hologram-text);
}

.model-dropdown-item.selected .model-item-name::after {
  content: ' ✓';
  color: #3b82f6;
}

.model-item-context {
  font-size: 10px;
  color: var(--hologram-text-muted);
  padding: 2px 6px;
  background: rgba(255, 255, 255, 0.06);
  border-radius: 4px;
}

/* 发送按钮 - 亮蓝圆形 */
.send-btn {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #3b82f6;
  border: none;
  border-radius: 50%;
  color: white;
  font-size: 16px;
  cursor: pointer;
  transition: all 0.15s ease;
  box-shadow: 0 4px 16px rgba(59, 130, 246, 0.4);
}

.send-btn:hover:not(.disabled) {
  transform: scale(1.08);
  box-shadow: 0 6px 24px rgba(59, 130, 246, 0.5);
}

.send-btn.disabled {
  opacity: 0.4;
  cursor: not-allowed;
  transform: none;
  box-shadow: none;
}

/* 状态 A: 新会话时隐藏底部操作栏 */
.state-new-session .input-actions {
  display: none;
}

/* 状态 B: 聊天进行时输入框靠近底部 */
.input-console.state-chatting {
  position: relative;
  padding: 12px 20px 20px;
}
```

- [ ] **Step 3: 提交**

```bash
git add src/views/AiChat/chat.css
git commit -m "refactor(chat): rewrite input console styles to glassmorphism"
```

---

### Task 2: 调整 index.vue 输入框模板结构

**文件:** `src/views/AiChat/index.vue`（输入框部分，约 170-240 行）

- [ ] **Step 1: 读取当前输入框区域代码确认结构**

确认 `.input-console` div 的起止位置和内部结构

- [ ] **Step 2: 修改 `.input-console` div，添加状态 class 绑定**

将:
```html
<div class="input-console">
```

改为:
```html
<div class="input-console" :class="inputConsoleState">
```

- [ ] **Step 3: 在 `<script setup>` 中添加状态计算属性**

在 computed 部分添加：
```typescript
const inputConsoleState = computed(() => {
  const hasMessages = store.currentSession?.messages.length > 0
  return hasMessages ? 'state-chatting' : 'state-new-session'
})
```

- [ ] **Step 4: 确认 textarea 和工具栏结构不变**

当前结构已经符合要求（textarea + 底部工具栏），只需确保 class 绑定正确

- [ ] **Step 5: 提交**

```bash
git add src/views/AiChat/index.vue
git commit -m "feat(chat): add input console state switching for new session vs chatting"
```

---

### Task 3: 验证效果

- [ ] **Step 1: 启动开发服务器**

```bash
pnpm tauri dev
```

或仅前端预览：
```bash
pnpm build && pnpm preview
```

- [ ] **Step 2: 验证状态 A（新会话）**
- 输入框居中悬浮在大约 50% 高度位置
- 底部工具栏不显示
- 大圆角 + 磨砂背景

- [ ] **Step 3: 验证状态 B（发送消息后）**
- 输入框回到底部
- 底部工具栏显示（`+` 按钮、Model 下拉、发送按钮）
- 磨砂背景 + 高光边框

- [ ] **Step 4: 验证交互**
- `/` 命令面板是否正常
- Model 下拉是否正常
- 发送按钮是否 disabled 状态正确

---

### Task 4: 适配浅色主题（如已实现）

**文件:** `src/views/AiChat/chat.css`（Light Theme 部分）

- [ ] **Step 1: 检查 Light Theme 的 `.input-console` 样式覆盖**

在 chat.css 约 1396-1445 行的 `.light-theme .input-console` 相关样式，确认是否有冲突

- [ ] **Step 2: 如有冲突，添加对应 .state-new-session 和 .state-chatting 的浅色主题样式**

```css
.light-theme .console-input {
  background: rgba(255, 255, 255, 0.6);
  border-color: rgba(0, 0, 0, 0.1);
  color: #1a1a1a;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
}

.light-theme .console-input:focus {
  background: rgba(255, 255, 255, 0.8);
  border-color: rgba(59, 130, 246, 0.3);
}

.light-theme .console-input::placeholder {
  color: rgba(0, 0, 0, 0.35);
}
```

- [ ] **Step 3: 提交**

```bash
git add src/views/AiChat/chat.css
git commit -m "fix(chat): adapt input console glassmorphism for light theme"
```

---

## 验证清单

- [ ] 新会话时输入框居中悬浮（约 50% 高度）
- [ ] 新会话时底部工具栏隐藏
- [ ] 发送消息后输入框回到底部
- [ ] 底部工具栏正常显示
- [ ] 输入框圆角 24px
- [ ] 磨砂背景 + 高光边框
- [ ] Model 文字下拉正常
- [ ] 发送按钮圆形亮蓝色
- [ ] 浅色主题适配正常
- [ ] 状态切换动画平滑（0.3s）
