// @ts-ignore
import { invoke } from '@tauri-apps/api/core';

export interface AppInfo {
  version: string;
  name: string;
}

/**
 * 获取应用信息
 * @returns 应用信息，包括版本号和名称
 */
export async function getAppInfo(): Promise<AppInfo> {
  return invoke('get_app_info');
}
