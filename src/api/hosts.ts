/**
 * Hosts管理 API
 * 封装与Tauri后端的交互
 */
import { invoke } from '@tauri-apps/api/core'
import type { Group, HostsSource } from '@/types/hosts'
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
 * 获取系统hosts文件路径
 * @returns hosts文件路径
 */
export async function getHostsPath(): Promise<string> {
  return await invoke<string>('get_hosts_path')
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
 * @param url 远程配置URL（返回 Jedi JSON 分组格式，必要参数由 URL 自身携带）
 * @returns 远程配置数据
 */
export async function fetchRemoteConfig(url: string): Promise<Group[]> {
  return await invoke<Group[]>('fetch_remote_config', { url })
}

/**
 * 读取远程数据源列表
 * @returns 数据源列表
 */
export async function getHostsSources(): Promise<HostsSource[]> {
  return await invoke<HostsSource[]>('get_hosts_sources')
}

/**
 * 保存/更新远程数据源（按分组名 upsert）
 * @param source 数据源
 * @returns 最新数据源列表
 */
export async function saveHostsSource(source: HostsSource): Promise<HostsSource[]> {
  return await invoke<HostsSource[]>('save_hosts_source', { source })
}

/**
 * 删除远程数据源
 * @param name 分组名
 * @returns 最新数据源列表
 */
export async function removeHostsSource(name: string): Promise<HostsSource[]> {
  return await invoke<HostsSource[]>('remove_hosts_source', { name })
}
