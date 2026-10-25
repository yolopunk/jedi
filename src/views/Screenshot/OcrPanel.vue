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
        <span>{{ language }}</span>
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
  copied.value = false
  copyError.value = ''
  try {
    const result = await recognizeScreenshot(props.path)
    if (version !== request) return
    text.value = result.text
    language.value = result.language
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
.ocr-panel { min-height: 240px; padding: 20px 24px; border-top: 1px solid var(--border); background: var(--bg); }
header, footer { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
header { justify-content: space-between; margin-bottom: 12px; }
h3 { font-size: 15px; margin: 0 0 6px; }
p, footer { font-size: 12px; color: var(--text-muted); line-height: 1.6; }
.result-label { display: block; font-size: 12px; margin-bottom: 8px; }
textarea { display: block; width: 100%; min-height: 160px; max-height: 40vh; padding: 12px; resize: vertical; border: 1px solid var(--border); border-radius: 10px; color: var(--text); background: var(--bg-elevated-2); font: 14px/1.7 var(--jedi-font-ui); }
textarea:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
footer { margin-top: 12px; }
footer > :first-child { margin-right: auto; }
</style>
