<template>
  <v-dialog
    :model-value="modelValue"
    @update:model-value="$emit('update:modelValue', $event)"
    max-width="520"
  >
    <v-card class="model-settings-card">
      <!-- Header -->
      <div class="card-header">
        <div class="header-brand">
          <div class="brand-icon">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" stroke="currentColor" stroke-width="1.5"/>
              <polyline points="7 10 12 15 17 10" stroke="currentColor" stroke-width="1.5"/>
              <line x1="12" y1="15" x2="12" y2="3" stroke="currentColor" stroke-width="1.5"/>
            </svg>
          </div>
          <div class="brand-text">
            <h2>Update Available</h2>
            <p v-if="!isInstalling">New version ready to install</p>
            <p v-else>Downloading...</p>
          </div>
        </div>
        <button class="close-btn" @click="$emit('update:modelValue', false)">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
            <line x1="18" y1="6" x2="6" y2="18" stroke="currentColor" stroke-width="2"/>
            <line x1="6" y1="6" x2="18" y2="18" stroke="currentColor" stroke-width="2"/>
          </svg>
        </button>
      </div>

      <!-- Content -->
      <div class="card-body">
        <div class="version-info">
          <div class="version-row">
            <span class="version-label">Current</span>
            <span class="version-value current">v{{ currentVersion }}</span>
          </div>
          <div class="version-arrow">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
              <line x1="5" y1="12" x2="19" y2="12" stroke="currentColor" stroke-width="2"/>
              <polyline points="12 5 19 12 12 19" stroke="currentColor" stroke-width="2"/>
            </svg>
          </div>
          <div class="version-row">
            <span class="version-label">New</span>
            <span class="version-value new">v{{ updateInfo.version }}</span>
          </div>
        </div>

        <div class="release-notes-section">
          <div class="section-title">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" stroke="currentColor" stroke-width="1.5"/>
              <polyline points="14 2 14 8 20 8" stroke="currentColor" stroke-width="1.5"/>
            </svg>
            Release Notes
          </div>
          <div class="release-notes-box">
            <div class="release-notes" v-html="formattedReleaseNotes"></div>
          </div>
        </div>

        <div v-if="isInstalling" class="install-progress">
          <div class="progress-bar">
            <div class="progress-fill"></div>
          </div>
          <span class="progress-text">Installing update...</span>
        </div>
      </div>

      <!-- Footer -->
      <div class="card-footer">
        <v-btn variant="text" @click="$emit('update:modelValue', false)" :disabled="isInstalling">
          Cancel
        </v-btn>
        <v-spacer />
        <v-btn variant="tonal" color="primary" @click="$emit('install')" :loading="isInstalling">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
            <polyline points="7 10 12 15 17 10"/>
            <line x1="12" y1="15" x2="12" y2="3"/>
          </svg>
          Download & Install
        </v-btn>
      </div>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import MarkdownIt from 'markdown-it'
import { computed } from 'vue'
import type { UpdateInfo } from '@/api/update'
import pkg from '../../../package.json'

const props = defineProps<{
  modelValue: boolean
  updateInfo: UpdateInfo
  isInstalling: boolean
}>()

defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'install'): void
}>()

const _currentVersion = pkg.version
const md = new MarkdownIt({ html: true, linkify: true, breaks: true })

const _formattedReleaseNotes = computed(() => {
  if (!props.updateInfo.body) return ''
  return md.render(props.updateInfo.body)
})
</script>

<style scoped>
.model-settings-card {
  background: #0a0e14 !important;
  border-radius: 16px !important;
  overflow: hidden;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: rgba(20, 30, 40, 0.6);
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  flex-shrink: 0;
}

.header-brand {
  display: flex;
  align-items: center;
  gap: 12px;
}

.brand-icon {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, rgba(0, 255, 136, 0.15) 0%, rgba(0, 255, 136, 0.05) 100%);
  border: 1px solid rgba(0, 255, 136, 0.25);
  border-radius: 10px;
  color: #00ff88;
}

.brand-text h2 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: #ffffff;
}

.brand-text p {
  margin: 2px 0 0;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.4);
}

.close-btn {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  color: rgba(255, 255, 255, 0.5);
  cursor: pointer;
  transition: all 0.2s ease;
}

.close-btn:hover {
  background: rgba(255, 107, 107, 0.1);
  border-color: rgba(255, 107, 107, 0.3);
  color: #ff6b6b;
}

.card-body {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.version-info {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
  padding: 16px;
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 12px;
}

.version-row {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.version-label {
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 1px;
  color: rgba(255, 255, 255, 0.35);
  text-transform: uppercase;
}

.version-value {
  font-family: 'JetBrains Mono', monospace;
  font-size: 18px;
  font-weight: 700;
}

.version-value.current {
  color: rgba(255, 255, 255, 0.5);
}

.version-value.new {
  color: #00ff88;
  text-shadow: 0 0 12px rgba(0, 255, 136, 0.4);
}

.version-arrow {
  color: rgba(255, 255, 255, 0.2);
}

.release-notes-section {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.section-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 1px;
  color: rgba(255, 255, 255, 0.5);
  text-transform: uppercase;
}

.section-title svg {
  color: #00ffff;
}

.release-notes-box {
  background: rgba(0, 255, 255, 0.02);
  border: 1px solid rgba(0, 255, 255, 0.08);
  border-radius: 10px;
  padding: 14px;
  max-height: 200px;
  overflow-y: auto;
}

.release-notes {
  font-family: 'JetBrains Mono', monospace;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.6);
  line-height: 1.6;
}

.release-notes :deep(h1),
.release-notes :deep(h2),
.release-notes :deep(h3) {
  margin: 8px 0 4px;
  font-weight: 700;
  color: #00ffff;
}

.release-notes :deep(h1) { font-size: 14px; }
.release-notes :deep(h2) { font-size: 13px; }
.release-notes :deep(h3) { font-size: 12px; }

.release-notes :deep(ul),
.release-notes :deep(ol) {
  padding-left: 18px;
  margin: 6px 0;
}

.release-notes :deep(p) {
  margin: 6px 0;
}

.release-notes :deep(code) {
  background: rgba(0, 255, 255, 0.08);
  padding: 2px 6px;
  border-radius: 4px;
  color: #00ffff;
}

.release-notes :deep(a) {
  color: #00ff88;
}

.install-progress {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.progress-bar {
  height: 4px;
  background: rgba(0, 255, 136, 0.1);
  border-radius: 2px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  width: 30%;
  background: linear-gradient(90deg, #00ffff, #00ff88);
  border-radius: 2px;
  animation: progress-slide 1.5s ease-in-out infinite;
}

@keyframes progress-slide {
  0% { transform: translateX(-100%); }
  100% { transform: translateX(400%); }
}

.progress-text {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.4);
  text-align: center;
  letter-spacing: 0.5px;
}

.card-footer {
  display: flex;
  align-items: center;
  padding: 16px 20px;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  flex-shrink: 0;
}

::-webkit-scrollbar {
  width: 4px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 2px;
}

::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.15);
}
</style>
