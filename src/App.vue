<template>
  <!-- 截图覆盖窗口等无框架路由：跳过主应用布局，只渲染页面本身 -->
  <router-view v-if="isBareLayout" />
  <v-app v-else>
    <div class="app-layout" :style="{ '--title-bar-height': `${TITLE_BAR_TOTAL_HEIGHT}px` }">
      <AppHeader
        @open-github="openGithubRepo"
        @show-help="showHelpDialog = true"
        @show-settings="showSettingsDialog = true"
        @show-about="showAboutDialog = true"
      />
      <div class="content-wrapper">
        <!-- Sidebar -->
        <app-sidebar
          :collapsed="sidebarCollapsed"
        />

        <!-- Main Content Area -->
        <div class="main-area">
          <!-- Content -->
          <v-main class="jedi-main-content">
            <v-container class="py-2 px-2" height="100%" fluid>
              <router-view v-slot="{ Component }">
                <div :key="route.path" class="workspace-page">
                  <component :is="Component" @open-settings="showSettingsDialog = true" />
                </div>
              </router-view>
            </v-container>
          </v-main>
        </div>
      </div>

      <!-- Full Width Bottom Status Bar -->
      <div class="bottom-status-bar">
        <div class="status-bar-left" data-tauri-no-drag>
          <v-btn
            icon
            size="x-small"
            variant="text"
            @click="toggleSidebar"
            class="sidebar-toggle-btn"
          >
            <v-icon :icon="sidebarCollapsed ? mdiMenu : mdiMenuOpen" size="16" />
            <v-tooltip activator="parent" location="top">
              {{ sidebarCollapsed ? t('sidebar.expand') : t('sidebar.collapse') }}
            </v-tooltip>
          </v-btn>
        </div>
        <div class="status-bar-center" data-tauri-drag-region>
          <system-info-bar></system-info-bar>
        </div>
        <div class="status-bar-right" data-tauri-no-drag>
          <button
            class="status-text clickable"
            :title="localeTooltip"
            :aria-label="localeTooltip"
            @click="toggleLanguage"
          >{{ currentLocale }}</button>
        </div>
      </div>
    </div>

    <!-- Dialog Components -->
    <help-dialog v-model="showHelpDialog" />
    <settings-dialog v-model="showSettingsDialog" />
    <about-dialog
      v-model="showAboutDialog"
      @open-github="openGithubRepo"
      @open-website="openProjectWebsite"
      @open-email="sendEmail"
    />

    <!-- Global Audio Element -->
    <audio
      ref="audioEl"
      class="d-none"
      :src="currentPlaying?.audio_url || ''"
      autoplay
    ></audio>

    <!-- Splash Screen (fades out once app init completes) -->
    <SplashScreen v-if="!splashDone" :leaving="splashLeaving" @ready="handleSplashReady" />
  </v-app>
</template>

<script setup lang="ts">
import { mdiMenu, mdiMenuOpen } from '@mdi/js'
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-shell'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'
import SplashScreen from '@/components/common/SplashScreen.vue'
import SystemInfoBar from '@/components/common/SystemInfoBar.vue'
import AboutDialog from '@/components/dialogs/AboutDialog.vue'
import HelpDialog from '@/components/dialogs/HelpDialog.vue'
import SettingsDialog from '@/components/dialogs/SettingsDialog.vue'
import AppHeader from '@/components/layout/AppHeader.vue'
import AppSidebar from '@/components/layout/AppSidebar.vue'
import { useAudioPlayer } from '@/composables/useAudioPlayer'
import { useStorage } from '@/composables/useStorage'
import { initTheme } from '@/composables/useTheme'
import { useUpdate } from '@/composables/useUpdate'
import { useWallpaper } from '@/composables/useWallpaper'
import { TITLE_BAR_TOTAL_HEIGHT } from '@/utils/platform'

const { locale, t } = useI18n()
const { getItem, setItem } = useStorage()
const { currentPlaying, setAudioRef } = useAudioPlayer()
const { startAutoCheck } = useUpdate()
const router = useRouter()
const route = useRoute()

// 覆盖窗口等独立路由不渲染主布局
const isBareLayout = computed(() => route.meta.bare === true)

// Tray navigation listener
let unlistenTray: (() => void) | null = null
let unlistenScreenshot: (() => void) | null = null

const audioEl = ref<HTMLAudioElement | null>(null)

// Dialog states
const showHelpDialog = ref(false)
const showSettingsDialog = ref(false)
const showAboutDialog = ref(false)

// Sidebar state
const sidebarCollapsed = ref(false)

// Splash state
const SPLASH_MIN_MS = 1800
const SPLASH_FADE_MS = 600
// 初始化耗时过长时，完成后仍保证启动页至少完整展示这一段，避免闪现即淡出
const SPLASH_LEAVE_FLOOR_MS = 400
const splashLeaving = ref(false)
const splashDone = ref(false)

// 计时起点 = 启动页组件实际渲染完成（@ready），与初始化 await 耗时无关
let splashStartedAt = 0
const handleSplashReady = () => {
  splashStartedAt = Date.now()
}

// 初始化完成后补足最短停留，再淡出并卸载
const scheduleSplashLeave = () => {
  const start = splashStartedAt || Date.now()
  const remain = Math.max(SPLASH_LEAVE_FLOOR_MS, SPLASH_MIN_MS - (Date.now() - start))
  window.setTimeout(() => {
    splashLeaving.value = true
    window.setTimeout(() => {
      splashDone.value = true
    }, SPLASH_FADE_MS)
  }, remain)
}

// Current locale display (text only: flag emojis render as "CH"/"US" letters on Windows)
const currentLocale = computed(() => (locale.value === 'zh' ? 'zh-CN' : 'en-US'))

const localeTooltip = computed(() => {
  return locale.value === 'zh' ? 'Switch to English' : '切换到中文'
})

// Toggle language
const toggleLanguage = async () => {
  const newLang = locale.value === 'zh' ? 'en' : 'zh'
  locale.value = newLang
  await setItem('language', newLang)
}

// Initialize Audio Player
watch(audioEl, el => {
  if (el) {
    setAudioRef(el)
  }
})

// Initialize language
const initLanguage = async () => {
  const savedLang = await getItem<string>('language')
  if (savedLang) {
    locale.value = savedLang
  }
}

// Toggle sidebar
const toggleSidebar = () => {
  sidebarCollapsed.value = !sidebarCollapsed.value
}

// Open GitHub repo
const openGithubRepo = async () => {
  try {
    await open('https://github.com/yolopunk/jedi')
  } catch (error) {
    console.error('Failed to open GitHub repo:', error)
  }
}

// Open project website
const openProjectWebsite = async () => {
  try {
    await open('https://yolopunk.github.io/jedi')
  } catch (error) {
    console.error('Failed to open project website:', error)
  }
}

// Send email
const sendEmail = async () => {
  try {
    await open('mailto:cynosurech@gmail.com')
  } catch (error) {
    console.error('Failed to open email client:', error)
  }
}

// Load sidebar state from storage
const loadSidebarState = async () => {
  const savedCollapsed = await getItem<boolean>('sidebar-collapsed')
  if (savedCollapsed !== undefined && savedCollapsed !== null) {
    sidebarCollapsed.value = savedCollapsed
  }
}

// Save sidebar state to storage
watch(sidebarCollapsed, async newVal => {
  await setItem('sidebar-collapsed', newVal)
})

onMounted(async () => {
  if (isBareLayout.value) return
  initTheme()
  await initLanguage()
  await loadSidebarState()

  // Check and run wallpaper auto-update
  const { startAutoUpdateCheck } = useWallpaper()
  startAutoUpdateCheck()

  // Initialize auto-update check
  startAutoCheck()

  // Listen for tray navigation events
  try {
    unlistenTray = await listen<string>('tray-navigate', event => {
      if (event.payload) {
        router.push(event.payload)
      }
    })
  } catch {
    // ignore if listen fails (e.g. non-Tauri env)
  }

  // Listen for screenshots captured with the "send to AI" action: bring the
  // window up, land on the chat view and queue the image as an attachment.
  try {
    unlistenScreenshot = await listen<string>('screenshot-captured', async event => {
      const path = event.payload
      if (!path) return
      const { getCurrentWindow } = await import('@tauri-apps/api/window')
      const win = getCurrentWindow()
      await win.show()
      await win.setFocus()
      await router.push('/chat')
      const { useAiChatStore } = await import('@/stores/aiChat')
      useAiChatStore().addPendingAttachment(path)
    })
  } catch {
    // ignore if listen fails (e.g. non-Tauri env)
  }
  scheduleSplashLeave()
})

onUnmounted(() => {
  unlistenTray?.()
  unlistenScreenshot?.()
})
</script>

<style scoped>
/* Ensure background is applied to the layout container */
.app-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
  background-color: rgb(var(--v-theme-background));
  padding-top: var(--title-bar-height);
}

.content-wrapper {
  display: flex;
  flex: 1;
  overflow: hidden;
  min-height: 0;
}

.main-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  overflow: hidden;
}

.jedi-main-content {
  background-color: rgb(var(--v-theme-background));
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.jedi-main-content > .v-container {
  flex: 1;
  overflow: hidden;
  padding-bottom: 0 !important; /* Remove padding to allow full height apps */
  display: flex;
  flex-direction: column;
}

.bottom-status-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 24px;
  background: linear-gradient(
    180deg,
    rgba(var(--v-theme-surface), 0.92) 0%,
    rgba(var(--v-theme-surface), 0.98) 100%
  );
  backdrop-filter: blur(12px);
  border-top: 1px solid rgba(var(--v-theme-on-surface), 0.08);
  flex-shrink: 0;
  user-select: none;
  font-size: 0.75rem; /* Smaller font for narrow bar */
}

.status-bar-left {
  display: flex;
  align-items: center;
  padding: 0 4px;
  min-width: 48px;
}

.status-bar-center {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.status-bar-center :deep(.system-bar-override) {
  position: static;
  width: 100%;
}

.status-bar-right {
  display: flex;
  align-items: center;
  padding: 0 8px;
  min-width: 48px;
  justify-content: flex-end;
}

.sidebar-toggle-btn {
  border-radius: 4px;
  height: 20px !important;
  width: 20px !important;
  min-width: 20px !important;
  padding: 0 !important;
  transition: all 0.2s cubic-bezier(0.25, 0.8, 0.25, 1);
}

.sidebar-toggle-btn:hover {
  background: rgba(var(--v-theme-primary), 0.1);
  transform: scale(1.05);
}

.sidebar-toggle-btn .v-icon {
  opacity: 0.7;
  transition: opacity 0.2s ease;
}

.sidebar-toggle-btn:hover .v-icon {
  opacity: 1;
}

.status-text {
  font-size: 0.75rem;
  color: rgba(var(--v-theme-on-surface), 0.5);
  padding: 0 8px;
  background: none;
  border: none;
  font-family: inherit;
}

.status-text.clickable {
  cursor: pointer;
  transition: color 0.15s ease;
}

.status-text.clickable:hover {
  color: rgb(var(--v-theme-primary));
}

.status-text.clickable:focus-visible {
  outline: 2px solid rgb(var(--v-theme-primary));
  outline-offset: 1px;
  border-radius: 3px;
}
</style>
