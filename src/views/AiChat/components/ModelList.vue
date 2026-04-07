<template>
  <div class="model-list">
    <!-- Provider Header -->
    <div class="provider-header">
      <button class="back-btn" @click="emit('back')">
        <span class="back-icon">←</span>
      </button>
      <div class="provider-info">
        <span class="provider-name">{{ providerName }}</span>
        <span class="model-count-chip">{{ filteredModels.length }} models</span>
      </div>
    </div>

    <!-- Search Field -->
    <div class="search-wrapper">
      <v-text-field
        v-model="searchQuery"
        placeholder="Search models..."
        prepend-inner-icon="mdi-magnify"
        variant="solo-filled"
        density="compact"
        hide-details
        single-line
        clearable
      />
    </div>

    <!-- Model Cards -->
    <div class="models-container">
      <div
        v-for="model in filteredModels"
        :key="model.id"
        class="model-card"
        :class="{ selected: isSelected(model) }"
        @click="emit('select', model)"
      >
        <div class="card-header">
          <span class="model-name">{{ model.name }}</span>
          <v-icon
            v-if="isSelected(model)"
            icon="mdi-check-circle"
            size="20"
            class="check-icon"
          />
        </div>

        <div class="badges">
          <span v-if="model.reasoning" class="badge reasoning">REASONING</span>
          <span v-if="model.tool_call" class="badge tools">TOOLS</span>
        </div>

        <div class="model-details">
          <div class="detail-row">
            <span class="detail-label">Context</span>
            <span class="detail-value">{{ formatContext(model.limit?.context) }}</span>
          </div>
          <div class="detail-row">
            <span class="detail-label">Cost</span>
            <span class="detail-value">{{ formatCost(model.cost) }}</span>
          </div>
        </div>

        <div class="modality-indicators">
          <span
            v-for="mod in getInputModalities(model)"
            :key="mod"
            class="modality-badge"
            :class="mod.toLowerCase()"
          >{{ mod }}</span>
        </div>
      </div>

      <!-- Empty State -->
      <div v-if="filteredModels.length === 0" class="empty-state">
        <v-icon icon="mdi-magnify-close" size="48" color="surface-variant" />
        <p class="empty-text">No models found</p>
        <p class="empty-hint">Try adjusting your search</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { ModelsDevModel } from '@/types/modelsDev'

const props = defineProps<{
  providerName: string
  models: ModelsDevModel[]
  selectedModelId?: string | null
}>()

const emit = defineEmits<{
  (e: 'back'): void
  (e: 'select', model: ModelsDevModel): void
}>()

const searchQuery = ref('')

const filteredModels = computed(() => {
  if (!searchQuery.value) return props.models

  const query = searchQuery.value.toLowerCase()
  return props.models.filter(m =>
    m.name.toLowerCase().includes(query) ||
    m.id.toLowerCase().includes(query)
  )
})

function isSelected(model: ModelsDevModel): boolean {
  return model.id === props.selectedModelId
}

function formatContext(context?: number): string {
  if (!context) return '-'
  if (context >= 1000000) {
    return `${(context / 1000000).toFixed(context % 1000000 === 0 ? 0 : 1)}M`
  }
  if (context >= 1000) {
    return `${Math.round(context / 1000)}K`
  }
  return context.toString()
}

function formatCost(cost?: ModelsDevModel['cost']): string {
  if (!cost) return '-'
  const input = cost.input !== undefined ? `$${cost.input.toFixed(2)}` : '-'
  const output = cost.output !== undefined ? `$${cost.output.toFixed(2)}` : '-'
  return `${input} / ${output}`
}

function getInputModalities(model: ModelsDevModel): string[] {
  const modalities: string[] = []
  if (model.modalities.input.includes('text')) modalities.push('T')
  if (model.modalities.input.includes('image')) modalities.push('I')
  if (model.modalities.input.includes('audio')) modalities.push('A')
  return modalities
}
</script>

<style scoped>
.model-list {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: rgba(10, 15, 20, 0.95);
  border-radius: 12px;
  overflow: hidden;
}

/* Provider Header */
.provider-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px;
  background: rgba(0, 255, 255, 0.05);
  border-bottom: 1px solid rgba(0, 255, 255, 0.1);
}

.back-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border: 1px solid rgba(0, 255, 255, 0.3);
  border-radius: 8px;
  background: transparent;
  color: #00ffff;
  cursor: pointer;
  transition: all 0.2s ease;
}

.back-btn:hover {
  background: rgba(0, 255, 255, 0.1);
  border-color: #00ffff;
}

.back-icon {
  font-size: 18px;
}

.provider-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.provider-name {
  font-size: 18px;
  font-weight: 600;
  color: #ffffff;
}

.model-count-chip {
  display: inline-flex;
  align-items: center;
  padding: 2px 8px;
  font-size: 11px;
  font-weight: 500;
  color: #00ffff;
  background: rgba(0, 255, 255, 0.1);
  border: 1px solid rgba(0, 255, 255, 0.2);
  border-radius: 12px;
  width: fit-content;
}

/* Search */
.search-wrapper {
  padding: 12px 16px;
  background: rgba(10, 15, 20, 0.5);
}

.search-wrapper :deep(.v-field) {
  background: rgba(30, 40, 50, 0.8) !important;
  border: 1px solid rgba(0, 255, 255, 0.15);
  border-radius: 8px;
}

.search-wrapper :deep(.v-field__input) {
  color: #ffffff;
}

.search-wrapper :deep(.v-field__input::placeholder) {
  color: rgba(255, 255, 255, 0.4);
}

/* Models Container */
.models-container {
  flex: 1;
  overflow-y: auto;
  padding: 12px 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

/* Model Card */
.model-card {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 16px;
  background: rgba(20, 30, 40, 0.8);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.model-card:hover {
  background: rgba(30, 40, 55, 0.9);
  border-color: rgba(0, 255, 255, 0.3);
}

.model-card.selected {
  background: rgba(0, 255, 136, 0.08);
  border-color: rgba(0, 255, 136, 0.4);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.model-name {
  font-size: 15px;
  font-weight: 600;
  color: #ffffff;
}

.check-icon {
  color: #00ff88 !important;
}

/* Badges */
.badges {
  display: flex;
  gap: 8px;
}

.badge {
  padding: 3px 8px;
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.5px;
  border-radius: 4px;
}

.badge.reasoning {
  color: #00ffff;
  background: rgba(0, 255, 255, 0.1);
  border: 1px solid rgba(0, 255, 255, 0.3);
}

.badge.tools {
  color: #00ff88;
  background: rgba(0, 255, 136, 0.1);
  border: 1px solid rgba(0, 255, 136, 0.3);
}

/* Model Details */
.model-details {
  display: flex;
  gap: 24px;
}

.detail-row {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.detail-label {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.4);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.detail-value {
  font-size: 13px;
  color: #ffffff;
  font-weight: 500;
}

/* Modality Indicators */
.modality-indicators {
  display: flex;
  gap: 6px;
}

.modality-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  font-size: 11px;
  font-weight: 700;
  border-radius: 4px;
}

.modality-badge.t {
  color: #ffffff;
  background: rgba(255, 255, 255, 0.15);
}

.modality-badge.i {
  color: #ff6b9d;
  background: rgba(255, 107, 157, 0.15);
}

.modality-badge.a {
  color: #ffd93d;
  background: rgba(255, 217, 61, 0.15);
}

/* Empty State */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px 24px;
  text-align: center;
}

.empty-text {
  margin-top: 16px;
  font-size: 16px;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.7);
}

.empty-hint {
  margin-top: 4px;
  font-size: 13px;
  color: rgba(255, 255, 255, 0.4);
}

/* Scrollbar */
.models-container::-webkit-scrollbar {
  width: 6px;
}

.models-container::-webkit-scrollbar-track {
  background: transparent;
}

.models-container::-webkit-scrollbar-thumb {
  background: rgba(0, 255, 255, 0.2);
  border-radius: 3px;
}

.models-container::-webkit-scrollbar-thumb:hover {
  background: rgba(0, 255, 255, 0.3);
}
</style>
