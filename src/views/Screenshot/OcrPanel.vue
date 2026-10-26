<template>
  <section ref="panel" class="ocr-panel" :aria-label="t('screenshot.ocr.title')" :aria-busy="busy">
    <header>
      <div><h3>{{ t('screenshot.ocr.title') }}</h3><p>{{ t('screenshot.ocr.local') }}</p></div>
      <v-btn variant="text" size="small" :loading="busy" @click="recognize">{{ t('common.retry') }}</v-btn>
    </header>
    <p v-if="busy" role="status">{{ t('screenshot.ocr.loading') }}</p>
    <v-alert v-else-if="error" type="error" variant="tonal" role="alert">{{ error }}</v-alert>
    <template v-else>
      <p v-if="!text.trim()" role="status">{{ t('screenshot.ocr.empty') }}</p>
      <label class="result-label" for="ocr-result">{{ t('screenshot.ocr.result') }}</label>
      <textarea id="ocr-result" v-model="text" spellcheck="false" />
      <footer>
        <span>{{ engine }}<template v-if="language"> · {{ language }}</template></span>
        <span role="status">{{ copied ? t('screenshot.ocr.copied') : '' }}</span>
        <v-btn variant="tonal" color="primary" :disabled="!text.trim()" :loading="copying" @click="copy">{{ t('screenshot.ocr.copy') }}</v-btn>
      </footer>
      <p v-if="copyError" role="alert">{{ copyError }}</p>
    </template>
  </section>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
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
const copyError = ref('')
let request = 0

async function recognize() {
  const version = ++request
  busy.value = true
  error.value = ''
  text.value = ''
  language.value = ''
  engine.value = ''
  copied.value = false
  copyError.value = ''
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
  copyError.value = ''
  try {
    await copyScreenshotText(text.value)
    copied.value = true
  } catch {
    copyError.value = t('screenshot.ocr.errors.copyFailed')
  } finally {
    copying.value = false
  }
}

onMounted(() => panel.value?.scrollIntoView({ block: 'nearest' }))
watch(() => props.path, recognize, { immediate: true })
watch(text, () => {
  copied.value = false
})
onBeforeUnmount(() => {
  request++
})
</script>

<style scoped>
.ocr-panel { min-width: 0; min-height: 0; overflow: auto; padding: 22px; border-left: 1px solid var(--border); background: linear-gradient(180deg, rgb(var(--accent-rgb) / 0.045), transparent 180px), var(--bg); }
header, footer { display: flex; align-items: center; gap: 10px; flex-wrap: wrap; }
header { justify-content: space-between; margin-bottom: 18px; }
header > div { min-width: 0; }
h3 { margin: 0 0 5px; font-size: 14px; font-weight: 650; }
p, footer { color: var(--text-muted); font-size: 11px; line-height: 1.65; }
header p { max-width: 250px; margin: 0; }
.result-label { display: block; margin: 12px 0 8px; color: var(--text-muted); font-size: 10px; font-weight: 600; letter-spacing: 0.7px; text-transform: uppercase; }
textarea { display: block; width: 100%; min-height: 230px; max-height: 48vh; padding: 13px 14px; resize: vertical; border: 1px solid var(--border); border-radius: 11px; background: var(--surface-canvas); color: var(--text); font: 13px/1.72 var(--jedi-font-ui); transition: border-color var(--motion-fast), box-shadow var(--motion-fast); }
textarea:focus-visible { border-color: rgb(var(--accent-rgb) / 0.65); outline: 0; box-shadow: 0 0 0 3px rgb(var(--accent-rgb) / 0.12); }
footer { margin-top: 13px; }
footer > :first-child { width: 100%; overflow-wrap: anywhere; font: 9px/1.6 var(--jedi-font-mono); }
footer > :nth-child(2) { margin-right: auto; color: var(--success); }
@media (max-width: 880px) { .ocr-panel { min-height: 360px; border-top: 1px solid var(--border); border-left: 0; } }
</style>
