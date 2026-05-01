<!-- Parent: ../AGENTS.md -->
<!-- Generated: 2026-04-18 | Updated: 2026-04-18 -->

# composables (Vue组合式函数)

## Purpose
存放可复用的Vue组合式逻辑，实现逻辑跨组件复用，分离关注点。

## Key Files
| File | Description |
|------|-------------|
| `useStorage.ts` | 本地存储封装，支持Tauri Store和localStorage降级 |
| `useTheme.ts` | 主题管理，支持深色/浅色/系统主题切换 |
| `useUpdate.ts` | 应用更新检查和安装逻辑 |
| `useWallpaper.ts` | 壁纸功能相关逻辑 |
| `useAudioPlayer.ts` | 音频播放器封装，用于播客功能 |
| `useHostsData.ts` | Hosts数据管理逻辑 |

## For AI Agents

### Working In This Directory
- 文件名使用kebab-case，前缀为`use-`
- 组合式函数命名使用camelCase，前缀为`use`
- 优先返回ref和可调用函数，避免返回复杂对象
- 副作用需要提供清理函数，支持onUnmounted自动清理
- 通用逻辑需要考虑多实例使用场景，避免全局状态污染

### Common Patterns
- 使用ref管理内部状态
- 提供响应式的返回值
- 支持传入配置选项定制行为
- 自动处理生命周期清理

## Dependencies

### External
- Vue 3.x - Composition API支持
- `@tauri-apps/api` - 系统API调用

<!-- MANUAL: -->
