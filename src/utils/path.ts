export function isAbsoluteProjectPath(path: string): boolean {
  const normalized = path.trim()
  if (
    !normalized ||
    [...normalized].some(char => char.charCodeAt(0) < 32 || char.charCodeAt(0) === 127)
  )
    return false
  return (
    normalized.startsWith('/') ||
    /^[A-Za-z]:[\\/]/.test(normalized) ||
    /^\\\\[^\\]+\\[^\\]+/.test(normalized)
  )
}
