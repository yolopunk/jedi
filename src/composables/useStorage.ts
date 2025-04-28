// @ts-ignore
import { Store } from '@tauri-apps/plugin-store'

// 创建存储实例
let store: Store | null = null

// 初始化存储
const initStore = async () => {
  if (!store) {
    try {
      // 使用 ensureJediDir API 确保 .jedi 目录存在
      const jediDir = await import('@/api/app').then(api => api.ensureJediDir())

      // 创建存储
      store = await Store.load(`${jediDir}/settings.json`)
    } catch (error) {
      console.error('初始化存储失败:', error)
      // 降级为使用localStorage
      store = null
    }
  }
  return store
}

// 存储工具
export function useStorage() {
  // 获取存储项
  const getItem = async <T>(key: string): Promise<T | null> => {
    try {
      const s = await initStore()
      if (s) {
        // 使用Tauri存储
        return await s.get(key) as T
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

  // 删除存储项
  const removeItem = async (key: string): Promise<void> => {
    try {
      const s = await initStore()
      if (s) {
        // 使用Tauri存储
        await s.delete(key)
        await s.save()
      } else {
        // 降级使用localStorage
        localStorage.removeItem(key)
      }
    } catch (error) {
      console.error(`删除存储项 ${key} 失败:`, error)
    }
  }

  return {
    getItem,
    setItem,
    removeItem
  }
}
