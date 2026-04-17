<template>
  <v-dialog v-model="dialogModel" max-width="1200">
    <v-card class="scifi-card models-browser-dialog">
      <v-card-title class="console-title-bar">
        <span class="dialog-title">[ MODEL_HUB ]</span>
        <v-spacer></v-spacer>
        <button class="console-btn icon-only" @click="dialogModel = false">
          <span class="btn-icon">✕</span>
        </button>
      </v-card-title>
      <v-card-text class="console-card-text models-browser-content">
        <div class="main-content d-flex flex-column overflow-hidden">
          <!-- Top Bar / Header -->
          <div class="console-header-bar">
            <div class="header-row">
              <div class="header-left">
                <div class="status-indicators">
                  <div class="status-light" :class="loading ? 'standby' : 'online'"></div>
                </div>
                <div class="header-metrics">
                  <span class="metric-item">
                    <span class="metric-label">PROVIDERS:</span>
                    <span class="metric-value">{{ providers.length }}</span>
                  </span>
                  <span class="metric-item">
                    <span class="metric-label">MODELS:</span>
                    <span class="metric-value">{{ totalModelCount }}</span>
                  </span>
                </div>
              </div>

              <div class="header-right">
                <div class="input-wrapper">
                  <span class="input-prompt">>></span>
                  <input
                    v-model="searchQuery"
                    type="text"
                    class="console-input"
                    placeholder="Search models..."
                  />
                </div>
                <button class="console-btn icon-only" @click="refreshData" :disabled="loading" title="Refresh">
                  <span class="btn-icon" :class="{ spinning: loading }">↻</span>
                </button>
              </div>
            </div>
          </div>

          <!-- Content Area -->
          <div class="main-content-area">
            <div class="browser-layout">
              <!-- Left: Provider List -->
              <div class="providers-panel">
                <div class="panel-header">
                  <span class="panel-title">PROVIDERS</span>
                </div>
                <div class="providers-list">
                  <div
                    v-for="provider in providers"
                    :key="provider.id"
                    class="provider-item"
                    :class="{ active: selectedProviderId === provider.id }"
                    @click="selectProvider(provider)"
                  >
                    <div class="provider-icon">
                      <v-icon :icon="getProviderIcon(provider.id)" size="24" />
                    </div>
                    <div class="provider-info">
                      <div class="provider-name">{{ provider.name }}</div>
                      <div class="provider-meta">{{ provider.model_count }} models</div>
                    </div>
                    <div class="provider-badge" v-if="provider.id === 'openai'">
                      OFFICIAL
                    </div>
                  </div>
                </div>
              </div>

              <!-- Right: Model List -->
              <div class="models-panel">
                <div class="panel-header">
                  <span class="panel-title">
                    {{ selectedProvider?.name || 'Select a provider' }}
                  </span>
                  <span v-if="selectedProvider" class="panel-count">
                    {{ filteredModels.length }} models
                  </span>
                </div>

                <div v-if="!selectedProvider" class="empty-state">
                  <div class="empty-icon">🤖</div>
                  <div class="empty-text">Select a provider to view models</div>
                </div>

                <div v-else class="models-list">
                  <div
                    v-for="model in filteredModels"
                    :key="model.id"
                    class="model-card"
                  >
                    <div class="model-header">
                      <div class="model-name">{{ model.name }}</div>
                      <div class="model-id">{{ model.id }}</div>
                    </div>

                    <div class="model-tags">
                      <span v-if="model.reasoning" class="tag reasoning">🧠 Reasoning</span>
                      <span v-if="model.tool_call" class="tag tool">🔧 Tools</span>
                      <span v-if="model.attachment" class="tag attachment">📎 Attachments</span>
                      <span v-if="model.open_weights" class="tag open">📖 Open Weights</span>
                    </div>

                    <div class="model-details">
                      <div v-if="model.limit?.context" class="detail-item">
                        <span class="detail-label">Context:</span>
                        <span class="detail-value">{{ formatContextLength(model.limit.context) }}</span>
                      </div>
                      <div v-if="model.cost" class="detail-item">
                        <span class="detail-label">Input:</span>
                        <span class="detail-value">${{ model.cost.input?.toFixed(4) || '-' }}/M</span>
                      </div>
                      <div v-if="model.cost" class="detail-item">
                        <span class="detail-label">Output:</span>
                        <span class="detail-value">${{ model.cost.output?.toFixed(4) || '-' }}/M</span>
                      </div>
                      <div v-if="model.release_date" class="detail-item">
                        <span class="detail-label">Released:</span>
                        <span class="detail-value">{{ model.release_date }}</span>
                      </div>
                    </div>

                    <div class="model-modalities">
                      <span class="modality-label">Input:</span>
                      <span v-for="mod in model.modalities.input" :key="mod" class="modality-tag">
                        {{ mod }}
                      </span>
                      <span class="modality-label output">→ Output:</span>
                      <span v-for="mod in model.modalities.output" :key="'out-' + mod" class="modality-tag">
                        {{ mod }}
                      </span>
                    </div>

                    <div class="model-actions">
                      <button class="console-btn small" @click="selectModel(model)">
                        <span class="btn-icon">✓</span>
                        <span class="btn-text">SELECT</span>
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </v-card-text>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import {
  mdiBrain,
  mdiChartBar,
  mdiCloud,
  mdiCodeTags,
  mdiCpu64Bit,
  mdiGoogle,
  mdiHelpCircle,
  mdiImage,
  mdiLan,
  mdiLightningBolt,
  mdiRobot,
  mdiStar,
  mdiWave,
} from '@mdi/js'
import { computed, ref, watch } from 'vue'
import {
  getModelsForProvider,
  getModelsProviders,
  type ModelsDevModel,
  type ProviderSummary,
} from '@/api/ai-chat'

const props = defineProps<{
  modelValue: boolean
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'select', model: ModelsDevModel, provider: string): void
}>()

const dialogModel = computed({
  get: () => props.modelValue,
  set: value => emit('update:modelValue', value),
})

// State
const loading = ref(false)
const providers = ref<ProviderSummary[]>([])
const selectedProviderId = ref<string | null>(null)
const selectedProviderModels = ref<ModelsDevModel[]>([])
const searchQuery = ref('')

// Computed
const selectedProvider = computed(() =>
  providers.value.find(p => p.id === selectedProviderId.value)
)

const totalModelCount = computed(() => providers.value.reduce((sum, p) => sum + p.model_count, 0))

const filteredModels = computed(() => {
  if (!searchQuery.value) {
    return selectedProviderModels.value
  }
  const query = searchQuery.value.toLowerCase()
  return selectedProviderModels.value.filter(
    m =>
      m.name.toLowerCase().includes(query) ||
      m.id.toLowerCase().includes(query) ||
      m.family?.toLowerCase().includes(query)
  )
})

// Methods
function getProviderIcon(providerId: string): any {
  const iconMap: Record<string, any> = {
    openai: mdiRobot,
    anthropic: mdiBrain,
    google: mdiGoogle,
    'ollama-cloud': mdiWave,
    cohere: mdiWave,
    'cloudflare-ai-gateway': mdiCloud,
    inference: mdiCpu64Bit,
    'io-net': mdiLan,
    drun: mdiStar,
    moark: mdiStar,
    bailing: mdiLightningBolt,
    'minimax-coding-plan': mdiCodeTags,
    wandb: mdiChartBar,
    'qiniu-ai': mdiImage,
  }
  return iconMap[providerId] || mdiHelpCircle
}

function formatContextLength(bytes: number): string {
  if (bytes >= 1000000) {
    return (bytes / 1000000).toFixed(1) + 'M'
  } else if (bytes >= 1000) {
    return (bytes / 1000).toFixed(1) + 'K'
  }
  return bytes.toString()
}

async function refreshData() {
  loading.value = true
  try {
    providers.value = await getModelsProviders()
  } catch (e) {
    console.error('Failed to load providers:', e)
  } finally {
    loading.value = false
  }
}

async function selectProvider(provider: ProviderSummary) {
  selectedProviderId.value = provider.id
  loading.value = true
  try {
    selectedProviderModels.value = await getModelsForProvider(provider.id)
  } catch (e) {
    console.error('Failed to load models:', e)
    selectedProviderModels.value = []
  } finally {
    loading.value = false
  }
}

function selectModel(model: ModelsDevModel) {
  if (selectedProviderId.value) {
    emit('select', model, selectedProviderId.value)
  }
  dialogModel.value = false
}

watch(dialogModel, newVal => {
  if (newVal) {
    refreshData()
  } else {
    selectedProviderId.value = null
    selectedProviderModels.value = []
    searchQuery.value = ''
  }
})
</script>

<style scoped>
.models-browser-dialog {
  max-height: 80vh;
}

.models-browser-content {
  padding: 0 !important;
  max-height: 70vh;
  overflow: hidden;
}

.main-content {
  background: rgba(24, 24, 27, 0.95);
  display: flex;
  flex-direction: column;
  height: 100%;
}

.console-header-bar {
  padding: 12px 20px;
  border-bottom: 1px solid rgba(0, 255, 255, 0.2);
  background: rgba(0, 255, 255, 0.05);
}

.header-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.status-indicators {
  display: flex;
  gap: 4px;
}

.status-light {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #00ff88;
  box-shadow: 0 0 8px #00ff88;
}

.status-light.standby {
  background: #ffaa00;
  box-shadow: 0 0 8px #ffaa00;
  animation: pulse 1s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.header-metrics {
  display: flex;
  gap: 16px;
}

.metric-item {
  font-family: 'Courier New', monospace;
  font-size: 0.75rem;
  color: #888;
}

.metric-label {
  color: #555;
}

.metric-value {
  color: #00ff88;
  font-weight: 600;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.input-wrapper {
  display: flex;
  align-items: center;
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid rgba(0, 255, 255, 0.3);
  border-radius: 6px;
  padding: 6px 10px;
  gap: 6px;
}

.input-prompt {
  color: #00ff88;
  font-family: 'Courier New', monospace;
  font-size: 0.875rem;
}

.console-input {
  background: transparent;
  border: none;
  outline: none;
  color: #fff;
  font-family: 'Courier New', monospace;
  font-size: 0.875rem;
  width: 180px;
}

.console-input::placeholder {
  color: #555;
}

.console-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  background: transparent;
  border: 1px solid rgba(0, 255, 255, 0.5);
  border-radius: 6px;
  color: #00ffff;
  font-family: 'Courier New', monospace;
  font-size: 0.75rem;
  cursor: pointer;
  transition: all 0.2s;
}

.console-btn:hover {
  background: rgba(0, 255, 255, 0.1);
  border-color: #00ffff;
}

.console-btn.icon-only {
  padding: 8px;
}

.console-btn.small {
  padding: 6px 10px;
  font-size: 0.6875rem;
}

.console-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-icon.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.main-content-area {
  flex: 1;
  overflow: hidden;
  min-height: 500px;
}

.browser-layout {
  display: flex;
  height: 100%;
  min-height: 500px;
}

.providers-panel {
  width: 250px;
  border-right: 1px solid rgba(0, 255, 255, 0.2);
  display: flex;
  flex-direction: column;
}

.models-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.panel-header {
  padding: 12px 16px;
  border-bottom: 1px solid rgba(0, 255, 255, 0.2);
  background: rgba(0, 255, 255, 0.03);
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.panel-title {
  font-family: 'Courier New', monospace;
  font-size: 0.75rem;
  font-weight: 600;
  color: #00ffff;
  text-transform: uppercase;
  letter-spacing: 0.1em;
}

.panel-count {
  font-family: 'Courier New', monospace;
  font-size: 0.6875rem;
  color: #666;
}

.providers-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.provider-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
  border: 1px solid transparent;
}

.provider-item:hover {
  background: rgba(0, 255, 255, 0.05);
}

.provider-item.active {
  background: rgba(0, 255, 255, 0.1);
  border-color: rgba(0, 255, 255, 0.5);
}

.provider-icon {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 255, 255, 0.1);
  border-radius: 6px;
  color: #00ffff;
}

.provider-info {
  flex: 1;
  min-width: 0;
}

.provider-name {
  font-size: 0.875rem;
  font-weight: 500;
  color: #fff;
}

.provider-meta {
  font-size: 0.6875rem;
  color: #666;
  font-family: 'Courier New', monospace;
}

.provider-badge {
  font-size: 0.5625rem;
  padding: 2px 6px;
  background: rgba(0, 255, 136, 0.2);
  border: 1px solid rgba(0, 255, 136, 0.5);
  border-radius: 4px;
  color: #00ff88;
  font-family: 'Courier New', monospace;
}

.models-list {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 12px;
  align-content: start;
}

.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  color: #555;
}

.empty-icon {
  font-size: 3rem;
}

.empty-text {
  font-family: 'Courier New', monospace;
  font-size: 0.875rem;
}

.model-card {
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid rgba(0, 255, 255, 0.2);
  border-radius: 8px;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  transition: all 0.2s;
}

.model-card:hover {
  border-color: rgba(0, 255, 255, 0.5);
  background: rgba(0, 255, 255, 0.05);
}

.model-header {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.model-name {
  font-size: 1rem;
  font-weight: 600;
  color: #fff;
}

.model-id {
  font-size: 0.75rem;
  color: #666;
  font-family: 'Courier New', monospace;
}

.model-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.tag {
  font-size: 0.625rem;
  padding: 3px 6px;
  border-radius: 4px;
  font-family: 'Courier New', monospace;
}

.tag.reasoning {
  background: rgba(255, 170, 0, 0.2);
  border: 1px solid rgba(255, 170, 0, 0.5);
  color: #ffaa00;
}

.tag.tool {
  background: rgba(0, 255, 136, 0.2);
  border: 1px solid rgba(0, 255, 136, 0.5);
  color: #00ff88;
}

.tag.attachment {
  background: rgba(0, 255, 255, 0.2);
  border: 1px solid rgba(0, 255, 255, 0.5);
  color: #00ffff;
}

.tag.open {
  background: rgba(255, 0, 255, 0.2);
  border: 1px solid rgba(255, 0, 255, 0.5);
  color: #ff00ff;
}

.model-details {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  padding: 6px 0;
  border-top: 1px solid rgba(0, 255, 255, 0.1);
  border-bottom: 1px solid rgba(0, 255, 255, 0.1);
}

.detail-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.detail-label {
  font-size: 0.625rem;
  color: #555;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.detail-value {
  font-size: 0.75rem;
  color: #00ffff;
  font-family: 'Courier New', monospace;
}

.model-modalities {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  align-items: center;
}

.modality-label {
  font-size: 0.6875rem;
  color: #555;
  font-family: 'Courier New', monospace;
}

.modality-label.output {
  margin-left: 4px;
}

.modality-tag {
  font-size: 0.625rem;
  padding: 2px 6px;
  background: rgba(0, 255, 136, 0.15);
  border: 1px solid rgba(0, 255, 136, 0.3);
  border-radius: 4px;
  color: #00ff88;
  font-family: 'Courier New', monospace;
}

.model-actions {
  display: flex;
  gap: 8px;
  margin-top: 4px;
}

/* Scrollbar styling */
.providers-list::-webkit-scrollbar,
.models-list::-webkit-scrollbar {
  width: 6px;
}

.providers-list::-webkit-scrollbar-track,
.models-list::-webkit-scrollbar-track {
  background: transparent;
}

.providers-list::-webkit-scrollbar-thumb,
.models-list::-webkit-scrollbar-thumb {
  background: rgba(0, 255, 255, 0.2);
  border-radius: 3px;
}
</style>
