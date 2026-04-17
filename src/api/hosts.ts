/**
 * Hosts管理 API
 * 封装与Tauri后端的交互
 */
import { invoke } from '@tauri-apps/api/core'
import type { Group } from '@/types/hosts'
import type { OsInfo } from '@/types/os'

/**
 * 获取系统信息
 * @returns 系统信息
 */
export async function getOsInfo(): Promise<OsInfo> {
  return await invoke('get_os_info')
}

/**
 * 读取系统hosts配置
 * @returns hosts配置数据
 */
export async function readSystemHosts(): Promise<Group[]> {
  return await invoke<Group[]>('read_system_hosts')
}

/**
 * 更新hosts文件
 * @param groups 分组数据
 * @returns 更新结果
 */
export async function updateHostsWithGroups(groups: Group[]): Promise<string> {
  return await invoke<string>('update_hosts_with_groups', {
    source: 'current',
    url: null,
    groups,
  })
}

/**
 * 恢复hosts文件
 * @returns 恢复结果
 */
export async function revertHosts(): Promise<string> {
  return await invoke<string>('revert_hosts')
}

/**
 * 初始化默认配置
 * @returns 初始化结果
 */
export async function initializeDefaultConfig(): Promise<string> {
  return await invoke<string>('update_hosts_with_groups', {
    source: 'default',
    url: null,
    groups: null,
  })
}

/**
 * 获取远程配置
 * @param url 远程配置URL
 * @returns 远程配置数据
 */
export async function fetchRemoteConfig(url: string): Promise<string> {
  return await invoke<string>('fetch_remote_config', { url })
}
