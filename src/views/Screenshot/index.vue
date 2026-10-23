<template>
  <main class="screenshot-page" :style="{ '--thumbnail-size': `${thumbnailSize}px` }">
    <header class="capture-header">
      <div class="heading">
        <span class="section-label">{{ t('screenshot.collectionLabel') }}</span>
        <h1>{{ t('screenshot.galleryTitle') }}<span class="title-dot" aria-hidden="true"></span></h1>
        <p>{{ t('screenshot.galleryDescription') }}</p>
      </div>
      <div class="capture-actions">
        <v-btn color="primary" rounded="lg" :prepend-icon="mdiPlus" :loading="capturing" @click="startCapture">{{ t('screenshot.start') }}</v-btn>
        <span class="shortcut"><v-icon :icon="mdiKeyboardOutline" size="16" aria-hidden="true" />Ctrl + Alt + S</span>
      </div>
    </header>
    <v-alert v-if="error" type="error" variant="tonal" closable @click:close="error = ''">{{ error }}</v-alert>
    <section class="gallery" :aria-label="t('screenshot.recent')" :aria-busy="loading">
      <div class="gallery-toolbar">
        <div class="collection-summary"><v-icon :icon="mdiImageMultipleOutline" size="18" aria-hidden="true" /><h2>{{ t('screenshot.recent') }}</h2><span class="count">{{ recent.length }}</span></div>
        <div class="gallery-controls">
          <label class="density-control">{{ t('screenshot.thumbnailSize') }}<input v-model.number="thumbnailSize" type="range" min="220" max="380" step="20" /></label>
          <v-btn :icon="mdiRefresh" variant="text" size="small" :loading="loading" :aria-label="t('common.refresh')" @click="loadRecent" />
        </div>
      </div>
      <div v-if="recent.length" class="gallery-collections">
        <section v-for="group in groups" :key="group.key" class="date-group" :aria-label="group.title">
          <div class="date-heading"><h3>{{ group.title }}</h3><span>{{ group.items.length.toString().padStart(2, '0') }}</span><div></div></div>
          <div class="photo-columns">
            <button v-for="(item, index) in group.items" :key="item.path" class="capture-card" type="button" :style="{ '--card-delay': `${Math.min(index, 5) * 35}ms` }" @click="selected = item; beautifying = false">
              <div class="thumbnail">
                <img :src="convertFileSrc(item.path)" :alt="item.name" loading="lazy" draggable="false" />
                <span class="preview-hint"><v-icon :icon="mdiMagnifyPlusOutline" size="18" aria-hidden="true" />{{ t('screenshot.preview') }}</span>
              </div>
              <div class="card-info"><span class="capture-name" :title="item.name">{{ item.name }}</span><span class="card-meta"><time>{{ formatDate(item.modifiedMs) }}</time><span>{{ formatSize(item.size) }}</span></span></div>
            </button>
          </div>
        </section>
      </div>
      <div v-else-if="loading" class="gallery-skeleton" :aria-label="t('common.refresh')"><div v-for="i in 6" :key="i" class="skeleton-frame"></div></div>
      <div v-else class="empty-state">
        <div class="empty-frames" aria-hidden="true"><span></span><span><v-icon :icon="mdiCropFree" size="32" /></span><span></span></div>
        <h3>{{ t('screenshot.recentEmpty') }}</h3><p>{{ t('screenshot.galleryDescription') }}</p>
        <v-btn variant="tonal" color="primary" :prepend-icon="mdiCrop" :loading="capturing" @click="startCapture">{{ t('screenshot.start') }}</v-btn>
      </div>
    </section>
    <v-dialog :model-value="!!selected" max-width="1200" @update:model-value="selected = null">
      <div v-if="selected" class="preview-panel">
        <header class="preview-header"><div><span class="section-label">{{ t(beautifying ? 'screenshot.beautify' : 'screenshot.preview') }}</span><h2>{{ selected.name }}</h2></div><v-btn :icon="mdiClose" variant="text" :aria-label="t('screenshot.closePreview')" @click="selected = null" /></header>
        <BeautifyEditor v-if="beautifying" :path="selected.path" @saved="onStyledSaved" />
        <div v-else class="preview-image"><img :src="convertFileSrc(selected.path)" :alt="selected.name" /></div>
        <footer class="preview-footer"><span>{{ formatDate(selected.modifiedMs) }} · {{ formatSize(selected.size) }}</span><div><v-btn variant="text" :prepend-icon="mdiFolderOpenOutline" @click="openFolder(selected.path)">{{ t('screenshot.openFolder') }}</v-btn><v-btn variant="tonal" color="primary" @click="beautifying = !beautifying">{{ t(beautifying ? 'screenshot.original' : 'screenshot.beautify') }}</v-btn></div></footer>
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
  mdiKeyboardOutline,
  mdiMagnifyPlusOutline,
  mdiPlus,
  mdiRefresh,
} from '@mdi/js'
import { convertFileSrc } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { computed, onMounted, onUnmounted, ref } from 'vue'
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
let unlisten: UnlistenFn | undefined
let disposed = false

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
.screenshot-page { height: 100%; overflow: auto; padding: clamp(24px, 3.5vw, 48px); color: var(--text); background: var(--bg); }
h1, h2, h3, p { margin: 0; }
.capture-header { display: flex; justify-content: space-between; align-items: center; gap: 24px; padding-bottom: 32px; }
.section-label { display: block; color: var(--text-muted); font-size: 10px; font-weight: 600; letter-spacing: 2px; line-height: 1.6; text-transform: uppercase; }
h1 { display: flex; align-items: center; gap: 12px; margin-top: 8px; font-size: 30px; font-weight: 600; letter-spacing: -1px; }
.title-dot { width: 6px; height: 6px; border-radius: 50%; background: var(--accent); }
.heading p { margin-top: 10px; color: var(--text-muted); font-size: 13px; line-height: 1.7; }
.capture-actions { display: flex; flex-direction: column; align-items: center; gap: 10px; flex-shrink: 0; }
.shortcut { display: flex; align-items: center; gap: 6px; color: var(--text-muted); font: 10px var(--jedi-font-mono); }
.gallery-toolbar { display: flex; justify-content: space-between; align-items: center; gap: 16px; border-top: 1px solid var(--border); border-bottom: 1px solid var(--border); padding: 12px 0; }
.collection-summary, .gallery-controls { display: flex; align-items: center; gap: 10px; }
.collection-summary > .v-icon { color: var(--text-muted); }
h2 { font-size: 13px; font-weight: 600; }
.count { border: 1px solid var(--border); border-radius: 6px; min-width: 26px; padding: 2px 6px; text-align: center; font: 10px var(--jedi-font-mono); color: var(--text-muted); }
.density-control { display: flex; gap: 12px; align-items: center; color: var(--text-muted); font-size: 11px; }
.density-control input { width: 88px; accent-color: var(--accent); }
.date-heading { display: flex; align-items: center; gap: 12px; margin: 28px 0 18px; }
.date-heading h3 { font-size: 12px; font-weight: 500; }
.date-heading > span { color: var(--text-muted); font: 10px var(--jedi-font-mono); }
.date-heading > div { height: 1px; flex: 1; background: var(--border); }
.photo-columns { columns: var(--thumbnail-size); column-gap: 24px; }
.capture-card { display: block; break-inside: avoid; width: 100%; margin: 0 0 24px; padding: 8px; text-align: left; color: inherit; background: var(--surface-raised); border: 1px solid var(--border); border-radius: 12px; cursor: pointer; box-shadow: var(--shadow-sm); transition: border-color var(--motion-normal), box-shadow var(--motion-normal), transform var(--motion-normal) var(--ease-out); animation: photo-reveal var(--motion-enter) var(--ease-out) var(--card-delay) backwards; }
.capture-card:hover { border-color: var(--border-strong); box-shadow: var(--shadow-lg); transform: translateY(-3px); }
.capture-card:focus-visible { outline: 2px solid var(--accent); outline-offset: 3px; }
.thumbnail { position: relative; min-height: 150px; padding: 24px; display: grid; place-items: center; background: var(--surface-canvas); border-radius: 7px; overflow: hidden; }
.thumbnail img { display: block; width: 100%; max-height: 340px; object-fit: contain; border-radius: 3px; box-shadow: 0 4px 14px rgb(var(--ink-rgb) / .18); }
.capture-card:nth-child(3n + 2) .thumbnail { background: color-mix(in srgb, var(--accent) 5%, var(--surface-canvas)); }
.capture-card:nth-child(3n) .thumbnail { background: color-mix(in srgb, var(--accent-2) 5%, var(--surface-canvas)); }
.preview-hint { position: absolute; display: flex; align-items: center; gap: 6px; right: 10px; bottom: 10px; padding: 7px 10px; font-size: 11px; border: 1px solid var(--border); border-radius: 8px; background: var(--surface-raised); opacity: 0; transform: translateY(4px); transition: opacity var(--motion-fast), transform var(--motion-fast); }
.capture-card:hover .preview-hint, .capture-card:focus-visible .preview-hint { opacity: 1; transform: none; }
.card-info { padding: 14px 6px 6px; }
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
.preview-header h2 { margin-top: 5px; overflow-wrap: anywhere; }
.preview-header { border-bottom: 1px solid var(--border); }
.preview-image { min-height: 250px; padding: 40px; background: var(--surface-canvas); display: grid; place-items: center; }
.preview-image img { max-width: 100%; max-height: 60vh; object-fit: contain; box-shadow: var(--shadow-lg); }
.preview-footer { flex-wrap: wrap; color: var(--text-muted); font-size: 11px; border-top: 1px solid var(--border); }
.preview-footer > div { display: flex; flex-wrap: wrap; gap: 8px; }
@keyframes photo-reveal { from { opacity: 0; transform: translateY(8px); } to { opacity: 1; transform: none; } }
@keyframes gallery-pulse { to { opacity: .45; } }
@media (max-width: 700px) { .capture-header { align-items: flex-start; flex-direction: column; } .capture-actions { flex-direction: row; } .density-control { display: none; } .screenshot-page { padding: 24px 18px; } .preview-image { padding: 20px; } }
@media (prefers-reduced-motion: reduce) { .capture-card, .skeleton-frame { animation: none; } .capture-card:hover { transform: none; } .capture-card, .preview-hint { transition: none; } }
</style>
