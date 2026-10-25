// Windows -> WSL path translation for external engines (docs §5.3 W5).
//
// Mirrors `wslpath -a` for the standard drvfs mounts without spawning a
// process: C:\Users\x\img.png -> /mnt/c/Users/x/img.png. UNC paths
// (\\wsl$\…) and non-Windows paths are not translatable (null).

const WIN_ABS = /^([A-Za-z]):([\\/].+)$/

export function toWslPath(winPath: string): string | null {
  const match = WIN_ABS.exec(winPath.trim())
  if (!match) return null
  const drive = match[1].toLowerCase()
  const rest = match[2].replace(/\\/g, '/')
  return `/mnt/${drive}${rest}`
}

/** True when the engine lives inside WSL and needs translated paths. */
export function isWslHost(host: string): boolean {
  return host.startsWith('wsl:')
}
