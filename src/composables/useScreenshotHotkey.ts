import { ref } from 'vue'
import { getCaptureHotkey, setCaptureHotkey } from '@/api/screenshot'
import { useStorage } from '@/composables/useStorage'

/** 默认截图快捷键 */
export const DEFAULT_SCREENSHOT_HOTKEY = 'F1'
const STORAGE_KEY = 'screenshotHotkey'

/** 当前生效的快捷键（accelerator 形式，如 F1 / Ctrl+Alt+S） */
const hotkey = ref(DEFAULT_SCREENSHOT_HOTKEY)
const hotkeyBusy = ref(false)
const hotkeyError = ref('')

let initialized = false

const MODIFIERS = new Set(['Control', 'Alt', 'Shift', 'Meta'])

/** event.key → tauri 全局快捷键接受的键名 */
const KEY_NAMES: Record<string, string> = {
  ' ': 'Space',
  Escape: 'Escape',
  Enter: 'Enter',
  Tab: 'Tab',
  Backspace: 'Backspace',
  Delete: 'Delete',
  Insert: 'Insert',
  Home: 'Home',
  End: 'End',
  PageUp: 'PageUp',
  PageDown: 'PageDown',
  ArrowUp: 'ArrowUp',
  ArrowDown: 'ArrowDown',
  ArrowLeft: 'ArrowLeft',
  ArrowRight: 'ArrowRight',
  '`': 'Backquote',
  '-': 'Minus',
  '=': 'Equal',
  '[': 'BracketLeft',
  ']': 'BracketRight',
  '\\': 'Backslash',
  ';': 'Semicolon',
  "'": 'Quote',
  ',': 'Comma',
  '.': 'Period',
  '/': 'Slash',
}

/**
 * 把键盘事件转成全局快捷键字符串（如 "Ctrl+Alt+S"、"F1"）。
 * 只按下修饰键时返回 null，便于录制态继续等待主键。
 */
export function acceleratorFromEvent(event: KeyboardEvent): string | null {
  if (MODIFIERS.has(event.key)) return null

  let key: string | null = null
  if (KEY_NAMES[event.key]) {
    key = KEY_NAMES[event.key]
  } else if (/^[a-zA-Z]$/.test(event.key)) {
    key = event.key.toUpperCase()
  } else if (/^[0-9]$/.test(event.key)) {
    key = event.key
  } else if (/^F([1-9]|1[0-9]|2[0-4])$/.test(event.key)) {
    key = event.key.toUpperCase()
  }
  if (!key) return null

  const parts: string[] = []
  if (event.ctrlKey) parts.push('Ctrl')
  if (event.altKey) parts.push('Alt')
  if (event.shiftKey) parts.push('Shift')
  if (event.metaKey) parts.push('Super')
  parts.push(key)
  return parts.join('+')
}

/** 应用并持久化快捷键；返回是否注册成功 */
async function applyHotkey(accelerator: string): Promise<boolean> {
  hotkeyBusy.value = true
  hotkeyError.value = ''
  try {
    await setCaptureHotkey(accelerator)
    hotkey.value = accelerator
    await useStorage().setItem(STORAGE_KEY, accelerator)
    return true
  } catch (err) {
    hotkeyError.value = String(err)
    return false
  } finally {
    hotkeyBusy.value = false
  }
}

/**
 * 启动时把已保存的快捷键同步给后端。
 * 后端 setup 阶段先注册默认值，这里不足一次重复注册（值相同就跳过）。
 */
export async function initScreenshotHotkey() {
  if (initialized) return
  initialized = true
  try {
    const saved = await useStorage().getItem<string>(STORAGE_KEY)
    const target = saved && saved.trim() ? saved.trim() : DEFAULT_SCREENSHOT_HOTKEY
    hotkey.value = target

    let current = ''
    try {
      current = await getCaptureHotkey()
    } catch {
      current = ''
    }
    if (current !== target) {
      await applyHotkey(target)
    }
  } catch (err) {
    hotkeyError.value = String(err)
  }
}

export function useScreenshotHotkey() {
  return {
    hotkey,
    hotkeyBusy,
    hotkeyError,
    applyHotkey,
    initScreenshotHotkey,
    defaultHotkey: DEFAULT_SCREENSHOT_HOTKEY,
  }
}
