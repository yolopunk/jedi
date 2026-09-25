/**
 * 自定义技能 API
 * 与后端 src-tauri/src/api/skills.rs 中的命令一一对应
 */

import { invoke } from '@tauri-apps/api/core'
import type { CustomSkillDef } from '@/skills/types'

/** 列出 ~/.jedi/skills/ 下的所有自定义技能（格式非法的文件会被后端跳过） */
export function listCustomSkills(): Promise<CustomSkillDef[]> {
  return invoke('skills_list_custom')
}

/** 保存（新建或覆盖）一个自定义技能，返回规范化后的定义 */
export function saveCustomSkill(skill: CustomSkillDef): Promise<CustomSkillDef> {
  return invoke('skills_save', { skill })
}

/** 删除一个自定义技能；返回文件原本是否存在 */
export function deleteCustomSkill(id: string): Promise<boolean> {
  return invoke('skills_delete', { id })
}

// ========== 技能执行统计 API ==========

/** 单条最近错误记录（msg 已截断，不含 args） */
export interface SkillStatError {
  ts: number
  msg: string
}

/** 单个技能的累计执行统计 */
export interface SkillStatEntry {
  calls: number
  failures: number
  totalMs: number
  lastUsedAt: number
  recentErrors: SkillStatError[]
}

/** skillId → 统计条目 */
export type SkillStatsMap = Record<string, SkillStatEntry>

/** 列出全部技能执行统计 */
export function listSkillStats(): Promise<SkillStatsMap> {
  return invoke('skill_stats_list')
}

/** 整表保存统计（前端防抖批量写；后端保存前强制截断防护） */
export function saveSkillStats(stats: SkillStatsMap): Promise<void> {
  return invoke('skill_stats_save', { stats })
}

/** 清零全部统计 */
export function clearSkillStats(): Promise<void> {
  return invoke('skill_stats_clear')
}
