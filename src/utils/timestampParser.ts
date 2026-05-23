export interface TimestampMarker {
  time: number // seconds
  label: string
}

/**
 * Parse timestamps from show notes text.
 * Supports formats: [HH:MM:SS], (HH:MM:SS), HH:MM:SS - text, MM:SS - text
 */
export function parseTimestamps(text: string): TimestampMarker[] {
  if (!text) return []

  const markers: TimestampMarker[] = []
  const seen = new Set<number>()

  // Match timestamps: [01:23:45], (01:23:45), 01:23:45 -, 01:23 -
  const patterns = [
    // [HH:MM:SS] or [MM:SS]
    /\[(\d{1,2}):(\d{2})(?::(\d{2}))?\](?:\s*[-–—]\s*(.+?))?(?:\n|$|<\/)/g,
    // (HH:MM:SS) or (MM:SS)
    /\((\d{1,2}):(\d{2})(?::(\d{2}))?\)(?:\s*[-–—]\s*(.+?))?(?:\n|$|<\/)/g,
    // HH:MM:SS - text or MM:SS - text (at line start)
    /^(\d{1,2}):(\d{2})(?::(\d{2}))?\s*[-–—]\s*(.+?)(?:\n|$)/gm,
    // bare HH:MM:SS or MM:SS on its own line
    /^(\d{1,2}):(\d{2})(?::(\d{2}))?\s*$/gm,
  ]

  for (const regex of patterns) {
    let match: RegExpExecArray | null
    while ((match = regex.exec(text)) !== null) {
      const hours = parseInt(match[1], 10)
      const minutes = parseInt(match[2], 10)
      const seconds = match[3] ? parseInt(match[3], 10) : 0
      const totalSeconds = hours * 3600 + minutes * 60 + seconds

      if (seen.has(totalSeconds)) continue
      seen.add(totalSeconds)

      const label = match[4]?.trim() || formatTime(totalSeconds)
      markers.push({ time: totalSeconds, label })
    }
  }

  return markers.sort((a, b) => a.time - b.time)
}

export function formatTime(seconds: number): string {
  if (!isFinite(seconds) || seconds < 0) return '00:00'
  const h = Math.floor(seconds / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  const s = Math.floor(seconds % 60)
  if (h > 0) {
    return `${h}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`
  }
  return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`
}
