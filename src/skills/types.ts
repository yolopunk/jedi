// Skill = a SKILL.md instruction package (Claude Code Agent Skills style).
// A skill carries NO executable body: frontmatter metadata (name/description)
// stays in the model's context, the markdown body is loaded on demand via the
// `skill` tool, and bundled files (references/ scripts/ assets/) are read as
// needed. Execution always happens through the regular agent tools
// (see src/agent/tools/), which keep their own risk gates.

// Where a skill came from. 'builtin' skills ship with the app (read-only);
// 'user' skills live in ~/.jedi/skills/<name>/SKILL.md and are editable.
export type SkillSource = 'builtin' | 'user'

export interface SkillManifest {
  /** kebab-case identifier, equals the skill directory name */
  name: string
  /** what it does + when to trigger; the primary auto-trigger signal */
  description: string
  icon: string
  source: SkillSource
  enabled: boolean
}

export interface SkillDetail {
  name: string
  description: string
  icon: string
  source: SkillSource
  body: string
  /** bundled files relative to the skill directory, e.g. references/aws.md */
  files: string[]
  /** absolute path of the skill directory (user skills; empty for builtin) */
  dir: string
}
