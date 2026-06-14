<template>
  <div
    id="app-header"
    class="app-header"
    data-tauri-drag-region
    @mousedown="onDragMouseDown"
    @dblclick="onDragDoubleClick"
  >
    <div class="header-left" :class="{ 'header-left--mac': isMac() }" />

    <div class="header-center" />

    <div class="header-right">
      <div class="global-actions">
        <v-btn
          icon
          size="x-small"
          variant="text"
          class="header-action-btn"
          @click="handleCheckUpdate"
        >
          <v-progress-circular
            v-if="isChecking"
            :size="15"
            :width="2"
            indeterminate
          />
          <v-icon
            v-else
            :icon="hasUpdate ? mdiDownloadCircle : mdiCloudSyncOutline"
            size="16"
            :class="{ 'update-available': hasUpdate }"
          />
          <v-tooltip activator="parent" location="bottom">
            {{ hasUpdate && updateInfo ? $t('settings.updateAvailable', { version: updateInfo.version }) : $t('settings.checkUpdate') }}
          </v-tooltip>
        </v-btn>

        <v-btn
          icon
          size="x-small"
          variant="text"
          class="header-action-btn"
          @click="toggleTheme"
        >
          <v-icon :icon="themeIcon" size="16" />
          <v-tooltip activator="parent" location="bottom">
            {{ $t(themeTooltip) }}
          </v-tooltip>
        </v-btn>

        <v-btn
          icon
          size="x-small"
          variant="text"
          class="header-action-btn"
          @click="$emit('show-help')"
        >
          <v-icon :icon="mdiHelpCircleOutline" size="16" />
          <v-tooltip activator="parent" location="bottom">
            {{ $t('help.title') }}
          </v-tooltip>
        </v-btn>

        <v-btn
          icon
          size="x-small"
          variant="text"
          class="header-action-btn"
          @click="$emit('show-about')"
        >
          <v-icon :icon="mdiInformationOutline" size="16" />
          <v-tooltip activator="parent" location="bottom">
            {{ $t('about.title') }}
          </v-tooltip>
        </v-btn>

        <v-btn
          icon
          size="x-small"
          variant="text"
          class="header-action-btn"
          @click="$emit('open-github')"
        >
          <v-icon :icon="mdiGithub" size="16" />
          <v-tooltip activator="parent" location="bottom">
            {{ $t('sidebar.github') }}
          </v-tooltip>
        </v-btn>

        <v-btn
          icon
          size="x-small"
          variant="text"
          class="header-action-btn action-primary"
          @click="$emit('show-settings')"
        >
          <v-icon :icon="mdiCog" size="16" />
          <v-tooltip activator="parent" location="bottom">
            {{ $t('header.settings') }}
          </v-tooltip>
        </v-btn>
      </div>

      <div v-if="SHOW_WINDOW_CONTROLS" class="window-controls">
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
    </div>

    <UpdateDialog
      v-if="updateInfo"
      v-model="showUpdateDialog"
      :update-info="updateInfo"
      :is-installing="isInstalling"
      @install="handleInstallUpdate"
    />
  </div>
</template>

<script setup lang="ts">
import {
  mdiClose,
  mdiCloudSyncOutline,
  mdiCog,
  mdiDownloadCircle,
  mdiGithub,
  mdiHelpCircleOutline,
  mdiInformationOutline,
  mdiWindowMaximize,
  mdiWindowMinimize,
  mdiWindowRestore,
} from '@mdi/js'
import { getCurrentWindow, type Window as TauriWindow } from '@tauri-apps/api/window'
import { onMounted, onUnmounted, ref } from 'vue'
import UpdateDialog from '@/components/dialogs/UpdateDialog.vue'
import { useThemeToggle } from '@/composables/useTheme'
import { useUpdate } from '@/composables/useUpdate'
import { isMac, SHOW_WINDOW_CONTROLS } from '@/utils/platform'

defineEmits<{
  (e: 'show-settings'): void
  (e: 'show-help'): void
  (e: 'show-about'): void
  (e: 'open-github'): void
}>()

let appWindow: TauriWindow | null = null

const isMaximized = ref(false)
const showUpdateDialog = ref(false)
const { themeIcon, themeTooltip, toggleTheme } = useThemeToggle()
const { hasUpdate, updateInfo, isChecking, isInstalling, checkForUpdate, installUpdate } =
  useUpdate()

const handleCheckUpdate = async () => {
  if (!hasUpdate.value) {
    await checkForUpdate()
  }
  if (updateInfo.value) {
    showUpdateDialog.value = true
  }
}

const handleInstallUpdate = async () => {
  try {
    await installUpdate()
    showUpdateDialog.value = false
  } catch (error) {
    console.error('Failed to install update:', error)
  }
}

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
  console.log(
    '[Jedi TitleBar] onMounted, isMac:',
    isMac(),
    'SHOW_WINDOW_CONTROLS:',
    SHOW_WINDOW_CONTROLS
  )
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
  console.log('[Jedi TitleBar] onDragMouseDown fired', {
    button: e.button,
    target: e.target,
    className: (e.target as HTMLElement)?.className,
  })
  if (!appWindow) {
    console.warn('[Jedi TitleBar] No appWindow')
    return
  }
  if (e.button !== 0) return
  const target = e.target as HTMLElement
  if (target.closest('.header-right, .global-actions')) {
    console.log('[Jedi TitleBar] Clicked on action area, skipping drag')
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
  if (target.closest('.header-right, .global-actions')) {
    console.log('[Jedi TitleBar] Double-clicked on action area, skipping maximize')
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
  min-width: 8px;
  pointer-events: none;
}

.header-left--mac {
  min-width: 78px;
}

.header-center {
  flex: 1;
  overflow: hidden;
}

.global-actions {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  height: 26px;
  padding: 1px 4px;
  border: 1px solid rgba(var(--v-theme-on-surface), 0.08);
  border-radius: 6px;
  background: rgba(var(--v-theme-surface), 0.62);
  -webkit-app-region: no-drag;
}

.header-action-btn {
  border-radius: 4px;
  height: 22px !important;
  width: 22px !important;
  min-width: 22px !important;
  padding: 0 !important;
  color: rgba(var(--v-theme-on-surface), 0.68);
  transition:
    background 0.15s ease,
    color 0.15s ease,
    transform 0.15s ease;
}

.header-action-btn:hover {
  background: rgba(var(--v-theme-primary), 0.12);
  color: rgb(var(--v-theme-primary));
}

.header-action-btn:active {
  transform: translateY(1px);
}

.action-primary {
  color: rgb(var(--v-theme-primary));
}

.update-available {
  color: rgb(var(--v-theme-error));
}

.header-right {
  display: flex;
  align-items: center;
  gap: 6px;
  padding-right: 4px;
  justify-content: flex-end;
  -webkit-app-region: no-drag;
}

.window-controls {
  display: flex;
  align-items: center;
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
