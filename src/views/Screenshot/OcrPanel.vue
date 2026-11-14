<template>
  <section ref="panel" class="ocr-panel" :aria-label="t('screenshot.ocr.title')" :aria-busy="busy">
    <header class="ocr-header">
      <span class="ocr-title">
        <v-icon :icon="mdiTextRecognition" size="14" aria-hidden="true" />
        {{ t('screenshot.ocr.title') }}
      </span>
      <span class="ocr-badge">{{ t('screenshot.ocr.offlineBadge') }}</span>
    </header>

    <div v-if="busy" class="ocr-body" role="status">
      <p class="ocr-note">{{ t('screenshot.ocr.loading') }}</p>
    </div>

    <div v-else-if="error" class="ocr-body">
      <v-alert type="error" variant="tonal" density="compact" role="alert">{{ error }}</v-alert>
      <div class="ocr-actions">
        <button class="ocr-btn primary" type="button" @click="recognize">{{ t('common.retry') }}</button>
      </div>
    </div>

    <div v-else class="ocr-body">
      <template v-if="lines.length">
        <p class="ocr-note">{{ t('screenshot.ocr.linesFound', { count: lines.length }) }}</p>
        <div class="ocr-lines">
          <div v-for="(line, index) in lines" :key="index" class="ocr-line">
            <div class="ocr-line-text">{{ line }}</div>
            <div class="ocr-line-pos">L{{ index + 1 }}</div>
          </div>
        </div>
      </template>
      <p v-else class="ocr-note">{{ t('screenshot.ocr.empty') }}</p>

      <div class="ocr-actions">
        <button
          class="ocr-btn primary"
          type="button"
          :disabled="!text.trim() || copying"
          @click="copy"
        >
          {{ copying ? t('screenshot.ocr.copying') : copied ? t('screenshot.ocr.copied') : t('screenshot.ocr.copyAll') }}
        </button>
      </div>
    </div>

    <footer v-if="engine && !busy && !error" class="ocr-footer">
      {{ engine }}<template v-if="language"> · {{ language }}</template>
    </footer>
  </section>
</template>

<script setup lang="ts">
import { mdiTextRecognition } from '@mdi/js'
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { copyScreenshotText, recognizeScreenshot } from '@/api/screenshot'

const props = defineProps<{ path: string }>()
const { t, te } = useI18n()
const panel = ref<HTMLElement | null>(null)
const busy = ref(false)
const text = ref('')
const language = ref('')
const engine = ref('')
const error = ref('')
const copied = ref(false)
const copying = ref(false)
let request = 0

/** 识别结果按行拆分，供逐行卡片展示 */
const lines = computed(() =>
  text.value
    .split(/\r?\n/)
    .map(line => line.trim())
    .filter(Boolean)
)

async function recognize() {
  const version = ++request
  busy.value = true
  error.value = ''
  text.value = ''
  language.value = ''
  engine.value = ''
  copied.value = false
  try {
    const result = await recognizeScreenshot(props.path)
    if (version !== request) return
    text.value = result.text
    language.value = result.language
    engine.value = result.engine
  } catch (cause) {
    if (version !== request) return
    const key = `screenshot.ocr.errors.${String(cause)}`
    error.value = t(te(key) ? key : 'screenshot.ocr.errors.failed')
  } finally {
    if (version === request) busy.value = false
  }
}

async function copy() {
  if (copying.value) return
  copying.value = true
  try {
    await copyScreenshotText(text.value)
    copied.value = true
    window.setTimeout(() => {
      copied.value = false
    }, 2200)
  } catch {
    error.value = t('screenshot.ocr.errors.copyFailed')
  } finally {
    copying.value = false
  }
}

onMounted(() => panel.value?.scrollIntoView({ block: 'nearest' }))
watch(() => props.path, recognize, { immediate: true })
onBeforeUnmount(() => {
  request++
})
</script>

<style scoped>
.ocr-panel {
  width: 296px;
  flex: none;
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
  border-left: 1px solid var(--border);
  background: var(--bg);
}

.ocr-header {
  height: 44px;
  flex: none;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 14px;
  border-bottom: 1px solid var(--border);
}

.ocr-title {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text);
}

.ocr-title :deep(.v-icon) {
  color: var(--accent);
}

.ocr-badge {
  font: 400 9px var(--jedi-font-mono);
  color: var(--text-muted);
}

.ocr-body {
  flex: 1;
  min-height: 0;
  padding: 14px;
  overflow-y: auto;
}

.ocr-note {
  margin: 0;
  font-size: 10.5px;
  line-height: 1.7;
  color: var(--text-muted);
}

.ocr-lines {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 10px;
}

.ocr-line {
  padding: 8px 10px;
  border: 1px solid var(--border);
  border-radius: 7px;
  background: rgb(var(--ink-rgb) / 0.03);
}

.ocr-line-text {
  font-size: 11px;
  line-height: 1.65;
  color: var(--text);
  overflow-wrap: anywhere;
  white-space: pre-wrap;
}

.ocr-line-pos {
  margin-top: 5px;
  font: 400 8.5px var(--jedi-font-mono);
  color: rgb(var(--ink-rgb) / 0.45);
}

.ocr-actions {
  display: flex;
  gap: 6px;
  margin-top: 12px;
}

.ocr-btn {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  height: 28px;
  padding: 0 10px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg);
  color: var(--text-muted);
  font: 400 11px var(--jedi-font-ui);
  cursor: pointer;
  transition: background var(--motion-fast), color var(--motion-fast), border-color var(--motion-fast);
}

.ocr-btn:hover:not(:disabled) {
  color: var(--text);
  border-color: var(--border-strong);
}

.ocr-btn.primary {
  border-color: var(--accent);
  background: var(--accent);
  color: var(--on-accent);
  font-weight: 600;
}

.ocr-btn.primary:hover:not(:disabled) {
  background: var(--accent-hover);
  border-color: var(--accent-hover);
  color: var(--on-accent);
}

.ocr-btn:disabled {
  opacity: 0.55;
  cursor: default;
}

.ocr-btn:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 1px;
}

.ocr-footer {
  flex: none;
  padding: 10px 14px;
  border-top: 1px solid var(--border);
  background: rgb(var(--ink-rgb) / 0.015);
  font: 400 9px var(--jedi-font-mono);
  color: rgb(var(--ink-rgb) / 0.45);
}

@media (max-width: 880px) {
  .ocr-panel {
    width: 100%;
    min-height: 360px;
    border-top: 1px solid var(--border);
    border-left: 0;
  }
}
</style>
