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
