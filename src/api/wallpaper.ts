/**
 * 壁纸管理 API
 */
import { invoke } from '@tauri-apps/api/core';

export interface WallpaperItem {
  id: string;
  title: string;
  url: string;
  category: string;
  tags: string[];
  description: string;
  content: string;
}

export interface WallpaperConfig {
  auto_update: boolean;
  frequency_hours: number;
  selected_categories: string[];
  last_update_ts: number;
}

/**
 * 获取壁纸列表
 */
export async function getWallpapers(): Promise<WallpaperItem[]> {
  try {
    return await invoke<WallpaperItem[]>('get_wallpapers');
  } catch (error) {
    console.error('Failed to get wallpapers:', error);
    throw error;
  }
}

/**
 * 设置桌面壁纸
 * @param url 图片URL
 */
export async function setDesktopWallpaper(url: string): Promise<void> {
  try {
    await invoke('set_desktop_wallpaper', { url });
  } catch (error) {
    console.error('Failed to set desktop wallpaper:', error);
    throw error;
  }
}
