<template>
  <div class="nav-footer px-4 pt-4">
    <v-divider class="mb-3"></v-divider>
    <div class="d-flex flex-column align-center">
      <!-- 底部按钮 -->
      <div class="d-flex justify-center w-100 mb-1">
        <v-btn :icon="mdiGithub" variant="text" color="var(--jedi-primary)" @click="$emit('open-github')" size="small" class="mx-1"></v-btn>
        <v-btn :icon="mdiHelpCircle" variant="text" color="var(--jedi-primary)" size="small" class="mx-1" @click="$emit('show-help')"></v-btn>
        <v-btn :icon="mdiCog" variant="text" color="var(--jedi-primary)" size="small" class="mx-1" @click="$emit('show-settings')"></v-btn>
        <v-btn :icon="mdiInformation" variant="text" color="var(--jedi-primary)" size="small" class="mx-1" @click="$emit('show-about')"></v-btn>
      </div>

      <!-- 版权信息 -->
      <div class="text-caption text-grey-darken-1 mt-1">© 2025 Jedi</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { mdiGithub, mdiHelpCircle, mdiCog, mdiInformation } from '@mdi/js'

// 定义组件事件
defineEmits<{
  (e: 'open-github'): void;
  (e: 'open-website'): void;
  (e: 'open-email'): void;
  (e: 'show-help'): void;
  (e: 'show-settings'): void;
  (e: 'show-about'): void;
}>()
</script>

<style scoped>
.nav-footer {
  position: fixed;
  bottom: 0;
  left: 0;
  width: 200px;
  background-color: #f8f9fa;
  z-index: 1;
  padding-bottom: 8px;
  box-shadow: 0 -4px 6px -2px rgba(0, 0, 0, 0.05);
}
</style>
