<template>
  <div class="provider-selector">
    <!-- Tabs: Popular / Other / Custom -->
    <v-tabs v-model="activeTab" color="primary" density="compact">
      <v-tab value="popular">
        <v-icon icon="mdi-star" start size="14" />
        Popular
        <v-chip v-if="popularCount > 0" size="x-small" class="ml-1">
          {{ popularCount }}
        </v-chip>
      </v-tab>
      <v-tab value="other">
        <v-icon icon="mdi-dots-horizontal" start size="14" />
        Other
        <v-chip v-if="otherCount > 0" size="x-small" class="ml-1">
          {{ otherCount }}
        </v-chip>
      </v-tab>
      <v-tab value="custom">
        <v-icon icon="mdi-plus-circle" start size="14" />
        Custom
        <v-chip v-if="customCount > 0" size="x-small" class="ml-1">
          {{ customCount }}
        </v-chip>
      </v-tab>
    </v-tabs>

    <v-divider />

    <!-- Provider Grid -->
    <div class="provider-grid">
      <div
        v-for="provider in displayedProviders"
        :key="provider.id"
        class="provider-card"
        :class="{
          selected: provider.id === store.selectedProviderId,
          configured: store.isProviderConfigured(provider.id),
        }"
        @click="selectProvider(provider)"
      >
        <div class="provider-icon">
          <v-icon :icon="getProviderIcon(provider.id)" size="24" />
        </div>
        <div class="provider-info">
          <span class="provider-name">{{ provider.name }}</span>
          <span class="provider-models">{{ Object.keys(provider.models).length }} models</span>
        </div>
        <div class="provider-status">
          <v-chip
            v-if="store.isProviderConfigured(provider.id)"
            size="x-small"
            color="success"
          >
            CONFIGURED
          </v-chip>
          <v-chip v-else size="x-small" variant="outlined">
            NOT SET
          </v-chip>
        </div>
        <v-icon
          v-if="provider.id === store.selectedProviderId"
          icon="mdi-check-circle"
          color="primary"
          size="18"
          class="check-icon"
        />
      </div>

      <!-- Empty state -->
      <div v-if="displayedProviders.length === 0" class="empty-state">
        <v-icon icon="mdi-inbox-outline" size="32" color="surface-variant" />
        <p>No providers in this category</p>
      </div>
    </div>

    <!-- Configure Button -->
    <div class="config-section">
      <v-btn
        block
        color="primary"
        :disabled="!store.selectedProviderId"
        @click="$emit('configure', store.selectedProviderId)"
      >
        <v-icon icon="mdi-cog" start />
        Configure Selected Provider
      </v-btn>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useModelsDevStore } from '@/stores/modelsDev'
import type { ModelsDevProvider } from '@/types/modelsDev'

const emit = defineEmits<{
  (e: 'configure', providerId: string): void
}>()

const store = useModelsDevStore()
const activeTab = ref<'popular' | 'other' | 'custom'>('popular')

const displayedProviders = computed(() => {
  switch (activeTab.value) {
    case 'popular':
      return store.popularProviders
    case 'other':
      return store.otherProviders
    case 'custom':
      return store.customProviders
    default:
      return []
  }
})

const popularCount = computed(() => store.popularProviders.length)
const otherCount = computed(() => store.otherProviders.length)
const customCount = computed(() => store.customProviders.length)

function selectProvider(provider: ModelsDevProvider) {
  store.selectProvider(provider.id)
}

function getProviderIcon(providerId: string): string {
  const icons: Record<string, string> = {
    openai: 'mdi-openai',
    anthropic: 'mdi-brain',
    google: 'mdi-google',
    deepseek: 'mdi-deepseek',
    xai: 'mdi-robot',
    mistral: 'mdi-weather-cloudy',
    moonshot: 'mdi-moon-waning-crescent',
    zhipu: 'mdi-alpha-z-circle',
  }
  return icons[providerId] || 'mdi-cloud'
}
</script>

<style scoped>
.provider-selector {
  display: flex;
  flex-direction: column;
  gap: 0;
}

.provider-grid {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px;
  max-height: 300px;
  overflow-y: auto;
}

.provider-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid rgba(0, 255, 255, 0.1);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.provider-card:hover {
  background: rgba(0, 255, 255, 0.05);
  border-color: rgba(0, 255, 255, 0.3);
}

.provider-card.selected {
  border-color: rgba(0, 255, 136, 0.5);
  background: rgba(0, 255, 136, 0.05);
}

.provider-card.configured {
  border-left: 3px solid #00ff88;
}

.provider-icon {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 255, 255, 0.05);
  border-radius: 8px;
}

.provider-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.provider-name {
  font-size: 14px;
  font-weight: 600;
  color: #e4e4e7;
}

.provider-models {
  font-size: 11px;
  color: #71717a;
}

.provider-status {
  display: flex;
  align-items: center;
}

.check-icon {
  position: absolute;
  right: 12px;
}

.config-section {
  padding: 12px;
  border-top: 1px solid rgba(255, 255, 255, 0.05);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 32px;
  color: rgba(255, 255, 255, 0.4);
  gap: 8px;
}
</style>
