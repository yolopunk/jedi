// src/skills/registry.ts
//
// In-memory registry of skill manifests (SKILL.md metadata, no executable
// body). Builtin skills register at module load; user skills register when
// stores/skills.ts pulls them from the backend. The enabled flag is applied
// in both directions by stores/skills.ts.

import { BUILTIN_SKILLS } from './builtin'
import type { SkillManifest } from './types'

export class SkillRegistry {
  private skills: Map<string, SkillManifest> = new Map()

  register(manifest: SkillManifest): void {
    this.skills.set(manifest.name, manifest)
  }

  unregister(name: string): void {
    this.skills.delete(name)
  }

  get(name: string): SkillManifest | undefined {
    return this.skills.get(name)
  }

  list(): SkillManifest[] {
    return Array.from(this.skills.values())
  }

  listEnabled(): SkillManifest[] {
    return this.list().filter(s => s.enabled)
  }

  setEnabled(name: string, enabled: boolean): void {
    const skill = this.skills.get(name)
    if (skill) {
      skill.enabled = enabled
    }
  }
}

export const skillRegistry = new SkillRegistry()

// Register builtin skills (user skills are registered by stores/skills.ts)
for (const doc of BUILTIN_SKILLS) {
  const { body: _body, ...manifest } = doc
  skillRegistry.register(manifest)
}
