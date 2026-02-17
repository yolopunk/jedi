<template>
  <v-app>
    <app-sidebar
      @open-github="openGithubRepo"
      @open-website="openProjectWebsite"
      @open-email="sendEmail"
      @show-help="showHelpDialog = true"
      @show-settings="showSettingsDialog = true"
      @show-about="showAboutDialog = true"
    />

    <v-main class="jedi-main-content">
      <v-container class="py-6 px-6" height="100%">
        <router-view v-slot="{ Component }">
          <component :is="Component" />
        </router-view>
      </v-container>
    </v-main>

    <!-- 系统信息栏 (Global Status Bar) -->
    <system-info-bar class="system-bar-override"></system-info-bar>

    <!-- 对话框组件 -->
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
      :src="currentPlaying?.audio_url"
      autoplay
    ></audio>
  </v-app>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useStorage } from '@/composables/useStorage'
import SystemInfoBar from '@/components/common/SystemInfoBar.vue'
import AppSidebar from '@/components/layout/AppSidebar.vue'
import HelpDialog from '@/components/dialogs/HelpDialog.vue'
import SettingsDialog from '@/components/dialogs/SettingsDialog.vue'
import AboutDialog from '@/components/dialogs/AboutDialog.vue'
import { open } from '@tauri-apps/plugin-shell'
import { initTheme } from '@/composables/useTheme'
import { useWallpaper } from '@/composables/useWallpaper'
import { useAudioPlayer } from '@/composables/useAudioPlayer'
import { useUpdate } from '@/composables/useUpdate'

const { locale } = useI18n()
const { getItem } = useStorage()
const { currentPlaying, setAudioRef } = useAudioPlayer()

const audioEl = ref<HTMLAudioElement | null>(null)

// 对话框状态
const showHelpDialog = ref(false)
const showSettingsDialog = ref(false)
const showAboutDialog = ref(false)

// Initialize Audio Player
watch(audioEl, (el) => {
  if (el) {
    setAudioRef(el)
  }
})

// 初始化语言
const initLanguage = async () => {
  const savedLang = await getItem<string>('language')
  if (savedLang) {
    locale.value = savedLang
  }
}

// 打开GitHub仓库
const openGithubRepo = async () => {
  try {
    await open('https://github.com/yolopunk/jedi')
  } catch (error) {
    console.error('Failed to open GitHub repo:', error)
  }
}

// 打开项目网站
const openProjectWebsite = async () => {
  try {
    await open('https://yolopunk.github.io/jedi')
  } catch (error) {
    console.error('Failed to open project website:', error)
  }
}

// 发送邮件
const sendEmail = async () => {
  try {
    await open('mailto:cynosurech@gmail.com')
  } catch (error) {
    console.error('Failed to open email client:', error)
  }
}

// 初始化主题
onMounted(async () => {
  initTheme()
  await initLanguage()

  // Check and run wallpaper auto-update
  const { startAutoUpdateCheck } = useWallpaper()
  startAutoUpdateCheck()

  // Initialize auto-update check
  const { startAutoCheck } = useUpdate()
  startAutoCheck()
})
</script>

<style scoped>
.jedi-main-content {
  background-color: rgb(var(--v-theme-background));
  min-height: 100vh;
}

.system-bar-override {
  z-index: 100;
}
</style>
