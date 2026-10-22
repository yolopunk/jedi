<template>
  <main class="screenshot-page">
    <header class="capture-header">
      <div class="heading"><span class="app-mark"><v-icon :icon="mdiCropFree" /></span><div><h1>{{ t('screenshot.title') }}</h1><p>{{ t('screenshot.galleryDescription') }}</p></div></div>
      <v-btn color="primary" rounded="lg" :prepend-icon="mdiPlus" :loading="capturing" @click="startCapture">{{ t('screenshot.start') }}</v-btn>
    </header>
    <div class="capture-note"><v-icon :icon="mdiKeyboardOutline" size="18" /><span>{{ t('screenshot.shortcutHint') }}</span></div>
    <v-alert v-if="error" type="error" variant="tonal" closable @click:close="error = ''">{{ error }}</v-alert>
    <section class="gallery" :aria-label="t('screenshot.recent')">
      <div class="gallery-header"><div><h2>{{ t('screenshot.recent') }}</h2><span class="count">{{ recent.length }}</span></div><v-btn :icon="mdiRefresh" variant="text" size="small" :loading="loading" :aria-label="t('common.refresh')" @click="loadRecent" /></div>
      <div v-if="recent.length" class="recent-grid">
        <button v-for="item in recent" :key="item.path" class="capture-card" type="button" @click="selected = item">
          <div class="thumbnail"><img :src="convertFileSrc(item.path)" :alt="item.name" loading="lazy" draggable="false" /><span class="preview-hint"><v-icon :icon="mdiMagnifyPlusOutline" size="20" />{{ t('screenshot.preview') }}</span></div>
          <div class="card-info"><span class="capture-name">{{ item.name }}</span><span class="card-meta"><time>{{ formatDate(item.modifiedMs) }}</time><span>{{ formatSize(item.size) }}</span></span></div>
        </button>
      </div>
      <div v-else-if="!loading" class="empty-state"><span class="empty-icon"><v-icon :icon="mdiImageMultipleOutline" size="36" /></span><h3>{{ t('screenshot.recentEmpty') }}</h3><p>{{ t('screenshot.galleryDescription') }}</p><v-btn variant="tonal" color="primary" :prepend-icon="mdiCrop" :loading="capturing" @click="startCapture">{{ t('screenshot.start') }}</v-btn></div>
      <v-progress-linear v-else indeterminate color="primary" :aria-label="t('common.refresh')" />
    </section>
    <v-dialog :model-value="!!selected" max-width="1120" @update:model-value="selected = null">
      <div v-if="selected" class="preview-panel">
        <header class="preview-header"><span>{{ selected.name }}</span><v-btn :icon="mdiClose" variant="text" :aria-label="t('screenshot.closePreview')" @click="selected = null" /></header>
        <div class="preview-image"><img :src="convertFileSrc(selected.path)" :alt="selected.name" /></div>
        <footer class="preview-footer"><span>{{ formatDate(selected.modifiedMs) }} · {{ formatSize(selected.size) }}</span><v-btn variant="tonal" :prepend-icon="mdiFolderOpenOutline" @click="openFolder(selected.path)">{{ t('screenshot.openFolder') }}</v-btn></footer>
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
import { onMounted, onUnmounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { listScreenshots, type ScreenshotItem, screenshotBegin } from '@/api/screenshot'
import { showInFolder } from '@/api/wallpaper'

const { t, locale } = useI18n()
const recent = ref<ScreenshotItem[]>([])
const selected = ref<ScreenshotItem | null>(null)
const capturing = ref(false)
const loading = ref(false)
const error = ref('')
let unlisten: UnlistenFn | undefined
let disposed = false

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
.screenshot-page { height: 100%; overflow: auto; padding: 32px; color: var(--text); background: var(--bg); }
.capture-header, .heading, .gallery-header, .gallery-header > div, .preview-header, .preview-footer { display: flex; align-items: center; }
.capture-header, .gallery-header, .preview-header, .preview-footer { justify-content: space-between; gap: 16px; }
.heading { gap: 16px; min-width: 0; }
h1, h2, h3, p { margin: 0; }
h1 { font-size: 25px; font-weight: 650; letter-spacing: -.8px; }
.heading p, .empty-state p { margin-top: 6px; color: var(--text-muted); font-size: 13px; line-height: 1.6; }
.app-mark, .empty-icon { display: grid; place-items: center; background: color-mix(in srgb, var(--accent) 12%, var(--bg)); color: var(--accent); border: 1px solid color-mix(in srgb, var(--accent) 20%, var(--border)); border-radius: 16px; }
.app-mark { width: 52px; height: 52px; flex-shrink: 0; }
.capture-note { display: flex; gap: 8px; align-items: center; color: var(--text-muted); font-size: 12px; padding: 20px 0 28px; }
.gallery { border-top: 1px solid var(--border); }
.gallery-header { padding: 20px 0; }
.gallery-header > div { gap: 10px; }
h2 { font-size: 15px; font-weight: 600; }
.count { font-size: 11px; padding: 2px 8px; background: var(--bg-elevated-2); border-radius: 6px; }
.recent-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(240px, 1fr)); gap: 20px; }
.capture-card { text-align: left; color: inherit; background: var(--bg-elevated); border: 1px solid var(--border); border-radius: 14px; overflow: hidden; cursor: pointer; transition: border-color .18s, box-shadow .18s; }
.capture-card:hover, .capture-card:focus-visible { border-color: var(--accent); box-shadow: 0 6px 24px rgb(0 0 0 / .08); }
.capture-card:focus-visible { outline: 2px solid var(--accent); outline-offset: 3px; }
.thumbnail { position: relative; height: 190px; padding: 18px; display: grid; place-items: center; background: color-mix(in srgb, var(--accent) 5%, var(--bg-elevated-2)); }
.thumbnail img { max-width: 100%; max-height: 154px; object-fit: contain; border-radius: 4px; box-shadow: 0 4px 14px rgb(0 0 0 / .14); }
.preview-hint { position: absolute; display: flex; align-items: center; gap: 6px; inset: auto 12px 12px auto; padding: 6px 10px; font-size: 12px; border-radius: 8px; background: var(--bg); opacity: 0; transition: opacity .18s; }
.capture-card:hover .preview-hint, .capture-card:focus-visible .preview-hint { opacity: 1; }
.card-info { padding: 14px 16px; }
.capture-name { display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 12px; font-weight: 550; }
.card-meta { display: flex; justify-content: space-between; gap: 8px; margin-top: 8px; font-size: 11px; color: var(--text-muted); }
.empty-state { min-height: 360px; display: flex; flex-direction: column; align-items: center; justify-content: center; text-align: center; }
.empty-icon { width: 80px; height: 80px; margin-bottom: 20px; border-radius: 24px; }
.empty-state h3 { font-size: 17px; font-weight: 550; }
.empty-state .v-btn { margin-top: 24px; }
.preview-panel { background: var(--bg); color: var(--text); border: 1px solid var(--border); border-radius: 16px; overflow: hidden; }
.preview-header, .preview-footer { padding: 12px 20px; font-size: 12px; }
.preview-header > span { overflow-wrap: anywhere; }
.preview-image { padding: 24px; background: var(--bg-elevated-2); display: grid; place-items: center; }
.preview-image img { max-width: 100%; max-height: 65vh; object-fit: contain; }
.preview-footer { flex-wrap: wrap; color: var(--text-muted); }
@media (max-width: 650px) { .screenshot-page { padding: 20px 16px; } .capture-header { align-items: flex-start; flex-direction: column; } .recent-grid { grid-template-columns: repeat(auto-fill, minmax(210px, 1fr)); gap: 14px; } }
@media (prefers-reduced-motion: reduce) { .capture-card, .preview-hint { transition: none; } }
</style>
