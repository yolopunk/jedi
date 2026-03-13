# Jedi 控制台布局 - 实施计划

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 重构应用顶部栏、侧边栏、底部状态栏，加入 IDE 风格和 Jedi 光剑元素；同时修复自定义标题栏的双击放大和全屏问题；重构 Chat 和 Hosts 模块。

**Architecture:** 渐进式重构，每个组件独立修改并测试，保持功能完整性。先修复窗口行为问题，再优化视觉效果，最后重构 Chat 和 Hosts 模块。

**Tech Stack:** Vue 3 + TypeScript + Vuetify 3 + Tauri v2

---

## Chunk 1: 修复自定义标题栏行为

**目标**: 修复双击放大、全屏占满等原生窗口行为

### Task 1.1: 顶部栏双击放大功能

**Files:**
- Modify: `src/components/layout/AppHeader.vue`

- [ ] **Step 1: 阅读现有 AppHeader.vue**

Read: `src/components/layout/AppHeader.vue`
Note: 当前有窗口控制按钮，但没有双击事件监听

- [ ] **Step 2: 添加双击事件监听**

在 `.app-header` 元素上添加 `@dblclick` 事件：
```vue
<div class="app-header" data-tauri-drag-region @dblclick="handleDoubleClick">
```

- [ ] **Step 3: 实现 handleDoubleClick 方法**

在 script setup 中添加：
```typescript
function handleDoubleClick() {
  toggleMaximize()
}
```

- [ ] **Step 4: 确保可点击区域有正确的拖拽属性**

确保中间拖拽区域可以接收双击，但按钮区域设置 `data-tauri-no-drag`

- [ ] **Step 5: 验证修改**

检查:
- 双击顶部栏中间区域触发最大化/还原
- 按钮区域不触发双击事件

---

### Task 1.2: 优化全屏显示

**Files:**
- Modify: `src/App.vue`
- Modify: `src/assets/theme.css` (or add inline styles)

- [ ] **Step 1: 阅读现有 App.vue**

Read: `src/App.vue`
Note: 当前有 `.app-layout` 和 `.os-macos` 样式

- [ ] **Step 2: 添加全屏状态监听**

在 App.vue 中添加全屏状态追踪：
```typescript
const isFullscreen = ref(false)

async function checkFullscreen() {
  isFullscreen.value = await appWindow.isFullscreen()
}

// 在 onMounted 中添加监听
unlistenResize = await appWindow.onResized(() => {
  checkMaximized()
  checkFullscreen()
})
await checkFullscreen()
```

- [ ] **Step 3: 添加全屏样式**

在 App.vue style 中添加：
```css
/* 全屏时移除圆角和阴影 */
.os-macos.fullscreen .app-layout {
  border-radius: 0;
  box-shadow: none;
}
```

- [ ] **Step 4: 绑定 fullscreen class**

在 v-app 上绑定 class:
```vue
<v-app :class="[platformClass, { fullscreen: isFullscreen }]">
```

- [ ] **Step 5: 验证修改**

检查:
- macOS 全屏时没有圆角
- 全屏时窗口完全占满屏幕

---

### Task 1.3: 检查 tauri.conf.json 配置

**Files:**
- Review: `src-tauri/tauri.conf.json`

- [ ] **Step 1: 验证当前配置**

Read: `src-tauri/tauri.conf.json`
Current:
```json
"decorations": false,
"transparent": true
```

- [ ] **Step 2: 确认配置无需修改**

保持 `decorations: false` 和 `transparent: true`，我们通过前端代码解决问题。

---

## Chunk 2: 优化顶部栏 (AppHeader)

### Task 2.1: 精简顶部栏高度

**Files:**
- Modify: `src/components/layout/AppHeader.vue`

- [ ] **Step 1: 修改高度为 32px**

在 style 中找到 `.app-header`，修改:
```css
.app-header {
  height: 32px; /* 从 36px 改为 32px */
  /* ... 其他保持不变 */
}
```

- [ ] **Step 2: 调整垂直间距**

确保内部元素在 32px 高度中居中显示，调整 padding/margin 如需要。

- [ ] **Step 3: 验证修改**

检查:
- 顶部栏高度为 32px
- 所有元素正确对齐
- 交通灯/窗口控制按钮位置正确

---

### Task 2.2: 优化迷你播放器样式

**Files:**
- Modify: `src/components/layout/AppHeader.vue`

- [ ] **Step 1: 添加光剑风格发光**

为迷你播放器激活状态添加微妙发光效果:
```css
.mini-player {
  /* 现有样式 */
  transition: all 0.2s ease;
}

.mini-player:hover {
  box-shadow: 0 0 8px rgba(96, 165, 250, 0.2);
}
```

---

## Chunk 3: 优化侧边栏 (AppSidebar)

### Task 3.1: 添加光剑激活效果

**Files:**
- Modify: `src/components/layout/AppSidebar.vue`

- [ ] **Step 1: 阅读现有 AppSidebar.vue**

Read: `src/components/layout/AppSidebar.vue`
Note: 当前有导航列表，但缺少激活状态的视觉效果

- [ ] **Step 2: 添加左侧光剑边框**

为激活的 v-list-item 添加左侧边框:
```css
.sidebar-item.v-list-item--active {
  border-left: 2px solid rgb(var(--v-theme-primary));
  background: rgba(96, 165, 250, 0.08);
}
```

- [ ] **Step 3: 添加图标发光效果**

为激活状态的图标添加发光:
```css
.sidebar-item.v-list-item--active .sidebar-icon-container {
  filter: drop-shadow(0 0 4px rgba(96, 165, 250, 0.6));
}
```

- [ ] **Step 4: 添加过渡动画**

确保状态变化有平滑过渡:
```css
.sidebar-item {
  transition: all 0.15s ease;
}

.sidebar-icon-container {
  transition: filter 0.15s ease;
}
```

- [ ] **Step 5: 验证修改**

检查:
- 激活项有 2px 左侧蓝色边框
- 激活项图标有发光效果
- 状态切换有平滑过渡

---

## Chunk 4: 优化底部状态栏

### Task 4.1: 优化状态栏信息展示

**Files:**
- Modify: `src/App.vue`
- Review: `src/components/common/SystemInfoBar.vue`

- [ ] **Step 1: 阅读现有状态栏代码**

Read: `src/App.vue` (底部状态栏部分)
Read: `src/components/common/SystemInfoBar.vue`

- [ ] **Step 2: 添加右侧状态指示器**

在 `.status-bar-right` 中添加:
```vue
<div class="status-bar-right" data-tauri-no-drag>
  <span class="status-text">{{ currentLocale }}</span>
</div>
```

在 script 中添加:
```typescript
const currentLocale = computed(() => {
  return locale.value === 'zh' ? 'zh-CN' : 'en-US'
})
```

- [ ] **Step 3: 添加状态栏样式**

在 App.vue style 中添加:
```css
.status-text {
  font-size: 0.75rem;
  color: rgba(var(--v-theme-on-surface), 0.5);
  padding: 0 8px;
}
```

- [ ] **Step 4: 验证修改**

检查:
- 状态栏右侧显示当前语言
- 样式与整体协调

---

## Chunk 5: 添加全局样式

### Task 5.1: 添加 Jedi 控制台样式

**Files:**
- Modify: `src/assets/theme.css`

- [ ] **Step 1: 阅读现有 theme.css**

Read: `src/assets/theme.css`

- [ ] **Step 2: 添加光剑发光类**

在文件末尾添加:
```css
/* =========================================
   Jedi Console - 光剑效果
   ========================================= */

/* 光剑发光 - 限制使用范围 */
.glow-saber {
  box-shadow: 0 0 8px rgba(96, 165, 250, 0.4);
  will-change: box-shadow;
}

/* 悬停发光 */
.glow-hover:hover {
  box-shadow: 0 0 12px rgba(96, 165, 250, 0.5);
}

/* 浅色主题下调低发光强度 */
.light-theme .glow-saber {
  box-shadow: 0 0 6px rgba(59, 130, 246, 0.3);
}

/* 焦点指示器 - 无障碍 */
.glow-focus:focus-visible,
.btn:focus-visible,
.sidebar-item:focus-visible {
  outline: 2px solid rgb(var(--v-theme-primary));
  outline-offset: 2px;
}
```

---

## Chunk 6: Chat 模块 - R2-D2 头像

### Task 6.1: 创建/优化 ChatMessage 组件

**Files:**
- Review: `src/views/AiChat/ChatMessage.vue` (若存在)
- Create/Modify: `src/views/AiChat/ChatMessage.vue`

- [ ] **Step 1: 检查现有文件**

Check: `src/views/AiChat/ChatMessage.vue`
- 若存在，阅读现有代码
- 若不存在，创建新组件

- [ ] **Step 2: 实现 R2-D2 头像组件**

创建 R2-D2 样式的头像:
```vue
<template>
  <div class="chat-message" :class="{ 'ai-message': isAi }">
    <div v-if="isAi" class="avatar-container">
      <div class="avatar-r2d2" :class="avatarState">
        <!-- R2-D2 中心细节 -->
      </div>
    </div>
    <div class="message-content">
      <!-- 消息内容 -->
    </div>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{
  isAi: boolean
  content: string
  state?: 'idle' | 'thinking' | 'speaking'
}>()

const avatarState = computed(() => {
  if (props.state === 'thinking') return 'thinking'
  if (props.state === 'speaking') return 'speaking'
  return ''
})
</script>

<style scoped>
.avatar-r2d2 {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background: linear-gradient(135deg, #1e3a5f 0%, #3b82f6 50%, #60a5fa 100%);
  box-shadow: 0 0 12px rgba(96, 165, 250, 0.4);
  position: relative;
  will-change: box-shadow, transform;
}

.avatar-r2d2::before {
  content: '';
  position: absolute;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: rgba(255,255,255,0.9);
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  box-shadow: 0 0 8px rgba(255,255,255,0.5);
}

/* 思考状态 */
@keyframes pulse-glow {
  0%, 100% {
    transform: scale(1);
    box-shadow: 0 0 12px rgba(96, 165, 250, 0.4);
  }
  50% {
    transform: scale(1.05);
    box-shadow: 0 0 20px rgba(96, 165, 250, 0.6);
  }
}

.avatar-r2d2.thinking {
  animation: pulse-glow 2s ease-in-out infinite;
}

/* AI 消息气泡 */
.ai-message .message-content {
  border-left: 2px solid rgba(96, 165, 250, 0.5);
  background: rgba(96, 165, 250, 0.05);
  border-radius: 0 4px 4px 0;
  padding: 12px 16px;
}
</style>
```

---

### Task 6.2: 集成到 Chat 主页面

**Files:**
- Modify: `src/views/AiChat/index.vue`

- [ ] **Step 1: 阅读现有 Chat 页面**

Read: `src/views/AiChat/index.vue`

- [ ] **Step 2: 集成 ChatMessage 组件**

用新的 ChatMessage 组件替换现有消息展示，传入适当的 state。

- [ ] **Step 3: 添加输入框装饰**

在输入框左下角添加小 R2-D2 图标装饰 (16px)。

---

## Chunk 7: Hosts 模块 - IDE 风格表格

### Task 7.1: 优化 Hosts 表格样式

**Files:**
- Modify: `src/views/hosts/HostsManager.vue`
- Review: `src/components/hosts/tables/HostsTable.vue`

- [ ] **Step 1: 阅读现有 Hosts 代码**

Read: `src/views/hosts/HostsManager.vue`
Read: `src/components/hosts/tables/HostsTable.vue`

- [ ] **Step 2: 设置表格行高为 32px**

在表格组件中添加/修改:
```css
.hosts-table .v-data-table__tr {
  height: 32px;
}
```

- [ ] **Step 3: 优化状态指示器**

为启用状态添加发光效果:
```css
.status-enabled {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: rgb(var(--v-theme-success));
  box-shadow: 0 0 6px rgba(74, 222, 128, 0.5);
  will-change: box-shadow;
}

.status-disabled {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: var(--jedi-text-tertiary, #52525b);
}

/* 浅色主题调整 */
.light-theme .status-enabled {
  background: #16a34a;
  box-shadow: 0 0 4px rgba(22, 163, 74, 0.4);
}
```

- [ ] **Step 4: 移除行号列**

根据设计，移除行号列，保持: 状态 | IP | 域名 | 备注 | 操作

- [ ] **Step 5: 添加响应式隐藏备注列**

使用 CSS 在小窗口下隐藏备注列:
```css
@media (max-width: 600px) {
  .hosts-table .column-remarks {
    display: none;
  }
}
```

---

## Chunk 8: 最终验证

### Task 8.1: 完整测试

**Files:**
- All modified files

- [ ] **Step 1: 启动开发服务器**

Run: `pnpm tauri dev`

- [ ] **Step 2: 测试窗口行为**

Check:
- [ ] 双击顶部栏放大/还原
- [ ] 全屏时窗口完全占满
- [ ] macOS 全屏无圆角
- [ ] 拖拽窗口正常工作

- [ ] **Step 3: 测试视觉效果**

Check:
- [ ] 顶部栏 32px 高度
- [ ] 侧边栏激活项有光剑边框 + 发光
- [ ] Chat 模块有 R2-D2 头像
- [ ] Hosts 表格状态指示器发光
- [ ] 深色/浅色主题均适配

- [ ] **Step 4: 测试响应式**

Check:
- [ ] 窗口 < 600px 时 Hosts 备注列隐藏
- [ ] 各组件在不同尺寸下布局正确

- [ ] **Step 5: 性能检查**

Check:
- [ ] 动画流畅 (目测 60fps)
- [ ] 无明显性能问题

---

## Commit 指南

每个任务完成后单独 commit:

```bash
# Chunk 1
git add src/components/layout/AppHeader.vue
git commit -m "feat: add double-click to maximize on titlebar"

git add src/App.vue
git commit -m "fix: fullscreen display with no corner radius"

# Chunk 2
git add src/components/layout/AppHeader.vue
git commit -m "style: reduce header height to 32px"

# Chunk 3
git add src/components/layout/AppSidebar.vue
git commit -m "style: add lightsaber glow effect to sidebar"

# Chunk 4
git add src/App.vue
git commit -m "style: add locale indicator to status bar"

# Chunk 5
git add src/assets/theme.css
git commit -m "style: add jedi console glow utilities"

# Chunk 6
git add src/views/AiChat/ChatMessage.vue src/views/AiChat/index.vue
git commit -m "feat: add R2-D2 avatar to chat module"

# Chunk 7
git add src/views/hosts/HostsManager.vue src/components/hosts/tables/HostsTable.vue
git commit -m "style: IDE-style hosts table with 32px row height"
```

---

## 文件清单汇总

| 操作 | 文件 |
|------|------|
| 修改 | `src/components/layout/AppHeader.vue` |
| 修改 | `src/components/layout/AppSidebar.vue` |
| 修改 | `src/App.vue` |
| 修改 | `src/assets/theme.css` |
| 创建/修改 | `src/views/AiChat/ChatMessage.vue` |
| 修改 | `src/views/AiChat/index.vue` |
| 修改 | `src/views/hosts/HostsManager.vue` |
| 审查 | `src/components/hosts/tables/HostsTable.vue` |
| 审查 | `src-tauri/tauri.conf.json` |

