# Jedi 控制台设计方案

**日期**: 2026-03-13
**版本**: 1.1
**状态**: 待审查

## 概述

本设计方案为 Jedi 工具箱应用提供一套 IDE 风格（Zed 风格）的界面，同时微妙地融入星球大战 / Jedi 元素。

## 设计原则

1. **极简高效**: 遵循 Zed 编辑器的美学，界面精简，信息密度适中
2. **微妙 Jedi**: 光剑发光效果、配色暗示，不喧宾夺主
3. **功能优先**: 装饰性元素最小化，专注实用性
4. **一致性**: 所有模块遵循统一的视觉语言
5. **向后兼容**: 保留现有功能，仅优化视觉表现
6. **原生体验**: 自定义标题栏支持双击放大、全屏占满等原生行为

---

## 0. 窗口配置优化

### Tauri 窗口配置问题
当前配置 `decorations: false` 导致:
- 双击标题栏不能放大/缩小
- 全屏后未占满屏幕

### 解决方案
1. **顶部栏双击事件**: 监听双击触发 `toggleMaximize`
2. **拖拽区域**: 确保 `data-tauri-drag-region` 正确设置
3. **全屏处理**: macOS 需要特殊处理圆角和边距

---

## 1. 顶部栏 (AppHeader)

### 规格
- **高度**: 32px (从 36px 精简)
- **背景**: `rgb(var(--v-theme-surface))` (#18181b)
- **底部边框**: 1px solid `var(--jedi-border)` (#27272a)

### 布局

```
┌─────────────────────────────────────────────────────────────────────┐
│ [红绿灯]                    [播放器] [☾] [⯑] [⚙] [_] [□] [✕] │
└─────────────────────────────────────────────────────────────────────┘
```

### 区域划分

| 区域 | 内容 | 说明 |
|------|------|------|
| 左侧 | 窗口控制 | macOS 交通灯 / Windows 最小化/最大化/关闭 |
| 中间 | 留白 | `data-tauri-drag-region` 拖拽区域 |
| 右侧 | 功能图标 | 迷你播放器（播放时显示）、主题切换、GitHub、设置 |

### 视觉效果

**迷你播放器激活时**:
- 背景: `rgba(96, 165, 250, 0.1)`
- 边框: 1px solid `rgba(96, 165, 250, 0.2)`

---

## 2. 侧边栏 (AppSidebar)

### 规格
- **图标模式宽度**: 64px (保持现有)
- **展开模式宽度**: 180px (保持现有)
- **背景**: `rgb(var(--v-theme-background))` (#111113)
- **右侧边框**: 1px solid `var(--jedi-border)` (#27272a)

### 布局

```
┌──────┐
│ [◎]  │ ← Jedi Logo (40px)
├──────┤
│ [🤖] │ ← Chat (激活)
│ [🌐] │ ← Hosts
│ [🖼]  │ ← Wallpapers
│ [🎙]  │ ← Podcast
│      │
│      │
└──────┘
```

### 导航项状态

| 状态 | 视觉效果 |
|------|----------|
| 默认 | 图标居中，无背景 |
| 悬停 | 轻微背景 `rgba(255,255,255,0.05)` |
| 激活 | 左侧 2px 蓝色边框 + 图标发光 `drop-shadow(0 0 4px rgba(96, 165, 250, 0.6))` |

### Tauri 拖拽注意事项
- 所有可点击元素添加 `data-tauri-no-drag`

---

## 3. 底部状态栏 (Bottom Status Bar)

### 规格
- **高度**: 24px (保持现有)
- **背景**: `rgb(var(--v-theme-surface))` (#18181b)
- **顶部边框**: 1px solid `var(--jedi-border)` (#27272a)

### 布局

```
┌─────────────────────────────────────────────────────────────────────┐
│ [≡] main  │  CPU: 12%  Mem: 45%  │  zh-CN  Ln 1, Col 1  UTF-8 │
└─────────────────────────────────────────────────────────────────────┘
```

### 区域划分

| 区域 | 内容 | 说明 |
|------|------|------|
| 左侧 | 侧边栏切换、分支/分组 | Hosts 模块显示当前分组 |
| 中间 | 系统信息 | SystemInfoBar 组件 (CPU、内存) |
| 右侧 | 语言、光标位置、编码 | 类似 IDE 状态栏 |

### 文字样式
- 字体大小: 0.75rem (12px)
- 默认颜色: `var(--jedi-text-tertiary)` (#52525b)
- 强调颜色: `rgb(var(--v-theme-primary))` (#60a5fa)

### 数据来源
- 系统信息: `src-tauri/src/api/os.rs` + `sysinfo` crate

---

## 4. Chat 模块设计

### R2-D2 机器人元素

#### AI 头像

**状态**:
- **静态**: R2-D2 风格圆形图标，蓝白配色
- **思考**: 脉冲发光动画 (性能优化版)
- **说话**: 声波波动动画 (简化版)

**样式**:
```css
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

/* 思考状态 - 使用 transform 而非 top/left */
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
```

#### 消息气泡

**AI 消息**:
- 左侧对齐
- 微妙蓝色左边框 (2px)
- 轻微圆角 (4px)
- 背景: `rgba(96, 165, 250, 0.05)`

**用户消息**:
- 右侧对齐
- 简洁设计
- 背景: `rgba(255,255,255,0.05)`

#### 输入区域

- 左下角: 小 R2-D2 图标装饰 (16px)
- 发送按钮: 光剑风格悬停效果
- 支持多行输入

#### 国际化支持
所有用户可见文本使用 Vue I18n:
- 输入框占位符: `chat.inputPlaceholder`
- 发送按钮: `chat.send`

---

## 5. Hosts 模块设计

### IDE 风格表格

#### 表格规格
- **行高**: 32px (紧凑模式)
- **表头背景**: `rgb(var(--v-theme-surface))`
- **边框**: 1px solid `var(--jedi-border)`

#### 列布局

| 列 | 宽度 | 内容 | 说明 |
|----|------|------|------|
| 状态 | 32px | ○ / ● | 启用/禁用指示器 |
| IP | 140px | 127.0.0.1 | 等宽字体 |
| 域名 | 自适应 | localhost | 主内容 |
| 备注 | 自适应 | 说明文字 | 浅色文字 |
| 操作 | 80px | 编辑/删除 | 悬停显示 |

#### 状态指示器

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
  background: var(--jedi-text-tertiary);
}

/* 浅色主题调整 */
.light-theme .status-enabled {
  background: #16a34a;
  box-shadow: 0 0 4px rgba(22, 163, 74, 0.4);
}
```

#### 分组设计

- 类似 VS Code 文件树
- 可折叠/展开
- 分组标题背景: `rgba(255,255,255,0.03)`
- 折叠箭头: 简单的三角形图标

#### 全局开关

- 光剑风格切换按钮
- 开启时: 蓝色发光效果
- 关闭时: 灰色

#### 响应式设计

| 断点 | 布局调整 |
|------|---------|
| < 800px | 侧边栏默认收起 |
| < 600px | 隐藏备注列 |

---

## 颜色系统

### Jedi 光剑配色 (基于 Vuetify 变量)

| 用途 | 颜色 | CSS 变量 |
|------|------|----------|
| 光剑蓝 (主色) | #60a5fa | `rgb(var(--v-theme-primary))` |
| 光剑绿 (成功) | #4ade80 | `rgb(var(--v-theme-success))` |
| 光剑红 (危险) | #f87171 | `rgb(var(--v-theme-error))` |
| 光剑黄 (警告) | #fbbf24 | `rgb(var(--v-theme-warning))` |

### 发光效果 (性能优化版)

```css
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
```

---

## 动画规范

### 持续时间
- 悬停: 150ms
- 切换: 200ms
- 加载: 300-500ms

### 缓动函数
- 默认: `cubic-bezier(0.25, 0.8, 0.25, 1)`
- 进入: `cubic-bezier(0, 0, 0.2, 1)`
- 离开: `cubic-bezier(0.4, 0, 1, 1)`

### 过渡动画

| 动画类型 | 持续时间 | 缓动函数 | 应用场景 |
|---------|---------|---------|---------|
| 侧边栏切换 | 200ms | cubic-bezier(0.4, 0, 0.2, 1) | 宽度变化 |
| 图标发光 | 150ms | ease-out | 悬停效果 |
| 表格行 | 100ms | ease | 悬停背景 |

---

## 无障碍设计 (Accessibility)

### 键盘导航
- 所有交互元素支持 `Tab` 键导航
- 焦点指示器清晰可见

```css
/* 焦点指示器 */
.sidebar-item:focus-visible,
.btn:focus-visible {
  outline: 2px solid rgb(var(--v-theme-primary));
  outline-offset: 2px;
}
```

### 屏幕阅读器
- 所有图标有 `aria-label` 属性
- 状态变化有适当的 ARIA 角色

### 颜色对比度
- 普通文本对比度至少 4.5:1
- 大文本对比度至少 3:1

---

## 修改文件清单

| 文件 | 操作 | 说明 |
|------|------|------|
| `src/components/layout/AppHeader.vue` | 优化 | 调整高度 36px → 32px，优化视觉效果 |
| `src/components/layout/AppSidebar.vue` | 优化 | 添加光剑激活效果，优化交互 |
| `src/components/layout/AppFooter.vue` | 检查 | 确认是否需要调整或保留 |
| `src/App.vue` | 优化 | 完善底部状态栏信息展示 |
| `src/components/common/SystemInfoBar.vue` | 优化 | 适配新的状态栏样式 |
| `src/views/AiChat/index.vue` | 重构 | 保留功能，优化 UI 添加 R2-D2 元素 |
| `src/views/AiChat/ChatMessage.vue` | 优化 | 已存在，添加 R2-D2 头像 |
| `src/views/hosts/HostsManager.vue` | 重构 | 保留功能，优化表格为 IDE 风格 |
| `src/assets/theme.css` | 补充 | 添加新样式类 |
| `src/assets/css/animations.css` | 补充 | 添加动画定义 |

---

## 验收标准

- [ ] 顶部栏 32px 高度，视觉精简
- [ ] 侧边栏 64px 图标模式，激活项有左侧光剑边框 + 发光
- [ ] 底部状态栏 24px 高度，信息布局合理
- [ ] Chat 模块 R2-D2 头像，三种状态（静态/思考/说话）
- [ ] Hosts 表格 32px 行高，状态指示器发光
- [ ] 所有动画 FPS 保持在 60 (使用 Chrome DevTools Performance 监测)
- [ ] 页面加载时间 < 2s（从冷启动到可交互）
- [ ] 内存占用增加 < 20MB（相比当前版本）
- [ ] 深色/浅色主题均适配
- [ ] 所有硬编码文本使用 i18n
- [ ] 键盘导航完整支持
- [ ] 窗口缩小时布局自适应

---

## 备注

本设计基于现有架构进行优化，保留所有现有功能，仅调整视觉表现和交互细节。

