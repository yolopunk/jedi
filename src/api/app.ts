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

/**
 * 启用开机自启动
 * @returns 操作结果
 */
export async function enableAutostart(): Promise<void> {
  return invoke('enable_autostart');
}

/**
 * 禁用开机自启动
 * @returns 操作结果
 */
export async function disableAutostart(): Promise<void> {
  return invoke('disable_autostart');
}

/**
 * 检查是否已启用开机自启动
 * @returns 是否已启用
 */
export async function isAutostartEnabled(): Promise<boolean> {
  return invoke('is_autostart_enabled');
}

/**
 * 确保 .jedi 目录存在
 * @returns .jedi 目录路径
 */
export async function ensureJediDir(): Promise<string> {
  return invoke('ensure_jedi_dir');
}
