<template>
  <component :is="iconComponents[providerId]" v-if="iconComponents[providerId]" class="provider-icon-svg" />
  <span v-else class="provider-icon-default">{{ fallbackIcon }}</span>
</template>

<script setup lang="ts">
defineProps<{
  providerId: string
  size?: number
}>()

defineOptions({ name: 'ProviderIcon' })

// Fallback for unknown providers
const fallbackIcon = '○'

// Provider icon components using inline SVGs
const iconComponents: Record<string, any> = {
  openai: {
    template: `<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
      <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="1.5"/>
      <circle cx="12" cy="12" r="4" fill="currentColor"/>
    </svg>`
  },
  anthropic: {
    template: `<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
      <circle cx="12" cy="12" r="9" stroke="currentColor" stroke-width="1.5"/>
      <circle cx="12" cy="12" r="3" fill="currentColor"/>
      <circle cx="12" cy="5" r="1.5" fill="currentColor"/>
      <circle cx="12" cy="19" r="1.5" fill="currentColor"/>
    </svg>`
  },
  google: {
    template: `<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
      <rect x="3" y="3" width="18" height="18" rx="3" stroke="currentColor" stroke-width="1.5"/>
      <line x1="12" y1="3" x2="12" y2="21" stroke="currentColor" stroke-width="1.5"/>
      <line x1="3" y1="12" x2="21" y2="12" stroke="currentColor" stroke-width="1.5"/>
    </svg>`
  },
  deepseek: {
    template: `<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
      <polygon points="12,2 22,12 12,22 2,12" stroke="currentColor" stroke-width="1.5" fill="none"/>
      <polygon points="12,7 17,12 12,17 7,12" fill="currentColor"/>
    </svg>`
  },
  openrouter: {
    template: `<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
      <circle cx="12" cy="12" r="9" stroke="currentColor" stroke-width="1.5"/>
      <circle cx="12" cy="12" r="3" stroke="currentColor" stroke-width="1.5"/>
      <line x1="12" y1="3" x2="12" y2="7" stroke="currentColor" stroke-width="1.5"/>
      <line x1="12" y1="17" x2="12" y2="21" stroke="currentColor" stroke-width="1.5"/>
      <line x1="3" y1="12" x2="7" y2="12" stroke="currentColor" stroke-width="1.5"/>
      <line x1="17" y1="12" x2="21" y2="12" stroke="currentColor" stroke-width="1.5"/>
    </svg>`
  },
  ollama: {
    template: `<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
      <circle cx="12" cy="12" r="9" stroke="currentColor" stroke-width="1.5"/>
      <circle cx="12" cy="12" r="5" stroke="currentColor" stroke-width="1.5"/>
      <circle cx="12" cy="12" r="2" fill="currentColor"/>
    </svg>`
  }
}
</script>

<style scoped>
.provider-icon-svg {
  width: 20px;
  height: 20px;
  color: currentColor;
}

.provider-icon-svg :deep(svg) {
  width: 100%;
  height: 100%;
}

.provider-icon-default {
  font-size: 14px;
}
</style>
