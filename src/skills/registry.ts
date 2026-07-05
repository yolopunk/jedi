// src/skills/registry.ts

import { browserSkill } from './browser'
import { filesystemSkill } from './filesystem'
import { hostsSkill } from './hosts'
import { podcastSkill } from './podcast'
import { terminalSkill } from './terminal'
import type { Skill } from './types'
import { wallpaperSkill } from './wallpaper'
import { webFetchSkill, webSearchSkill } from './web'

export class SkillRegistry {
  private skills: Map<string, Skill> = new Map()

  register(skill: Skill): void {
    this.skills.set(skill.id, skill)
  }

  get(id: string): Skill | undefined {
    return this.skills.get(id)
  }

  list(): Skill[] {
    return Array.from(this.skills.values())
  }

  listEnabled(): Skill[] {
    return this.list().filter(s => s.enabled)
  }

  listAutoCallable(): Skill[] {
    return this.listEnabled().filter(s => s.autoCallable)
  }

  setEnabled(id: string, enabled: boolean): void {
    const skill = this.skills.get(id)
    if (skill) {
      skill.enabled = enabled
    }
  }
}

export const skillRegistry = new SkillRegistry()

// Register built-in skills
skillRegistry.register(terminalSkill)
skillRegistry.register(filesystemSkill)
skillRegistry.register(hostsSkill)
skillRegistry.register(browserSkill)
skillRegistry.register(podcastSkill)
skillRegistry.register(wallpaperSkill)
skillRegistry.register(webSearchSkill)
skillRegistry.register(webFetchSkill)

// Set initial enabled states
skillRegistry.setEnabled('browser', false)
skillRegistry.setEnabled('podcast', false)
skillRegistry.setEnabled('wallpaper', false)
