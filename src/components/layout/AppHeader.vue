<template>
  <div
    id="app-header"
    class="app-header"
    data-tauri-drag-region
    @mousedown="onDragMouseDown"
    @dblclick="onDragDoubleClick"
  >
    <div class="header-left" :class="{ 'header-left--mac': isMac() }" />

    <div class="header-center">
      <button
        v-if="hasUpdate || isInstalling || installError"
        type="button"
        class="update-notice"
        :class="{ 'update-notice--error': installError }"
        :aria-label="updateStatus"
        @click="handleCheckUpdate"
      >
        <v-icon :icon="installError ? mdiAlertCircle : mdiDownloadCircle" size="16" aria-hidden="true" />
        <span role="status">{{ updateStatus }}</span>
      </button>
    </div>

    <div class="header-right">
      <div class="global-actions">
        <v-btn
          icon
          size="x-small"
          variant="text"
          class="header-action-btn"
          :class="{ 'update-progress-btn': isInstalling }"
          :aria-label="updateStatus"
          @click="handleCheckUpdate"
        >
          <v-progress-circular
            v-if="isChecking || isInstalling"
            :size="15"
            :width="2"
            :indeterminate="!isInstalling || installProgress == null || installProgress >= 100"
            :model-value="installProgress ?? 0"
          />
          <span v-if="isInstalling" class="update-progress-label">{{ installProgress != null && installProgress < 100 ? `${installProgress}%` : $t('dialogs.update.updatingShort') }}</span>
          <v-icon
            v-if="!isChecking && !isInstalling"
            :icon="installError ? mdiAlertCircle : hasUpdate ? mdiDownloadCircle : mdiCloudSyncOutline"
            size="16"
            :class="{ 'update-available': hasUpdate || installError }"
          />
          <v-tooltip activator="parent" location="bottom">
            {{ updateStatus }}
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
          :aria-label="$t('help.title')"
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
          :aria-label="$t('about.title')"
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
          :aria-label="$t('sidebar.github')"
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
          :aria-label="$t('header.settings')"
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
          :aria-label="$t('windowControls.minimize')"
          @click="handleMinimize"
        >
          <v-icon :icon="mdiWindowMinimize" size="16" />
        </v-btn>
        <v-btn
          icon
          size="x-small"
          variant="text"
          class="window-btn"
          :aria-label="isMaximized ? $t('windowControls.restore') : $t('windowControls.maximize')"
          @click="handleMaximize"
        >
          <v-icon :icon="isMaximized ? mdiWindowRestore : mdiWindowMaximize" size="16" />
        </v-btn>
        <v-btn
          icon
          size="x-small"
          variant="text"
          class="window-btn window-btn-close"
          :aria-label="$t('windowControls.close')"
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
      :progress="installProgress"
      :error="installError"
      @install="handleInstallUpdate"
    />
  </div>
</template>

<script setup lang="ts">
import {
  mdiAlertCircle,
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
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
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
const { t } = useI18n()
const { themeIcon, themeTooltip, toggleTheme } = useThemeToggle()
const {
  hasUpdate,
  updateInfo,
  isChecking,
  isInstalling,
  installProgress,
  installError,
  checkForUpdate,
  installUpdate,
} = useUpdate()

const updateStatus = computed(() => {
  if (isInstalling.value) {
    return installProgress.value != null && installProgress.value < 100
      ? `${t('dialogs.update.downloading')} ${installProgress.value}%`
      : t('dialogs.update.inProgress')
  }
  if (installError.value) return t('dialogs.update.failed')
  if (isChecking.value) return t('settings.updateChecking')
  return hasUpdate.value && updateInfo.value
    ? t('settings.updateAvailable', { version: updateInfo.value.version })
    : t('settings.checkUpdate')
})

const handleCheckUpdate = async () => {
  if (!hasUpdate.value) {
    await checkForUpdate()
  }
  if (updateInfo.value) {
    showUpdateDialog.value = true
  }
}

const handleInstallUpdate = async () => {
  showUpdateDialog.value = false
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
  try {
    appWindow = getCurrentWindow()
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
  if (!appWindow) return
  if (e.button !== 0) return
  const target = e.target as HTMLElement
  if (target.closest('.header-right, .global-actions, .update-notice')) {
    return
  }
  try {
    await appWindow.startDragging()
  } catch (err) {
    console.error('[Jedi TitleBar] startDragging failed:', err)
  }
}

const onDragDoubleClick = async (e: MouseEvent) => {
  if (isMac()) return

  e.preventDefault()
  if (!appWindow) return
  const target = e.target as HTMLElement
  if (target.closest('.header-right, .global-actions, .update-notice')) {
    return
  }
  try {
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
  min-width: 0;
  display: flex;
  justify-content: center;
  padding: 0 8px;
  overflow: hidden;
}

.update-notice {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  max-width: 100%;
  min-width: 0;
  height: 26px;
  padding: 0 12px;
  border: 1px solid rgb(var(--v-theme-primary));
  border-radius: 6px;
  background: rgb(var(--v-theme-primary));
  color: rgb(var(--v-theme-on-primary));
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  -webkit-app-region: no-drag;
}

.update-notice span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.update-notice:hover { filter: brightness(1.1); }
.update-notice:focus-visible {
  outline: 2px solid rgb(var(--v-theme-on-surface));
  outline-offset: 1px;
}
.update-notice--error {
  background: rgb(var(--v-theme-error));
  border-color: rgb(var(--v-theme-error));
  color: rgb(var(--v-theme-on-error));
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
    color 0.15s ease;
}

.update-progress-btn {
  width: auto !important;
  padding: 0 4px !important;
  gap: 4px;
  color: rgb(var(--v-theme-primary));
}

.update-progress-label {
  font-size: 11px;
  white-space: nowrap;
}

.header-action-btn:hover {
  background: rgba(var(--v-theme-primary), 0.12);
  color: rgb(var(--v-theme-primary));
}

.header-action-btn:active {
  background: rgba(var(--v-theme-primary), 0.2);
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
  background: rgb(var(--danger-rgb) / 0.9) !important;
}

.window-btn-close:hover .v-icon {
  color: var(--text-inverse) !important;
  opacity: 1;
}
</style>
