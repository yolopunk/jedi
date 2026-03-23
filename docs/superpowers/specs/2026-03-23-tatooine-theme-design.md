# Tatooine 主题设计方案

**日期**: 2026-03-23
**版本**: 1.0
**状态**: 待审查

## 概述

为 Jedi 工具箱应用设计一套全新的"浅色主题"——不再是传统的纯白亮色，而是基于《星球大战》塔图因（Tatooine）星球的暖沙黄金属质感主题。

**设计理念**:
- Light theme ≠ 纯白亮色
- 暖沙黄背景 + 黄铜/铜色金属装饰
- 去除霓虹发光，改用微妙的金属质感
- 保持科幻风格的同时确保优秀的可读性

---

## 设计原则

1. **沙黄质感**: 以塔图因沙漠为灵感的暖色调
2. **金属光泽**: 黄铜、铜色代替霓虹发光
3. **可读性优先**: 确保文字对比度 >= 4.5:1
4. **无霓虹**: 完全去除 neon glow 效果
5. **一致**: 所有模块遵循统一的视觉语言
6. **保留功能**: 深色主题完全不受影响

---

## 色彩系统

### 核心配色（Tatooine 前哨站）

| 用途 | 颜色值 | 对比度 (vs bg) | 说明 |
|------|--------|----------------|------|
| **背景 (app)** | `#f5e6d3` → `#e8d4bc` | - | 暖沙黄渐变 |
| **背景 (surface)** | `#efe0cc` | - | 比背景稍浅，用于卡片/面板 |
| **背景 (sidebar)** | `#e8d4bc` | - | 侧边栏稍深一点 |
| **背景 (input)** | `#faf3e8` | - | 输入框背景 |
| **边框** | `#b8860b` | - | 黄铜色，纤细线条 |
| **边框 (hover)** | `#cd7f32` | - | 悬停时稍亮 |
| **文字 (primary)** | `#3d2914` | ~7:1 | 深棕色，高对比度 |
| **文字 (secondary)** | `#6b4423` | ~4.5:1 | 中棕色 |
| **文字 (tertiary)** | `#9a7b5a` | ~2.5:1 | 浅棕色 |
| **主色** | `#cd7f32` | - | 铜色 |
| **成功** | `#daa520` | - | 暗金色 |
| **警告** | `#cd853f` | - | 琥珀铜 |
| **错误** | `#b22222` | - | 铁锈红 |

### 金属高光系统（替代霓虹发光）

| 效果 | 值 | 说明 |
|------|-----|------|
| 边框高光 | `rgba(205, 127, 50, 0.15)` | 细微的铜色光晕 |
| 悬停阴影 | `0 2px 8px rgba(184, 134, 11, 0.2)` | 柔和阴影 |
| 按钮高光 | `linear-gradient(135deg, #e6be8a 0%, #cd7f32 100%)` | 金属渐变 |
| 输入聚焦 | `0 0 0 3px rgba(205, 127, 50, 0.15)` | 细微光晕 |

---

## 组件设计规范

### 1. 顶部栏 (AppHeader)

**规格**:
- **高度**: 32px（保持不变）
- **背景**: `#efe0cc`（surface 色）
- **底部边框**: 1px solid `#b8860b`（黄铜色）

**视觉元素**:
- 窗口控制按钮: 深铜色圆圈（红/黄/绿）
- 标题文字: 深棕色，小型字体
- 无发光效果

### 2. 侧边栏 (AppSidebar)

**规格**:
- **图标模式宽度**: 64px
- **背景**: `#e8d4bc`（比主背景稍深）
- **右侧边框**: 1px solid `#b8860b`

**导航项状态**:
- **默认**: 图标居中，无背景
- **悬停**: 浅铜色背景 `rgba(205, 127, 50, 0.08)`
- **激活**: 左侧 2px 铜色边框 + 浅铜色背景 `rgba(205, 127, 50, 0.15)`
- **无霓虹发光**，只用边框和背景变化

### 3. 底部状态栏

**规格**:
- **高度**: 24px
- **背景**: `#efe0cc`
- **顶部边框**: 1px solid `#b8860b`
- **文字**: 浅棕色 `#9a7b5a`

### 4. 按钮样式

```css
/* 默认按钮 */
.btn {
  border: 1px solid #b8860b;
  background: #efe0cc;
  color: #3d2914;
  transition: all 150ms ease;
}

.btn:hover {
  border-color: #cd7f32;
  box-shadow: 0 2px 8px rgba(184, 134, 11, 0.2);
}

/* 主要按钮 - 金属渐变 */
.btn-primary {
  background: linear-gradient(135deg, #e6be8a 0%, #cd7f32 100%);
  border-color: #cd7f32;
  color: #fff;
  text-shadow: 0 1px 2px rgba(0,0,0,0.2);
}

/* 状态按钮 */
.btn-success { border-color: #daa520; color: #daa520; }
.btn-warning { border-color: #cd853f; color: #cd853f; }
.btn-error   { border-color: #b22222; color: #b22222; }
```

### 5. 输入框

```css
.console-input {
  background: #faf3e8;
  border: 1px solid #b8860b;
  color: #3d2914;
}

.console-input:focus {
  outline: none;
  border-color: #cd7f32;
  box-shadow: 0 0 0 3px rgba(205, 127, 50, 0.15);
}
```

### 6. 卡片/面板

```css
.card {
  background: #efe0cc;
  border: 1px solid #b8860b;
  border-radius: 6px;
}
```

---

## 模块适配

### Hosts 表格

**规格**:
- **行高**: 32px
- **表头背景**: `#e8d4bc`
- **边框**: 1px solid `#b8860b`
- **悬停行**: 浅铜色背景 `rgba(205, 127, 50, 0.08)`

**状态指示器**:
- **启用**: 暗金色 `#daa520`，无光晕
- **禁用**: 浅棕色 `#9a7b5a`

### Chat 模块

**保留科幻控制台风格**，但调整色调：
- 背景: 浅沙色 `#faf3e8`
- 边框: 黄铜色
- 扫描线/CRT效果: 调整为适合浅色背景的版本（降低不透明度）
- R2-D2 头像: 调整颜色以适配沙黄背景

### 播客/壁纸模块

- 统一使用新的色彩系统
- 卡片使用沙色背景 + 黄铜边框
- 去除所有霓虹发光效果

---

## CSS 变量结构

```css
.light-theme {
  /* Backgrounds */
  --jedi-bg-app: #f5e6d3;
  --jedi-bg-surface: #efe0cc;
  --jedi-bg-surface-hover: #e8d4bc;
  --jedi-bg-sidebar: #e8d4bc;
  --jedi-bg-input: #faf3e8;

  /* Borders - Brass */
  --jedi-border: #b8860b;
  --jedi-border-focus: #cd7f32;

  /* Text - Sandstone Browns */
  --jedi-text-primary: #3d2914;
  --jedi-text-secondary: #6b4423;
  --jedi-text-tertiary: #9a7b5a;
  --jedi-text-inverse: #ffffff;

  /* Functional - Metal Tones */
  --jedi-primary: #cd7f32;
  --jedi-primary-hover: #e6be8a;
  --jedi-accent: #daa520;
  --jedi-success: #daa520;
  --jedi-warning: #cd853f;
  --jedi-danger: #b22222;

  /* Sci-Fi Console (Tatooine Version) */
  --scifi-bg: #f5e6d3;
  --scifi-bg-terminal: #faf3e8;
  --scifi-cyan: #b8860b;
  --scifi-green: #daa520;
  --scifi-magenta: #cd7f32;
  --scifi-amber: #cd853f;
  --scifi-red: #b22222;
  --scifi-border: #b8860b;

  /* Shadows - Metal Glow (No Neon) */
  --jedi-shadow-sm: 0 2px 4px rgba(184, 134, 11, 0.1);
  --jedi-shadow-md: 0 2px 8px rgba(184, 134, 11, 0.2);

  /* Glow effects - SUBTLE METAL ONLY */
  --glow-border: rgba(205, 127, 50, 0.15);
  --glow-text: rgba(184, 134, 11, 0.3);
  --glow-hover: rgba(205, 127, 50, 0.08);

  /* Vuetify Theme Variables (RGB channels) */
  --v-theme-background: 245, 230, 211;
  --v-theme-surface: 239, 224, 204;
  --v-theme-surface-light: 232, 212, 188;
  --v-theme-surface-dark: 250, 243, 232;
  --v-theme-primary: 205, 127, 50;
  --v-theme-primary-light: 230, 190, 138;
  --v-theme-primary-dark: 184, 134, 11;
  --v-theme-secondary: 107, 68, 35;
  --v-theme-secondary-light: 154, 123, 90;
  --v-theme-secondary-dark: 61, 41, 20;
  --v-theme-error: 178, 34, 34;
  --v-theme-error-light: 205, 68, 68;
  --v-theme-error-dark: 139, 0, 0;
  --v-theme-info: 184, 134, 11;
  --v-theme-info-light: 218, 165, 32;
  --v-theme-info-dark: 139, 105, 20;
  --v-theme-success: 218, 165, 32;
  --v-theme-success-light: 230, 190, 138;
  --v-theme-success-dark: 139, 105, 20;
  --v-theme-warning: 205, 133, 63;
  --v-theme-warning-light: 222, 170, 120;
  --v-theme-warning-dark: 139, 90, 43;
  --v-theme-on-background: 61, 41, 20;
  --v-theme-on-surface: 61, 41, 20;
  --v-theme-on-primary: 255, 255, 255;
  --v-theme-on-secondary: 255, 255, 255;
  --v-theme-on-error: 255, 255, 255;
  --v-theme-on-info: 61, 41, 20;
  --v-theme-on-success: 61, 41, 20;
  --v-theme-on-warning: 61, 41, 20;
}
```

---

## 实施计划

### Phase 1: CSS 变量更新
- 重写 `src/assets/theme.css` 中的 `.light-theme` 变量
- 确保所有硬编码颜色使用 CSS 变量

### Phase 2: Vuetify 主题配置
- 更新 `src/plugins/vuetify.ts` 中的 light theme 配置

### Phase 3: 组件验证
- AppHeader.vue
- AppSidebar.vue
- AppFooter.vue
- HostsManager.vue / HostsTable.vue
- AiChat 页面
- PodcastManager.vue
- WallpaperManager.vue

### Phase 4: 测试验证
- 深色/浅色主题切换
- 各模块在不同主题下的显示
- 对比度验证

---

## 修改文件清单

| 文件 | 操作 |
|------|------|
| `src/assets/theme.css` | 重写 `.light-theme` 变量 |
| `src/plugins/vuetify.ts` | 配置 Vuetify light 主题 |
| 各组件文件 | 确保使用 CSS 变量而非硬编码色 |

---

## 验收标准

- [ ] 主题切换流畅，无闪烁
- [ ] 所有模块在 Tatooine 主题下正常显示
- [ ] 文字对比度 >= 4.5:1
- [ ] 无霓虹发光效果残留
- [ ] 深色主题不受影响
- [ ] 所有交互状态（hover/focus/active）正常
- [ ] 窗口缩小时布局自适应

---

## 备注

本设计将"light theme"重新定义为"日间主题"而非"亮色主题"，采用暖沙黄配色方案，既保持了科幻风格，又避免了纯白背景的刺眼感。
