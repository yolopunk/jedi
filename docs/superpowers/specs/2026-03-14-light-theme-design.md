# Jedi 浅色主题设计方案

**日期**: 2026-03-14
**版本**: 1.0
**状态**: 已批准

## 概述

为 Jedi 工具箱应用设计全局适配的浅色主题，保持科幻赛博风格的同时降低发光强度，确保在浅色背景下的可读性和视觉舒适度。

## 设计原则

1. **色彩对比**: 浅色背景 + 深色文字，确保 WCAG 可访问性
2. **科幻风格**: 保留边框设计，弱化 neon glow 效果
3. **Jedi 元素**: 保留光剑蓝配色，但降低饱和度
4. **代码优先**: 通过 CSS 变量实现主题切换

---

## 配色方案

### 核心颜色

| 用途 | 深色主题 | 浅色主题 |
|------|---------|---------|
| 背景 (app) | #111113 | #fafafa |
| 背景 (surface) | #18181b | #f4f4f5 |
| 背景 (sidebar) | #111113 | #f4f4f5 |
| 边框 | #27272a | #e4e4e7 |
| 文字 (primary) | #f4f4f5 | #18181b |
| 文字 (secondary) | #a1a1aa | #52525b |
| 主色 (primary) | #60a5fa | #0891b2 |
| 成功 (success) | #4ade80 | #10b981 |
| 警告 (warning) | #fbbf24 | #f59e0b |
| 错误 (error) | #f87171 | #ef4444 |

### Sci-Fi 霓虹色

| 用途 | 深色主题 | 浅色主题 |
|------|---------|---------|
| 青色 (cyan) | #00ffff | #0891b2 |
| 绿色 (green) | #00ff88 | #059669 |
| 洋红 (magenta) | #ff00ff | #a855f7 |
| 琥珀 (amber) | #ffaa00 | #d97706 |

### 发光效果

| 用途 | 深色主题 | 浅色主题 |
|------|---------|---------|
| 边框发光 | rgba(0, 255, 255, 0.2) | rgba(8, 145, 178, 0.15) |
| 文字发光 | rgba(0, 255, 255, 0.8) | rgba(8, 145, 178, 0.5) |
| 按钮悬停 | rgba(0, 255, 136, 0.07) | rgba(8, 145, 178, 0.05) |

---

## 组件适配规范

### 1. 顶部栏 (AppHeader)

```css
/* 深色 */
.app-header {
  background: #18181b;
  border-bottom: 1px solid #27272a;
}

/* 浅色 */
.light-theme .app-header {
  background: #ffffff;
  border-bottom: 1px solid #e4e4e7;
}
```

- 背景改为白色
- 边框改为浅灰色
- 窗口控制按钮颜色反转

### 2. 侧边栏 (AppSidebar)

```css
/* 深色 */
.jedi-sidebar {
  background: linear-gradient(180deg, #0d0d12, #0a0a0f);
  border-right: 1px solid rgba(0, 255, 255, 0.15);
}

/* 浅色 */
.light-theme .jedi-sidebar {
  background: linear-gradient(180deg, #f4f4f5, #fafafa);
  border-right: 1px solid rgba(8, 145, 178, 0.15);
}
```

- 激活项发光效果弱化
- 背景改为浅灰渐变

### 3. 状态指示器

```css
/* 深色 */
.status-light.online {
  background: #00ff88;
  box-shadow: 0 0 10px #00ff88;
}

/* 浅色 */
.light-theme .status-light.online {
  background: #10b981;
  box-shadow: 0 0 6px rgba(16, 185, 129, 0.4);
}
```

- 颜色更改为深绿色
- 发光强度降低

### 4. 按钮样式

```css
/* 深色 */
.console-btn {
  border-color: #00ff88;
  color: #00ff88;
}

/* 浅色 */
.light-theme .console-btn {
  border-color: #0891b2;
  color: #0891b2;
}
```

- 边框和文字颜色改为更深的青色
- 悬停背景更淡

### 5. 输入框

```css
/* 深色 */
.console-input {
  background: rgba(5, 5, 8, 0.9);
  border: 1px solid #1a1a3a;
  color: #00ffff;
}

/* 浅色 */
.light-theme .console-input {
  background: #ffffff;
  border: 1px solid #d4d4d8;
  color: #18181b;
}
```

### 6. 表格

```css
/* 深色 */
.hosts-table tr:hover {
  background: rgba(0, 255, 255, 0.05);
}

/* 浅色 */
.light-theme .hosts-table tr:hover {
  background: rgba(8, 145, 178, 0.05);
}
```

---

## CSS 变量结构

```css
:root {
  /* 基础变量 - 两个主题共用 */
  --jedi-font-ui: ...;
  --jedi-font-mono: ...;
  
  /* 浅色主题变量 */
  --jedi-bg-app: #fafafa;
  --jedi-bg-surface: #f4f4f5;
  --jedi-border: #e4e4e7;
  --jedi-text-primary: #18181b;
  --jedi-primary: #0891b2;
  --scifi-cyan: #0891b2;
  --scifi-green: #059669;
  
  /* Vuetify 变量覆盖 */
  --v-theme-background: 250, 250, 250;
  --v-theme-surface: 244, 244, 245;
  --v-theme-primary: 8, 145, 178;
}

/* 深色主题 */
.dark-theme {
  --jedi-bg-app: #111113;
  --jedi-bg-surface: #18181b;
  --jedi-border: #27272a;
  --jedi-text-primary: #f4f4f5;
  --jedi-primary: #60a5fa;
  --scifi-cyan: #00ffff;
  --scifi-green: #00ff88;
  
  --v-theme-background: 17, 17, 19;
  --v-theme-surface: 24, 24, 27;
  --v-theme-primary: 96, 165, 250;
}
```

---

## 实施计划

### Phase 1: CSS 变量完善
- 调整 `src/assets/theme.css` 中的 `.light-theme` 变量
- 确保所有硬编码颜色使用 CSS 变量

### Phase 2: 布局组件
- AppHeader.vue
- AppSidebar.vue
- AppFooter.vue

### Phase 3: 功能模块
- HostsManager.vue / HostsTable.vue
- PodcastManager.vue
- WallpaperManager.vue
- Chat 页面

### Phase 4: 测试验证
- 深色/浅色主题切换
- 各组件在不同主题下的显示

---

## 验收标准

- [ ] 切换到浅色主题后所有页面正常显示
- [ ] 文字可读性符合 WCAG AA 标准
- [ ] 发光效果在浅色下不刺眼
- [ ] 深色主题不受影响
- [ ] 主题切换无闪烁
- [ ] i18n 在两种主题下正常显示
