<template>
  <v-dialog v-model="dialogModel" max-width="600" class="jedi-dialog-card">
    <v-card class="jedi-dialog-card">
      <v-toolbar color="surface" class="px-4 jedi-dialog-header border-b">
        <v-icon :icon="mdiDownload" color="primary" class="mr-2"></v-icon>
        <v-toolbar-title class="font-weight-medium">{{ $t('update.title') }}</v-toolbar-title>
        <v-spacer></v-spacer>
        <v-btn :icon="mdiClose" variant="text" color="medium-emphasis" @click="dialogModel = false"></v-btn>
      </v-toolbar>
      <v-card-text class="pa-6">
        <v-alert
          type="info"
          variant="tonal"
          class="mb-4"
          density="compact"
        >
          <div class="d-flex align-center">
            <span class="font-weight-bold mr-2">{{ $t('update.currentVersion') }}:</span>
            <span>{{ currentVersion }}</span>
          </div>
        </v-alert>

        <v-alert
          type="success"
          variant="tonal"
          class="mb-4"
          density="compact"
        >
          <div class="d-flex align-center">
            <span class="font-weight-bold mr-2">{{ $t('update.newVersion') }}:</span>
            <span>{{ updateInfo.version }}</span>
          </div>
        </v-alert>

        <div class="mb-4">
          <div class="text-subtitle-2 font-weight-bold mb-2">{{ $t('update.releaseNotes') }}</div>
          <v-card variant="outlined" class="pa-3 bg-surface-variant-opt">
            <div class="text-body-2 release-notes" v-html="formattedReleaseNotes"></div>
          </v-card>
        </div>

        <v-progress-linear
          v-if="isInstalling"
          indeterminate
          color="primary"
          height="6"
          rounded
          class="mb-4"
        ></v-progress-linear>
      </v-card-text>
      <v-card-actions class="pa-4 pt-0">
        <v-spacer></v-spacer>
        <v-btn variant="text" @click="dialogModel = false" rounded="sm" class="mr-2" :disabled="isInstalling">
          {{ $t('common.cancel') }}
        </v-btn>
        <v-btn
          color="var(--jedi-accent)"
          variant="elevated"
          @click="handleInstall"
          rounded="sm"
          :loading="isInstalling"
          :disabled="isInstalling"
        >
          {{ isInstalling ? $t('update.installing') : $t('update.downloadAndInstall') }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { mdiDownload, mdiClose } from '@mdi/js'
import { UpdateInfo } from '@/api/update'
import pkg from '../../../package.json'

// Define props
const props = defineProps<{
  modelValue: boolean
  updateInfo: UpdateInfo
  isInstalling: boolean
}>()

// Define emits
const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'install'): void
}>()

const currentVersion = pkg.version

// Dialog model
const dialogModel = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

// Format release notes (convert newlines to HTML)
const formattedReleaseNotes = computed(() => {
  if (!props.updateInfo.body) return ''
  return props.updateInfo.body
    .split('\n')
    .map(line => {
      // Handle markdown-style headers
      if (line.startsWith('##')) {
        return `<h3 class="text-subtitle-2 font-weight-bold mt-3 mb-2">${line.replace('## ', '')}</h3>`
      }
      if (line.startsWith('#')) {
        return `<h2 class="text-h6 font-weight-bold mt-4 mb-2">${line.replace('# ', '')}</h2>`
      }
      // Handle bullet points
      if (line.trim().startsWith('-')) {
        return `<div class="ml-4 my-1">• ${line.trim().replace('- ', '')}</div>`
      }
      // Handle empty lines
      if (!line.trim()) {
        return '<div class="my-1"></div>'
      }
      return `<div class="my-1">${line}</div>`
    })
    .join('')
})

const handleInstall = () => {
  emit('install')
}
</script>

<style scoped>
.release-notes {
  line-height: 1.6;
}

.release-notes :deep(h3) {
  margin-top: 1rem;
  margin-bottom: 0.5rem;
}

.bg-surface-variant-opt {
  background-color: rgba(var(--v-theme-surface-variant), 0.05);
}
</style>
