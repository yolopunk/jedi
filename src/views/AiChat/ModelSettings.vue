<template>
  <v-dialog
    :model-value="modelValue"
    @update:model-value="$emit('update:modelValue', $event)"
    max-width="600"
  >
    <v-card class="scifi-card model-settings-dialog">
      <v-card-title class="console-title-bar">
        <span class="dialog-title">[ MODEL_SETTINGS ]</span>
        <v-spacer />
        <button class="console-btn icon-only" @click="$emit('update:modelValue', false)">
          <span class="btn-icon">✕</span>
        </button>
      </v-card-title>

      <v-card-text class="console-card-text settings-content">
        <!-- Providers List -->
        <div class="section-header">
          <span class="section-title">CONFIGURE PROVIDERS</span>
        </div>

        <div class="providers-list">
          <div
            v-for="p in providerList"
            :key="p.id"
            class="provider-item"
            :class="{ configured: isProviderConfigured(p.id) }"
          >
            <ProviderIcon :provider-id="p.id" class="provider-icon" />
            <div class="provider-info">
              <span class="provider-name">{{ p.name }}</span>
              <span class="provider-status">
                {{ isProviderConfigured(p.id) ? 'CONFIGURED' : 'NOT CONFIGURED' }}
              </span>
            </div>
            <button class="console-btn small" @click="openConfig(p.id)">
              {{ isProviderConfigured(p.id) ? 'EDIT' : 'CONFIGURE' }}
            </button>
          </div>
        </div>

        <!-- API Key Config Dialog -->
        <v-dialog v-model="showConfigDialog" max-width="400">
          <v-card class="scifi-card">
            <v-card-title class="console-title-bar">
              <span class="dialog-title">[ {{ currentProvider?.name }}_CONFIG ]</span>
            </v-card-title>
            <v-card-text class="console-card-text">
              <div class="form-field">
                <label class="field-label">API KEY</label>
                <div class="input-wrapper">
                  <span class="input-prompt">>></span>
                  <input
                    v-model="configApiKey"
                    :type="showKey ? 'text' : 'password'"
                    class="console-input"
                    placeholder="sk-..."
                  />
                  <button class="console-btn icon-only small" @click="showKey = !showKey">
                    <span class="btn-icon">{{ showKey ? '👁' : '👁‍🗨' }}</span>
                  </button>
                </div>
              </div>
              <div class="form-field mt-3">
                <label class="field-label">API ENDPOINT (OPTIONAL)</label>
                <div class="input-wrapper">
                  <span class="input-prompt">>></span>
                  <input
                    v-model="configEndpoint"
                    type="text"
                    class="console-input"
                    :placeholder="currentProvider?.api"
                  />
                </div>
              </div>
            </v-card-text>
            <v-card-actions class="console-card-actions">
              <button
                v-if="isProviderConfigured(currentProvider?.id)"
                class="console-btn danger"
                @click="deleteKey"
              >
                <span class="btn-text">DELETE</span>
              </button>
              <v-spacer />
              <button class="console-btn" @click="showConfigDialog = false">
                <span class="btn-text">CANCEL</span>
              </button>
              <button class="console-btn primary" @click="saveKey">
                <span class="btn-text">SAVE</span>
              </button>
            </v-card-actions>
          </v-card>
        </v-dialog>

      </v-card-text>

      <v-card-actions class="console-card-actions">
        <v-spacer />
        <button class="console-btn" @click="saveAndClose">
          <span class="btn-text">SAVE</span>
        </button>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useAiChatStore } from '@/stores/aiChat'
import ProviderIcon from '@/components/common/ProviderIcon.vue'

const props = defineProps<{ modelValue: boolean }>()
const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
}>()

const store = useAiChatStore()

const providerList = computed(() => store.getProvidersFromModelsDev())

const showConfigDialog = ref(false)
const currentProvider = ref<any>(null)
const configApiKey = ref('')
const configEndpoint = ref('')
const showKey = ref(false)

watch(() => props.modelValue, (open) => {
  if (open) {
    store.loadProviders()
    if (!store.modelsDevData) {
      store.fetchModelsDev()
    }
  }
})

function isProviderConfigured(id: string | undefined) {
  return store.providers.some(p => p.provider === id && p.has_key)
}

function openConfig(id: string) {
  const providers = store.getProvidersFromModelsDev()
  currentProvider.value = providers.find(p => p.id === id) || null
  configApiKey.value = ''
  configEndpoint.value = currentProvider.value?.api || ''
  showConfigDialog.value = true
}

async function saveKey() {
  if (!currentProvider.value) return
  await store.saveApiKey(
    currentProvider.value.id,
    configApiKey.value,
    configEndpoint.value || undefined
  )
  showConfigDialog.value = false
}

async function deleteKey() {
  if (!currentProvider.value) return
  await store.deleteApiKey(currentProvider.value.id)
  showConfigDialog.value = false
}

function saveAndClose() {
  emit('update:modelValue', false)
}
</script>

<style scoped>
.settings-content {
  padding: 20px !important;
}

.section-header {
  margin-bottom: 12px;
}

.section-title {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 2px;
  color: #00ff88;
}

.providers-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.provider-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid rgba(0, 255, 255, 0.1);
  border-radius: 6px;
}

.provider-item.configured {
  border-color: rgba(0, 255, 136, 0.3);
  background: rgba(0, 255, 136, 0.03);
}

.provider-icon {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 255, 255, 0.05);
  border-radius: 6px;
  font-size: 16px;
}

.provider-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.provider-name {
  font-size: 13px;
  font-weight: 600;
  color: #e4e4e7;
}

.provider-status {
  font-size: 10px;
  color: #52525b;
  letter-spacing: 1px;
}

.provider-item.configured .provider-status {
  color: #00ff88;
}

.form-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field-label {
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 1px;
  color: #00ffff;
}

</style>
