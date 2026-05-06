<template>
  <div
    id="app-header"
    class="app-header"
    data-tauri-drag-region
    @mousedown="onDragMouseDown"
    @dblclick="onDragDoubleClick"
  >
    <div class="header-left" />

    <div class="header-center" />

    <div v-if="SHOW_WINDOW_CONTROLS" class="header-right">
      <v-btn
        icon
        size="x-small"
        variant="text"
        class="window-btn"
        @click="handleMinimize"
      >
        <v-icon :icon="mdiWindowMinimize" size="16" />
      </v-btn>
      <v-btn
        icon
        size="x-small"
        variant="text"
        class="window-btn"
        @click="handleMaximize"
      >
        <v-icon :icon="isMaximized ? mdiWindowRestore : mdiWindowMaximize" size="16" />
      </v-btn>
      <v-btn
        icon
        size="x-small"
        variant="text"
        class="window-btn window-btn-close"
        @click="handleClose"
      >
        <v-icon :icon="mdiClose" size="16" />
      </v-btn>
    </div>
    <div v-else class="header-right" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { mdiWindowMinimize, mdiWindowMaximize, mdiWindowRestore, mdiClose } from '@mdi/js'
import { getCurrentWindow, type Window as TauriWindow } from '@tauri-apps/api/window'
import { isMac, SHOW_WINDOW_CONTROLS } from '@/utils/platform'

let appWindow: TauriWindow | null = null

const isMaximized = ref(false)

const checkMaximized = async () => {
  if (!appWindow) return
  try {
    isMaximized.value = await appWindow.isMaximized()
  } catch {
    // ignore
  }
}

let unlisten: (() => void) | null = null

onMounted(async () => {
  console.log('[Jedi TitleBar] onMounted, isMac:', isMac(), 'SHOW_WINDOW_CONTROLS:', SHOW_WINDOW_CONTROLS)
  try {
    appWindow = getCurrentWindow()
    console.log('[Jedi TitleBar] appWindow obtained:', appWindow.label)
  } catch (err) {
    console.error('[Jedi TitleBar] Failed to getCurrentWindow:', err)
    return
  }
  await checkMaximized()
  try {
    unlisten = await appWindow.onResized(() => {
      checkMaximized()
    })
  } catch {
    // ignore
  }
})

onUnmounted(() => {
  unlisten?.()
})

const onDragMouseDown = async (e: MouseEvent) => {
  console.log('[Jedi TitleBar] onDragMouseDown fired', { button: e.button, target: e.target, className: (e.target as HTMLElement)?.className })
  if (!appWindow) {
    console.warn('[Jedi TitleBar] No appWindow')
    return
  }
  if (e.button !== 0) return
  const target = e.target as HTMLElement
  if (target.closest('.header-right')) {
    console.log('[Jedi TitleBar] Clicked on header-right, skipping drag')
    return
  }
  try {
    console.log('[Jedi TitleBar] Calling startDragging()')
    await appWindow.startDragging()
  } catch (err) {
    console.error('[Jedi TitleBar] startDragging failed:', err)
  }
}

const onDragDoubleClick = async (e: MouseEvent) => {
  if (isMac()) return
  
  console.log('[Jedi TitleBar] onDragDoubleClick fired', { target: e.target, detail: e.detail })
  e.preventDefault()
  if (!appWindow) {
    console.warn('[Jedi TitleBar] No appWindow for dblclick')
    return
  }
  const target = e.target as HTMLElement
  if (target.closest('.header-right')) {
    console.log('[Jedi TitleBar] Double-clicked on header-right, skipping maximize')
    return
  }
  try {
    console.log('[Jedi TitleBar] Calling toggleMaximize()')
    await appWindow.toggleMaximize()
  } catch (err) {
    console.error('[Jedi TitleBar] toggleMaximize failed:', err)
  }
}

const handleMinimize = async () => {
  if (!appWindow) return
  try {
    await appWindow.minimize()
  } catch {
    // ignore
  }
}

const handleMaximize = async () => {
  if (!appWindow) return
  try {
    await appWindow.toggleMaximize()
  } catch {
    // ignore
  }
}

const handleClose = async () => {
  if (!appWindow) return
  try {
    await appWindow.close()
  } catch {
    // ignore
  }
}
</script>

<style scoped>
.app-header {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  z-index: 50;
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 32px;
  background: linear-gradient(
    180deg,
    rgba(var(--v-theme-surface), 0.92) 0%,
    rgba(var(--v-theme-surface), 0.98) 100%
  );
  backdrop-filter: blur(12px);
  border-bottom: 1px solid rgba(var(--v-theme-on-surface), 0.08);
  user-select: none;
  -webkit-user-select: none;
  padding: 0 4px;
  -webkit-app-region: drag;
}

.header-left {
  display: flex;
  align-items: center;
  min-width: 78px;
  padding-left: 4px;
  pointer-events: none;
}

.header-center {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  pointer-events: none;
}

.app-title {
  font-size: 0.8rem;
  font-weight: 500;
  color: rgba(var(--v-theme-on-surface), 0.7);
  letter-spacing: 0.02em;
  pointer-events: none;
}

.header-right {
  display: flex;
  align-items: center;
  min-width: 78px;
  justify-content: flex-end;
  -webkit-app-region: no-drag;
}

.window-btn {
  border-radius: 4px;
  height: 24px !important;
  width: 24px !important;
  min-width: 24px !important;
  padding: 0 !important;
  margin: 0 1px;
  transition: all 0.15s ease;
}

.window-btn:hover {
  background: rgba(var(--v-theme-on-surface), 0.08);
}

.window-btn .v-icon {
  opacity: 0.6;
  transition: opacity 0.15s ease;
}

.window-btn:hover .v-icon {
  opacity: 1;
}

.window-btn-close:hover {
  background: rgba(224, 67, 58, 0.9) !important;
}

.window-btn-close:hover .v-icon {
  color: white !important;
  opacity: 1;
}
</style>
