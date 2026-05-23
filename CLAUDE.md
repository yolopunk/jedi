# Jedi 工具箱 - 代码库概览

## 项目概述

Jedi 是一个基于 Tauri v2 和 Vue 3 构建的跨平台桌面应用程序，为开发者提供一系列实用工具来提升日常开发效率。

**项目类型**: 桌面应用程序 (Tauri + Vue 3)
**主要功能**: 
- Hosts 管理（分组管理、一键切换、权限控制）
- 知识壁纸（分类浏览、预览、一键设置）
- 小宇宙播客（订阅管理、播放、OPML导入）
- 系统信息展示与设置

---

## 技术栈

### 前端
- **框架**: Vue 3 + TypeScript
- **UI库**: Vuetify 3
- **构建工具**: Vite 6
- **包管理器**: pnpm
- **路由**: Vue Router 4
- **国际化**: Vue I18n
- **3D渲染**: Three.js + TresJS
- **状态管理**: Vue 3 Composables (no Vuex/Pinia)

### 后端
- **语言**: Rust
- **框架**: Tauri v2
- **关键依赖**:
  - sysinfo - 系统信息获取
  - reqwest - HTTP请求
  - tokio - 异步运行时
  - regex - 正则表达式处理
  - rss - RSS解析
  - wallpaper - 壁纸设置
  - nvml-wrapper - NVIDIA GPU信息

### 核心插件
- tauri-plugin-fs - 文件系统访问
- tauri-plugin-shell - 系统命令执行
- tauri-plugin-autostart - 开机自启
- tauri-plugin-store - 数据存储
- tauri-plugin-updater - 自动更新
- tauri-plugin-process - 进程管理

---

## 项目结构

```
jedi/
├── src/                      # 前端源代码
│   ├── api/                  # API 接口层（与后端通信）
│   │   ├── app.ts            # 应用信息API
│   │   ├── hosts.ts          # Hosts管理API
│   │   ├── podcast.ts        # 播客API
│   │   ├── wallpaper.ts      # 壁纸API
│   │   ├── update.ts         # 更新API
│   │   └── index.ts          # 统一导出
│   ├── assets/               # 静态资源
│   ├── components/           # Vue组件
│   │   ├── common/           # 通用组件
│   │   ├── dialogs/          # 对话框组件
│   │   ├── hosts/            # Hosts管理组件
│   │   ├── layout/           # 布局组件
│   │   └── podcast/          # 播客组件
│   ├── composables/          # Vue 3组合式函数
│   │   ├── useHostsData.ts   # Hosts数据管理
│   │   ├── useStorage.ts     # 本地存储
│   │   ├── useTheme.ts       # 主题管理
│   │   ├── useUpdate.ts      # 更新检查
│   │   ├── useWallpaper.ts   # 壁纸管理
│   │   └── useAudioPlayer.ts # 音频播放器
│   ├── i18n/                 # 国际化配置
│   ├── plugins/              # Vue插件
│   ├── router/               # 路由配置
│   ├── types/                # TypeScript类型定义
│   ├── utils/                # 工具函数
│   ├── views/                # 页面组件
│   │   ├── hosts/            # Hosts管理页面
│   │   ├── podcast/          # 播客页面
│   │   └── wallpapers/       # 壁纸页面
│   ├── App.vue               # 根组件
│   └── main.ts               # 应用入口
├── src-tauri/                # Tauri后端源代码
│   ├── src/
│   │   ├── api/              # Rust API实现
│   │   │   ├── app.rs        # 应用信息
│   │   │   ├── hosts.rs      # Hosts管理
│   │   │   ├── os.rs         # 系统信息
│   │   │   ├── podcast.rs    # 播客管理
│   │   │   └── wallpapers.rs # 壁纸管理
│   │   ├── config/           # 配置管理
│   │   ├── utils/            # Rust工具函数
│   │   └── main.rs           # 后端入口
│   ├── Cargo.toml            # Rust依赖配置
│   ├── tauri.conf.json       # Tauri配置
│   └── rustfmt.toml          # Rust格式化配置
├── public/                   # 公共资源
├── scripts/                  # 脚本文件
│   └── release.js            # 版本发布脚本
├── .github/
│   └── workflows/
│       └── release.yml       # GitHub自动发布工作流
├── package.json              # 前端依赖配置
├── tsconfig.json             # TypeScript配置
├── vite.config.ts            # Vite配置
└── README.md                 # 项目说明
```

---

## 开发环境搭建

### 前置要求
- Node.js >= 18
- pnpm >= 8
- Rust >= 1.70
- Tauri CLI v2.x

### 安装依赖
```bash
pnpm install
```

### 开发模式
```bash
pnpm tauri dev
```

### 构建生产版本
```bash
pnpm tauri build
```

### 前端构建（仅用于预览）
```bash
pnpm build          # 构建前端
pnpm preview        # 预览构建结果
```

---

## 架构设计

### 通信模式
- **前端 → 后端**: Tauri invoke 命令（通过 `@tauri-apps/api/core`）
- **后端 → 前端**: Tauri events 或直接返回值
- **API定义**: 在 `src/api/` 中封装，对应 `src-tauri/src/api/` 中的Rust实现

### 数据存储
- **本地存储**: tauri-plugin-store（JSON格式）
- **配置文件**: JSON格式，存储在系统配置目录
- **Hosts文件**: 系统级文件（需要管理员权限）
- **播客数据**: OPML导入 + 本地缓存

### 状态管理
- 使用 Vue 3 Composables (in `src/composables/`)
- 无全局状态管理库，采用"组件 → composable → API"模式

---

## 关键开发命令

### 版本管理
```bash
pnpm release [major|minor|patch|x.y.z]  # 自动更新版本号并创建Git标签
```
该脚本会更新以下文件：
- package.json (version)
- src-tauri/tauri.conf.json (version)
- src-tauri/Cargo.toml (version)
- src-tauri/Cargo.lock (dependency sync)

### 代码质量
```bash
cargo check          # 检查Rust代码
cargo fmt            # 格式化Rust代码
cargo clippy         # Rust代码 linting
```

### 测试
```bash
cargo test           # 运行Rust测试
```

---

## CI/CD 流程

### GitHub Actions 自动发布

**.github/workflows/release.yml** 配置了自动构建和发布流程：

1. **触发条件**: 
   - 推送到 `v*` 标签
   - 手动 workflow dispatch

2. **构建矩阵**:
   - macOS (x86_64 + arm64 通用二进制)
   - Windows (x86_64)
   - Linux (amd64)

3. **发布产物**:
   - macOS: .dmg 安装包
   - Windows: .msi 安装包
   - Linux: .AppImage 和 .deb 包

4. **签名**: 使用 Tauri 签名密钥（通过 GitHub Secrets 配置）

---

## 核心功能实现

### Hosts 管理
- **文件位置**: 系统默认 hosts 文件（需要管理员权限）
- **格式**: 使用特殊标记 `# === JEDI HOSTS MANAGER ===` 标记管理区域
- **权限**: 在 Windows 上请求 UAC 提升，macOS/Linux 需要 sudo
- **API**: `src/api/hosts.ts` ←→ `src-tauri/src/api/hosts.rs`

### 壁纸管理
- **源**: 本地 JSON 文件 + 远程同步
- **API**: `src/api/wallpaper.ts` ←→ `src-tauri/src/api/wallpapers.rs`
- **设置**: 使用 system wallpaper API（跨平台支持）

### 播客管理
- **源**: RSS订阅 + 小宇宙API
- **功能**: OPML导入、播放、搜索
- **API**: `src/api/podcast.ts` ←→ `src-tauri/src/api/podcast.rs`

---

## 代码风格与规范

### TypeScript/Vue
- 使用 ESLint（默认Vue配置）
- 组件命名: PascalCase 或 kebab-case
- 文件命名: kebab-case
- 类型定义: 在 `src/types/` 中集中管理

### Rust
- 使用 rustfmt 格式化
- 代码风格: 标准Rust风格（cargo fmt）
- Linting: cargo clippy
- 文档: 使用 /// 注释格式

---

## 重要注意事项

### 权限要求
- Hosts管理需要管理员权限
- 某些系统API需要相应的安全权限配置（在 tauri.conf.json 中）

### 跨平台兼容性
- 大部分功能支持 Windows/macOS/Linux
- 系统特定功能（如GPU信息）可能有平台限制
- macOS 支持 Apple Silicon 和 Intel

### 自动更新
- 配置在 tauri.conf.json 的 updater 部分
- 使用 GitHub Releases 作为更新源
- 支持公钥验证

---

## 开发提示

1. **首次运行**: 可能需要接受防火墙或安全权限提示
2. **Hosts权限**: 在Windows上会触发UAC提升，在macOS上需要输入密码
3. **调试**: 使用 `cargo tauri dev` 可以查看详细日志
4. **版本更新**: 使用 `pnpm release` 脚本而不是手动修改版本号

---

## gstack

这个项目使用 gstack 技能来增强开发工作流。

### Web浏览
- 使用 **/browse** 技能进行所有网页浏览
- 永远不要使用 mcp__claude-in-chrome__* 工具

### 可用技能
- **/office-hours** - 头脑风暴新想法
- **/plan-ceo-review** - 评审计划（战略层面）
- **/plan-eng-review** - 评审计划（架构层面）
- **/plan-design-review** - 评审计划（设计层面）
- **/design-consultation** - 创建设计系统
- **/design-shotgun** - 快速设计评审
- **/design-html** - HTML设计稿评审
- **/review** - 合并前代码审查
- **/ship** - 准备部署/创建PR
- **/land-and-deploy** - 落地部署
- **/canary** - 金丝雀发布
- **/benchmark** - 性能基准测试
- **/browse** - 网页浏览
- **/connect-chrome** - 连接Chrome调试
- **/qa** - 应用测试
- **/qa-only** - 仅QA测试
- **/design-review** - 视觉设计审计
- **/setup-browser-cookies** - 设置浏览器cookies
- **/setup-deploy** - 设置部署
- **/setup-gbrain** - 设置大脑知识库
- **/retro** - 每周回顾
- **/investigate** - 调试错误
- **/document-release** - 发布后文档更新
- **/codex** - 获得第二意见或对抗性代码审查
- **/goal** - 设定当前任务目标并持续对齐
- **/cso** - 首席安全官模式
- **/autoplan** - 自动规划
- **/plan-devex-review** - 开发者体验评审
- **/devex-review** - 开发者体验评审
- **/careful** - 处理生产或实时系统
- **/freeze** - 将编辑范围限定在一个模块/目录
- **/guard** - 最大安全模式（破坏性警告+编辑限制）
- **/unfreeze** - 移除编辑限制
- **/gstack-upgrade** - 升级gstack到最新版本
- **/learn** - 学习技能

### 故障排除
如果 gstack 技能无法正常工作，运行以下命令来构建二进制文件并注册技能：
```bash
cd .claude/skills/gstack && ./setup
```

---

## 相关文档

- **README.md**: 详细的使用说明和功能介绍
- **src-tauri/tauri.conf.json**: Tauri配置说明
- **.github/workflows/release.yml**: CI/CD配置
- **scripts/release.js**: 版本发布脚本说明
