/**
 * JEDIHOSTS管理器 - API服务
 * 封装与Tauri API的交互
 */
import { invoke } from '@tauri-apps/api/core'
import { Group, Tag, groupToTag } from '@/types/hosts'

/**
 * 获取系统信息
 * @returns 系统信息
 */
export async function getOsInfo() {
  return await invoke('get_os_info')
}

/**
 * 读取系统hosts配置
 * @returns hosts配置数据
 */
export async function readSystemHosts() {
  return await invoke('read_system_hosts')
}

/**
 * 更新hosts文件
 * @param groups 分组数据
 * @returns 更新结果
 */
export async function updateHostsWithGroups(groups: Group[]) {
  // 将新格式转换为旧格式
  const tags = groups.map(group => groupToTag(group))
  
  return await invoke('update_hosts_with_tag', {
    source: 'current',
    url: null,
    tags
  })
}

/**
 * 更新hosts文件 (兼容层)
 * @param tags 分组数据
 * @returns 更新结果
 */
export async function updateHostsWithTag(tags: Tag[]) {
  return await invoke('update_hosts_with_tag', {
    source: 'current',
    url: null,
    tags
  })
}

/**
 * 恢复hosts文件
 * @returns 恢复结果
 */
export async function revertHosts() {
  return await invoke('revert_hosts')
}

/**
 * 初始化默认配置
 * @returns 初始化结果
 */
export async function initializeDefaultConfig() {
  return await invoke('update_hosts_with_tag', {
    source: 'default',
    url: null,
    tags: null
  })
}

/**
 * 获取远程配置
 * @param url 远程配置URL
 * @returns 远程配置数据
 */
export async function fetchRemoteConfig(url: string) {
  return await invoke('fetch_remote_config', { url })
}
