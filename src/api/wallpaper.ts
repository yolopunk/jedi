/**
 * 壁纸管理 API
 */
import { invoke } from '@tauri-apps/api/core'

export interface WallpaperItem {
  id: string
  title: string
  url: string
  category: string
  tags: string[]
  description: string
  content: string
}

export enum WallpaperMode {
  Center = 'Center',
  Crop = 'Crop',
  Fit = 'Fit',
  Span = 'Span',
  Stretch = 'Stretch',
  Tile = 'Tile',
}

export interface WallpaperConfig {
  auto_update: boolean
  frequency_hours: number
  selected_categories: string[]
  last_update_ts: number
}

/**
 * 获取壁纸列表
 */
export async function getWallpapers(): Promise<WallpaperItem[]> {
  try {
    return await invoke<WallpaperItem[]>('get_wallpapers')
  } catch (error) {
    console.error('Failed to get wallpapers:', error)
    throw error
  }
}

/**
 * 同步壁纸数据（增量更新）
 */
export async function syncWallpapers(): Promise<void> {
  try {
    await invoke('sync_wallpapers')
  } catch (error) {
    console.error('Failed to sync wallpapers:', error)
    throw error
  }
}

/**
 * 设置桌面壁纸
 * @param url 图片URL
 * @param mode 壁纸模式 (可选，默认为 Crop)
 */
export async function setDesktopWallpaper(url: string, mode?: WallpaperMode): Promise<void> {
  try {
    await invoke('set_desktop_wallpaper', { url, mode })
  } catch (error) {
    console.error('Failed to set desktop wallpaper:', error)
    throw error
  }
}

/**
 * 获取当前壁纸
 * @returns 当前壁纸的 URL 或路径，如果没有壁纸则返回 null
 */
export async function getCurrentWallpaper(): Promise<string | null> {
  try {
    return await invoke<string | null>('get_current_wallpaper')
  } catch (error) {
    console.error('Failed to get current wallpaper:', error)
    return null
  }
}

/**
 * 在文件管理器中显示壁纸文件
 * @param path 壁纸路径
 */
export async function showInFolder(path: string): Promise<void> {
  try {
    await invoke('show_in_folder', { path })
  } catch (error) {
    console.error('Failed to show in folder:', error)
    throw error
  }
}
