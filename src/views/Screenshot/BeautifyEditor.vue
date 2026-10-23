<template>
  <div class="beautify-editor">
    <div class="canvas-stage"><canvas ref="canvas" :aria-label="t('screenshot.beautify')" /></div>
    <aside class="controls">
      <h3>{{ t('screenshot.beautify') }}</h3>
      <p>{{ t('screenshot.beautifyHint') }}</p>
      <div class="swatches" :aria-label="t('screenshot.background')">
        <button v-for="(preset, i) in presets" :key="i" type="button" :style="{ background: `linear-gradient(135deg, ${preset[0]}, ${preset[1]})` }" :aria-label="t('screenshot.preset', { number: i + 1 })" :aria-pressed="from === preset[0] && to === preset[1]" @click="from = preset[0]; to = preset[1]" />
      </div>
      <div class="color-row"><label>{{ t('screenshot.colorFrom') }}<input v-model="from" type="color" /></label><label>{{ t('screenshot.colorTo') }}<input v-model="to" type="color" /></label></div>
      <label class="slider-label">{{ t('screenshot.padding') }} <output>{{ padding }} px</output><input v-model.number="padding" type="range" min="0" max="160" step="4" /></label>
      <label class="slider-label">{{ t('screenshot.radius') }} <output>{{ radius }} px</output><input v-model.number="radius" type="range" min="0" max="64" step="2" /></label>
      <label class="slider-label">{{ t('screenshot.shadow') }} <output>{{ shadow }} px</output><input v-model.number="shadow" type="range" min="0" max="80" step="2" /></label>
      <v-alert v-if="error" type="error" variant="tonal" density="compact">{{ error }}</v-alert>
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
  ['#a5b4fc', '#fbcfe8'],
  ['#38bdf8', '#134e4a'],
  ['#fb923c', '#fce7f3'],
  ['#27272a', '#52525b'],
]
const from = ref(presets[0][0])
const to = ref(presets[0][1])
const padding = ref(64)
const radius = ref(16)
const shadow = ref(32)
const canvas = ref<HTMLCanvasElement | null>(null)
const ready = ref(false)
const saving = ref(false)
const error = ref('')
let image: HTMLImageElement | null = null
let loadVersion = 0

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
    emit('saved', await saveStyledScreenshot(canvas.value.toDataURL('image/png')))
  } catch (err) {
    error.value = t('screenshot.captureFailed', { message: String(err) })
  } finally {
    saving.value = false
  }
}
watch(() => props.path, load)
watch([from, to, padding, radius, shadow], draw)
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
.slider-label { display: flex; flex-wrap: wrap; justify-content: space-between; gap: 10px; font-size: 12px; }
input[type="range"] { width: 100%; accent-color: var(--accent); } output { color: var(--text-muted); }
@media (max-width: 720px) { .beautify-editor { grid-template-columns: 1fr; } canvas { max-height: 40vh; } }
</style>
