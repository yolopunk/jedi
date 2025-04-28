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
        <hosts-resolver></hosts-resolver>
      </v-container>
    </v-main>

    <!-- 系统信息栏 -->
    <system-info-bar></system-info-bar>

    <!-- 对话框组件 -->
    <help-dialog v-model="showHelpDialog" />
    <settings-dialog v-model="showSettingsDialog" />
    <about-dialog
      v-model="showAboutDialog"
      @open-github="openGithubRepo"
      @open-website="openProjectWebsite"
      @open-email="sendEmail"
    />
  </v-app>
</template>
<script setup lang="ts">
import { ref, onMounted } from 'vue'
import HostsResolver from '@/components/HostsResolver.vue'
import SystemInfoBar from '@/components/common/SystemInfoBar.vue'
import AppSidebar from '@/components/layout/AppSidebar.vue'
import HelpDialog from '@/components/dialogs/HelpDialog.vue'
import SettingsDialog from '@/components/dialogs/SettingsDialog.vue'
import AboutDialog from '@/components/dialogs/AboutDialog.vue'
import { open } from '@tauri-apps/plugin-shell'
import { initTheme } from '@/composables/useTheme'

// 对话框状态
const showHelpDialog = ref(false)
const showSettingsDialog = ref(false)
const showAboutDialog = ref(false)

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
onMounted(() => {
  initTheme()
})
</script>


<style scoped>
.jedi-main-content {
  background-color: var(--jedi-bg-color) !important;
}
</style>
