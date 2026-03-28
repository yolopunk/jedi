<template>
  <v-dialog v-model="dialogModel" max-width="600">
    <v-card class="scifi-card">
      <v-card-title class="console-title-bar">
        <span class="dialog-title">[ UPDATE_AVAILABLE ]</span>
      </v-card-title>
      <v-card-text class="console-card-text">
        <div class="alert-box info mb-4">
          <span class="alert-label">{{ $t('update.currentVersion') }}:</span>
          <span class="alert-value">{{ currentVersion }}</span>
        </div>

        <div class="alert-box success mb-4">
          <span class="alert-label">{{ $t('update.newVersion') }}:</span>
          <span class="alert-value">{{ updateInfo.version }}</span>
        </div>

        <div class="release-notes-section mb-4">
          <div class="section-title">&gt; {{ $t('update.releaseNotes') }}</div>
          <div class="release-notes-box">
            <div class="release-notes" v-html="formattedReleaseNotes"></div>
          </div>
        </div>

        <div v-if="isInstalling" class="progress-container">
          <div class="progress-bar">
            <div class="progress-indeterminate"></div>
          </div>
          <div class="progress-text">{{ $t('update.installing') }}...</div>
        </div>
      </v-card-text>
      <v-card-actions class="console-card-actions">
        <v-spacer></v-spacer>
        <button class="console-btn" @click="dialogModel = false" :disabled="isInstalling">
          <span class="btn-text">{{ $t('common.cancel') }}</span>
        </button>
        <button class="console-btn primary ml-2" @click="handleInstall" :disabled="isInstalling">
          <span class="btn-text" v-if="!isInstalling">{{ $t('update.downloadAndInstall') }}</span>
          <span class="btn-text" v-else>{{ $t('update.installing') }}...</span>
        </button>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { UpdateInfo } from '@/api/update'
import pkg from '../../../package.json'
import MarkdownIt from 'markdown-it'

const props = defineProps<{
  modelValue: boolean
  updateInfo: UpdateInfo
  isInstalling: boolean
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'install'): void
}>()

const currentVersion = pkg.version
const md = new MarkdownIt({
  html: true,
  linkify: true,
  breaks: true
})

const dialogModel = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

const formattedReleaseNotes = computed(() => {
  if (!props.updateInfo.body) return ''
  return md.render(props.updateInfo.body)
})

const handleInstall = () => {
  emit('install')
}
</script>

<style scoped>
.alert-box {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  border-radius: 4px;
  border: 1px solid;
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', monospace;
}

.alert-box.info {
  background: rgba(0, 170, 255, 0.05);
  border-color: rgba(0, 170, 255, 0.3);
}

.alert-box.success {
  background: rgba(0, 255, 136, 0.05);
  border-color: rgba(0, 255, 136, 0.3);
}

.alert-label {
  font-size: 11px;
  font-weight: 700;
  color: #52525b;
  letter-spacing: 1px;
}

.alert-value {
  font-size: 12px;
  font-weight: 600;
  color: #e4e4e7;
}

.alert-box.info .alert-value {
  color: #00aaff;
}

.alert-box.success .alert-value {
  color: #00ff88;
}

.release-notes-section {
  margin-bottom: 16px;
}

.section-title {
  font-size: 11px;
  font-weight: 700;
  color: #a1a1aa;
  letter-spacing: 1px;
  margin-bottom: 8px;
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', monospace;
}

.release-notes-box {
  background: rgba(5, 5, 8, 0.9);
  border: 1px solid #1a1a3a;
  border-radius: 4px;
  padding: 16px;
  max-height: 200px;
  overflow-y: auto;
}

.release-notes {
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', monospace;
  font-size: 11px;
  color: #71717a;
  line-height: 1.6;
}

.release-notes :deep(h1),
.release-notes :deep(h2),
.release-notes :deep(h3),
.release-notes :deep(h4) {
  margin-top: 12px;
  margin-bottom: 6px;
  font-weight: bold;
  color: #00ffff;
}

.release-notes :deep(h1) { font-size: 14px; }
.release-notes :deep(h2) { font-size: 13px; }
.release-notes :deep(h3) { font-size: 12px; }

.release-notes :deep(ul),
.release-notes :deep(ol) {
  padding-left: 20px;
  margin-bottom: 8px;
}

.release-notes :deep(p) {
  margin-bottom: 8px;
}

.release-notes :deep(a) {
  color: #00ff88;
  text-decoration: underline;
}

.release-notes :deep(code) {
  background: rgba(0, 255, 255, 0.1);
  padding: 2px 6px;
  border-radius: 3px;
  color: #00ffff;
}

.progress-container {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.progress-bar {
  height: 4px;
  background: rgba(0, 255, 255, 0.1);
  border-radius: 2px;
  overflow: hidden;
}

.progress-indeterminate {
  width: 50%;
  height: 100%;
  background: linear-gradient(90deg, transparent, #00ffff, transparent);
  animation: progress-slide 1.5s ease-in-out infinite;
}

@keyframes progress-slide {
  0% { transform: translateX(-100%); }
  100% { transform: translateX(300%); }
}

.progress-text {
  font-size: 10px;
  color: #52525b;
  text-align: center;
  font-family: 'JetBrains Mono', monospace;
  letter-spacing: 1px;
}

/* Custom scrollbar for release notes */
.release-notes-box::-webkit-scrollbar {
  width: 6px;
}

.release-notes-box::-webkit-scrollbar-track {
  background: transparent;
}

.release-notes-box::-webkit-scrollbar-thumb {
  background: rgba(0, 255, 255, 0.2);
  border-radius: 3px;
}

.release-notes-box::-webkit-scrollbar-thumb:hover {
  background: rgba(0, 255, 255, 0.3);
}
</style>
