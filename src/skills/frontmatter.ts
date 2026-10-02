// Minimal SKILL.md frontmatter parser, mirroring the Rust parser in
// src-tauri/src/api/skills.rs. Returns null when the document has no/unclosed
// frontmatter or lacks a description — such files are invalid skills.

export interface ParsedSkillDoc {
  /** frontmatter name; absent means "use the directory name" */
  name?: string
  description: string
  icon: string
  body: string
}

export function parseSkillDoc(content: string): ParsedSkillDoc | null {
  const trimmed = content.replace(/^\uFEFF/, '')
  if (!trimmed.startsWith('---\n')) return null
  const rest = trimmed.slice(4)
  const end = rest.indexOf('\n---')
  if (end < 0) return null
  const header = rest.slice(0, end)
  let body = rest.slice(end + 4)
  if (body.startsWith('\n')) body = body.slice(1)
  else if (body.startsWith('\r')) body = body.replace(/^[\r\n]+/, '')

  let name: string | undefined
  let description: string | undefined
  let icon = '🧩'
  for (const line of header.split('\n')) {
    if (!line.trim()) continue
    const idx = line.indexOf(':')
    if (idx < 0) continue
    const key = line.slice(0, idx).trim()
    const value = line.slice(idx + 1).trim()
    if (key === 'name' && value) name = value
    else if (key === 'description' && value) description = value
    else if (key === 'icon' && value) icon = value
  }
  if (!description) return null
  return { name, description, icon, body }
}
