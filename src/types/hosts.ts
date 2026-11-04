/**
 * Jedi Hosts 管理器类型定义
 *
 * 本文件定义了 Jedi Hosts 管理器中使用的主要数据类型
 */

/**
 * 主机条目类型
 */
export interface HostEntry {
  ip: string
  domain: string
  disabled: boolean
}

/**
 * 分组类型
 * 使用 name 作为分组名称
 */
export interface Group {
  name: string
  hosts: HostEntry[]
}

/**
 * 远程 hosts 数据源（与分组一一对应）
 *
 * 一个返回 Jedi JSON 分组格式的 URL；URL 本身携带全部必要参数
 * （如兜底 IP、命名空间等），Jedi 不感知数据源背后的实现细节。
 */
export interface HostsSource {
  name: string
  url: string
  /** 上次同步时间（Unix 秒） */
  lastSyncedAt?: number | null
}
