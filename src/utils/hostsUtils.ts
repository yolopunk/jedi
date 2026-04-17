/**
 * JEDIHOSTS管理器 - 工具函数
 * 提供处理hosts文件的通用工具函数
 */

import { open } from '@tauri-apps/plugin-shell'
import type { Group, HostEntry } from '@/types/hosts'

/**
 * 将hosts数组转换为数据表格项目
 * @param hosts hosts条目数组
 * @returns 格式化后的数据表格项目
 */
export function getHostsAsItems(hosts: HostEntry[]) {
  return hosts.map((host, index) => {
    return {
      id: index,
      domain: host.domain,
      ip: host.ip,
      enabled: !host.disabled,
      originalMap: host,
    }
  })
}

/**
 * 验证主机输入
 * @param ip IP地址
 * @param domain 域名
 * @returns 验证是否通过
 */
export function validateHostInput(ip: string, domain: string): boolean {
  return !!(ip.trim() && domain.trim())
}

/**
 * 查找主机条目
 * @param group 分组
 * @param host 主机信息
 * @returns 找到的主机条目
 */
export function findHostEntry(group: Group, host: HostEntry): HostEntry | undefined {
  return group.hosts.find(h => h.domain === host.domain && h.ip === host.ip)
}

/**
 * 查找主机条目索引
 * @param group 分组
 * @param host 主机信息
 * @returns 找到的主机条目索引
 */
export function findHostIndex(group: Group, host: HostEntry): number {
  return group.hosts.findIndex(h => h.domain === host.domain && h.ip === host.ip)
}

/**
 * 更新主机条目状态
 * @param hostEntry 主机条目
 * @param enabled 是否启用
 */
export function updateHostEntryStatus(hostEntry: HostEntry, enabled: boolean): void {
  hostEntry.disabled = !enabled
}

/**
 * 启用所有hosts条目
 * @param groups 分组数据
 */
export function enableAllHosts(groups: Group[]): void {
  for (const group of groups) {
    for (const host of group.hosts) {
      host.disabled = false
    }
  }
}

/**
 * 禁用所有hosts条目
 * @param groups 分组数据
 */
export function disableAllHosts(groups: Group[]): void {
  for (const group of groups) {
    for (const host of group.hosts) {
      host.disabled = true
    }
  }
}

/**
 * 打开域名链接
 * @param domain 要打开的域名
 * @returns 通知消息
 */
export async function openDomainLink(domain: string): Promise<string> {
  try {
    // 根据域名构建 URL
    let url = domain
    if (!url.startsWith('http://') && !url.startsWith('https://')) {
      url = `http://${url}`
    }

    // 在默认浏览器中打开链接
    await open(url)
    return `正在打开: ${domain}`
  } catch (error) {
    console.error('打开域名链接失败:', error)
    throw new Error(`打开域名失败: ${domain}`)
  }
}
