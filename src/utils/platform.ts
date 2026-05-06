/**
 * Platform detection utilities for cross-platform title bar and drag region support.
 * Borrowed from CC Switch's platform.ts pattern.
 */

const platform = navigator.platform.toLowerCase()
const userAgent = navigator.userAgent.toLowerCase()

export const isMac = (): boolean => platform.includes('mac')
export const isWindows = (): boolean => platform.includes('win')
export const isLinux = (): boolean => userAgent.includes('linux') && !isMac()
export const isWayland = (): boolean => {
  // Wayland detection is best-effort; XDG_SESSION_TYPE is not available in browser
  // We conservatively disable drag region on all Linux due to Wayland issues
  return isLinux()
}

/**
 * Drag region is disabled on Linux due to Wayland compatibility issues.
 * See: https://github.com/tauri-apps/tauri/issues/11719
 */
export const DRAG_REGION_ENABLED = !isLinux()

/**
 * Whether to show custom window control buttons (minimize, maximize, close).
 * On macOS with Overlay titleBarStyle, native controls are rendered by the system.
 */
export const SHOW_WINDOW_CONTROLS = !isMac()

export const HEADER_HEIGHT = 32

export const TITLE_BAR_TOTAL_HEIGHT = HEADER_HEIGHT

/**
 * Spreadable drag-region HTML attributes (CC Switch pattern).
 * Uses the stable `data-tauri-drag-region` attribute — NOT the unreleased
 * container pattern from PR 6362 (`data-tauri-drag-region-container`).
 */
export const DRAG_REGION_ATTR: Record<string, unknown> = DRAG_REGION_ENABLED
  ? { 'data-tauri-drag-region': true }
  : {}

export const DRAG_REGION_STYLE: Record<string, unknown> = DRAG_REGION_ENABLED
  ? { WebkitAppRegion: 'drag' }
  : {}
