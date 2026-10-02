// Builtin skills ship with the app as raw-imported SKILL.md packages.
// Metadata is parsed from the SKILL.md frontmatter at load time (same format
// as user skills), so the files on disk are the single source of truth.
// Builtin skills are read-only: users can enable/disable them but not edit.

import { parseSkillDoc } from '../frontmatter'
import type { SkillManifest } from '../types'
import commitMessageRaw from './commit-message/SKILL.md?raw'
import hostsTroubleshootRaw from './hosts-troubleshoot/SKILL.md?raw'
import rustReviewRaw from './rust-review/SKILL.md?raw'

const RAW_SOURCES: Array<{ name: string; raw: string }> = [
  { name: 'commit-message', raw: commitMessageRaw },
  { name: 'hosts-troubleshoot', raw: hostsTroubleshootRaw },
  { name: 'rust-review', raw: rustReviewRaw },
]

export interface BuiltinSkillDoc extends SkillManifest {
  body: string
}

export const BUILTIN_SKILLS: BuiltinSkillDoc[] = RAW_SOURCES.flatMap(({ name, raw }) => {
  const parsed = parseSkillDoc(raw)
  if (!parsed) {
    console.error(`Builtin skill ${name} has invalid SKILL.md frontmatter`)
    return []
  }
  return [
    {
      name,
      description: parsed.description,
      icon: parsed.icon,
      source: 'builtin' as const,
      enabled: true,
      body: parsed.body,
    },
  ]
})
