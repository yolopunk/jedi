/**
 * Jedi Hosts 管理器类型定义
 *
 * 本文件定义了 Jedi Hosts 管理器中使用的主要数据类型
 */

/**
 * 主机条目类型
 * 键为域名，值为IP地址
 * 可能包含 __disabled 属性表示禁用状态
 */
export type HostEntry = Record<string, string>;

/**
 * 分组类型
 * 使用 name 作为分组名称
 */
export interface Group {
  name: string;
  hosts: HostEntry[];
}


