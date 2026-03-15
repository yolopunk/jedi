# Jedi 浅色主题实施计划

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 全局适配科幻赛博风格的浅色主题，保持 Jedi 元素，降低发光强度

**Architecture:** 通过 CSS 变量实现主题切换，完善 theme.css 中的浅色变量，逐步适配各组件

**Tech Stack:** Vue 3 + CSS Variables + Vuetify 3

---

## 实施步骤

### Phase 1: 完善 CSS 变量系统

#### Task 1.1: 完善 theme.css 浅色主题变量

**Files:**
- Modify: `src/assets/theme.css:98-186`

- [ ] **Step 1: 检查当前浅色变量**

Read: `src/assets/theme.css` (lines 98-186)
Note: 确认现有 `.light-theme` 变量定义

- [ ] **Step 2: 更新浅色主题核心变量**

Edit: `src/assets/theme.css`

替换 `.light-theme` 块中的变量：

```css
.light-theme {
  /* Backgrounds */
  --jedi-bg-app: #fafafa;
  --jedi-bg-surface: #f4f4f5;
  --jedi-bg-surface-hover: #e4e4e7;
  --jedi-bg-sidebar: #f4f4f5;
  --jedi-bg-input: #ffffff;

  /* Borders */
  --jedi-border: #e4e4e7;
  --jedi-border-focus: #0891b2;

  /* Text */
  --jedi-text-primary: #18181b;
  --jedi-text-secondary: #52525b;
  --jedi-text-tertiary: #a1a1aa;
  --jedi-text-inverse: #ffffff;

  /* Functional */
  --jedi-primary: #0891b2;
  --jedi-primary-hover: #0e7490;
  --jedi-accent: #6366f1;
  --jedi-success: #10b981;
  --jedi-warning: #f59e0b;
  --jedi-danger: #ef4444;

  /* Syntax */
  --jedi-syntax-ip: #7c3aed;
  --jedi-syntax-domain: #059669;

  /* Sci-Fi Console (Light Theme) - 弱化发光 */
  --scifi-bg: #f8f8fa;
  --scifi-bg-terminal: #f0f0f4;
  --scifi-cyan: #0891b2;
  --scifi-green: #059669;
  --scifi-magenta: #a855f7;
  --scifi-amber: #d97706;
  --scifi-red: #dc2626;
  --scifi-border: #d4d4d8;

  /* Shadows */
  --jedi-shadow-sm: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
  --jedi-shadow-md: 0 4px 6px -1px rgba(0, 0, 0, 0.1);

  /* Vuetify Theme Variables */
  --v-theme-background: 250, 250, 250;
  --v-theme-surface: 244, 244, 245;
  --v-theme-surface-light: 255, 255, 255;
  --v-theme-surface-dark: 228, 228, 231;
  --v-theme-primary: 8, 145, 178;
  --v-theme-primary-light: 6, 182, 212;
  --v-theme-primary-dark: 7, 130, 153;
  --v-theme-secondary: 82, 82, 91;
  --v-theme-secondary-light: 113, 113, 122;
  --v-theme-secondary-dark: 63, 63, 70;
  --v-theme-error: 239, 68, 68;
  --v-theme-success: 16, 185, 129;
  --v-theme-warning: 245, 158, 11;
  --v-theme-on-background: 24, 24, 27;
  --v-theme-on-surface: 24, 24, 27;
  --v-theme-on-primary: 255, 255, 255;
}
```

- [ ] **Step 3: 更新发光效果变量**

在 `.light-theme` 块末尾添加发光效果变量：

```css
  /* 发光效果 - 弱化 */
  --glow-border: rgba(8, 145, 178, 0.15);
  --glow-text: rgba(8, 145, 178, 0.5);
  --glow-hover: rgba(8, 145, 178, 0.08);
}
```

- [ ] **Step 4: 验证构建**

Run: `cd /Users/cynosure/workspace/github/jedi && pnpm build`
Expected: Build successful

---

### Phase 2: 布局组件适配

#### Task 2.1: AppHeader.vue 浅色适配

**Files:**
- Modify: `src/components/layout/AppHeader.vue:230-280`

- [ ] **Step 1: 检查现有样式**

Read: `src/components/layout/AppHeader.vue` (style section)
Search for: `.app-header`, `.window-controls`

- [ ] **Step 2: 添加浅色主题样式**

在 AppHeader.vue 的 `<style>` 中添加：

```css
/* 浅色主题 - 顶部栏 */
.light-theme .app-header {
  background: #ffffff;
  border-bottom: 1px solid #e4e4e7;
}

.light-theme .header-center .console-title {
  color: #0891b2;
  text-shadow: none;
}

.light-theme .header-center .console-prefix,
.light-theme .header-center .console-suffix {
  color: #94a3b8;
}

.light-theme .status-light.online {
  background: #10b981;
  box-shadow: 0 0 6px rgba(16, 185, 129, 0.4);
}

.light-theme .status-light.standby {
  background: #f59e0b;
  box-shadow: 0 0 6px rgba(245, 158, 11, 0.4);
}

.light-theme .status-light.scanning {
  background: #0891b2;
  box-shadow: 0 0 6px rgba(8, 145, 178, 0.4);
}
```

#### Task 2.2: AppSidebar.vue 浅色适配

**Files:**
- Modify: `src/components/layout/AppSidebar.vue:230-280`

- [ ] **Step 1: 检查现有样式**

Read: `src/components/layout/AppSidebar.vue` (style section)
Search for: `.jedi-sidebar`, `.sidebar-item`

- [ ] **Step 2: 添加浅色主题样式**

```css
/* 浅色主题 - 侧边栏 */
.light-theme .jedi-sidebar {
  background: linear-gradient(180deg, #f4f4f5 0%, #fafafa 100%);
  border-right-color: rgba(8, 145, 178, 0.15);
}

.light-theme .logo-area {
  border-bottom-color: rgba(8, 145, 178, 0.15) !important;
}

.light-theme .sidebar-title {
  color: #0891b2 !important;
  text-shadow: none;
}

.light-theme .sidebar-item::before {
  background: #0891b2;
  box-shadow: 0 0 6px rgba(8, 145, 178, 0.5);
}

.light-theme .sidebar-item.v-list-item--active::before {
  opacity: 1;
  transform: scaleY(1);
}

.light-theme .sidebar-item.v-list-item--active .sidebar-icon-container {
  filter: drop-shadow(0 0 4px rgba(8, 145, 178, 0.5));
}

.light-theme .nav-text {
  color: #52525b;
}

.light-theme .sidebar-item.v-list-item--active .nav-text {
  color: #0891b2;
  text-shadow: none;
}

.light-theme .sidebar-footer {
  border-top-color: rgba(8, 145, 178, 0.15);
  background: linear-gradient(0deg, #f4f4f5 0%, transparent 100%);
}

.light-theme .status-label {
  color: #a1a1aa;
}

.light-theme .status-value.online {
  color: #10b981;
  text-shadow: none;
}

.light-theme .footer-time {
  color: #0891b2;
  text-shadow: none;
}
```

#### Task 2.3: AppFooter.vue 浅色适配

**Files:**
- Modify: `src/components/layout/AppFooter.vue:150-200`

- [ ] **Step 1: 检查现有样式**

Read: `src/components/layout/AppFooter.vue` (style section)

- [ ] **Step 2: 添加浅色主题样式**

```css
/* 浅色主题 - 底部状态栏 */
.light-theme .app-footer {
  background: #ffffff;
  border-top: 1px solid #e4e4e7;
}

.light-theme .footer-btn {
  color: #52525b;
}

.light-theme .footer-btn:hover {
  color: #0891b2;
}
```

---

### Phase 3: 功能模块适配

#### Task 3.1: HostsTable.vue 浅色适配

**Files:**
- Modify: `src/components/hosts/tables/HostsTable.vue:600-800`

- [ ] **Step 1: 检查现有表格样式**

Read: `src/components/hosts/tables/HostsTable.vue` (style section)

- [ ] **Step 2: 添加浅色主题表格样式**

```css
/* 浅色主题 - Hosts 表格 */
.light-theme .hosts-console-container {
  background: #fafafa;
}

.light-theme .console-header {
  background: linear-gradient(180deg, #ffffff 0%, #f4f4f5 100%);
  border-bottom-color: rgba(8, 145, 178, 0.15);
}

.light-theme .console-toolbar {
  background: #f4f4f5;
  border-bottom-color: rgba(8, 145, 178, 0.1);
}

.light-theme .table-header-row {
  background: #f4f4f5;
  border-bottom-color: #e4e4e7;
}

.light-theme .table-cell {
  color: #52525b;
}

.light-theme .cell-label {
  color: #0891b2;
}

.light-theme .table-row {
  border-bottom-color: #f4f4f5;
}

.light-theme .table-row:hover {
  background: rgba(8, 145, 178, 0.05);
}

.light-theme .status-enabled {
  background: #10b981;
  box-shadow: 0 0 6px rgba(16, 185, 129, 0.4);
}

.light-theme .status-disabled {
  background: #d4d4d8;
  box-shadow: none;
}
```

#### Task 3.2: SettingsDialog.vue 浅色适配

**Files:**
- Modify: `src/components/dialogs/SettingsDialog.vue:800-900`

- [ ] **Step 1: 检查设置对话框样式**

Read: `src/components/dialogs/SettingsDialog.vue` (style section)
Search for: `.scifi-card`, `.setting-item`

- [ ] **Step 2: 添加浅色主题样式**

```css
/* 浅色主题 - 设置对话框 */
.light-theme .scifi-card {
  background: linear-gradient(135deg, #ffffff 0%, #f4f4f5 100%);
  border-color: rgba(8, 145, 178, 0.2);
}

.light-theme .scifi-card::before {
  background: linear-gradient(90deg, transparent, #0891b2, transparent);
}

.light-theme .console-title-bar {
  background: linear-gradient(180deg, #ffffff 0%, #f4f4f5 100%);
  border-bottom-color: rgba(8, 145, 178, 0.15);
}

.light-theme .dialog-title {
  color: #0891b2;
  text-shadow: none;
}

.light-theme .console-card-text {
  background: #fafafa;
}

.light-theme .setting-item {
  border-bottom-color: #f4f4f5;
}

.light-theme .setting-label {
  color: #18181b;
}

.light-theme .setting-subtitle {
  color: #71717a;
}

.light-theme .console-menu {
  background: #ffffff !important;
  border-color: rgba(8, 145, 178, 0.2) !important;
}

.light-theme .menu-item {
  color: #52525b;
}

.light-theme .menu-item:hover {
  background: rgba(8, 145, 178, 0.08);
  color: #0891b2;
}

.light-theme .menu-check {
  color: #10b981;
}

.light-theme .tab-button {
  color: #71717a;
}

.light-theme .tab-button.active {
  color: #0891b2;
  border-bottom-color: #0891b2;
}
```

---

### Phase 4: 验证测试

#### Task 4.1: 主题切换验证

- [ ] **Step 1: 启动开发服务器**

Run: `cd /Users/cynosure/workspace/github/jedi && pnpm tauri dev`

- [ ] **Step 2: 测试浅色主题**

Check:
- [ ] 切换到浅色主题
- [ ] 顶部栏显示白色背景
- [ ] 侧边栏显示浅灰背景
- [ ] 状态指示器颜色正确
- [ ] 按钮边框颜色正确

- [ ] **Step 3: 测试深色主题**

Check:
- [ ] 切换回深色主题
- [ ] 所有样式正常恢复

- [ ] **Step 4: 测试各页面**

Check:
- [ ] Hosts 页面浅色正常
- [ ] Podcast 页面浅色正常
- [ ] Wallpapers 页面浅色正常
- [ ] Chat 页面浅色正常

---

## 提交指南

每个阶段完成后单独提交：

```bash
# Phase 1
git add src/assets/theme.css
git commit -m "feat: enhance light theme CSS variables"

# Phase 2
git add src/components/layout/AppHeader.vue
git add src/components/layout/AppSidebar.vue
git add src/components/layout/AppFooter.vue
git commit -m "style: add light theme support to layout components"

# Phase 3
git add src/components/hosts/tables/HostsTable.vue
git add src/components/dialogs/SettingsDialog.vue
git commit -m "style: add light theme support to modules"

# Phase 4
git commit -m "test: verify light/dark theme switching"
```
