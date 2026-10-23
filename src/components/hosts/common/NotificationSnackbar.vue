<template>
  <v-snackbar v-model="show" :scrim="false" :timeout="timeout" location="top right" class="console-snackbar-root" :elevation="0">
    <div class="notice" :class="`notice-${color}`" role="status" aria-live="polite">
      <span class="notice-icon"><v-icon :icon="icon" size="20" aria-hidden="true" /></span>
      <div class="notice-content"><strong>{{ t(`notifications.${color}`) }}</strong><p>{{ text }}</p></div>
      <button type="button" class="notice-close" :aria-label="t('common.close')" @click="show = false"><v-icon :icon="mdiClose" size="16" aria-hidden="true" /></button>
    </div>
  </v-snackbar>
</template>

<script setup lang="ts">
import {
  mdiAlertCircleOutline,
  mdiAlertOutline,
  mdiCheckCircleOutline,
  mdiClose,
  mdiInformationOutline,
} from '@mdi/js'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

const props = defineProps<{
  modelValue: boolean
  text: string
  color: 'success' | 'error' | 'info' | 'warning'
  timeout?: number
}>()
const emit = defineEmits<(e: 'update:modelValue', value: boolean) => void>()
const { t } = useI18n()
const show = computed({
  get: () => props.modelValue,
  set: value => emit('update:modelValue', value),
})
const icon = computed(
  () =>
    ({
      success: mdiCheckCircleOutline,
      error: mdiAlertCircleOutline,
      warning: mdiAlertOutline,
      info: mdiInformationOutline,
    })[props.color]
)
</script>

<style scoped>
.console-snackbar-root { padding-top: 44px; padding-right: 12px; }
.console-snackbar-root :deep(.v-snackbar__wrapper) { background: transparent; box-shadow: none; border-radius: 14px; min-width: 0; }
.console-snackbar-root :deep(.v-snackbar__content) { padding: 0; }
.notice { --notice-color: var(--accent); display: flex; align-items: flex-start; gap: 12px; width: min(380px, calc(100vw - 48px)); padding: 16px; color: var(--text); background: var(--surface-raised); border: 1px solid var(--border-strong); border-radius: 14px; box-shadow: var(--shadow-lg); }
.notice-success { --notice-color: var(--success); }
.notice-warning { --notice-color: var(--warning); }
.notice-error { --notice-color: var(--danger); }
.notice-icon { display: grid; place-items: center; width: 34px; height: 34px; flex-shrink: 0; border-radius: 10px; color: var(--notice-color); background: color-mix(in srgb, var(--notice-color) 10%, transparent); }
.notice-content { flex: 1; min-width: 0; }
.notice-content strong { display: block; font-size: 12px; font-weight: 600; margin-bottom: 4px; }
.notice-content p { margin: 0; color: var(--text-muted); font-size: 12px; line-height: 1.6; overflow-wrap: anywhere; }
.notice-close { display: grid; place-items: center; width: 28px; height: 28px; flex-shrink: 0; color: var(--text-muted); border-radius: 7px; transition: background var(--motion-fast); }
.notice-close:hover { background: var(--bg-elevated-2); color: var(--text); }
</style>
