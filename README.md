# Jedi 工具箱

> 一个多功能开发辅助工具集，提升开发效率，目前包含 Hosts 管理模块。

[English](README.en.md) | 简体中文

## 📷 截图

![Jedi 工具箱截图](public/screenshot.png)

## 📖 简介

Jedi 工具箱是一个基于 Tauri v2 和 Vue 3 构建的跨平台应用，旨在为开发者提供一系列实用工具，提高开发效率。目前已实现的 Hosts 管理模块可以帮助开发者轻松创建、编辑和切换不同的 hosts 配置组，使开发、测试和部署过程更加高效。

名称 "Jedi" 灵感来自《星球大战》中的绝地武士，象征着这个工具可以帮助开发者像绝地武士一样轻松掌控不同的开发环境配置。

## ✨ 功能特性

### Hosts 管理模块

- **分组管理**: 按项目或环境创建 hosts 配置组
- **一键切换**: 快速切换不同的配置
- **全局开关**: 一键启用/禁用所有 hosts 配置
- **单条目控制**: 启用/禁用特定的 hosts 条目
- **域名快速访问**: 直接点击域名在浏览器中打开

### 系统功能

- **系统托盘**: 最小化到系统托盘，随时可用
- **跨平台支持**: 支持 Windows、macOS 和 Linux
- **现代界面**: 基于 Vuetify 的美观 UI
- **轻量级**: 资源占用少，启动快速
- **动态版本号**: 自动从 Cargo.toml 读取版本信息
- **开源免费**: 完全开源，免费使用

## 🚀 安装与运行

### 预先准备

- [Node.js](https://nodejs.org/) (v18+)
- [pnpm](https://pnpm.io/) (v8+)
- [Rust](https://www.rust-lang.org/) (v1.70+)
- [Tauri CLI](https://tauri.app/v2/guides/getting-started/prerequisites) (v2.x)

### 开发步骤

1. 克隆仓库

```bash
git clone https://github.com/yolopunk/jedi.git
cd jedi
```

2. 安装依赖

```bash
pnpm install
```

3. 开发模式运行

```bash
pnpm tauri dev
```

4. 构建生产版本

```bash
pnpm tauri build
```

### 下载安装

您也可以直接从 [GitHub Releases](https://github.com/yolopunk/jedi/releases) 页面下载预编译的安装包。

- Windows: `.msi` 安装包
- macOS: `.dmg` 安装包
- Linux: `.AppImage` 或 `.deb` 包

## 🔧 使用指南

### Hosts 管理模块

1. **添加分组**: 点击 "+" 按钮创建新的 hosts 配置组
2. **添加条目**: 在分组中点击 "添加条目" 按钮添加新的 hosts 条目
3. **启用/禁用条目**: 使用每个条目旁边的开关控制其状态
4. **全局开关**: 使用顶部的主开关控制所有 hosts 配置
5. **访问域名**: 点击域名可在浏览器中直接访问

### Hosts 文件格式

Jedi 使用特殊的格式标记来管理 hosts 文件中的内容：

```
# === JEDI HOSTS MANAGER ===
# +默认+
127.0.0.1 localhost
# +开发环境+
10.50.128.32 docker.dev.io
10.50.128.32 api.dev.io
# === END JEDI HOSTS MANAGER ===
```

Jedi 只管理这些标记之间的内容，不会修改文件的其他部分。

### 快捷键

- **Ctrl+N**: 新建分组
- **Ctrl+A**: 添加条目
- **Ctrl+S**: 保存当前配置
- **Ctrl+R**: 重新加载配置
- **F1**: 打开帮助对话框

## 🔐 权限说明

Jedi 需要管理员权限来修改 hosts 文件。在 Windows 上，它会请求 UAC 提升；在 macOS 和 Linux 上，您可能需要输入管理员密码。

## 💻 技术栈

- **前端**: Vue 3 + TypeScript + Vuetify 3
- **后端**: Rust + Tauri v2
- **构建工具**: Vite
- **包管理器**: pnpm

## 📚 项目结构

```
├── src/               # 前端源代码
│   ├── api/           # API 调用
│   ├── assets/        # 静态资源
│   ├── components/    # Vue 组件
│   ├── types/         # TypeScript 类型定义
│   ├── utils/         # 工具函数
│   ├── App.vue        # 主应用组件
│   └── main.ts        # 应用入口
├── src-tauri/         # Tauri/Rust 后端
│   ├── src/           # Rust 源代码
│   │   ├── api/       # API 实现
│   │   ├── config/    # 配置相关
│   │   └── utils/     # 工具函数
│   ├── Cargo.toml     # Rust 依赖配置
│   └── tauri.conf.json # Tauri 配置
├── public/            # 公共资源
├── index.html         # HTML 模板
├── package.json       # 项目配置
└── vite.config.ts     # Vite 配置
```

## 👨‍💻 贡献指南

欢迎对项目进行贡献！如果您想贡献代码，请遵循以下步骤：

1. Fork 这个仓库
2. 创建您的特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交您的更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 打开一个 Pull Request

## 📝 许可证

本项目采用 MIT 许可证。
