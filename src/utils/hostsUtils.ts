/**
 * JEDIHOSTS管理器 - 工具函数
 * 提供处理hosts文件的通用工具函数
 */

import { Group, Tag, HostEntry } from '@/types/hosts'

/**
 * 将hosts数组转换为数据表格项目
 * @param hosts hosts条目数组
 * @returns 格式化后的数据表格项目
 */
export function getHostsAsItems(hosts: HostEntry[]) {
  return hosts.map((hostMap, index) => {
    // 检查是否禁用
    const isDisabled = hostMap.hasOwnProperty('__disabled')

    // 提取域名和IP（跳过特殊键）
    let domain = ''
    let ip = ''

    for (const key in hostMap) {
      if (key !== '__disabled') {
        domain = key
        ip = hostMap[key]
        break
      }
    }

    return {
      id: index,
      domain,
      ip,
      enabled: !isDisabled,
      originalMap: hostMap
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
export function findHostEntry(group: Group | Tag, host: any): HostEntry | undefined {
  return group.hosts.find(h => {
    for (const key in h) {
      if (key !== '__disabled' && key === host.domain && h[key] === host.ip) {
        return true;
      }
    }
    return false;
  });
}

/**
 * 查找主机条目索引
 * @param group 分组
 * @param host 主机信息
 * @returns 找到的主机条目索引
 */
export function findHostIndex(group: Group | Tag, host: any): number {
  return group.hosts.findIndex(h => {
    for (const key in h) {
      if (key !== '__disabled' && key === host.domain && h[key] === host.ip) {
        return true;
      }
    }
    return false;
  });
}

/**
 * 更新主机条目状态
 * @param hostEntry 主机条目
 * @param enabled 是否启用
 */
export function updateHostEntryStatus(hostEntry: HostEntry, enabled: boolean): void {
  if (enabled) {
    // 如果启用，删除禁用标记
    delete hostEntry['__disabled']
  } else {
    // 如果禁用，添加禁用标记
    hostEntry['__disabled'] = 'true'
  }
}

/**
 * 启用所有hosts条目
 * @param groups 分组数据
 */
export function enableAllHosts(groups: Group[] | Tag[]): void {
  for (const group of groups) {
    for (const host of group.hosts) {
      if (host.hasOwnProperty('__disabled')) {
        delete host['__disabled']
      }
    }
  }
}

/**
 * 禁用所有hosts条目
 * @param groups 分组数据
 */
export function disableAllHosts(groups: Group[] | Tag[]): void {
  for (const group of groups) {
    for (const host of group.hosts) {
      if (!host.hasOwnProperty('__disabled')) {
        host['__disabled'] = 'true'
      }
    }
  }
}

/**
 * 打开域名链接
 * @param domain 要打开的域名
 * @returns 通知消息
 */
export function openDomainLink(domain: string): string {
  // 根据域名构建 URL
  let url = domain
  if (!url.startsWith('http://') && !url.startsWith('https://')) {
    url = 'http://' + url
  }
  
  // 在新标签页中打开链接
  window.open(url, '_blank')
  return `正在打开: ${domain}`
}
