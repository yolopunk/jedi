<template>
  <v-dialog
    :model-value="modelValue"
    @update:model-value="$emit('update:modelValue', $event)"
    max-width="400"
  >
    <v-card class="model-settings-card">
      <!-- Header -->
      <div class="card-header">
        <div class="header-brand">
          <div class="brand-icon">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
              <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="1.5"/>
              <circle cx="12" cy="12" r="4" fill="currentColor"/>
              <line x1="12" y1="2" x2="12" y2="6" stroke="currentColor" stroke-width="1.5"/>
              <line x1="12" y1="18" x2="12" y2="22" stroke="currentColor" stroke-width="1.5"/>
              <line x1="2" y1="12" x2="6" y2="12" stroke="currentColor" stroke-width="1.5"/>
              <line x1="18" y1="12" x2="22" y2="12" stroke="currentColor" stroke-width="1.5"/>
            </svg>
          </div>
          <div class="brand-text">
            <h2>About</h2>
            <p>Application info</p>
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
        <div class="about-logo">
          <img src="/icon.png" alt="Jedi Logo" class="app-logo" />
          <div class="logo-glow"></div>
        </div>

        <h3 class="app-name">{{ appInfo.name }}</h3>
        <p class="app-version">v{{ appInfo.version }}</p>
        <p class="app-desc">{{ $t('about.description') }}</p>

        <div class="divider"></div>

        <div class="about-links">
          <v-tooltip text="GitHub">
            <template v-slot:activator="{ props }">
              <v-btn v-bind="props" icon variant="text" size="small" @click="$emit('open-github')">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
                </svg>
              </v-btn>
            </template>
          </v-tooltip>
          <v-tooltip text="Website">
            <template v-slot:activator="{ props }">
              <v-btn v-bind="props" icon variant="text" size="small" @click="$emit('open-website')">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="12" cy="12" r="10"/>
                  <line x1="2" y1="12" x2="22" y2="12"/>
                  <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>
                </svg>
              </v-btn>
            </template>
          </v-tooltip>
          <v-tooltip text="Email">
            <template v-slot:activator="{ props }">
              <v-btn v-bind="props" icon variant="text" size="small" @click="$emit('open-email')">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <rect x="2" y="4" width="20" height="16" rx="2"/>
                  <path d="m22 7-8.97 5.7a1.94 1.94 0 0 1-2.06 0L2 7"/>
                </svg>
              </v-btn>
            </template>
          </v-tooltip>
        </div>

        <p class="copyright">&copy; {{ currentYear }} All rights reserved.</p>
      </div>

      <!-- Footer -->
      <div class="card-footer">
        <v-btn variant="tonal" color="primary" @click="$emit('update:modelValue', false)">
          Close
        </v-btn>
      </div>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { type AppInfo, getAppInfo } from '@/api/app'

defineProps<{ modelValue: boolean }>()
defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'open-github'): void
  (e: 'open-website'): void
  (e: 'open-email'): void
}>()

const currentYear = new Date().getFullYear()
const appInfo = ref<AppInfo>({ version: '0.0.0', name: 'Jedi' })

onMounted(async () => {
  try {
    appInfo.value = await getAppInfo()
  } catch (error) {
    console.error('Failed to get app info:', error)
  }
})
</script>

<style scoped>
.model-settings-card {
  background: var(--bg-terminal) !important;
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
  background: rgb(var(--bg-rgb) / 0.6);
  border-bottom: 1px solid rgb(var(--text-rgb) / 0.06);
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
  background: linear-gradient(135deg, rgb(var(--accent-rgb) / 0.15) 0%, rgb(var(--success-rgb) / 0.05) 100%);
  border: 1px solid rgb(var(--accent-rgb) / 0.25);
  border-radius: 10px;
  color: var(--accent);
}

.brand-text h2 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--text);
}

.brand-text p {
  margin: 2px 0 0;
  font-size: 12px;
  color: rgb(var(--text-rgb) / 0.4);
}

.close-btn {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: 1px solid rgb(var(--text-rgb) / 0.08);
  border-radius: 8px;
  color: rgb(var(--text-rgb) / 0.5);
  cursor: pointer;
  transition: all 0.2s ease;
}

.close-btn:hover {
  background: rgb(var(--danger-rgb) / 0.1);
  border-color: rgb(var(--danger-rgb) / 0.3);
  color: var(--danger);
}

.card-body {
  flex: 1;
  overflow-y: auto;
  padding: 28px 24px;
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
}

.about-logo {
  position: relative;
  width: 80px;
  height: 80px;
  margin-bottom: 16px;
}

.app-logo {
  width: 72px;
  height: 72px;
  object-fit: contain;
  border-radius: 50%;
  position: relative;
  z-index: 2;
}

.logo-glow {
  position: absolute;
  inset: -8px;
  border-radius: 50%;
  background: radial-gradient(circle, rgb(var(--accent-rgb) / 0.3) 0%, transparent 70%);
  animation: logoPulse 2s ease-in-out infinite;
  z-index: 0;
}

@keyframes logoPulse {
  0%, 100% { opacity: 0.5; transform: scale(1); }
  50% { opacity: 1; transform: scale(1.1); }
}

.app-name {
  margin: 0;
  font-size: 18px;
  font-weight: 700;
  color: var(--success);
  text-shadow: 0 0 16px rgb(var(--success-rgb) / 0.4);
  letter-spacing: 2px;
}

.app-version {
  margin: 4px 0 0;
  font-size: 13px;
  color: var(--accent);
  font-family: 'JetBrains Mono', monospace;
}

.app-desc {
  margin: 16px 0 0;
  font-size: 12px;
  color: rgb(var(--text-rgb) / 0.4);
  line-height: 1.6;
  max-width: 280px;
}

.divider {
  width: 100%;
  height: 1px;
  margin: 20px 0;
  background: linear-gradient(90deg, transparent, rgb(var(--accent-rgb) / 0.2), transparent);
}

.about-links {
  display: flex;
  gap: 8px;
}

.about-links :deep(.v-btn) {
  color: rgb(var(--text-rgb) / 0.5) !important;
}

.about-links :deep(.v-btn:hover) {
  color: var(--accent) !important;
  background: rgb(var(--accent-rgb) / 0.08) !important;
}

.copyright {
  margin: 16px 0 0;
  font-size: 11px;
  color: rgb(var(--text-rgb) / 0.25);
  letter-spacing: 0.5px;
}

.card-footer {
  display: flex;
  justify-content: center;
  padding: 16px 20px;
  border-top: 1px solid rgb(var(--text-rgb) / 0.06);
  flex-shrink: 0;
}

::-webkit-scrollbar {
  width: 4px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background:rgb(var(--ink-rgb) / 0.1);
  border-radius: 2px;
}

::-webkit-scrollbar-thumb:hover {
  background:rgb(var(--ink-rgb) / 0.15);
}
</style>
