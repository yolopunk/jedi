<template>
  <div class="custom-provider-form">
    <div class="form-field">
      <label class="field-label">PROVIDER NAME</label>
      <div class="input-wrapper">
        <span class="input-prompt">>></span>
        <input
          v-model="formData.name"
          type="text"
          class="console-input"
          placeholder="My Custom Provider"
        />
      </div>
    </div>

    <div class="form-field">
      <label class="field-label">BASE URL</label>
      <div class="input-wrapper">
        <span class="input-prompt">>></span>
        <input
          v-model="formData.baseUrl"
          type="text"
          class="console-input"
          placeholder="https://api.example.com/v1"
        />
      </div>
    </div>

    <div class="form-field">
      <label class="field-label">API KEY</label>
      <div class="input-wrapper">
        <span class="input-prompt">>></span>
        <input
          v-model="formData.apiKey"
          :type="showApiKey ? 'text' : 'password'"
          class="console-input"
          placeholder="sk-..."
        />
        <button class="console-btn icon-only small toggle-btn" @click="showApiKey = !showApiKey">
          <span class="btn-icon">{{ showApiKey ? '👁' : '👁‍🗨' }}</span>
        </button>
      </div>
    </div>

    <div class="form-field">
      <label class="field-label">MODEL ID <span class="optional">(OPTIONAL)</span></label>
      <div class="input-wrapper">
        <span class="input-prompt">>></span>
        <input
          v-model="formData.modelId"
          type="text"
          class="console-input"
          placeholder="gpt-4o-mini"
        />
      </div>
    </div>

    <div class="form-actions">
      <button class="console-btn" @click="handleCancel">
        <span class="btn-text">CANCEL</span>
      </button>
      <button
        class="console-btn primary"
        :disabled="!isValid"
        @click="handleSave"
      >
        <span class="btn-text">SAVE</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

const emit = defineEmits<{
  (e: 'save', data: { name: string; baseUrl: string; apiKey: string; modelId?: string }): void
  (e: 'cancel'): void
}>()

const formData = ref({
  name: '',
  baseUrl: '',
  apiKey: '',
  modelId: ''
})

const showApiKey = ref(false)

const isValid = computed(() => {
  return formData.value.name.trim() !== '' &&
         formData.value.baseUrl.trim() !== '' &&
         formData.value.apiKey.trim() !== ''
})

function handleSave() {
  if (!isValid.value) return

  emit('save', {
    name: formData.value.name.trim(),
    baseUrl: formData.value.baseUrl.trim(),
    apiKey: formData.value.apiKey.trim(),
    modelId: formData.value.modelId.trim() || undefined
  })
}

function handleCancel() {
  emit('cancel')
}
</script>

<style scoped>
.custom-provider-form {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.form-field {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.field-label {
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 1px;
  color: #00ffff;
}

.field-label .optional {
  font-weight: 400;
  color: rgba(255, 255, 255, 0.4);
  font-size: 9px;
  letter-spacing: 0;
}

.input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
  background: rgba(5, 5, 8, 0.9);
  border: 1px solid #1a1a3a;
  border-radius: 4px;
  padding: 8px 12px;
}

.input-prompt {
  color: rgba(0, 255, 255, 0.5);
  font-size: 12px;
  margin-right: 8px;
  user-select: none;
}

.console-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  color: #00ffff;
  font-family: 'JetBrains Mono', monospace;
  font-size: 12px;
}

.console-input::placeholder {
  color: #52525b;
}

.toggle-btn {
  margin-left: 8px;
  padding: 4px 8px;
}

.btn-icon {
  font-size: 14px;
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 8px;
}

/* Console button styles */
.console-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  background: transparent;
  border: 1px solid #00ff88;
  color: #00ff88;
  padding: 6px 12px;
  border-radius: 4px;
  font-family: inherit;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 1px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.console-btn:hover:not(:disabled) {
  background: rgba(0, 255, 136, 0.07);
  box-shadow: 0 0 15px rgba(0, 255, 136, 0.27), inset 0 0 15px rgba(0, 255, 136, 0.07);
}

.console-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.console-btn.primary {
  border-color: #00ffff;
  color: #00ffff;
}

.console-btn.primary:hover:not(:disabled) {
  background: rgba(0, 255, 255, 0.07);
  box-shadow: 0 0 15px rgba(0, 255, 255, 0.27), inset 0 0 15px rgba(0, 255, 255, 0.07);
}

.console-btn.small {
  padding: 4px 10px;
  font-size: 9px;
}

.console-btn.icon-only {
  padding: 4px 8px;
}
</style>
