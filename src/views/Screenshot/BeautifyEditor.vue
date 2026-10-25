<template>
  <div class="beautify-editor">
    <div class="canvas-stage"><canvas ref="canvas" :aria-label="t('screenshot.beautify')" /></div>
    <aside class="controls">
      <h3>{{ t('screenshot.beautify') }}</h3>
      <p>{{ t('screenshot.beautifyHint') }}</p>
      <div class="swatches" :aria-label="t('screenshot.background')">
        <button v-for="preset in presets" :key="preset.id" type="button" :style="{ background: `linear-gradient(135deg, ${preset.from}, ${preset.to})` }" :aria-label="t(`screenshot.presetNames.${preset.id}`)" :aria-pressed="from === preset.from && to === preset.to" @click="from = preset.from; to = preset.to" />
      </div>
      <div class="color-row"><label>{{ t('screenshot.colorFrom') }}<input v-model="from" type="color" /></label><label>{{ t('screenshot.colorTo') }}<input v-model="to" type="color" /></label></div>
      <label class="slider-label"><span>{{ t('screenshot.padding') }}</span><input v-model.number="padding" class="num-input" type="number" min="0" max="160" step="4" /><input v-model.number="padding" type="range" min="0" max="160" step="4" :aria-label="t('screenshot.padding')" /></label>
      <label class="slider-label"><span>{{ t('screenshot.radius') }}</span><input v-model.number="radius" class="num-input" type="number" min="0" max="64" step="2" /><input v-model.number="radius" type="range" min="0" max="64" step="2" :aria-label="t('screenshot.radius')" /></label>
      <label class="slider-label"><span>{{ t('screenshot.shadow') }}</span><input v-model.number="shadow" class="num-input" type="number" min="0" max="80" step="2" /><input v-model.number="shadow" type="range" min="0" max="80" step="2" :aria-label="t('screenshot.shadow')" /></label>
      <button type="button" class="reset-btn" @click="resetParams">{{ t('screenshot.resetParams') }}</button>
      <v-alert v-if="error" type="error" variant="tonal" density="compact">{{ error }}</v-alert>
      <div v-if="ready" class="export-info">
        <p>{{ t('screenshot.exportSize', { w: exportWidth, h: exportHeight }) }}</p>
        <p>{{ t('screenshot.saveAsCopy') }}</p>
      </div>
      <v-btn color="primary" block :loading="saving" :disabled="!ready" @click="save">{{ t('screenshot.saveStyled') }}</v-btn>
    </aside>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { readImageDataUrl, saveStyledScreenshot } from '@/api/screenshot'

const props = defineProps<{ path: string }>()
const emit = defineEmits<{ saved: [path: string] }>()
const { t } = useI18n()
const presets = [
  { id: 'mist', from: '#a5b4fc', to: '#fbcfe8' },
  { id: 'ocean', from: '#38bdf8', to: '#134e4a' },
  { id: 'sunset', from: '#fb923c', to: '#fce7f3' },
  { id: 'slate', from: '#27272a', to: '#52525b' },
]
const from = ref(presets[0].from)
const to = ref(presets[0].to)
const padding = ref(64)
const radius = ref(16)
const shadow = ref(32)
const canvas = ref<HTMLCanvasElement | null>(null)
const ready = ref(false)
const saving = ref(false)
const error = ref('')
const exportWidth = ref(0)
const exportHeight = ref(0)
let image: HTMLImageElement | null = null
let loadVersion = 0
let draftTimer: number | undefined
let draftTimerPath = ''

function draftKey(path: string) {
  return `beautify-draft:${path}`
}

function resetParams() {
  from.value = presets[0].from
  to.value = presets[0].to
  padding.value = 64
  radius.value = 16
  shadow.value = 32
}

function draftNumber(value: unknown, min: number, max: number, fallback: number) {
  const n = typeof value === 'number' ? value : Number(value)
  return Number.isFinite(n) ? Math.min(max, Math.max(min, n)) : fallback
}

function applyDraft(path: string) {
  let draft: unknown = null
  try {
    const raw = sessionStorage.getItem(draftKey(path))
    if (raw) draft = JSON.parse(raw)
  } catch {
    draft = null
  }
  if (!draft || typeof draft !== 'object') {
    resetParams()
    return
  }
  const saved = draft as Record<string, unknown>
  from.value = typeof saved.from === 'string' && saved.from ? saved.from : presets[0].from
  to.value = typeof saved.to === 'string' && saved.to ? saved.to : presets[0].to
  padding.value = draftNumber(saved.padding, 0, 160, 64)
  radius.value = draftNumber(saved.radius, 0, 64, 16)
  shadow.value = draftNumber(saved.shadow, 0, 80, 32)
}

function writeDraft(path: string) {
  try {
    sessionStorage.setItem(
      draftKey(path),
      JSON.stringify({
        from: from.value,
        to: to.value,
        padding: padding.value,
        radius: radius.value,
        shadow: shadow.value,
      })
    )
  } catch {
    // 草稿写入失败可忽略（不影响编辑与导出）
  }
}

function cancelDraft() {
  if (draftTimer !== undefined) {
    clearTimeout(draftTimer)
    draftTimer = undefined
  }
}

function flushDraft() {
  if (draftTimer === undefined) return
  cancelDraft()
  writeDraft(draftTimerPath)
}

function draw() {
  const target = canvas.value
  if (!target || !image) return
  const ctx = target.getContext('2d')
  if (!ctx) return
  const w = image.naturalWidth
  const h = image.naturalHeight
  // 限制画布尺寸和内存；一般截图按原始像素导出。
  const scale = Math.min(1, 6000 / Math.max(w + padding.value * 2, h + padding.value * 2))
  target.width = Math.round((w + padding.value * 2) * scale)
  target.height = Math.round((h + padding.value * 2) * scale)
  exportWidth.value = target.width
  exportHeight.value = target.height
  ctx.scale(scale, scale)
  const gradient = ctx.createLinearGradient(0, 0, w + padding.value * 2, h + padding.value * 2)
  gradient.addColorStop(0, from.value)
  gradient.addColorStop(1, to.value)
  ctx.fillStyle = gradient
  ctx.fillRect(0, 0, target.width / scale, target.height / scale)
  const r = Math.min(radius.value, w / 2, h / 2)
  ctx.save()
  ctx.beginPath()
  ctx.roundRect(padding.value, padding.value, w, h, r)
  ctx.shadowColor = 'rgba(0, 0, 0, 0.35)'
  ctx.shadowBlur = shadow.value
  ctx.shadowOffsetY = shadow.value / 3
  ctx.fillStyle = '#fff'
  ctx.fill()
  ctx.restore()
  ctx.save()
  ctx.beginPath()
  ctx.roundRect(padding.value, padding.value, w, h, r)
  ctx.clip()
  ctx.drawImage(image, padding.value, padding.value, w, h)
  ctx.restore()
  ready.value = true
}
async function load() {
  const version = ++loadVersion
  ready.value = false
  image = null
  error.value = ''
  flushDraft()
  applyDraft(props.path)
  const next = new Image()
  try {
    next.src = await readImageDataUrl(props.path)
    await next.decode()
    if (version !== loadVersion) return
    image = next
    draw()
  } catch (err) {
    error.value = t('screenshot.loadFailed', { message: String(err) })
  }
}
async function save() {
  if (!canvas.value || !ready.value || saving.value) return
  saving.value = true
  error.value = ''
  try {
    const savedPath = await saveStyledScreenshot(canvas.value.toDataURL('image/png'))
    cancelDraft()
    try {
      sessionStorage.removeItem(draftKey(props.path))
    } catch {
      // 草稿清理失败可忽略
    }
    emit('saved', savedPath)
  } catch (err) {
    error.value = t('screenshot.captureFailed', { message: String(err) })
  } finally {
    saving.value = false
  }
}
watch(() => props.path, load)
watch([from, to, padding, radius, shadow], draw)
watch([from, to, padding, radius, shadow], () => {
  cancelDraft()
  draftTimerPath = props.path
  draftTimer = window.setTimeout(() => {
    draftTimer = undefined
    writeDraft(draftTimerPath)
  }, 300)
})
onMounted(load)
</script>

<style scoped>
.beautify-editor { display: grid; grid-template-columns: minmax(0, 1fr) 248px; min-height: 420px; }
.canvas-stage { display: grid; place-items: center; padding: 28px; min-width: 0; background: var(--bg-elevated-2); }
canvas { max-width: 100%; max-height: 62vh; object-fit: contain; }
.controls { padding: 20px; display: flex; flex-direction: column; gap: 20px; background: var(--bg); }
h3, p { margin: 0; } h3 { font-size: 16px; } p { color: var(--text-muted); font-size: 12px; line-height: 1.6; }
.swatches { display: flex; gap: 10px; }
.swatches button { width: 38px; height: 38px; border: 2px solid var(--border); border-radius: 10px; cursor: pointer; }
.swatches button[aria-pressed="true"] { outline: 2px solid var(--accent); outline-offset: 3px; }
button:focus-visible, input:focus-visible { outline: 2px solid var(--accent); outline-offset: 3px; }
.color-row { display: flex; gap: 16px; } .color-row label { display: flex; flex-direction: column; gap: 8px; flex: 1; font-size: 12px; }
input[type="color"] { width: 100%; height: 32px; cursor: pointer; }
.slider-label { display: flex; flex-wrap: wrap; justify-content: space-between; align-items: center; gap: 10px; font-size: 12px; }
input[type="range"] { width: 100%; accent-color: var(--accent); }
.num-input { width: 64px; padding: 3px 8px; border: 1px solid var(--border); border-radius: 6px; background: var(--bg); color: var(--text); font: inherit; }
.reset-btn { align-self: flex-start; padding: 6px 14px; border: 1px solid var(--border); border-radius: 8px; background: transparent; color: var(--text-muted); font-size: 12px; cursor: pointer; }
.reset-btn:hover { border-color: var(--accent); color: var(--text); }
.export-info { display: flex; flex-direction: column; gap: 4px; }
@media (max-width: 720px) { .beautify-editor { grid-template-columns: 1fr; } canvas { max-height: 40vh; } }
</style>
