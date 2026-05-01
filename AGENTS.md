<!-- Generated: 2026-04-18 | Updated: 2026-04-18 -->

# Jedi 工具箱

## Purpose
跨平台桌面应用，为开发者提供实用工具集，包括Hosts管理、知识壁纸、播客客户端和AI聊天助手等功能。基于Tauri + Vue 3技术栈构建，支持Windows/macOS/Linux。

## Key Files
| File | Description |
|------|-------------|
| `package.json` | 前端依赖管理和项目脚本 |
| `pnpm-lock.yaml` | pnpm依赖锁定文件 |
| `tsconfig.json` | TypeScript编译器配置 |
| `vite.config.ts` | Vite构建工具配置 |
| `README.md` | 项目说明文档和使用指南 |
| `CLAUDE.md` | AI协作规则和项目上下文 |
| `.gitignore` | Git忽略规则配置 |
| `.biome.json` | Biome代码检查和格式化配置 |

## Subdirectories
| Directory | Purpose |
|-----------|---------|
| `docs/` | 项目文档和架构设计 (see `docs/AGENTS.md`) |
| `public/` | 静态资源文件 (字体、图片等) |
| `scripts/` | 构建、发布和自动化脚本 |
| `src/` | 前端Vue应用源代码 (see `src/AGENTS.md`) |
| `src-tauri/` | Tauri后端Rust源代码 (see `src-tauri/AGENTS.md`) |

## For AI Agents

### Working In This Directory
- 使用 `pnpm` 作为包管理器，不要使用npm/yarn
- 开发命令：`pnpm tauri dev`
- 构建命令：`pnpm tauri build`
- 版本发布使用 `pnpm release [major|minor|patch]` 脚本自动更新版本号
- 代码风格遵循Biome配置，提交前自动格式化

### Testing Requirements
- Rust后端测试：`cargo test`
- 前端单元测试：暂无测试体系，主要做功能验证
- 跨平台兼容性需要在所有支持的系统上测试

### Common Patterns
- 前端使用Vue 3 Composition API + TypeScript
- 后端使用Rust，通过Tauri invoke与前端通信
- 状态管理使用Vue Composables，无全局状态库
- 本地存储使用Tauri Store插件，浏览器降级使用localStorage

## Dependencies

### Internal
- 前端调用后端API通过 `@/api/` 目录下的封装
- 后端API定义在 `src-tauri/src/api/` 目录

### External
- Vue 3.x - 前端框架
- Tauri 2.x - 桌面应用框架
- Vuetify 3.x - UI组件库
- TypeScript 5.x - 类型安全
- Vite 6.x - 构建工具
- Rust 1.70+ - 后端编程语言

<!-- MANUAL: -->
