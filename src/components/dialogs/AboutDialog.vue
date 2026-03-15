<template>
  <v-dialog v-model="dialogModel" max-width="500">
    <v-card class="scifi-card">
      <v-card-title class="console-title-bar">
        <span class="dialog-title">[ SYSTEM_INFO ]</span>
      </v-card-title>
      <v-card-text class="console-card-text text-center">
        <div class="about-logo mb-4">
          <img src="/icon.png" alt="Jedi Logo" class="app-logo" />
          <div class="logo-glow"></div>
        </div>
        <h2 class="about-title mb-1">{{ $t('about.appName') }}</h2>
        <p class="about-version mb-4">&gt; {{ $t('about.version', { version: appInfo.version }) }}</p>
        <p class="about-desc mb-2">{{ $t('about.description') }}</p>
        <p class="about-desc mb-4">{{ $t('about.techStack') }}</p>

        <div class="divider-line mb-4"></div>

        <div class="about-links d-flex justify-center mb-2">
          <button class="console-btn icon-only small" @click="$emit('open-github')" title="GitHub">
            <span class="btn-icon">⌘</span>
          </button>
          <button class="console-btn icon-only small ml-2" @click="$emit('open-website')" title="Website">
            <span class="btn-icon">⌗</span>
          </button>
          <button class="console-btn icon-only small ml-2" @click="$emit('open-email')" title="Email">
            <span class="btn-icon">@</span>
          </button>
        </div>

        <p class="about-copyright">{{ $t('about.copyright', { year: currentYear }) }}</p>
      </v-card-text>
      <v-card-actions class="console-card-actions">
        <v-spacer></v-spacer>
        <button class="console-btn primary" @click="dialogModel = false">
          <span class="btn-text">{{ $t('common.close') }}</span>
        </button>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'
import { getAppInfo, type AppInfo } from '@/api/app'

const currentYear = new Date().getFullYear()

const props = defineProps<{
  modelValue: boolean;
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
  (e: 'open-github'): void;
  (e: 'open-website'): void;
  (e: 'open-email'): void;
}>()

const dialogModel = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

const appInfo = ref<AppInfo>({
  version: '0.0.0',
  name: 'Jedi'
})

onMounted(async () => {
  try {
    appInfo.value = await getAppInfo()
  } catch (error) {
    console.error('Failed to get app info:', error)
  }
})
</script>

<style scoped>
.about-logo {
  position: relative;
  width: 80px;
  height: 80px;
  margin: 0 auto;
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
  background: radial-gradient(circle, rgba(0, 255, 255, 0.3) 0%, transparent 70%);
  animation: logoPulse 2s ease-in-out infinite;
  z-index: 0;
}

@keyframes logoPulse {
  0%, 100% { opacity: 0.5; transform: scale(1); }
  50% { opacity: 1; transform: scale(1.1); }
}

.about-title {
  font-size: 16px;
  font-weight: 700;
  color: #00ff88;
  text-shadow: 0 0 10px rgba(0, 255, 136, 0.5);
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', monospace;
  letter-spacing: 2px;
  margin: 0;
}

.about-version {
  font-size: 12px;
  color: #00ffff;
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', monospace;
  text-shadow: 0 0 8px rgba(0, 255, 255, 0.3);
  margin: 0;
}

.about-desc {
  font-size: 11px;
  color: #71717a;
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', monospace;
  line-height: 1.6;
  margin: 0;
}

.divider-line {
  height: 1px;
  background: linear-gradient(90deg, transparent, rgba(0, 255, 255, 0.3), transparent);
  margin: 16px 0;
}

.about-links {
  gap: 8px;
}

.about-copyright {
  font-size: 10px;
  color: #52525b;
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', monospace;
  letter-spacing: 1px;
  margin: 8px 0 0;
}
</style>
