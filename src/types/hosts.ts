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

/**
 * 旧的分组类型（兼容层）
 * 使用 tag 作为分组名称
 */
export interface Tag {
  tag: string;
  hosts: HostEntry[];
}

/**
 * 将 Tag 转换为 Group
 */
export function tagToGroup(tag: Tag): Group {
  return {
    name: tag.tag,
    hosts: tag.hosts
  };
}

/**
 * 将 Group 转换为 Tag
 */
export function groupToTag(group: Group): Tag {
  return {
    tag: group.name,
    hosts: group.hosts
  };
}
