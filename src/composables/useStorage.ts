import { Store } from '@tauri-apps/plugin-store'

// 创建存储实例
let store: Store | null = null
let initPromise: Promise<Store | null> | null = null

// 初始化存储（加锁防止并发初始化导致多次密钥环授权）
const initStore = async () => {
  if (store) return store
  if (initPromise) return initPromise

  initPromise = (async () => {
    try {
      // 检查是否在Tauri环境中运行
      const isTauri = typeof window !== 'undefined' && '__TAURI__' in window

      if (isTauri) {
        // 使用 ensureJediDir API 确保 .jedi 目录存在
        const jediDir = await import('@/api/app').then(api => api.ensureJediDir())
        // 创建存储（禁用加密，避免密钥环授权提示）
        // 不传 encryptionKey：默认即不加密，避免 Linux 下反复请求密钥环授权
        // （存储的都是非敏感配置数据，无需加密）
        store = await Store.load(`${jediDir}/settings.json`, {
          autoSave: true,
          defaults: {},
        })
      } else {
        // 非Tauri环境，直接使用localStorage
        store = null
      }
      return store
    } catch (error) {
      console.error('初始化存储失败:', error)
      // 降级为使用localStorage
      store = null
      return null
    } finally {
      initPromise = null
    }
  })()

  return initPromise
}

// 存储工具
export function useStorage() {
  // 获取存储项
  const getItem = async <T>(key: string): Promise<T | null> => {
    try {
      const s = await initStore()
      if (s) {
        // 使用Tauri存储
        return (await s.get(key)) as T
      } else {
        // 降级使用localStorage
        const value = localStorage.getItem(key)
        return value ? JSON.parse(value) : null
      }
    } catch (error) {
      console.error(`获取存储项 ${key} 失败:`, error)
      return null
    }
  }

  // 设置存储项
  const setItem = async <T>(key: string, value: T): Promise<void> => {
    try {
      const s = await initStore()
      if (s) {
        // 使用Tauri存储
        await s.set(key, value)
        await s.save()
      } else {
        // 降级使用localStorage
        localStorage.setItem(key, JSON.stringify(value))
      }
    } catch (error) {
      console.error(`设置存储项 ${key} 失败:`, error)
    }
  }

  return {
    getItem,
    setItem,
  }
}
