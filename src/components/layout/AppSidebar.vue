<template>
  <v-navigation-drawer permanent :border="0" :width="200" class="jedi-nav-drawer text-center">
    <v-container class="py-8 pb-32">
      <div class="app-logo-container mb-6">
        <v-img src="/icon.png" width="120" class="mx-auto jedi-hover-scale"></v-img>
      </div>
      <v-list nav class="jedi-nav-list pa-2">
        <v-list-item
          :active="true"
          color="var(--jedi-accent)"
          rounded="sm"
          title="Hosts 管理"
          :prepend-icon="mdiDns"
          class="jedi-nav-item mb-1"
        >
        </v-list-item>
      </v-list>
    </v-container>
    <app-footer 
      @open-github="$emit('open-github')"
      @open-website="$emit('open-website')"
      @open-email="$emit('open-email')"
      @show-help="$emit('show-help')"
      @show-settings="$emit('show-settings')"
      @show-about="$emit('show-about')"
    />
  </v-navigation-drawer>
</template>

<script setup lang="ts">
import { mdiDns } from '@mdi/js'
import AppFooter from './AppFooter.vue'

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
.jedi-nav-drawer {
  background-color: #f8f9fa !important;
  border-right: 1px solid rgba(0, 0, 0, 0.05) !important;
  overflow-y: auto !important;
}

.app-logo-container {
  transition: all 0.3s ease;
}

.jedi-nav-list {
  background-color: transparent !important;
}

.jedi-nav-item {
  margin-bottom: 4px;
  transition: all 0.2s ease;
}

.jedi-nav-item:hover {
  background-color: rgba(66, 165, 245, 0.05) !important;
}
</style>
