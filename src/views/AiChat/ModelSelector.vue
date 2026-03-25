<template>
  <v-menu
    v-model="showMenu"
    :close-on-content-click="true"
    location="bottom"
    offset="4"
  >
    <template v-slot:activator="{ props }">
      <v-btn
        variant="outlined"
        class="model-selector-btn"
        v-bind="props"
        :disabled="disabled"
      >
        <v-icon icon="mdi-brain" start size="18" />
        <span class="model-name">
          {{ selectedModel?.name || 'Select Model' }}
        </span>
        <v-icon icon="mdi-chevron-down" end size="18" />
      </v-btn>
    </template>

    <v-card min-width="280" class="model-menu">
      <v-card-text class="pa-0">
        <!-- Search -->
        <div class="search-wrapper pa-2">
          <v-text-field
            v-model="searchQuery"
            :placeholder="$t('chat.modelSearchPlaceholder')"
            prepend-inner-icon="mdi-magnify"
            variant="solo-filled"
            density="compact"
            hide-details
            single-line
            clearable
          />
        </div>

        <v-divider />

        <!-- Model List -->
        <v-list density="compact" class="model-list">
          <template v-for="provider in groupedModels" :key="provider.name">
            <v-list-subheader>
              {{ provider.name }}
            </v-list-subheader>

            <v-list-item
              v-for="model in provider.models"
              :key="model.id"
              :active="model.id === modelValue"
              @click="selectModel(model.id)"
            >
              <template v-slot:prepend>
                <v-icon 
                  :icon="model.id === modelValue ? 'mdi-check-circle' : 'mdi-circle-outline'"
                  :color="model.id === modelValue ? 'primary' : 'default'"
                  size="20"
                />
              </template>

              <v-list-item-title>{{ model.name }}</v-list-item-title>

              <v-list-item-subtitle v-if="model.contextLength">
                {{ formatContext(model.contextLength) }} context
              </v-list-item-subtitle>
            </v-list-item>
          </template>

          <div v-if="filteredModels.length === 0" class="empty-state pa-4 text-center">
            <v-icon icon="mdi-magnify-close" size="32" color="surface-variant" />
            <p class="mt-2 text-medium-emphasis">No models found</p>
          </div>
        </v-list>
      </v-card-text>
    </v-card>
  </v-menu>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { Model } from '@/stores/aiChat'

const props = defineProps<{
  modelValue: string | null
  models: (Model & { providerName?: string })[]
  disabled?: boolean
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void
}>()

const showMenu = ref(false)
const searchQuery = ref('')

const selectedModel = computed(() => {
  return props.models.find(m => m.id === props.modelValue)
})

const filteredModels = computed(() => {
  if (!searchQuery.value) return props.models
  
  const query = searchQuery.value.toLowerCase()
  return props.models.filter(m => 
    m.name.toLowerCase().includes(query) ||
    m.providerName?.toLowerCase().includes(query)
  )
})

const groupedModels = computed(() => {
  const groups: Record<string, (Model & { providerName?: string })[]> = {}
  
  for (const model of filteredModels.value) {
    const provider = model.providerName || 'Unknown'
    if (!groups[provider]) {
      groups[provider] = []
    }
    groups[provider].push(model)
  }
  
  return Object.entries(groups).map(([name, models]) => ({
    name,
    models,
  }))
})

function selectModel(modelId: string) {
  emit('update:modelValue', modelId)
}

function formatContext(length: number): string {
  if (length >= 1000000) {
    return `${(length / 1000000).toFixed(1)}M`
  } else if (length >= 1000) {
    return `${(length / 1000).toFixed(0)}K`
  }
  return length.toString()
}
</script>

<style scoped>
.model-selector-btn {
  border-radius: 12px;
  text-transform: none;
  letter-spacing: 0;
  font-weight: 500;
}

.model-name {
  max-width: 150px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.model-menu {
  border-radius: 12px;
  overflow: hidden;
}

.search-wrapper {
  background: rgba(var(--v-theme-surface-variant), 0.3);
}

.model-list {
  max-height: 400px;
  overflow-y: auto;
}

.model-list :deep(.v-list-item) {
  padding-left: 12px;
}

.model-list :deep(.v-list-item--active) {
  background: rgba(var(--v-theme-primary), 0.1);
}

.empty-state {
  color: rgba(var(--v-theme-on-surface), 0.5);
}

/* Scrollbar styling */
.model-list::-webkit-scrollbar {
  width: 6px;
}

.model-list::-webkit-scrollbar-track {
  background: transparent;
}

.model-list::-webkit-scrollbar-thumb {
  background: rgba(var(--v-theme-on-surface), 0.2);
  border-radius: 3px;
}
</style>
