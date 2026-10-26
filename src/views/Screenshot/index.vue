<template>
  <main class="screenshot-page" :style="{ '--thumbnail-size': `${thumbnailSize}px` }">
    <div class="screenshot-body">
      <header class="capture-header">
        <div class="heading">
          <span class="eyebrow">
            <v-icon :icon="mdiImageMultipleOutline" size="15" aria-hidden="true" />
            {{ t('screenshot.collectionLabel') }}
          </span>
          <h1>{{ t('screenshot.galleryTitle') }}</h1>
          <p>{{ t('screenshot.galleryDescription') }}</p>
        </div>
        <div class="capture-actions">
          <v-btn class="capture-button" color="primary" rounded="lg" size="large" :prepend-icon="mdiCropFree" :loading="capturing" @click="startCapture">{{ t('screenshot.start') }}</v-btn>
          <span class="shortcut"><v-icon :icon="mdiKeyboardOutline" size="15" aria-hidden="true" /><kbd>Ctrl</kbd><span>+</span><kbd>Alt</kbd><span>+</span><kbd>S</kbd></span>
        </div>
      </header>
      <v-alert v-if="error" class="gallery-alert" type="error" variant="tonal" closable @click:close="error = ''">{{ error }}</v-alert>
      <section class="gallery" :aria-label="t('screenshot.recent')" :aria-busy="loading">
        <div class="gallery-toolbar">
          <div class="collection-summary">
            <span class="toolbar-icon"><v-icon :icon="mdiViewGridOutline" size="18" aria-hidden="true" /></span>
            <span><strong>{{ t('screenshot.recent') }}</strong><small>{{ recent.length.toString().padStart(2, '0') }}</small></span>
          </div>
          <div class="gallery-controls">
            <label class="density-control"><v-icon :icon="mdiImageSizeSelectLarge" size="16" aria-hidden="true" /><span>{{ t('screenshot.thumbnailSize') }}</span><input v-model.number="thumbnailSize" type="range" min="220" max="380" step="20" /></label>
            <span class="toolbar-divider" aria-hidden="true"></span>
            <v-btn :icon="mdiRefresh" variant="text" density="comfortable" :loading="loading" :aria-label="t('common.refresh')" @click="loadRecent" />
          </div>
        </div>
        <div v-if="recent.length" class="gallery-collections">
          <section v-for="group in groups" :key="group.key" class="date-group" :aria-label="group.title">
            <div class="date-heading"><span class="date-mark" aria-hidden="true"></span><h3>{{ group.title }}</h3><span>{{ group.items.length.toString().padStart(2, '0') }}</span><div></div></div>
            <div class="photo-columns">
              <article v-for="(item, index) in group.items" :key="item.path" class="capture-card" :style="{ '--card-delay': `${Math.min(index, 5) * 35}ms` }">
                <button class="capture-open" type="button" @click="selected = item; beautifying = false">
                  <div class="thumbnail">
                    <div class="image-mat">
                      <img v-if="!brokenImages.has(item.path)" :key="`${item.path}:${imageRetries[item.path] || 0}`" :src="thumbSrc(item.path)" :alt="item.name" loading="lazy" draggable="false" @error="onThumbError(item.path)" />
                      <div v-else class="thumb-broken">
                        <v-icon :icon="mdiImageOffOutline" size="24" aria-hidden="true" />
                        <span class="thumb-broken-text">{{ t('screenshot.imageMissing') }}</span>
                      </div>
                    </div>
                    <span v-if="!brokenImages.has(item.path)" class="preview-hint"><v-icon :icon="mdiMagnifyPlusOutline" size="17" aria-hidden="true" />{{ t('screenshot.preview') }}</span>
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
    <v-dialog :model-value="!!selected" max-width="1440" transition="dialog-bottom-transition" @update:model-value="selected = null">
      <div v-if="selected" class="preview-panel" :class="{ 'is-beautifying': beautifying }">
        <header class="preview-header">
          <div class="preview-title">
            <span class="section-label">{{ t('screenshot.collectionLabel') }}</span>
            <h2>{{ selected.name }}</h2>
            <span class="pixel-size"><template v-if="previewPixelLabel">{{ previewPixelLabel }} · </template>{{ formatSize(selected.size) }}</span>
          </div>
          <div class="mode-switch" :aria-label="t('screenshot.preview')">
            <button type="button" :class="{ active: !beautifying }" :aria-pressed="!beautifying" @click="beautifying = false"><v-icon :icon="mdiEyeOutline" size="16" aria-hidden="true" />{{ t('screenshot.preview') }}</button>
            <button type="button" :class="{ active: beautifying }" :aria-pressed="beautifying" @click="beautifying = true"><v-icon :icon="mdiAutoFix" size="16" aria-hidden="true" />{{ t('screenshot.beautify') }}</button>
          </div>
          <v-btn class="close-preview" :icon="mdiClose" variant="text" :aria-label="t('screenshot.closePreview')" @click="selected = null" />
        </header>
        <BeautifyEditor v-if="beautifying" :path="selected.path" @saved="onStyledSaved" />
        <div v-else class="preview-workspace" :class="{ 'has-ocr': showOcr }">
          <div class="preview-image" tabindex="0" :aria-label="t('screenshot.preview')" :class="{ 'is-actual': previewZoom === 'actual', 'is-panning': panning }" @pointerdown="onPanStart" @pointermove="onPanMove" @pointerup="onPanEnd" @pointercancel="onPanEnd" @lostpointercapture="onPanEnd">
            <div v-if="previewFailed" role="alert" class="thumb-broken"><span>{{ t('screenshot.imageMissing') }}</span><v-btn variant="text" @click="retryThumb(selected.path); previewFailed = false">{{ t('screenshot.retryImage') }}</v-btn></div>
            <img v-else :src="thumbSrc(selected.path)" :alt="selected.name" draggable="false" :style="previewStyle" @load="onPreviewLoad" @error="previewFailed = true" />
          </div>
          <OcrPanel v-if="showOcr" :path="selected.path" />
        </div>
        <footer class="preview-footer">
          <span>{{ formatDate(selected.modifiedMs) }}</span>
          <div v-if="!beautifying" class="preview-actions">
            <div v-if="!beautifying" class="zoom-toggle">
              <v-btn size="small" :variant="previewZoom === 'fit' ? 'tonal' : 'text'" :color="previewZoom === 'fit' ? 'primary' : undefined" @click="previewZoom = 'fit'">{{ t('screenshot.fitWindow') }}</v-btn>
              <v-btn size="small" :variant="previewZoom === 'actual' ? 'tonal' : 'text'" :color="previewZoom === 'actual' ? 'primary' : undefined" :disabled="!previewPixels" @click="previewZoom = 'actual'">{{ t('screenshot.actualSize') }}</v-btn>
            </div>
            <span class="footer-divider" aria-hidden="true"></span>
            <v-btn variant="tonal" :prepend-icon="mdiTextRecognition" :aria-pressed="showOcr" @click="showOcr = !showOcr">{{ t('screenshot.ocr.title') }}</v-btn>
            <v-btn variant="text" :prepend-icon="mdiFolderOpenOutline" @click="openFolder(selected.path)">{{ t('screenshot.openFolder') }}</v-btn>
          </div>
          <span v-else>{{ t('screenshot.saveAsCopy') }}</span>
        </footer>
      </div>
    </v-dialog>
  </main>
</template>

<script setup lang="ts">
import {
  mdiAutoFix,
  mdiClose,
  mdiCrop,
  mdiCropFree,
  mdiEyeOutline,
  mdiFolderOpenOutline,
  mdiImageMultipleOutline,
  mdiImageOffOutline,
  mdiImageSizeSelectLarge,
  mdiKeyboardOutline,
  mdiMagnifyPlusOutline,
  mdiRefresh,
  mdiTextRecognition,
  mdiViewGridOutline,
} from '@mdi/js'
import { convertFileSrc } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { listScreenshots, type ScreenshotItem, screenshotBegin } from '@/api/screenshot'
import { showInFolder } from '@/api/wallpaper'
import BeautifyEditor from './BeautifyEditor.vue'
import OcrPanel from './OcrPanel.vue'

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
const showOcr = ref(false)
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
let pendingOcrPath = ''

async function openCapturedOcr(event: Event) {
  const path = (event as CustomEvent<string>).detail
  if (!path) return
  pendingOcrPath = path
  await loadRecent()
  await revealPendingOcr()
}

async function revealPendingOcr() {
  if (!pendingOcrPath) return
  const item = recent.value.find(item => item.path === pendingOcrPath)
  if (!item) return
  pendingOcrPath = ''
  selected.value = item
  beautifying.value = false
  await nextTick()
  showOcr.value = true
}

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
  showOcr.value = false
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
    await revealPendingOcr()
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
  window.addEventListener('jedi-screenshot-ocr', openCapturedOcr)
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
  window.removeEventListener('jedi-screenshot-ocr', openCapturedOcr)
})
</script>

<style scoped>
.screenshot-page {
  height: 100%;
  overflow: auto;
  color: var(--text);
  background:
    radial-gradient(circle at 84% 2%, rgb(var(--accent-rgb) / 0.08), transparent 28rem),
    var(--bg);
  container-type: inline-size;
}
.screenshot-body { width: min(1500px, 100%); min-height: 100%; margin: 0 auto; padding: clamp(28px, 4vw, 58px); }
h1, h2, h3, p { margin: 0; }
.capture-header { display: flex; justify-content: space-between; align-items: flex-end; gap: 32px; padding: 8px 2px 30px; }
.heading { max-width: 620px; }
.eyebrow, .section-label { display: flex; align-items: center; gap: 8px; color: var(--accent); font: 600 10px/1.6 var(--jedi-font-mono); letter-spacing: 1.6px; text-transform: uppercase; }
h1 { margin-top: 8px; font-size: clamp(27px, 3vw, 38px); font-weight: 650; letter-spacing: -1.4px; }
.heading p { max-width: 520px; margin-top: 10px; color: var(--text-muted); font-size: 13px; line-height: 1.7; }
.capture-actions { display: flex; flex-direction: column; align-items: center; gap: 9px; flex-shrink: 0; }
.capture-button { min-width: 142px; box-shadow: 0 10px 30px rgb(var(--accent-rgb) / 0.22); }
.shortcut { display: flex; align-items: center; gap: 4px; color: var(--text-muted); font: 10px var(--jedi-font-mono); }
.shortcut kbd { min-width: 22px; padding: 2px 6px; border: 1px solid var(--border); border-bottom-color: var(--border-strong); border-radius: 5px; background: var(--surface-raised); color: var(--text-muted); font: inherit; text-align: center; box-shadow: 0 1px 0 rgb(var(--text-rgb) / 0.08); }
.gallery-alert { margin-bottom: 18px; }
.gallery { padding-bottom: 40px; }
.gallery-toolbar { position: sticky; top: 0; z-index: 5; display: flex; justify-content: space-between; align-items: center; gap: 18px; min-height: 58px; padding: 9px 12px; border: 1px solid rgb(var(--text-rgb) / 0.08); border-radius: 14px; background: rgb(var(--bg-rgb) / 0.82); box-shadow: 0 12px 36px rgb(var(--ink-rgb) / 0.08); backdrop-filter: blur(18px) saturate(140%); }
.collection-summary, .gallery-controls, .density-control { display: flex; align-items: center; }
.collection-summary { gap: 11px; }
.toolbar-icon { display: grid; width: 36px; height: 36px; place-items: center; border: 1px solid rgb(var(--accent-rgb) / 0.22); border-radius: 10px; background: rgb(var(--accent-rgb) / 0.1); color: var(--accent); }
.collection-summary > span:last-child { display: flex; flex-direction: column; gap: 1px; }
.collection-summary strong { font-size: 12px; font-weight: 650; }
.collection-summary small { color: var(--text-muted); font: 10px var(--jedi-font-mono); }
.gallery-controls { gap: 8px; }
.density-control { gap: 10px; padding: 0 8px; color: var(--text-muted); font-size: 11px; }
.density-control input { width: 94px; accent-color: var(--accent); cursor: pointer; }
.toolbar-divider, .footer-divider { width: 1px; height: 22px; background: var(--border); }
.date-heading { display: flex; align-items: center; gap: 10px; margin: 32px 2px 15px; }
.date-mark { width: 5px; height: 5px; border-radius: 50%; background: var(--accent); box-shadow: 0 0 0 5px rgb(var(--accent-rgb) / 0.1); }
.date-heading h3 { font-size: 12px; font-weight: 600; }
.date-heading > span:not(.date-mark) { color: var(--text-muted); font: 10px var(--jedi-font-mono); }
.date-heading > div { height: 1px; flex: 1; background: linear-gradient(90deg, var(--border), transparent); }
.photo-columns { display: grid; grid-template-columns: repeat(auto-fill, minmax(min(var(--thumbnail-size), 100%), 1fr)); gap: clamp(16px, 2vw, 24px); }
.capture-card { position: relative; min-width: 0; border: 1px solid rgb(var(--text-rgb) / 0.08); border-radius: 16px; background: rgb(var(--text-rgb) / 0.025); overflow: hidden; animation: photo-reveal var(--motion-enter) var(--ease-out) var(--card-delay) backwards; transition: border-color var(--motion-fast), background var(--motion-fast), transform var(--motion-fast), box-shadow var(--motion-normal); }
.capture-card:hover { border-color: rgb(var(--accent-rgb) / 0.32); background: rgb(var(--text-rgb) / 0.045); transform: translateY(-3px); box-shadow: 0 16px 38px rgb(var(--ink-rgb) / 0.16); }
.capture-open { display: block; width: 100%; padding: 0; border: 0; border-radius: inherit; background: transparent; color: inherit; text-align: left; cursor: pointer; }
.capture-open:focus-visible { outline: 2px solid var(--accent); outline-offset: -2px; }
.thumbnail { position: relative; aspect-ratio: 16 / 10; margin: 8px 8px 0; border-radius: 11px; overflow: hidden; background:
    linear-gradient(135deg, rgb(var(--text-rgb) / 0.035), transparent 60%),
    var(--surface-canvas); }
.thumbnail::before { position: absolute; inset: 0; content: ''; background-image: radial-gradient(rgb(var(--text-rgb) / 0.08) 0.7px, transparent 0.7px); background-size: 10px 10px; opacity: 0.45; }
.image-mat { position: absolute; inset: 12px; display: grid; place-items: center; min-width: 0; min-height: 0; }
.thumbnail img { position: relative; display: block; width: 100%; height: 100%; object-fit: contain; filter: saturate(0.96); box-shadow: 0 10px 24px rgb(var(--ink-rgb) / 0.22); transition: filter var(--motion-normal), transform var(--motion-normal), box-shadow var(--motion-normal); }
.capture-card:hover .thumbnail img { filter: saturate(1.03); transform: scale(1.012); box-shadow: 0 14px 30px rgb(var(--ink-rgb) / 0.28); }
.thumb-broken { position: absolute; inset: 0; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 8px; padding: 16px; text-align: center; color: var(--text-muted); }
.thumb-broken-text { font-size: 12px; line-height: 1.6; }
.thumb-retry { position: absolute; top: 18px; right: 18px; padding: 5px 10px; border: 1px solid var(--border); border-radius: 7px; background: var(--surface-raised); color: var(--text-muted); font-size: 11px; cursor: pointer; }
.thumb-retry:hover { border-color: var(--accent); color: var(--text); }
.thumb-retry:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
.preview-hint { position: absolute; right: 10px; bottom: 10px; display: flex; align-items: center; gap: 6px; padding: 7px 10px; border: 1px solid rgb(var(--text-rgb) / 0.12); border-radius: 8px; background: rgb(var(--bg-rgb) / 0.84); color: var(--text); font-size: 11px; opacity: 0; transform: translateY(5px); backdrop-filter: blur(12px); transition: opacity var(--motion-fast), transform var(--motion-fast); }
.capture-card:hover .preview-hint, .capture-open:focus-visible .preview-hint { opacity: 1; transform: none; }
.card-info { padding: 14px 15px 15px; }
.capture-name { display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 12px; font-weight: 600; }
.card-meta { display: flex; justify-content: space-between; gap: 8px; margin-top: 7px; color: var(--text-muted); font: 10px var(--jedi-font-mono); }
.empty-state { min-height: 400px; display: flex; flex-direction: column; align-items: center; justify-content: center; text-align: center; }
.empty-frames { display: flex; align-items: center; height: 120px; margin-bottom: 30px; }
.empty-frames span { display: grid; width: 94px; height: 110px; place-items: center; border: 1px solid var(--border-strong); border-radius: 14px; background: var(--surface-raised); color: var(--accent); box-shadow: var(--shadow-md); }
.empty-frames span:first-child { transform: rotate(-10deg) translateX(18px); background: var(--surface-canvas); }
.empty-frames span:last-child { transform: rotate(10deg) translateX(-18px); background: var(--surface-canvas); }
.empty-frames span:nth-child(2) { z-index: 1; }
.empty-state h3 { font-size: 17px; font-weight: 650; }
.empty-state p { max-width: 400px; margin-top: 8px; color: var(--text-muted); font-size: 12px; line-height: 1.7; }
.empty-state .v-btn { margin-top: 22px; }
.gallery-skeleton { display: grid; grid-template-columns: repeat(auto-fill, minmax(240px, 1fr)); gap: 20px; padding-top: 32px; }
.skeleton-frame { aspect-ratio: 16 / 11; border: 1px solid var(--border); border-radius: 16px; background: linear-gradient(105deg, var(--surface-canvas) 25%, rgb(var(--text-rgb) / 0.065) 40%, var(--surface-canvas) 58%); background-size: 220% 100%; animation: gallery-pulse 1.6s ease-in-out infinite; }
.preview-panel { display: grid; grid-template-rows: auto minmax(0, 1fr) auto; width: min(1440px, 94vw); height: min(880px, 92vh); overflow: hidden; border: 1px solid rgb(var(--text-rgb) / 0.12); border-radius: 18px; background: var(--bg); color: var(--text); box-shadow: 0 30px 90px rgb(var(--ink-rgb) / 0.42); }
.preview-header, .preview-footer { display: flex; align-items: center; gap: 18px; padding: 14px 18px; background: rgb(var(--bg-rgb) / 0.9); backdrop-filter: blur(18px); }
.preview-header { min-height: 72px; border-bottom: 1px solid var(--border); }
.preview-title { min-width: 0; flex: 1; }
.preview-header h2 { max-width: 520px; margin-top: 3px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 13px; font-weight: 650; }
.pixel-size { display: block; margin-top: 3px; color: var(--text-muted); font: 10px var(--jedi-font-mono); }
.mode-switch { display: flex; gap: 3px; padding: 4px; border: 1px solid var(--border); border-radius: 11px; background: var(--surface-canvas); }
.mode-switch button { display: flex; align-items: center; gap: 7px; min-height: 34px; padding: 0 13px; border: 0; border-radius: 7px; background: transparent; color: var(--text-muted); font: 600 11px var(--jedi-font-ui); cursor: pointer; transition: color var(--motion-fast), background var(--motion-fast), box-shadow var(--motion-fast); }
.mode-switch button:hover { color: var(--text); }
.mode-switch button.active { background: var(--surface-raised); color: var(--text); box-shadow: 0 3px 10px rgb(var(--ink-rgb) / 0.14); }
.mode-switch button:focus-visible { outline: 2px solid var(--accent); outline-offset: 1px; }
.close-preview { margin-left: auto; }
.preview-workspace { min-height: 0; display: grid; grid-template-columns: minmax(0, 1fr); }
.preview-workspace.has-ocr { grid-template-columns: minmax(0, 1fr) minmax(300px, 360px); }
.preview-image { min-width: 0; min-height: 320px; display: grid; place-items: center; overflow: auto; padding: clamp(26px, 4vw, 56px); background:
    radial-gradient(circle at 50% 35%, rgb(var(--accent-rgb) / 0.075), transparent 34rem),
    radial-gradient(rgb(var(--text-rgb) / 0.07) 0.7px, transparent 0.7px),
    var(--surface-canvas); background-size: auto, 12px 12px, auto; }
.preview-image img { max-width: 100%; max-height: 68vh; object-fit: contain; box-shadow: 0 24px 64px rgb(var(--ink-rgb) / 0.34), 0 0 0 1px rgb(var(--text-rgb) / 0.1); }
.preview-image.is-actual { display: block; cursor: grab; user-select: none; touch-action: none; }
.preview-image.is-actual img { display: block; max-width: none; max-height: none; }
.preview-image.is-actual.is-panning { cursor: grabbing; }
.preview-footer { min-height: 64px; justify-content: space-between; flex-wrap: wrap; border-top: 1px solid var(--border); color: var(--text-muted); font: 10px var(--jedi-font-mono); }
.preview-actions, .zoom-toggle { display: flex; align-items: center; gap: 6px; }
@keyframes photo-reveal { from { opacity: 0; transform: translateY(10px) scale(0.985); } to { opacity: 1; transform: none; } }
@keyframes gallery-pulse { from { background-position: 100% 0; } to { background-position: -120% 0; } }
@container (max-width: 760px) {
  .screenshot-body { padding: 24px 18px 40px; }
  .capture-header { align-items: flex-start; flex-direction: column; }
  .capture-actions { width: 100%; align-items: stretch; }
  .capture-button { width: 100%; }
  .shortcut { justify-content: center; }
  .density-control span, .density-control > .v-icon, .toolbar-divider { display: none; }
}
@media (max-width: 880px) {
  .preview-panel { width: 96vw; height: 94vh; }
  .preview-header { flex-wrap: wrap; }
  .mode-switch { order: 3; width: 100%; }
  .mode-switch button { flex: 1; justify-content: center; }
  .preview-workspace.has-ocr { grid-template-columns: 1fr; overflow: auto; }
  .preview-image { min-height: 300px; padding: 22px; }
  .preview-footer { align-items: flex-start; }
  .preview-actions { width: 100%; flex-wrap: wrap; }
  .footer-divider { display: none; }
}
@media (prefers-reduced-motion: reduce) {
  .capture-card, .skeleton-frame { animation: none; }
  .capture-card, .preview-hint, .thumbnail img, .mode-switch button { transition: none; }
}
</style>
