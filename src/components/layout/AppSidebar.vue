<template>
  <v-navigation-drawer permanent :width="220" class="jedi-sidebar">
    <!-- Logo Area -->
    <div class="d-flex flex-column align-center py-6 border-bottom">
      <v-img src="/icon.png" width="64" class="mb-3 jedi-logo-glow"></v-img>
      <h2 class="text-h6 font-weight-bold text-primary">JEDI</h2>
      <div class="text-caption text-secondary">Hosts Manager</div>
    </div>

    <!-- Navigation -->
    <v-list nav class="pa-2 mt-2">
      <v-list-item
        :active="true"
        rounded="lg"
        class="mb-1 sidebar-item"
        color="primary"
      >
        <template v-slot:prepend>
          <v-icon :icon="mdiDns" size="small" class="mr-2"></v-icon>
        </template>
        <v-list-item-title>Hosts 管理</v-list-item-title>
      </v-list-item>
    </v-list>

    <!-- Footer Area -->
    <template v-slot:append>
      <div class="pa-2">
        <app-footer
          @open-github="$emit('open-github')"
          @open-website="$emit('open-website')"
          @open-email="$emit('open-email')"
          @show-help="$emit('show-help')"
          @show-settings="$emit('show-settings')"
          @show-about="$emit('show-about')"
        />
      </div>
    </template>
  </v-navigation-drawer>
</template>

<script setup lang="ts">
import { mdiDns } from '@mdi/js'
import AppFooter from './AppFooter.vue'

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
/* Minimal scoped styles, rely on global theme */
.jedi-sidebar {
  /* Ensure border is visible */
  border-right: 1px solid var(--jedi-border) !important;
}

.border-bottom {
  border-bottom: 1px solid var(--jedi-border);
}

.jedi-logo-glow {
  filter: drop-shadow(0 0 10px rgba(59, 130, 246, 0.3));
}

/* Sidebar Item Active State Override for Light Theme */
:global(.light-theme) .sidebar-item.v-list-item--active {
  background-color: #e0f2fe !important; /* Light Blue-50 */
  color: #0369a1 !important; /* Sky-700 */
}

/* Use a subtle gradient for dark mode active state */
:global(.dark-theme) .sidebar-item.v-list-item--active {
  background: linear-gradient(90deg, rgba(59, 130, 246, 0.15), transparent) !important;
  color: #60a5fa !important; /* Blue-400 */
  border-left: 2px solid #60a5fa;
}
</style>
