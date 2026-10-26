<template>
  <div class="beautify-editor">
    <div class="canvas-stage">
      <div class="stage-bar">
        <span><span class="live-dot" aria-hidden="true"></span>{{ t('screenshot.livePreview') }}</span>
        <span v-if="ready">{{ exportWidth }} × {{ exportHeight }}</span>
      </div>
      <div class="canvas-viewport"><canvas ref="canvas" :aria-label="t('screenshot.beautify')" /></div>
    </div>
    <aside class="controls">
      <div class="controls-scroll">
        <header class="controls-header">
          <span class="controls-icon"><v-icon :icon="mdiAutoFix" size="18" aria-hidden="true" /></span>
          <div><h3>{{ t('screenshot.beautify') }}</h3><p>{{ t('screenshot.beautifyHint') }}</p></div>
        </header>
        <section class="control-section">
          <div class="section-heading"><span>{{ t('screenshot.background') }}</span></div>
          <div class="swatches" :aria-label="t('screenshot.background')">
            <button v-for="preset in presets" :key="preset.id" type="button" :aria-label="t(`screenshot.presetNames.${preset.id}`)" :aria-pressed="from === preset.from && to === preset.to" @click="from = preset.from; to = preset.to">
              <span :style="{ background: `linear-gradient(135deg, ${preset.from}, ${preset.to})` }"></span>
              <small>{{ t(`screenshot.presetNames.${preset.id}`) }}</small>
            </button>
          </div>
          <div class="color-row">
            <label><span>{{ t('screenshot.colorFrom') }}</span><span class="color-field"><input v-model="from" type="color" /><code>{{ from.toUpperCase() }}</code></span></label>
            <label><span>{{ t('screenshot.colorTo') }}</span><span class="color-field"><input v-model="to" type="color" /><code>{{ to.toUpperCase() }}</code></span></label>
          </div>
        </section>
        <section class="control-section geometry-section">
          <div class="section-heading"><span>{{ t('screenshot.appearance') }}</span><button type="button" class="reset-btn" @click="resetParams"><v-icon :icon="mdiRestore" size="15" aria-hidden="true" />{{ t('screenshot.resetParams') }}</button></div>
          <label class="slider-label"><span>{{ t('screenshot.padding') }}</span><span class="value-input"><input v-model.number="padding" class="num-input" type="number" min="0" max="160" step="4" /><small>px</small></span><input v-model.number="padding" type="range" min="0" max="160" step="4" :aria-label="t('screenshot.padding')" /></label>
          <label class="slider-label"><span>{{ t('screenshot.radius') }}</span><span class="value-input"><input v-model.number="radius" class="num-input" type="number" min="0" max="64" step="2" /><small>px</small></span><input v-model.number="radius" type="range" min="0" max="64" step="2" :aria-label="t('screenshot.radius')" /></label>
          <label class="slider-label"><span>{{ t('screenshot.shadow') }}</span><span class="value-input"><input v-model.number="shadow" class="num-input" type="number" min="0" max="80" step="2" /><small>px</small></span><input v-model.number="shadow" type="range" min="0" max="80" step="2" :aria-label="t('screenshot.shadow')" /></label>
        </section>
        <v-alert v-if="error" type="error" variant="tonal" density="compact">{{ error }}</v-alert>
      </div>
      <div class="export-panel">
        <div v-if="ready" class="export-info">
          <strong>{{ t('screenshot.exportSize', { w: exportWidth, h: exportHeight }) }}</strong>
          <span>{{ t('screenshot.saveAsCopy') }}</span>
        </div>
        <v-btn color="primary" rounded="lg" :prepend-icon="mdiContentSaveOutline" :loading="saving" :disabled="!ready" @click="save">{{ t('screenshot.saveStyled') }}</v-btn>
      </div>
    </aside>
  </div>
</template>

<script setup lang="ts">
import { mdiAutoFix, mdiContentSaveOutline, mdiRestore } from '@mdi/js'
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
.beautify-editor { min-height: 0; display: grid; grid-template-columns: minmax(0, 1fr) 328px; overflow: hidden; }
.canvas-stage { min-width: 0; min-height: 0; display: grid; grid-template-rows: auto minmax(0, 1fr); padding: 14px; border-right: 1px solid var(--border); background:
    radial-gradient(circle at 50% 36%, rgb(var(--accent-rgb) / 0.09), transparent 32rem),
    var(--surface-canvas); }
.stage-bar { display: flex; align-items: center; justify-content: space-between; gap: 16px; padding: 2px 4px 12px; color: var(--text-muted); font: 10px var(--jedi-font-mono); }
.stage-bar > span:first-child { display: flex; align-items: center; gap: 7px; }
.live-dot { width: 6px; height: 6px; border-radius: 50%; background: var(--success); box-shadow: 0 0 0 4px rgb(var(--success-rgb) / 0.12); }
.canvas-viewport { min-width: 0; min-height: 0; display: grid; place-items: center; overflow: auto; padding: clamp(18px, 3vw, 38px); border: 1px solid rgb(var(--text-rgb) / 0.07); border-radius: 13px; background-image: radial-gradient(rgb(var(--text-rgb) / 0.075) 0.7px, transparent 0.7px); background-size: 12px 12px; }
canvas { max-width: 100%; max-height: 64vh; object-fit: contain; box-shadow: 0 24px 70px rgb(var(--ink-rgb) / 0.24); }
.controls { min-width: 0; min-height: 0; display: flex; flex-direction: column; background: var(--bg); }
.controls-scroll { min-height: 0; flex: 1; overflow: auto; padding: 22px; }
.controls-header { display: flex; align-items: flex-start; gap: 12px; padding-bottom: 20px; }
.controls-icon { display: grid; width: 36px; height: 36px; flex: 0 0 auto; place-items: center; border: 1px solid rgb(var(--accent-rgb) / 0.22); border-radius: 10px; background: rgb(var(--accent-rgb) / 0.1); color: var(--accent); }
h3, p { margin: 0; }
h3 { font-size: 15px; font-weight: 650; }
p { margin-top: 5px; color: var(--text-muted); font-size: 11px; line-height: 1.6; }
.control-section { padding: 18px 0; border-top: 1px solid var(--border); }
.section-heading { min-height: 28px; display: flex; align-items: center; justify-content: space-between; gap: 12px; margin-bottom: 13px; color: var(--text); font-size: 11px; font-weight: 650; }
.swatches { display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px; }
.swatches button { min-width: 0; padding: 4px; border: 1px solid transparent; border-radius: 9px; background: transparent; color: var(--text-muted); cursor: pointer; transition: border-color var(--motion-fast), background var(--motion-fast), color var(--motion-fast); }
.swatches button > span { display: block; aspect-ratio: 1.25; border: 1px solid rgb(var(--text-rgb) / 0.12); border-radius: 7px; box-shadow: inset 0 0 0 1px rgb(255 255 255 / 0.08); }
.swatches button small { display: block; margin-top: 6px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 9px; }
.swatches button:hover { background: rgb(var(--text-rgb) / 0.045); color: var(--text); }
.swatches button[aria-pressed="true"] { border-color: rgb(var(--accent-rgb) / 0.5); background: rgb(var(--accent-rgb) / 0.1); color: var(--accent); }
button:focus-visible, input:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
.color-row { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; margin-top: 14px; }
.color-row label { display: flex; flex-direction: column; gap: 7px; color: var(--text-muted); font-size: 10px; }
.color-field { display: flex; align-items: center; gap: 7px; min-width: 0; padding: 5px 7px; border: 1px solid var(--border); border-radius: 8px; background: var(--surface-canvas); }
input[type="color"] { width: 26px; height: 24px; flex: 0 0 auto; padding: 0; border: 0; border-radius: 5px; background: transparent; cursor: pointer; }
.color-field code { min-width: 0; overflow: hidden; color: var(--text-muted); font: 9px var(--jedi-font-mono); text-overflow: ellipsis; }
.geometry-section { display: flex; flex-direction: column; gap: 16px; }
.geometry-section .section-heading { margin-bottom: -2px; }
.slider-label { display: grid; grid-template-columns: 1fr auto; align-items: center; gap: 8px 12px; color: var(--text-muted); font-size: 11px; }
.slider-label > input[type="range"] { grid-column: 1 / -1; width: 100%; margin: 0; accent-color: var(--accent); cursor: pointer; }
.value-input { display: flex; align-items: center; gap: 2px; padding: 2px 6px; border: 1px solid var(--border); border-radius: 7px; background: var(--surface-canvas); }
.value-input small { color: var(--text-muted); font: 9px var(--jedi-font-mono); }
.num-input { width: 38px; padding: 2px 0; border: 0; outline: 0; background: transparent; color: var(--text); font: 10px var(--jedi-font-mono); text-align: right; }
.reset-btn { display: flex; align-items: center; gap: 5px; padding: 4px 7px; border: 0; border-radius: 6px; background: transparent; color: var(--text-muted); font: 10px var(--jedi-font-ui); cursor: pointer; }
.reset-btn:hover { background: rgb(var(--text-rgb) / 0.06); color: var(--text); }
.export-panel { display: flex; flex-direction: column; gap: 13px; padding: 16px 20px 18px; border-top: 1px solid var(--border); background: rgb(var(--text-rgb) / 0.018); }
.export-info { display: flex; flex-direction: column; gap: 3px; }
.export-info strong { font: 600 10px var(--jedi-font-mono); }
.export-info span { color: var(--text-muted); font-size: 10px; }
@media (max-width: 900px) {
  .beautify-editor { grid-template-columns: 1fr; overflow: auto; }
  .canvas-stage { min-height: 360px; border-right: 0; border-bottom: 1px solid var(--border); }
  canvas { max-height: 48vh; }
  .controls { min-height: 430px; }
}
@media (prefers-reduced-motion: reduce) { .swatches button { transition: none; } }
</style>
