<template>
  <main class="screenshot-page" :style="{ '--thumbnail-size': `${thumbnailSize}px` }">
    <div class="screenshot-body">
      <header class="capture-header">
        <div class="heading">
          <h1>{{ t('screenshot.galleryTitle') }}<span class="title-dot" aria-hidden="true"></span></h1>
        </div>
        <div class="capture-actions">
          <v-btn color="primary" rounded="lg" :prepend-icon="mdiPlus" :loading="capturing" @click="startCapture">{{ t('screenshot.start') }}</v-btn>
          <span class="shortcut"><v-icon :icon="mdiKeyboardOutline" size="16" aria-hidden="true" />Ctrl + Alt + S</span>
        </div>
      </header>
      <v-alert v-if="error" type="error" variant="tonal" closable @click:close="error = ''">{{ error }}</v-alert>
      <section class="gallery" :aria-label="t('screenshot.recent')" :aria-busy="loading">
        <div class="gallery-toolbar">
          <div class="collection-summary"><v-icon :icon="mdiImageMultipleOutline" size="18" aria-hidden="true" /><span class="collection-label">{{ t('screenshot.recent') }}</span><span class="count">{{ recent.length }}</span></div>
          <div class="gallery-controls">
            <label class="density-control">{{ t('screenshot.thumbnailSize') }}<input v-model.number="thumbnailSize" type="range" min="220" max="380" step="20" /></label>
            <v-btn :icon="mdiRefresh" variant="text" size="small" :loading="loading" :aria-label="t('common.refresh')" @click="loadRecent" />
          </div>
        </div>
        <div v-if="recent.length" class="gallery-collections">
          <section v-for="group in groups" :key="group.key" class="date-group" :aria-label="group.title">
            <div class="date-heading"><h3>{{ group.title }}</h3><span>{{ group.items.length.toString().padStart(2, '0') }}</span><div></div></div>
            <div class="photo-columns">
              <article v-for="(item, index) in group.items" :key="item.path" class="capture-card" :style="{ '--card-delay': `${Math.min(index, 5) * 35}ms` }">
                <button class="capture-open" type="button" @click="selected = item; beautifying = false">
                <div class="thumbnail">
                  <img v-if="!brokenImages.has(item.path)" :key="`${item.path}:${imageRetries[item.path] || 0}`" :src="thumbSrc(item.path)" :alt="item.name" loading="lazy" draggable="false" @error="onThumbError(item.path)" />
                  <div v-else class="thumb-broken">
                    <v-icon :icon="mdiImageOffOutline" size="22" aria-hidden="true" />
                    <span class="thumb-broken-text">{{ t('screenshot.imageMissing') }}</span>
                  </div>
                  <span v-if="!brokenImages.has(item.path)" class="preview-hint"><v-icon :icon="mdiMagnifyPlusOutline" size="18" aria-hidden="true" />{{ t('screenshot.preview') }}</span>
                </div>
                <div class="card-info"><span class="capture-name" :title="item.name">{{ item.name }}</span><span class="card-meta"><time>{{ formatDate(item.modifiedMs) }}</time><span>{{ formatSize(item.size) }}</span></span></div>
                </button>
                <button v-if="brokenImages.has(item.path)" type="button" class="thumb-retry" @click="retryThumb(item.path)">{{ t('screenshot.retryImage') }}</button>
              </article>
            </div>
          </section>
        </div>
        <div v-else-if="loading" class="gallery-skeleton" :aria-label="t('common.refresh')"><div v-for="i in 6" :key="i" class="skeleton-frame"></div></div>
        <div v-else-if="!error" class="empty-state">
          <div class="empty-frames" aria-hidden="true"><span></span><span><v-icon :icon="mdiCropFree" size="32" /></span><span></span></div>
          <h3>{{ t('screenshot.recentEmpty') }}</h3><p>{{ t('screenshot.galleryDescription') }}</p>
          <v-btn variant="tonal" color="primary" :prepend-icon="mdiCrop" :loading="capturing" @click="startCapture">{{ t('screenshot.start') }}</v-btn>
        </div>
      </section>
    </div>
    <v-dialog :model-value="!!selected" max-width="1200" @update:model-value="selected = null">
      <div v-if="selected" class="preview-panel">
        <header class="preview-header">
          <div>
            <span class="section-label">{{ t(beautifying ? 'screenshot.beautify' : 'screenshot.preview') }}</span>
            <h2>{{ selected.name }}</h2>
            <span v-if="!beautifying && previewPixelLabel" class="pixel-size">{{ previewPixelLabel }}</span>
          </div>
          <v-btn :icon="mdiClose" variant="text" :aria-label="t('screenshot.closePreview')" @click="selected = null" />
        </header>
        <BeautifyEditor v-if="beautifying" :path="selected.path" @saved="onStyledSaved" />
        <div v-else class="preview-image" tabindex="0" :aria-label="t('screenshot.preview')" :class="{ 'is-actual': previewZoom === 'actual', 'is-panning': panning }" @pointerdown="onPanStart" @pointermove="onPanMove" @pointerup="onPanEnd" @pointercancel="onPanEnd" @lostpointercapture="onPanEnd">
          <div v-if="previewFailed" role="alert" class="thumb-broken"><span>{{ t('screenshot.imageMissing') }}</span><v-btn variant="text" @click="retryThumb(selected.path); previewFailed = false">{{ t('screenshot.retryImage') }}</v-btn></div>
          <img v-else :src="thumbSrc(selected.path)" :alt="selected.name" draggable="false" :style="previewStyle" @load="onPreviewLoad" @error="previewFailed = true" />
        </div>
        <footer class="preview-footer">
          <span>{{ formatDate(selected.modifiedMs) }} · {{ formatSize(selected.size) }}</span>
          <div>
            <div v-if="!beautifying" class="zoom-toggle">
              <v-btn size="small" :variant="previewZoom === 'fit' ? 'tonal' : 'text'" :color="previewZoom === 'fit' ? 'primary' : undefined" @click="previewZoom = 'fit'">{{ t('screenshot.fitWindow') }}</v-btn>
              <v-btn size="small" :variant="previewZoom === 'actual' ? 'tonal' : 'text'" :color="previewZoom === 'actual' ? 'primary' : undefined" :disabled="!previewPixels" @click="previewZoom = 'actual'">{{ t('screenshot.actualSize') }}</v-btn>
            </div>
            <v-btn variant="text" :prepend-icon="mdiFolderOpenOutline" @click="openFolder(selected.path)">{{ t('screenshot.openFolder') }}</v-btn>
            <v-btn variant="tonal" color="primary" :title="beautifying ? t('screenshot.backToOriginalHint') : undefined" @click="beautifying = !beautifying">{{ t(beautifying ? 'screenshot.backToOriginal' : 'screenshot.beautify') }}</v-btn>
          </div>
        </footer>
      </div>
    </v-dialog>
  </main>
</template>

<script setup lang="ts">
import {
  mdiClose,
  mdiCrop,
  mdiCropFree,
  mdiFolderOpenOutline,
  mdiImageMultipleOutline,
  mdiImageOffOutline,
  mdiKeyboardOutline,
  mdiMagnifyPlusOutline,
  mdiPlus,
  mdiRefresh,
} from '@mdi/js'
import { convertFileSrc } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { listScreenshots, type ScreenshotItem, screenshotBegin } from '@/api/screenshot'
import { showInFolder } from '@/api/wallpaper'
import BeautifyEditor from './BeautifyEditor.vue'

const { t, locale } = useI18n()
const recent = ref<ScreenshotItem[]>([])
const thumbnailSize = ref(280)
const groups = computed(() => {
  const result: { key: string; title: string; items: ScreenshotItem[] }[] = []
  for (const item of recent.value) {
    const date = new Date(item.modifiedMs)
    const key = `${date.getFullYear()}-${date.getMonth()}-${date.getDate()}`
    let group = result.find(group => group.key === key)
    if (!group) {
      group = {
        key,
        title: new Intl.DateTimeFormat(locale.value, {
          year: 'numeric',
          month: 'long',
          day: 'numeric',
        }).format(date),
        items: [],
      }
      result.push(group)
    }
    group.items.push(item)
  }
  return result
})
const selected = ref<ScreenshotItem | null>(null)
const beautifying = ref(false)
const capturing = ref(false)
const loading = ref(false)
const error = ref('')
const previewZoom = ref<'fit' | 'actual'>('fit')
const previewPixels = ref<{ w: number; h: number } | null>(null)
const panning = ref(false)
const previewFailed = ref(false)
let panOrigin = { x: 0, y: 0, left: 0, top: 0 }
const imageRetries = ref<Record<string, number>>({})
const brokenImages = ref(new Set<string>())
let unlisten: UnlistenFn | undefined
let disposed = false

const previewPixelLabel = computed(() =>
  previewPixels.value ? t('screenshot.pixelSize', previewPixels.value) : ''
)
const previewStyle = computed(() => {
  if (previewZoom.value === 'actual' && previewPixels.value) {
    return { width: `${previewPixels.value.w}px`, maxWidth: 'none' }
  }
  return undefined
})

watch(selected, () => {
  previewFailed.value = false
  previewZoom.value = 'fit'
  previewPixels.value = null
  panning.value = false
})

function onPreviewLoad(event: Event) {
  const img = event.target as HTMLImageElement
  previewPixels.value = { w: img.naturalWidth, h: img.naturalHeight }
}

function onPanStart(event: PointerEvent) {
  if (previewZoom.value !== 'actual' || event.button !== 0) return
  const el = event.currentTarget as HTMLElement
  if (el.scrollWidth <= el.clientWidth && el.scrollHeight <= el.clientHeight) return
  panOrigin = { x: event.clientX, y: event.clientY, left: el.scrollLeft, top: el.scrollTop }
  panning.value = true
  el.setPointerCapture(event.pointerId)
  el.focus({ preventScroll: true })
  event.preventDefault()
}

function onPanMove(event: PointerEvent) {
  if (!panning.value) return
  const el = event.currentTarget as HTMLElement
  el.scrollLeft = panOrigin.left - (event.clientX - panOrigin.x)
  el.scrollTop = panOrigin.top - (event.clientY - panOrigin.y)
}

function onPanEnd(event: PointerEvent) {
  panning.value = false
  const el = event.currentTarget as HTMLElement
  if (el.hasPointerCapture(event.pointerId)) el.releasePointerCapture(event.pointerId)
}

function thumbSrc(path: string): string {
  const retry = imageRetries.value[path]
  return retry ? `${convertFileSrc(path)}?retry=${retry}` : convertFileSrc(path)
}

function onThumbError(path: string) {
  brokenImages.value.add(path)
}

function retryThumb(path: string) {
  imageRetries.value = { ...imageRetries.value, [path]: (imageRetries.value[path] || 0) + 1 }
  brokenImages.value.delete(path)
}

async function onStyledSaved() {
  selected.value = null
  await loadRecent()
}

async function startCapture() {
  if (capturing.value) return
  capturing.value = true
  error.value = ''
  try {
    await screenshotBegin()
  } catch (err) {
    error.value = t('screenshot.captureFailed', { message: String(err) })
  } finally {
    capturing.value = false
  }
}

async function loadRecent() {
  if (loading.value) return
  loading.value = true
  error.value = ''
  try {
    recent.value = await listScreenshots()
  } catch (err) {
    error.value = t('screenshot.loadFailed', { message: String(err) })
  } finally {
    loading.value = false
  }
}

async function openFolder(path: string) {
  try {
    await showInFolder(path)
  } catch (err) {
    error.value = t('screenshot.folderFailed', { message: String(err) })
    selected.value = null
  }
}

function formatSize(bytes: number): string {
  if (bytes < 1024 * 1024) return `${Math.max(1, Math.round(bytes / 1024))} KB`
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`
}
function formatDate(ms: number): string {
  return new Intl.DateTimeFormat(locale.value, {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(ms)
}

onMounted(async () => {
  window.addEventListener('focus', loadRecent)
  try {
    const stop = await listen('screenshot-saved', loadRecent)
    if (disposed) stop()
    else unlisten = stop
  } catch (err) {
    console.error('Screenshot listener failed:', err)
  }
  if (!disposed) await loadRecent()
})
onUnmounted(() => {
  disposed = true
  unlisten?.()
  window.removeEventListener('focus', loadRecent)
})
</script>

<style scoped>
.screenshot-page { height: 100%; overflow: auto; color: var(--text); background: var(--bg); container-type: inline-size; }
.screenshot-body { padding: clamp(24px, 3.5vw, 48px); }
h1, h2, h3, p { margin: 0; }
.capture-header { display: flex; justify-content: space-between; align-items: center; gap: 24px; padding-bottom: 24px; }
.section-label { display: block; color: var(--text-muted); font-size: 10px; font-weight: 600; letter-spacing: 2px; line-height: 1.6; text-transform: uppercase; }
h1 { display: flex; align-items: center; gap: 12px; font-size: 21px; font-weight: 600; letter-spacing: -0.5px; }
.title-dot { width: 6px; height: 6px; border-radius: 50%; background: var(--accent); }
.capture-actions { display: flex; flex-direction: column; align-items: center; gap: 10px; flex-shrink: 0; }
.shortcut { display: flex; align-items: center; gap: 6px; color: var(--text-muted); font: 10px var(--jedi-font-mono); }
.gallery-toolbar { display: flex; justify-content: space-between; align-items: center; gap: 16px; border-top: 1px solid var(--border); border-bottom: 1px solid var(--border); padding: 12px 0; }
.collection-summary, .gallery-controls { display: flex; align-items: center; gap: 10px; }
.collection-summary > .v-icon { color: var(--text-muted); }
.collection-label { font-size: 13px; font-weight: 600; }
.count { border: 1px solid var(--border); border-radius: 6px; min-width: 26px; padding: 2px 6px; text-align: center; font: 10px var(--jedi-font-mono); color: var(--text-muted); }
.density-control { display: flex; gap: 12px; align-items: center; color: var(--text-muted); font-size: 11px; }
.density-control input { width: 88px; accent-color: var(--accent); }
.date-heading { display: flex; align-items: center; gap: 12px; margin: 28px 0 18px; }
.date-heading h3 { font-size: 12px; font-weight: 500; }
.date-heading > span { color: var(--text-muted); font: 10px var(--jedi-font-mono); }
.date-heading > div { height: 1px; flex: 1; background: var(--border); }
.photo-columns { display: grid; grid-template-columns: repeat(auto-fill, minmax(var(--thumbnail-size), 1fr)); gap: 24px; }
.capture-open { display: block; width: 100%; text-align: left; color: inherit; border-radius: inherit; }
.capture-card { display: block; width: 100%; margin: 0; padding: 0; text-align: left; color: inherit; background: transparent; border: 0; border-radius: 12px; cursor: pointer; animation: photo-reveal var(--motion-enter) var(--ease-out) var(--card-delay) backwards; }
.capture-open:focus-visible { outline: 2px solid var(--accent); outline-offset: 3px; }
.thumbnail { position: relative; aspect-ratio: 16 / 10; border-radius: 10px; overflow: hidden; }
.thumbnail img { display: block; width: 100%; height: 100%; object-fit: contain; border-radius: 8px; box-shadow: 0 4px 14px rgb(var(--ink-rgb) / .18); transition: box-shadow var(--motion-normal); }
.capture-card:hover .thumbnail img { box-shadow: 0 6px 18px rgb(var(--ink-rgb) / .24); }
.thumb-broken { position: absolute; inset: 0; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 8px; padding: 16px; text-align: center; color: var(--text-muted); }
.thumb-broken-text { font-size: 12px; line-height: 1.6; }
.thumb-retry { padding: 4px 12px; font-size: 12px; color: var(--text-muted); background: transparent; border: 1px solid var(--border); border-radius: 6px; cursor: pointer; transition: border-color var(--motion-fast), color var(--motion-fast); }
.thumb-retry:hover { border-color: var(--border-strong); color: var(--text); }
.thumb-retry:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
.preview-hint { position: absolute; display: flex; align-items: center; gap: 6px; right: 10px; bottom: 10px; padding: 7px 10px; font-size: 11px; border: 1px solid var(--border); border-radius: 8px; background: var(--surface-raised); opacity: 0; transform: translateY(4px); transition: opacity var(--motion-fast), transform var(--motion-fast); }
.capture-card:hover .preview-hint, .capture-open:focus-visible .preview-hint { opacity: 1; transform: none; }
.card-info { padding: 10px 2px 0; }
.capture-name { display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 12px; font-weight: 500; }
.card-meta { display: flex; justify-content: space-between; gap: 8px; margin-top: 7px; font-size: 10px; color: var(--text-muted); }
.empty-state { min-height: 370px; display: flex; flex-direction: column; align-items: center; justify-content: center; text-align: center; }
.empty-frames { display: flex; align-items: center; margin-bottom: 30px; height: 120px; }
.empty-frames span { display: grid; place-items: center; width: 94px; height: 110px; border: 1px solid var(--border-strong); border-radius: 10px; background: var(--surface-raised); box-shadow: var(--shadow-md); color: var(--accent); }
.empty-frames span:first-child { transform: rotate(-12deg) translateX(18px); background: var(--surface-canvas); }
.empty-frames span:last-child { transform: rotate(12deg) translateX(-18px); background: var(--surface-canvas); }
.empty-frames span:nth-child(2) { z-index: 1; }
.empty-state h3 { font-size: 16px; font-weight: 550; }
.empty-state p { color: var(--text-muted); font-size: 12px; margin-top: 8px; }
.empty-state .v-btn { margin-top: 22px; }
.gallery-skeleton { display: grid; grid-template-columns: repeat(auto-fill, minmax(220px, 1fr)); gap: 24px; padding-top: 28px; }
.skeleton-frame { height: 220px; border: 1px solid var(--border); border-radius: 12px; background: var(--surface-canvas); animation: gallery-pulse 1.5s ease-in-out infinite alternate; }
.preview-panel { color: var(--text); overflow: auto !important; }
.preview-header, .preview-footer { display: flex; align-items: center; justify-content: space-between; gap: 16px; padding: 18px 24px; }
.preview-header h2 { margin-top: 5px; overflow-wrap: anywhere; font-size: 13px; font-weight: 600; }
.pixel-size { display: block; margin-top: 4px; color: var(--text-muted); font: 11px var(--jedi-font-mono); }
.preview-header { border-bottom: 1px solid var(--border); }
.preview-image { min-height: 250px; padding: 40px; background: var(--surface-canvas); display: grid; place-items: center; overflow: auto; }
.preview-image img { max-width: 100%; max-height: 60vh; object-fit: contain; box-shadow: var(--shadow-lg); }
.preview-image.is-actual { max-height: 60vh; display: block; cursor: grab; user-select: none; touch-action: none; }
.preview-image.is-actual img { display: block; max-width: none; max-height: none; }
.preview-image.is-actual.is-panning { cursor: grabbing; }
.preview-footer { flex-wrap: wrap; color: var(--text-muted); font-size: 11px; border-top: 1px solid var(--border); }
.preview-footer > div { display: flex; flex-wrap: wrap; align-items: center; gap: 8px; }
.zoom-toggle { display: flex; gap: 4px; }
@keyframes photo-reveal { from { opacity: 0; transform: translateY(8px); } to { opacity: 1; transform: none; } }
@keyframes gallery-pulse { to { opacity: .45; } }
/* In-page layout responds to the real content-area width (container query), not the viewport */
@container (max-width: 700px) { .screenshot-body { padding: 24px 18px; } .capture-header { align-items: flex-start; flex-direction: column; } .capture-actions { flex-direction: row; } .density-control { display: none; } }
/* v-dialog content is teleported to the viewport-level overlay container, outside .screenshot-page; keep it viewport-based */
@media (max-width: 700px) { .preview-image { padding: 20px; } }
@media (prefers-reduced-motion: reduce) { .capture-card, .skeleton-frame { animation: none; } .capture-card, .preview-hint, .thumbnail img { transition: none; } }
</style>
