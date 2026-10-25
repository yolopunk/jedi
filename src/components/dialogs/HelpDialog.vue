<template>
  <v-dialog
    :model-value="modelValue"
    @update:model-value="$emit('update:modelValue', $event)"
    max-width="480"
  >
    <v-card class="model-settings-card">
      <!-- Header -->
      <div class="card-header">
        <div class="header-brand">
          <div class="brand-icon">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
              <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="1.5"/>
              <path d="M9 9a3 3 0 1 1 4 2.83V14" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
              <circle cx="12" cy="17" r="1" fill="currentColor"/>
            </svg>
          </div>
          <div class="brand-text">
            <h2>{{ $t('dialogs.help.title') }}</h2>
            <p>{{ $t('dialogs.help.subtitle') }}</p>
          </div>
        </div>
        <button class="close-btn" type="button" :aria-label="$t('common.close')" @click="$emit('update:modelValue', false)">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
            <line x1="18" y1="6" x2="6" y2="18" stroke="currentColor" stroke-width="2"/>
            <line x1="6" y1="6" x2="18" y2="18" stroke="currentColor" stroke-width="2"/>
          </svg>
        </button>
      </div>

      <!-- Content -->
      <div class="card-body">
        <!-- Current page tips -->
        <section class="help-section">
          <h3 class="section-title">{{ pageTitle }}</h3>
          <div class="help-item" v-for="(text, index) in pageItems" :key="`page-${index}`">
            <div class="help-icon">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
                <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="1.5"/>
                <line x1="12" y1="8" x2="12" y2="12" stroke="currentColor" stroke-width="2"/>
                <circle cx="12" cy="16" r="1" fill="currentColor"/>
              </svg>
            </div>
            <span class="help-text">{{ text }}</span>
          </div>
        </section>

        <!-- Global actions -->
        <section class="help-section">
          <h3 class="section-title">{{ $t('dialogs.help.globalSection') }}</h3>
          <div class="help-item" v-for="(text, index) in globalItems" :key="`global-${index}`">
            <div class="help-icon">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
                <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="1.5"/>
                <line x1="12" y1="8" x2="12" y2="12" stroke="currentColor" stroke-width="2"/>
                <circle cx="12" cy="16" r="1" fill="currentColor"/>
              </svg>
            </div>
            <span class="help-text">{{ text }}</span>
          </div>
        </section>

        <!-- Feature catalog -->
        <section class="help-section">
          <h3 class="section-title">{{ $t('dialogs.help.catalogSection') }}</h3>
          <div class="help-item catalog-item" v-for="row in catalogRows" :key="row.name">
            <div class="help-icon">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
                <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="1.5"/>
                <line x1="12" y1="8" x2="12" y2="12" stroke="currentColor" stroke-width="2"/>
                <circle cx="12" cy="16" r="1" fill="currentColor"/>
              </svg>
            </div>
            <div class="catalog-body">
              <span class="catalog-name">{{ row.name }}</span>
              <span class="help-text">{{ row.desc }}</span>
            </div>
          </div>
        </section>
      </div>

      <!-- Footer -->
      <div class="card-footer">
        <v-btn variant="tonal" color="primary" @click="$emit('update:modelValue', false)">
          {{ $t('dialogs.help.gotIt') }}
        </v-btn>
      </div>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute } from 'vue-router'

defineProps<{ modelValue: boolean }>()
defineEmits<(e: 'update:modelValue', value: boolean) => void>()

const { t } = useI18n()
const route = useRoute()

type HelpPageKey = 'chat' | 'skills' | 'hosts' | 'wallpapers' | 'podcast' | 'screenshot'

const pageConfig: Record<HelpPageKey, { nameKey: string; tipKeys: string[] }> = {
  chat: {
    nameKey: 'sidebar.chat',
    tipKeys: ['dialogs.help.chatTip1', 'dialogs.help.chatTip2', 'dialogs.help.chatTip3'],
  },
  skills: {
    nameKey: 'sidebar.skills',
    tipKeys: ['dialogs.help.skillsTip1', 'dialogs.help.skillsTip2'],
  },
  hosts: {
    nameKey: 'sidebar.hostsManager',
    tipKeys: [
      'dialogs.help.itemGroup',
      'dialogs.help.itemEntry',
      'dialogs.help.itemToggle',
      'dialogs.help.itemGlobal',
    ],
  },
  wallpapers: {
    nameKey: 'sidebar.wallpapers',
    tipKeys: ['dialogs.help.wallpapersTip1', 'dialogs.help.wallpapersTip2'],
  },
  podcast: {
    nameKey: 'sidebar.podcast',
    tipKeys: ['dialogs.help.podcastTip1', 'dialogs.help.podcastTip2'],
  },
  screenshot: {
    nameKey: 'sidebar.screenshot',
    tipKeys: ['dialogs.help.screenshotTip1', 'dialogs.help.screenshotTip2'],
  },
}

const currentPage = computed<HelpPageKey>(() => {
  const path = route.path
  if (path.startsWith('/chat')) return 'chat'
  if (path.startsWith('/skills')) return 'skills'
  if (path.startsWith('/hosts')) return 'hosts'
  if (path.startsWith('/wallpapers')) return 'wallpapers'
  if (path.startsWith('/podcast')) return 'podcast'
  if (path.startsWith('/screenshot')) return 'screenshot'
  return 'chat'
})

const currentConfig = computed(() => pageConfig[currentPage.value])

const pageTitle = computed(
  () => `${t('dialogs.help.pageSection')} · ${t(currentConfig.value.nameKey)}`
)

const pageItems = computed(() => currentConfig.value.tipKeys.map(key => t(key)))

const globalItems = computed(() => [t('dialogs.help.paletteTip'), t('dialogs.help.settingsTip')])

const catalogRows = computed(() => [
  { name: t('sidebar.chat'), desc: t('dialogs.help.catalogChat') },
  { name: t('sidebar.skills'), desc: t('dialogs.help.catalogSkills') },
  { name: t('sidebar.hostsManager'), desc: t('dialogs.help.catalogHosts') },
  { name: t('sidebar.wallpapers'), desc: t('dialogs.help.catalogWallpapers') },
  { name: t('sidebar.podcast'), desc: t('dialogs.help.catalogPodcast') },
  { name: t('sidebar.screenshot'), desc: t('dialogs.help.catalogScreenshot') },
])
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
  padding: 20px;
}

.help-section {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.help-item {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 12px 14px;
  background:rgb(var(--ink-rgb) / 0.02);
  border: 1px solid rgb(var(--text-rgb) / 0.06);
  border-radius: 10px;
  transition: all 0.15s ease;
}

.help-item:hover {
  background: rgb(var(--accent-rgb) / 0.04);
  border-color: rgb(var(--accent-rgb) / 0.15);
}

.help-icon {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgb(var(--accent-rgb) / 0.08);
  border-radius: 8px;
  color: var(--accent);
  flex-shrink: 0;
}

.help-text {
  font-size: 13px;
  color: rgb(var(--text-rgb) / 0.75);
  line-height: 1.4;
}

.help-section + .help-section {
  margin-top: 18px;
}

.section-title {
  margin: 0;
  font-size: 12px;
  font-weight: 600;
  color: rgb(var(--text-rgb) / 0.4);
  letter-spacing: 0.02em;
}

.catalog-item {
  gap: 10px;
  padding: 8px 10px;
}

.catalog-item .help-icon {
  width: 26px;
  height: 26px;
}

.catalog-item .help-text {
  font-size: 12px;
  color: rgb(var(--text-rgb) / 0.55);
}

.catalog-body {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.catalog-name {
  font-size: 12px;
  font-weight: 600;
  color: var(--text);
}

.card-footer {
  display: flex;
  justify-content: flex-end;
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
