/**
 * 技能（SKILL.md 指令包）API
 * 与后端 src-tauri/src/api/skills.rs 中的命令一一对应
 */

import { invoke } from '@tauri-apps/api/core'
import type { SkillDetail, SkillManifest } from '@/skills/types'

/** 编辑器保存载荷 */
export interface SkillSaveDef {
  name: string
  description: string
  icon: string
  body: string
}

/** 列出 ~/.jedi/skills/ 下所有用户技能（格式非法的目录会被后端跳过） */
export function listSkills(): Promise<SkillManifest[]> {
  return invoke('skills_list')
}

/** 读取一个技能的完整内容（正文 + 附属文件清单） */
export function readSkill(name: string): Promise<SkillDetail> {
  return invoke('skills_read', { name })
}

/** 读取技能的附属文件（rel_path 相对技能目录，禁止穿越） */
export function readSkillFile(name: string, relPath: string): Promise<string> {
  return invoke('skills_read_file', { name, relPath })
}

/** 保存（新建或覆盖）一个用户技能，返回规范化后的 manifest */
export function saveSkill(def: SkillSaveDef): Promise<SkillManifest> {
  return invoke('skills_save', { def })
}

/** 删除一个用户技能目录；返回原本是否存在 */
export function deleteSkill(name: string): Promise<boolean> {
  return invoke('skills_delete', { name })
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
