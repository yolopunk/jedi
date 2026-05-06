<template>
  <v-app>
    <div class="app-layout" :style="{ '--title-bar-height': `${TITLE_BAR_TOTAL_HEIGHT}px` }">
      <AppHeader />
      <div class="content-wrapper">
        <!-- Sidebar -->
        <app-sidebar
          :collapsed="sidebarCollapsed"
          @open-github="openGithubRepo"
          @open-website="openProjectWebsite"
          @open-email="sendEmail"
          @show-help="showHelpDialog = true"
          @show-settings="showSettingsDialog = true"
          @show-about="showAboutDialog = true"
        />

        <!-- Main Content Area -->
        <div class="main-area">
          <!-- Content -->
          <v-main class="jedi-main-content">
            <v-container class="py-2 px-2" height="100%" fluid>
              <router-view v-slot="{ Component }">
                <component :is="Component" @open-settings="showSettingsDialog = true" />
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
              {{ sidebarCollapsed ? '展开侧边栏' : '收起侧边栏' }}
            </v-tooltip>
          </v-btn>
        </div>
        <div class="status-bar-center" data-tauri-drag-region>
          <system-info-bar></system-info-bar>
        </div>
        <div class="status-bar-right" data-tauri-no-drag>
          <span class="status-text clickable" @click="toggleLanguage" :title="localeTooltip">{{ currentLocale }}</span>
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
  </v-app>
</template>

<script setup lang="ts">
import { open } from '@tauri-apps/plugin-shell'
import { listen } from '@tauri-apps/api/event'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { mdiMenu, mdiMenuOpen } from '@mdi/js'
import { useAudioPlayer } from '@/composables/useAudioPlayer'
import { useStorage } from '@/composables/useStorage'
import { initTheme } from '@/composables/useTheme'
import { useUpdate } from '@/composables/useUpdate'
import { useWallpaper } from '@/composables/useWallpaper'
import AppSidebar from '@/components/layout/AppSidebar.vue'
import AppHeader from '@/components/layout/AppHeader.vue'
import { TITLE_BAR_TOTAL_HEIGHT } from '@/utils/platform'
import SystemInfoBar from '@/components/common/SystemInfoBar.vue'
import HelpDialog from '@/components/dialogs/HelpDialog.vue'
import SettingsDialog from '@/components/dialogs/SettingsDialog.vue'
import AboutDialog from '@/components/dialogs/AboutDialog.vue'

const { locale } = useI18n()
const { getItem, setItem } = useStorage()
const { setAudioRef } = useAudioPlayer()
const router = useRouter()

// Tray navigation listener
let unlistenTray: (() => void) | null = null

const audioEl = ref<HTMLAudioElement | null>(null)
const currentPlaying = computed(() => useAudioPlayer().currentPlaying)

// Dialog states
const showHelpDialog = ref(false)
const showSettingsDialog = ref(false)
const showAboutDialog = ref(false)

// Sidebar state
const sidebarCollapsed = ref(false)

// Current locale display
const currentLocale = computed(() => {
  const flag = locale.value === 'zh' ? '🇨🇳 ' : '🇺🇸 '
  const text = locale.value === 'zh' ? 'zh-CN' : 'en-US'
  return flag + text
})

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
  initTheme()
  await initLanguage()
  await loadSidebarState()

  // Check and run wallpaper auto-update
  const { startAutoUpdateCheck } = useWallpaper()
  startAutoUpdateCheck()

  // Initialize auto-update check
  const { startAutoCheck } = useUpdate()
  startAutoCheck()

  // Listen for tray navigation events
  try {
    unlistenTray = await listen<string>('tray-navigate', (event) => {
      if (event.payload) {
        router.push(event.payload)
      }
    })
  } catch {
    // ignore if listen fails (e.g. non-Tauri env)
  }
})

onUnmounted(() => {
  unlistenTray?.()
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
}

.status-text.clickable {
  cursor: pointer;
  transition: color 0.15s ease;
}

.status-text.clickable:hover {
  color: rgb(var(--v-theme-primary));
}
</style>
