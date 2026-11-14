<template>
  <main class="screenshot-page" :style="{ '--thumbnail-size': `${thumbnailSize}px` }">
    <div class="screenshot-body">
      <header class="capture-header">
        <div class="heading">
          <span class="eyebrow-pill">
            <span class="pill-dot" aria-hidden="true"></span>
            {{ t('screenshot.collectionLabel') }}
          </span>
          <h1>{{ t('screenshot.galleryTitle') }}</h1>
          <p>{{ t('screenshot.galleryDescription') }}</p>
        </div>
        <div class="capture-actions">
          <v-btn
            class="capture-button"
            color="primary"
            rounded="lg"
            size="large"
            :loading="capturing"
            @click="startCapture"
          >
            <template #prepend><v-icon :icon="mdiCropFree" size="18" aria-hidden="true" /></template>
            {{ t('screenshot.start') }}
            <span v-for="part in hotkeyParts" :key="part" class="kbd-chip">{{ part }}</span>
          </v-btn>
          <v-btn class="import-button" rounded="lg" size="large" variant="outlined" :loading="importing" @click="importImage">{{ t('screenshot.import') }}</v-btn>
        </div>
      </header>
      <v-alert v-if="error" class="gallery-alert" type="error" variant="tonal" closable @click:close="error = ''">{{ error }}</v-alert>
      <section class="gallery" :aria-label="t('screenshot.recent')" :aria-busy="loading">
        <div class="gallery-toolbar">
          <div class="collection-summary">
            <span class="toolbar-icon"><v-icon :icon="mdiViewGridOutline" size="17" aria-hidden="true" /></span>
            <span class="summary-text">
              <strong>{{ t('screenshot.recent') }}</strong>
              <small>{{ recent.length.toString().padStart(2, '0') }} · {{ formatSize(totalSize) }}</small>
            </span>
          </div>

          <span class="toolbar-divider" aria-hidden="true"></span>

          <div class="filter-chips" role="group" :aria-label="t('screenshot.filter.label')">
            <button
              v-for="option in filterOptions"
              :key="option.kind"
              type="button"
              class="filter-chip"
              :class="{ active: filterKind === option.kind }"
              :aria-pressed="filterKind === option.kind"
              @click="filterKind = option.kind"
            >
              {{ option.label }}
              <span class="chip-count">{{ option.count.toString().padStart(2, '0') }}</span>
            </button>
          </div>

          <div class="gallery-controls">
            <label class="density-control">
              <v-icon :icon="mdiImageSizeSelectLarge" size="15" aria-hidden="true" />
              <span>{{ t('screenshot.thumbnailSize') }}</span>
              <input v-model.number="thumbnailSize" type="range" min="220" max="380" step="20" />
            </label>
            <span class="toolbar-divider" aria-hidden="true"></span>
            <button
              type="button"
              class="select-toggle"
              :class="{ active: selectMode }"
              :aria-pressed="selectMode"
              @click="toggleSelectMode"
            >
              <v-icon :icon="mdiCheckCircleOutline" size="15" aria-hidden="true" />
              {{ t('screenshot.select') }}
            </button>
            <v-btn :icon="mdiRefresh" variant="text" density="comfortable" :loading="loading" :aria-label="t('common.refresh')" @click="loadRecent" />
          </div>
        </div>

        <div v-if="visibleGroups.length" class="gallery-collections">
          <section v-for="group in visibleGroups" :key="group.key" class="date-group" :aria-label="group.title">
            <div class="date-heading">
              <span class="date-mark" aria-hidden="true"></span>
              <h3>{{ group.title }}</h3>
              <span>{{ group.items.length.toString().padStart(2, '0') }}</span>
              <div></div>
            </div>
            <div class="photo-columns">
              <article
                v-for="(item, index) in group.items"
                :key="item.path"
                class="capture-card"
                :class="{
                  selected: selectedPaths.has(item.path),
                  fresh: isFresh(item),
                  developing: developing.has(item.path),
                }"
                :style="{ '--card-delay': `${Math.min(index, 5) * 35}ms` }"
              >
                <button
                  class="capture-open"
                  type="button"
                  @click="onCardClick(item)"
                  @pointerenter="hoverPath = item.path"
                  @pointerleave="hoverPath = hoverPath === item.path ? '' : hoverPath"
                >
                  <div class="thumbnail">
                    <div class="paper-mat">
                      <template v-if="item.kind === 'video'">
                        <div class="thumb-video">
                          <v-icon :icon="mdiFileVideoOutline" size="28" aria-hidden="true" />
                          <span>{{ t('screenshot.videoBadge') }}</span>
                        </div>
                      </template>
                      <img v-else-if="!brokenImages.has(item.path)" :key="`${item.path}:${imageRetries[item.path] || 0}`" :src="thumbSrc(item.path)" :alt="item.name" loading="lazy" draggable="false" @error="onThumbError(item.path)" />
                      <div v-else class="thumb-broken">
                        <v-icon :icon="mdiImageOffOutline" size="22" aria-hidden="true" />
                        <span class="thumb-broken-text">{{ t('screenshot.imageMissing') }}</span>
                      </div>
                    </div>
                    <span v-if="kindOf(item) === 'long'" class="kind-badge">{{ t('screenshot.longBadge') }}</span>
                    <span v-else-if="item.kind === 'video'" class="kind-badge">{{ t('screenshot.videoBadge') }}</span>
                    <span v-if="isFresh(item)" class="fresh-banner">
                      <span class="fresh-dot" aria-hidden="true"></span>{{ t('screenshot.freshBadge') }}
                    </span>
                  </div>
                  <div class="card-info">
                    <span class="capture-name" :title="item.name">{{ item.name }}</span>
                    <span class="card-meta"><time>{{ formatDate(item.modifiedMs) }}</time><span>{{ formatSize(item.size) }}</span></span>
                  </div>
                </button>
                <button v-if="brokenImages.has(item.path)" type="button" class="thumb-retry" @click="retryThumb(item.path)">{{ t('screenshot.retryImage') }}</button>
                <button
                  v-if="selectMode"
                  type="button"
                  class="card-check"
                  :class="{ checked: selectedPaths.has(item.path) }"
                  :aria-label="t('screenshot.select')"
                  :aria-pressed="selectedPaths.has(item.path)"
                  @click.stop="toggleSelect(item.path)"
                >
                  <v-icon v-if="selectedPaths.has(item.path)" :icon="mdiCheck" size="12" aria-hidden="true" />
                </button>
                <button
                  v-else
                  type="button"
                  class="card-delete"
                  :aria-label="`${t('screenshot.delete')} ${item.name}`"
                  :title="t('screenshot.delete')"
                  @click.stop="askDelete(item)"
                >
                  <v-icon :icon="mdiTrashCanOutline" size="14" aria-hidden="true" />
                </button>
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

    <!-- 多选操作条 -->
    <transition name="bar-rise">
      <div v-if="selectMode && selectedPaths.size" class="select-bar" role="toolbar" :aria-label="t('screenshot.select')">
        <span class="sb-count">{{ t('screenshot.selectedCount', { count: selectedPaths.size }) }}</span>
        <span class="sb-size">{{ formatSize(selectedSize) }}</span>
        <span class="sb-divider" aria-hidden="true"></span>
        <button class="sb-btn sb-danger" type="button" :disabled="deleteBusy" @click="deleteSelected">
          <v-icon :icon="mdiTrashCanOutline" size="13" aria-hidden="true" />
          {{ t('screenshot.delete') }}
        </button>
        <button class="sb-btn" type="button" @click="clearSelection">{{ t('common.cancel') }}</button>
        <span class="sb-hint">Del</span>
      </div>
    </transition>

    <!-- 快速预览（空格 / 单击卡片） -->
    <v-dialog :model-value="!!selected" max-width="1440" transition="dialog-bottom-transition" @update:model-value="selected = null">
      <div v-if="selected" class="quicklook" :class="{ 'is-beautifying': beautifying }">
        <header class="ql-topbar">
          <button class="ql-back" type="button" @click="closePreview">
            <v-icon :icon="mdiChevronLeft" size="15" aria-hidden="true" />
            {{ t('screenshot.backToLibrary') }}
          </button>
          <span class="ql-divider" aria-hidden="true"></span>
          <div class="ql-title">
            <strong>{{ selected.name }}</strong>
            <small><template v-if="previewPixelLabel">{{ previewPixelLabel }} · </template>{{ formatSize(selected.size) }} · {{ formatDate(selected.modifiedMs) }}</small>
          </div>
          <div class="ql-actions">
            <template v-if="selected.kind === 'video'">
              <v-btn variant="flat" color="primary" size="small" :prepend-icon="mdiPlayCircleOutline" @click="openVideo(selected.path)">{{ t('screenshot.openFile') }}</v-btn>
            </template>
            <template v-else-if="!beautifying">
              <v-btn variant="flat" color="primary" size="small" :loading="copyingImage" @click="copyImage">{{ t('screenshot.copyImage') }}</v-btn>
              <v-btn variant="outlined" size="small" @click="pinCurrent">{{ t('screenshot.pin.action') }}</v-btn>
            </template>
            <v-btn v-if="selected.kind !== 'video'" variant="outlined" size="small" :class="{ 'ql-editing': beautifying }" @click="beautifying = !beautifying">
              {{ t('screenshot.beautify') }}
            </v-btn>
            <v-btn variant="outlined" size="small" color="error" @click="askDelete(selected)">{{ t('screenshot.delete') }}</v-btn>
          </div>
        </header>

        <div class="ql-body">
          <BeautifyEditor v-if="beautifying && selected.kind !== 'video'" class="ql-editor" :path="selected.path" @saved="onStyledSaved" />
          <template v-else>
            <!-- 日期时间轴 -->
            <aside class="ql-rail" aria-label="Dates">
              <span class="rail-year">{{ railYear }}</span>
              <button
                v-for="group in groups"
                :key="group.key"
                type="button"
                class="rail-day"
                :class="{ active: group.items.some(gi => gi.path === selected?.path) }"
                @click="jumpToGroup(group)"
              >
                <span class="rail-label">{{ railLabel(group) }}</span>
                <span class="rail-dot" aria-hidden="true"></span>
              </button>
            </aside>

            <!-- 画面 + 胶片条 -->
            <div class="ql-stage" :class="{ 'has-ocr': showOcr }">
              <div
                class="stage-canvas"
                :class="{ 'is-actual': previewZoom === 'actual', 'is-panning': panning }"
                tabindex="0"
                :aria-label="t('screenshot.preview')"
                @pointerdown="onPanStart"
                @pointermove="onPanMove"
                @pointerup="onPanEnd"
                @pointercancel="onPanEnd"
                @lostpointercapture="onPanEnd"
              >
                <div v-if="previewFailed" role="alert" class="thumb-broken"><span>{{ t('screenshot.imageMissing') }}</span><v-btn variant="text" @click="retryThumb(selected.path); previewFailed = false">{{ t('screenshot.retryImage') }}</v-btn></div>
                <div v-else-if="selected.kind === 'video'" class="stage-video">
                  <video class="preview-video" :src="thumbSrc(selected.path)" controls playsinline></video>
                </div>
                <div v-else class="stage-card">
                  <img :src="thumbSrc(selected.path)" :alt="selected.name" draggable="false" :style="previewStyle" @load="onPreviewLoad" @error="previewFailed = true" />
                </div>
              </div>

              <div class="ql-strip">
                <span class="strip-keys" aria-hidden="true">← →</span>
                <div v-if="selected.kind !== 'video'" class="zoom-toggle">
                  <button type="button" class="zoom-chip" :class="{ active: previewZoom === 'fit' }" @click="previewZoom = 'fit'">{{ t('screenshot.fitWindow') }}</button>
                  <button type="button" class="zoom-chip" :class="{ active: previewZoom === 'actual' }" :disabled="!previewPixels" @click="previewZoom = 'actual'">{{ t('screenshot.actualSize') }}</button>
                </div>
                <div class="strip-thumbs">
                  <button
                    v-for="(item, index) in recent"
                    :key="item.path"
                    type="button"
                    class="strip-thumb"
                    :class="{ current: item.path === selected.path }"
                    :aria-label="item.name"
                    @click="selectByIndex(index)"
                  >
                    <img v-if="item.kind !== 'video'" :src="thumbSrc(item.path)" alt="" loading="lazy" draggable="false" />
                    <v-icon v-else :icon="mdiFileVideoOutline" size="18" aria-hidden="true" />
                    <span v-if="item.path === selected.path" class="strip-underline" aria-hidden="true"></span>
                  </button>
                </div>
                <span class="strip-count">{{ stripIndex.toString().padStart(2, '0') }} / {{ recent.length }}</span>
              </div>
            </div>

            <!-- 识别文字 -->
            <OcrPanel v-if="showOcr && selected.kind !== 'video'" :path="selected.path" />
          </template>
        </div>
      </div>
    </v-dialog>

    <!-- 删除确认 -->
    <v-dialog :model-value="!!deleting" max-width="400" @update:model-value="v => { if (!v) deleting = null }">
      <div v-if="deleting" class="delete-panel" role="alertdialog" :aria-label="t('screenshot.deleteTitle')">
        <h3 class="delete-title">{{ deleting.name ? t('screenshot.deleteTitle') : t('screenshot.deleteBatchTitle', { count: selectedPaths.size }) }}</h3>
        <p class="delete-text">{{ deleting.name ? t('screenshot.deleteConfirm', { name: deleting.name }) : t('screenshot.deleteBatchConfirm', { count: selectedPaths.size }) }}</p>
        <div class="delete-actions">
          <button class="console-btn" type="button" @click="deleting = null">{{ t('common.cancel') }}</button>
          <button class="console-btn danger" type="button" :disabled="deleteBusy" @click="confirmDelete">
            {{ deleteBusy ? '…' : t('screenshot.delete') }}
          </button>
        </div>
      </div>
    </v-dialog>

    <!-- 存入提示（捕捉反馈 · 第三帧） -->
    <transition name="bar-rise">
      <div v-if="savedToast" class="saved-toast" role="status">
        <span class="st-check" aria-hidden="true">
          <v-icon :icon="mdiCheck" size="9" aria-hidden="true" />
        </span>
        <span class="st-text">{{ t('screenshot.savedToast') }}</span>
        <button v-if="savedToastPath" class="st-action" type="button" @click="pinFresh(savedToastPath)">{{ t('screenshot.pin.action') }}</button>
      </div>
    </transition>
  </main>
</template>

<script setup lang="ts">
import {
  mdiCheck,
  mdiCheckCircleOutline,
  mdiChevronLeft,
  mdiCrop,
  mdiCropFree,
  mdiFileVideoOutline,
  mdiImageOffOutline,
  mdiImageSizeSelectLarge,
  mdiPlayCircleOutline,
  mdiRefresh,
  mdiTrashCanOutline,
  mdiViewGridOutline,
} from '@mdi/js'
import { convertFileSrc } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  copyScreenshotFile,
  deleteScreenshot,
  importScreenshot,
  listScreenshots,
  openScreenshotFile,
  pinScreenshotFile,
  type ScreenshotItem,
  screenshotBegin,
} from '@/api/screenshot'
import { useScreenshotHotkey } from '@/composables/useScreenshotHotkey'
import BeautifyEditor from './BeautifyEditor.vue'
import OcrPanel from './OcrPanel.vue'

type FilterKind = 'all' | 'image' | 'long' | 'video'

const { t, locale } = useI18n()
const { hotkey } = useScreenshotHotkey()
/** F1 / Ctrl + Alt + S 的分段展示 */
const hotkeyParts = computed(() =>
  hotkey.value
    .split('+')
    .map(p => p.trim())
    .filter(Boolean)
)
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

// ---- 类型与筛选 ----

/** 条目类型：长图按文件名前缀（Scroll_）识别 */
function kindOf(item: ScreenshotItem): Exclude<FilterKind, 'all'> {
  if (item.kind === 'video') return 'video'
  if (item.name.startsWith('Scroll_')) return 'long'
  return 'image'
}

const filterKind = ref<FilterKind>('all')
const filterOptions = computed(() => [
  { kind: 'all' as FilterKind, label: t('screenshot.filter.all'), count: recent.value.length },
  {
    kind: 'image' as FilterKind,
    label: t('screenshot.filter.image'),
    count: recent.value.filter(item => kindOf(item) === 'image').length,
  },
  {
    kind: 'long' as FilterKind,
    label: t('screenshot.filter.long'),
    count: recent.value.filter(item => kindOf(item) === 'long').length,
  },
  {
    kind: 'video' as FilterKind,
    label: t('screenshot.filter.video'),
    count: recent.value.filter(item => kindOf(item) === 'video').length,
  },
])

const visibleGroups = computed(() =>
  groups.value
    .map(group => ({
      ...group,
      items:
        filterKind.value === 'all'
          ? group.items
          : group.items.filter(item => kindOf(item) === filterKind.value),
    }))
    .filter(group => group.items.length > 0)
)

const totalSize = computed(() => recent.value.reduce((sum, item) => sum + item.size, 0))

// ---- 多选 ----

const selectMode = ref(false)
const selectedPaths = ref(new Set<string>())
const selectedSize = computed(() =>
  recent.value
    .filter(item => selectedPaths.value.has(item.path))
    .reduce((sum, item) => sum + item.size, 0)
)

function toggleSelectMode() {
  selectMode.value = !selectMode.value
  if (!selectMode.value) selectedPaths.value = new Set()
}

function toggleSelect(path: string) {
  const next = new Set(selectedPaths.value)
  if (next.has(path)) next.delete(path)
  else next.add(path)
  selectedPaths.value = next
}

function clearSelection() {
  selectedPaths.value = new Set()
}

function deleteSelected() {
  if (!selectedPaths.value.size) return
  deleting.value = { name: '', path: '', size: 0, modifiedMs: 0, kind: 'image' }
}

// ---- 快取新片（捕捉反馈）----

const FRESH_WINDOW_MS = 5 * 60 * 1000
/** 本会话内新入库的路径：播放显影动画 */
const developing = ref(new Set<string>())
const savedToastPath = ref('')
let savedToastTimer: number | null = null
const savedToast = computed(() => !!savedToastPath.value)

function isFresh(item: ScreenshotItem): boolean {
  return Date.now() - item.modifiedMs < FRESH_WINDOW_MS
}

function onScreenshotSaved(path: string) {
  if (!path) return
  developing.value = new Set([...developing.value, path])
  savedToastPath.value = path
  if (savedToastTimer) window.clearTimeout(savedToastTimer)
  savedToastTimer = window.setTimeout(() => {
    savedToastPath.value = ''
  }, 4200)
}

async function pinFresh(path: string) {
  savedToastPath.value = ''
  try {
    await pinScreenshotFile(path)
  } catch (err) {
    error.value = t('screenshot.pin.failed', { message: String(err) })
  }
}

// ---- 预览（快速预览）----

const selected = ref<ScreenshotItem | null>(null)
const beautifying = ref(false)
const showOcr = ref(false)
const copyingImage = ref(false)
const copyDone = ref(false)
const previewZoom = ref<'fit' | 'actual'>('fit')
const previewPixels = ref<{ w: number; h: number } | null>(null)
const panning = ref(false)
const previewFailed = ref(false)
let panOrigin = { x: 0, y: 0, left: 0, top: 0 }
const imageRetries = ref<Record<string, number>>({})
const brokenImages = ref(new Set<string>())

const stripIndex = computed(() =>
  selected.value
    ? Math.max(
        0,
        recent.value.findIndex(item => item.path === selected.value!.path)
      )
    : 0
)

const railYear = computed(() =>
  selected.value ? new Date(selected.value.modifiedMs).getFullYear().toString() : ''
)

function railLabel(group: { items: ScreenshotItem[] }): string {
  const date = new Date(group.items[0]?.modifiedMs ?? Date.now())
  return `${(date.getMonth() + 1).toString().padStart(2, '0')}·${date.getDate().toString().padStart(2, '0')}`
}

function jumpToGroup(group: { items: ScreenshotItem[] }) {
  const [first] = group.items
  if (first) selectItem(first)
}

function selectItem(item: ScreenshotItem) {
  selected.value = item
  beautifying.value = false
}

function onCardClick(item: ScreenshotItem) {
  if (selectMode.value) {
    toggleSelect(item.path)
    return
  }
  selectItem(item)
}

function selectByIndex(index: number) {
  const item = recent.value[index]
  if (item) selected.value = item
}

function closePreview() {
  selected.value = null
}

function onPreviewKeyDown(event: KeyboardEvent) {
  if (!selected.value || beautifying.value) return
  if (event.key === 'Escape') {
    event.preventDefault()
    closePreview()
  } else if (event.key === 'ArrowLeft') {
    event.preventDefault()
    const prev = (stripIndex.value - 1 + recent.value.length) % recent.value.length
    selectByIndex(prev)
  } else if (event.key === 'ArrowRight') {
    event.preventDefault()
    selectByIndex((stripIndex.value + 1) % recent.value.length)
  } else if (event.key === ' ') {
    // 空格：收回快速预览（Finder 式开关）
    event.preventDefault()
    closePreview()
  }
}

async function copyImage() {
  const target = selected.value
  if (!target || copyingImage.value) return
  copyingImage.value = true
  try {
    await copyScreenshotFile(target.path)
    error.value = ''
    copyDone.value = true
    window.setTimeout(() => {
      copyDone.value = false
    }, 2200)
  } catch (err) {
    error.value = t('screenshot.copyImageFailed', { message: String(err) })
  } finally {
    copyingImage.value = false
  }
}

async function pinCurrent() {
  const target = selected.value
  if (!target) return
  try {
    await pinScreenshotFile(target.path)
  } catch (err) {
    error.value = t('screenshot.pin.failed', { message: String(err) })
  }
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

watch(selected, item => {
  showOcr.value = !!item && item.kind !== 'video'
  previewFailed.value = false
  previewZoom.value = 'fit'
  previewPixels.value = null
  panning.value = false
  beautifying.value = false
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

const capturing = ref(false)
const importing = ref(false)
const loading = ref(false)
const error = ref('')

/** 后端警告码（拼接降级、录屏失败等）映射为可见提示 */
function showNoticeCodes(codes: unknown) {
  if (!Array.isArray(codes) || codes.length === 0) return
  const messages = codes
    .map(code => t(`screenshot.warnings.${String(code)}`))
    .filter(message => !message.startsWith('screenshot.warnings.'))
  if (messages.length > 0) error.value = messages.join(' ')
}

async function openCapturedOcr(event: Event) {
  const path = (event as CustomEvent<string>).detail
  if (!path) return
  await loadRecent()
  const item = recent.value.find(entry => entry.path === path)
  if (item) {
    selected.value = item
    await Promise.resolve()
    showOcr.value = true
  }
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

/** 导入本地图片到截图库 */
async function importImage() {
  if (importing.value) return
  importing.value = true
  error.value = ''
  try {
    const path = await importScreenshot()
    if (path) {
      await loadRecent()
      const item = recent.value.find(entry => entry.path === path)
      if (item) onScreenshotSaved(item.path)
    }
  } catch (err) {
    error.value = t('screenshot.importFailed', { message: String(err) })
  } finally {
    importing.value = false
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

// ---- 删除 ----
const deleting = ref<ScreenshotItem | null>(null)
const deleteBusy = ref(false)

function askDelete(item: ScreenshotItem) {
  if (!item) return
  deleting.value = item
}

async function confirmDelete() {
  const target = deleting.value
  if (!target || deleteBusy.value) return
  deleteBusy.value = true
  try {
    if (target.name) {
      // 单条删除
      await deleteScreenshot(target.path)
      recent.value = recent.value.filter(i => i.path !== target.path)
      brokenImages.value.delete(target.path)
      selectedPaths.value = new Set([...selectedPaths.value].filter(p => p !== target.path))
      if (selected.value?.path === target.path) selected.value = null
    } else {
      // 批量删除
      const paths = [...selectedPaths.value]
      const failures: string[] = []
      for (const path of paths) {
        try {
          await deleteScreenshot(path)
          brokenImages.value.delete(path)
        } catch {
          failures.push(path)
        }
      }
      const removed = new Set(paths.filter(p => !failures.includes(p)))
      recent.value = recent.value.filter(i => !removed.has(i.path))
      selectedPaths.value = new Set(failures)
      if (selected.value && removed.has(selected.value.path)) selected.value = null
      if (!failures.length) selectMode.value = false
    }
    deleting.value = null
  } catch (err) {
    error.value = t('screenshot.deleteFailed', { message: String(err) })
    deleting.value = null
  } finally {
    deleteBusy.value = false
  }
}

/** 用系统默认播放器打开录屏 MP4 */
async function openVideo(path: string) {
  try {
    await openScreenshotFile(path)
  } catch (err) {
    error.value = t('screenshot.openFileFailed', { message: String(err) })
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

let unlisten: UnlistenFn | undefined
let unlistenNotice: UnlistenFn | undefined
let disposed = false

/** 悬停中的卡片：空格快速预览的目标 */
const hoverPath = ref('')

function onGlobalKeyDown(event: KeyboardEvent) {
  if (selectMode.value && (event.key === 'Delete' || event.key === 'Backspace')) {
    if (selectedPaths.value.size && !deleting.value) {
      event.preventDefault()
      deleteSelected()
    }
    return
  }
  if (selected.value) {
    onPreviewKeyDown(event)
    return
  }
  // 快速预览：空格打开悬停项（Finder 式），再按一次收回
  if (event.key === ' ' && !selectMode.value) {
    const item = recent.value.find(entry => entry.path === hoverPath.value)
    if (item) {
      event.preventDefault()
      selectItem(item)
    }
  }
}

onMounted(async () => {
  window.addEventListener('focus', loadRecent)
  window.addEventListener('keydown', onGlobalKeyDown)
  window.addEventListener('jedi-screenshot-ocr', openCapturedOcr)
  try {
    const stop = await listen<string>('screenshot-saved', event => {
      onScreenshotSaved(event.payload)
      void loadRecent()
    })
    if (disposed) stop()
    else unlisten = stop
    const stopNotice = await listen<string[]>('screenshot-notice', event =>
      showNoticeCodes(event.payload)
    )
    if (disposed) stopNotice()
    else unlistenNotice = stopNotice
  } catch (err) {
    console.error('Screenshot listener failed:', err)
  }
  if (!disposed) await loadRecent()
})
onUnmounted(() => {
  disposed = true
  unlisten?.()
  unlistenNotice?.()
  if (savedToastTimer) window.clearTimeout(savedToastTimer)
  window.removeEventListener('focus', loadRecent)
  window.removeEventListener('keydown', onGlobalKeyDown)
  window.removeEventListener('jedi-screenshot-ocr', openCapturedOcr)
})
</script>

<style scoped>
.screenshot-page {
  position: relative;
  height: 100%;
  overflow: auto;
  color: var(--text);
  background: var(--bg);
  container-type: inline-size;
}
.screenshot-body { width: min(1500px, 100%); min-height: 100%; margin: 0 auto; padding: clamp(26px, 3.4vw, 46px) clamp(28px, 4vw, 58px) 40px; }
h1, h2, h3, p { margin: 0; }
.capture-header { display: flex; justify-content: space-between; align-items: flex-start; gap: 32px; padding: 4px 2px 0; }
.heading { max-width: 620px; }
.eyebrow-pill {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 24px;
  padding: 0 9px 0 7px;
  border: 1px solid rgb(var(--accent-rgb) / 0.28);
  border-radius: 999px;
  background: rgb(var(--accent-rgb) / 0.08);
  color: var(--accent);
  font: 600 10px/1.6 var(--jedi-font-ui);
  letter-spacing: 0.06em;
}
.pill-dot { width: 12px; height: 12px; border-radius: 3px; background: var(--accent); }
h1 { margin-top: 12px; font-size: clamp(26px, 2.6vw, 30px); font-weight: 700; letter-spacing: -0.02em; line-height: 1.16; }
.heading p { max-width: 520px; margin-top: 8px; color: var(--text-muted); font-size: 12.5px; line-height: 1.7; }
.capture-actions { display: flex; gap: 8px; flex-shrink: 0; padding-top: 4px; }
.capture-button { min-width: 132px; }
.kbd-chip {
  margin-left: 4px;
  padding: 2px 6px;
  border-radius: 5px;
  background: rgb(255 255 255 / 0.22);
  font: 700 10px var(--jedi-font-mono);
  letter-spacing: 0.04em;
}
.import-button { background: var(--bg); }
.gallery-alert { margin-bottom: 18px; }
.gallery { padding-bottom: 40px; }

/* 工具条 */
.gallery-toolbar {
  position: sticky;
  top: 0;
  z-index: 5;
  display: flex;
  align-items: center;
  gap: 14px;
  height: 58px;
  margin-top: 20px;
  padding: 0 14px 0 16px;
  border: 1px solid var(--border);
  border-radius: 12px;
  background: var(--bg);
  box-shadow: 0 1px 2px rgb(var(--ink-rgb) / 0.04);
  overflow: hidden;
}
.collection-summary { display: flex; align-items: center; gap: 10px; }
.toolbar-icon { display: grid; width: 30px; height: 30px; place-items: center; border-radius: 8px; background: rgb(var(--accent-rgb) / 0.1); color: var(--accent); }
.summary-text { display: flex; flex-direction: column; gap: 1px; min-width: 0; }
.summary-text strong { font-size: 13px; font-weight: 600; line-height: 1.3; }
.summary-text small { color: var(--text-muted); font: 10px var(--jedi-font-mono); }
.toolbar-divider { width: 1px; height: 24px; background: var(--border); flex: none; }
.filter-chips { display: flex; align-items: center; gap: 4px; min-width: 0; overflow: hidden; }
.filter-chip {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  height: 26px;
  padding: 0 10px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: var(--text-muted);
  font: 400 11.5px var(--jedi-font-ui);
  cursor: pointer;
  transition: background var(--motion-fast), color var(--motion-fast);
  white-space: nowrap;
}
.filter-chip .chip-count { font: 400 10px var(--jedi-font-mono); opacity: 0.72; }
.filter-chip:hover { background: rgb(var(--ink-rgb) / 0.05); color: var(--text); }
.filter-chip.active { background: rgb(var(--accent-rgb) / 0.12); color: var(--accent); font-weight: 600; }
.filter-chip.active .chip-count { opacity: 0.9; }
.gallery-controls { display: flex; align-items: center; gap: 8px; margin-left: auto; }
.density-control { display: flex; align-items: center; gap: 8px; color: var(--text-muted); font-size: 11.5px; white-space: nowrap; }
.density-control input { width: 96px; accent-color: var(--accent); cursor: pointer; }
.select-toggle {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 30px;
  padding: 0 11px;
  border: 0;
  border-radius: 7px;
  background: var(--accent);
  color: var(--on-accent);
  font: 600 11.5px var(--jedi-font-ui);
  cursor: pointer;
  transition: background var(--motion-fast), color var(--motion-fast), box-shadow var(--motion-fast);
  white-space: nowrap;
}
.select-toggle:hover { background: var(--accent-hover); }
.select-toggle.active { background: rgb(var(--ink-rgb) / 0.86); color: var(--text-inverse); }

/* 日期分组 */
.date-heading { display: flex; align-items: center; gap: 9px; margin: 26px 2px 15px; }
.date-mark { width: 6px; height: 6px; border-radius: 50%; background: var(--accent); }
.date-heading h3 { font-size: 12px; font-weight: 600; }
.date-heading > span:not(.date-mark) { color: var(--text-muted); font: 10px var(--jedi-font-mono); }
.date-heading > div { height: 1px; flex: 1; background: var(--border); }

/* 卡片：相框卡纸 */
.photo-columns { display: grid; grid-template-columns: repeat(auto-fill, minmax(min(var(--thumbnail-size), 100%), 1fr)); gap: 18px 16px; }
.capture-card {
  position: relative;
  min-width: 0;
  border: 1px solid var(--border);
  border-radius: 13px;
  background: var(--surface-raised);
  overflow: hidden;
  animation: photo-reveal var(--motion-enter) var(--ease-out) var(--card-delay) backwards;
  transition: border-color var(--motion-fast), box-shadow var(--motion-normal), transform var(--motion-fast);
}
.capture-card:hover { border-color: rgb(var(--accent-rgb) / 0.4); box-shadow: 0 6px 18px rgb(var(--ink-rgb) / 0.08); transform: translateY(-2px); }
.capture-card.selected { border-color: var(--accent); box-shadow: 0 6px 18px rgb(var(--accent-rgb) / 0.14); }
.capture-card.fresh { border-color: rgb(var(--accent-rgb) / 0.5); box-shadow: 0 6px 18px rgb(var(--accent-rgb) / 0.14); }
.capture-card.developing .thumbnail img,
.capture-card.developing .thumb-video {
  animation: card-develop 420ms var(--ease-out) backwards;
}
.capture-card.developing::after {
  content: '';
  position: absolute;
  left: 0;
  right: 0;
  top: 0;
  height: 38px;
  background: linear-gradient(180deg, rgb(var(--accent-rgb) / 0.22), rgb(var(--accent-rgb) / 0));
  animation: card-sweep 320ms ease-out 120ms backwards;
  pointer-events: none;
  z-index: 2;
}
.capture-open { display: block; width: 100%; padding: 0; border: 0; border-radius: inherit; background: transparent; color: inherit; text-align: left; cursor: pointer; }
.capture-open:focus-visible { outline: 2px solid var(--accent); outline-offset: -2px; }
.thumbnail { position: relative; aspect-ratio: 16 / 10; margin: 8px 8px 0; border-radius: 8px; overflow: hidden; background: linear-gradient(135deg, rgb(var(--ink-rgb) / 0.035), rgb(var(--ink-rgb) / 0.012)); }
.paper-mat {
  position: absolute;
  inset: 10px;
  display: grid;
  place-items: center;
  min-width: 0;
  min-height: 0;
  border-radius: 4px;
  background: var(--surface-raised);
  box-shadow: 0 2px 7px rgb(var(--ink-rgb) / 0.12);
  overflow: hidden;
  padding: 6px;
}
.paper-mat img { display: block; max-width: 100%; max-height: 100%; object-fit: contain; }
.thumb-broken { position: absolute; inset: 0; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 8px; padding: 16px; text-align: center; color: var(--text-muted); }
.thumb-broken-text { font-size: 12px; line-height: 1.6; }
.thumb-video { display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 10px; color: var(--accent); }
.thumb-video span { font: 600 11px/1.4 var(--jedi-font-mono); letter-spacing: 1px; text-transform: uppercase; color: var(--text-muted); }
.thumb-retry { position: absolute; top: 18px; right: 18px; cursor: pointer; padding: 5px 10px; border: 1px solid var(--border); border-radius: 7px; background: var(--surface-raised); color: var(--text-muted); font-size: 11px; }
.thumb-retry:hover { border-color: var(--accent); color: var(--text); }
.thumb-retry:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }

/* 角标 */
.kind-badge {
  position: absolute;
  left: 6px;
  bottom: 6px;
  display: inline-flex;
  align-items: center;
  height: 18px;
  padding: 0 6px;
  border: 1px solid rgb(var(--ink-rgb) / 0.1);
  border-radius: 5px;
  background: rgb(var(--bg-rgb) / 0.94);
  color: var(--text-muted);
  font: 600 9.5px var(--jedi-font-ui);
  pointer-events: none;
}
.fresh-banner {
  position: absolute;
  left: 0;
  top: 0;
  right: 0;
  display: flex;
  align-items: center;
  gap: 5px;
  height: 20px;
  padding: 0 7px;
  background: var(--accent);
  color: #fff;
  font: 600 9.5px var(--jedi-font-ui);
  pointer-events: none;
}
.fresh-dot { width: 4px; height: 4px; border-radius: 999px; background: #fff; }

/* 删除 / 多选圆点 */
.card-delete {
  position: absolute;
  top: 6px;
  right: 6px;
  z-index: 3;
  display: grid;
  width: 24px;
  height: 24px;
  place-items: center;
  border: 1px solid rgb(var(--ink-rgb) / 0.12);
  border-radius: 7px;
  background: rgb(var(--bg-rgb) / 0.94);
  color: var(--danger);
  cursor: pointer;
  box-shadow: 0 2px 6px rgb(var(--ink-rgb) / 0.12);
  transition: background var(--motion-fast), border-color var(--motion-fast);
}
.card-delete:hover { background: rgb(var(--danger-rgb) / 0.1); border-color: rgb(var(--danger-rgb) / 0.5); }
.card-delete:focus-visible { outline: 2px solid var(--danger); outline-offset: 2px; }
.card-check {
  position: absolute;
  left: 6px;
  top: 26px;
  z-index: 3;
  display: grid;
  width: 19px;
  height: 19px;
  place-items: center;
  border: 1.5px solid rgb(var(--ink-rgb) / 0.3);
  border-radius: 999px;
  background: rgb(var(--bg-rgb) / 0.92);
  color: transparent;
  cursor: pointer;
  transition: background var(--motion-fast), border-color var(--motion-fast), color var(--motion-fast);
}
.card-check.checked { border-color: var(--accent); background: var(--accent); color: var(--on-accent); }
.card-check:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }
.selected-mark { position: absolute; inset: 0; border: 2px solid var(--accent); border-radius: 8px; background: rgb(var(--accent-rgb) / 0.12); pointer-events: none; }
.capture-card.selected .thumbnail::after {
  content: '';
  position: absolute;
  inset: 0;
  border: 2px solid var(--accent);
  border-radius: 8px;
  background: rgb(var(--accent-rgb) / 0.1);
  pointer-events: none;
  z-index: 1;
}

.card-info { padding: 9px 11px 11px; }
.capture-name { display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 11.5px; font-weight: 500; }
.card-meta { display: flex; justify-content: space-between; gap: 8px; margin-top: 3px; color: var(--text-muted); font: 9.5px var(--jedi-font-mono); }
.empty-state { min-height: 400px; display: flex; flex-direction: column; align-items: center; justify-content: center; text-align: center; }
.empty-frames { display: flex; align-items: center; height: 120px; margin-bottom: 30px; }
.empty-frames span { display: grid; width: 94px; height: 110px; place-items: center; border: 1px solid var(--border-strong); border-radius: 14px; background: var(--surface-raised); color: var(--accent); box-shadow: var(--shadow-md); }
.empty-frames span:first-child { transform: rotate(-10deg) translateX(18px); background: var(--bg-elevated); }
.empty-frames span:last-child { transform: rotate(10deg) translateX(-18px); background: var(--bg-elevated); }
.empty-frames span:nth-child(2) { z-index: 1; }
.empty-state h3 { font-size: 17px; font-weight: 650; }
.empty-state p { max-width: 400px; margin-top: 8px; color: var(--text-muted); font-size: 12px; line-height: 1.7; }
.empty-state .v-btn { margin-top: 22px; }
.gallery-skeleton { display: grid; grid-template-columns: repeat(auto-fill, minmax(240px, 1fr)); gap: 20px; padding-top: 32px; }
.skeleton-frame { aspect-ratio: 16 / 11; border: 1px solid var(--border); border-radius: 13px; background: linear-gradient(105deg, var(--bg-elevated) 25%, rgb(var(--ink-rgb) / 0.05) 40%, var(--bg-elevated) 58%); background-size: 220% 100%; animation: gallery-pulse 1.6s ease-in-out infinite; }

/* 多选操作条 */
.select-bar {
  position: fixed;
  left: 50%;
  transform: translateX(-50%);
  bottom: 22px;
  z-index: 30;
  display: flex;
  align-items: center;
  gap: 14px;
  height: 44px;
  padding: 0 8px 0 16px;
  border-radius: 11px;
  background: rgb(var(--ink-rgb) / 0.96);
  box-shadow: 0 14px 34px rgb(var(--ink-rgb) / 0.28);
}
.sb-count { font-size: 12px; color: #fff; white-space: nowrap; }
.sb-size { font: 400 10.5px var(--jedi-font-mono); color: rgb(255 255 255 / 0.6); }
.sb-divider { width: 1px; height: 18px; background: rgb(255 255 255 / 0.2); }
.sb-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 30px;
  padding: 0 12px;
  border: 0;
  border-radius: 7px;
  background: rgb(255 255 255 / 0.1);
  color: #fff;
  font: 400 11.5px var(--jedi-font-ui);
  cursor: pointer;
}
.sb-btn:hover { background: rgb(255 255 255 / 0.16); }
.sb-btn.sb-danger { background: rgb(var(--danger-rgb) / 0.2); color: #ff9aa5; font-weight: 600; }
.sb-btn.sb-danger:hover { background: rgb(var(--danger-rgb) / 0.3); }
.sb-btn:disabled { opacity: 0.55; cursor: default; }
.sb-hint { padding-right: 4px; font: 400 9.5px var(--jedi-font-mono); color: rgb(255 255 255 / 0.45); }

/* 存入提示 */
.saved-toast {
  position: fixed;
  left: 50%;
  transform: translateX(-50%);
  bottom: 22px;
  z-index: 31;
  display: flex;
  align-items: center;
  gap: 8px;
  height: 40px;
  padding: 0 12px 0 10px;
  border-radius: 10px;
  background: rgb(var(--ink-rgb) / 0.96);
  box-shadow: 0 10px 24px rgb(var(--ink-rgb) / 0.24);
}
.st-check {
  display: grid;
  place-items: center;
  width: 15px;
  height: 15px;
  border-radius: 999px;
  background: var(--accent);
  color: #fff;
}
.st-text { font-size: 11.5px; color: #fff; white-space: nowrap; }
.st-action {
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: #9cc2ff;
  font: 600 11.5px var(--jedi-font-ui);
  cursor: pointer;
  padding: 4px 6px;
}
.st-action:hover { background: rgb(255 255 255 / 0.08); }

/* 快速预览 */
.quicklook {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  width: min(1440px, 96vw);
  height: min(900px, 94vh);
  overflow: hidden;
  border: 1px solid var(--border);
  border-radius: 14px;
  background: var(--bg);
  color: var(--text);
  box-shadow: var(--shadow-dialog, 0 30px 90px rgb(var(--ink-rgb) / 0.42));
}
.ql-topbar {
  display: flex;
  align-items: center;
  gap: 12px;
  height: 56px;
  padding: 0 16px;
  border-bottom: 1px solid var(--border);
  background: var(--bg);
  flex: none;
}
.ql-back {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 30px;
  padding: 0 10px;
  border: 1px solid var(--border);
  border-radius: 7px;
  background: var(--bg-elevated);
  color: var(--text-muted);
  font: 400 11.5px var(--jedi-font-ui);
  cursor: pointer;
}
.ql-back:hover { color: var(--text); border-color: var(--border-strong); }
.ql-divider { width: 1px; height: 22px; background: var(--border); }
.ql-title { display: flex; flex-direction: column; gap: 2px; min-width: 0; flex: 1; }
.ql-title strong { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 12.5px; font-weight: 600; }
.ql-title small { color: var(--text-muted); font: 9.5px var(--jedi-font-mono); }
.ql-actions { display: flex; align-items: center; gap: 6px; flex: none; }

.ql-body { display: flex; min-height: 0; min-width: 0; }
.ql-editor { flex: 1; min-width: 0; }

/* 日期时间轴 */
.ql-rail {
  width: 56px;
  flex: none;
  border-right: 1px solid var(--border);
  background: rgb(var(--ink-rgb) / 0.015);
  display: flex;
  flex-direction: column;
  align-items: stretch;
  padding: 14px 0;
  overflow-y: auto;
  overflow-x: hidden;
}
.rail-year { align-self: center; margin-bottom: 8px; font: 400 8.5px var(--jedi-font-mono); color: var(--text-muted); }
.rail-day {
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 3px;
  padding: 4px 0;
  border: 0;
  border-left: 2px solid transparent;
  background: transparent;
  cursor: pointer;
}
.rail-day:hover { background: rgb(var(--ink-rgb) / 0.04); }
.rail-day.active { background: rgb(var(--accent-rgb) / 0.1); border-left-color: var(--accent); }
.rail-label { font: 400 8.5px var(--jedi-font-mono); color: var(--text-muted); }
.rail-day.active .rail-label { color: var(--accent); font-weight: 700; }
.rail-dot { width: 4px; height: 4px; border-radius: 999px; background: var(--border-strong); }
.rail-day.active .rail-dot { background: var(--accent); }

/* 画面与胶片条 */
.ql-stage { flex: 1; min-width: 0; display: flex; flex-direction: column; background: var(--surface-canvas); }
.stage-canvas {
  flex: 1;
  min-height: 0;
  display: grid;
  place-items: center;
  padding: 26px 30px;
  overflow: auto;
}
.stage-canvas.is-actual { display: block; cursor: grab; user-select: none; touch-action: none; }
.stage-canvas.is-actual .stage-card { display: inline-block; }
.stage-canvas.is-actual.is-panning { cursor: grabbing; }
.stage-card {
  width: fit-content;
  max-width: 100%;
  padding: 0;
  border: 1px solid rgb(var(--ink-rgb) / 0.14);
  border-radius: 8px;
  background: #fff;
  box-shadow: 0 18px 44px rgb(var(--ink-rgb) / 0.16);
  overflow: hidden;
  line-height: 0;
}
.stage-card img { max-width: 100%; max-height: 58vh; object-fit: contain; line-height: initial; }
.stage-canvas.is-actual .stage-card img { max-width: none; max-height: none; }
.stage-video { display: grid; place-items: center; width: 100%; }
.preview-video { max-width: 100%; max-height: 56vh; border-radius: 10px; background: #000; box-shadow: 0 24px 64px rgb(var(--ink-rgb) / 0.24), 0 0 0 1px rgb(var(--ink-rgb) / 0.1); }

.ql-strip {
  flex: none;
  height: 86px;
  border-top: 1px solid var(--border);
  background: var(--bg);
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 0 18px;
}
.strip-keys { font: 400 9.5px var(--jedi-font-mono); color: var(--text-muted); white-space: nowrap; }
.zoom-toggle { display: flex; gap: 4px; flex: none; }
.zoom-chip {
  height: 24px;
  padding: 0 8px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg);
  color: var(--text-muted);
  font: 400 10px var(--jedi-font-ui);
  cursor: pointer;
  white-space: nowrap;
}
.zoom-chip.active { border-color: rgb(var(--accent-rgb) / 0.5); background: rgb(var(--accent-rgb) / 0.1); color: var(--accent); }
.zoom-chip:disabled { opacity: 0.5; cursor: default; }
.strip-thumbs { flex: 1; display: flex; align-items: center; gap: 8px; min-width: 0; overflow-x: auto; padding: 4px 0; }
.strip-thumb {
  position: relative;
  width: 104px;
  height: 58px;
  flex: none;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: rgb(var(--ink-rgb) / 0.015);
  overflow: hidden;
  cursor: pointer;
  padding: 0;
}
.strip-thumb img { width: 100%; height: 100%; object-fit: cover; display: block; }
.strip-thumb:hover { border-color: var(--border-strong); }
.strip-thumb.current { border-color: var(--accent); box-shadow: 0 0 0 2px rgb(var(--accent-rgb) / 0.18); }
.strip-underline { position: absolute; left: 0; right: 0; bottom: 0; height: 2px; background: var(--accent); }
.strip-count { font: 400 9.5px var(--jedi-font-mono); color: var(--text-muted); white-space: nowrap; }

/* 删除确认 */
.delete-panel {
  padding: 22px 22px 18px;
  border: 1px solid var(--border-strong);
  border-radius: 14px;
  background: var(--surface-raised);
  color: var(--text);
  box-shadow: var(--shadow-dialog, 0 24px 60px rgb(var(--ink-rgb) / 0.32));
}
.delete-title { margin: 0 0 8px; font-size: 15px; font-weight: 650; letter-spacing: -0.2px; }
.delete-text { margin: 0 0 18px; color: var(--text-muted); font-size: 12px; line-height: 1.6; }
.delete-actions { display: flex; justify-content: flex-end; gap: 8px; }

/* 过渡 */
.bar-rise-enter-active, .bar-rise-leave-active { transition: opacity var(--motion-normal), transform var(--motion-enter) var(--ease-out); }
.bar-rise-enter-from, .bar-rise-leave-to { opacity: 0; transform: translateX(-50%) translateY(14px); }

@keyframes photo-reveal { from { opacity: 0; transform: translateY(10px) scale(0.985); } to { opacity: 1; transform: none; } }
@keyframes card-develop { from { filter: blur(1.2px); opacity: 0.86; transform: scale(0.985); } to { filter: none; opacity: 1; transform: none; } }
@keyframes card-sweep { from { transform: translateY(-38px); } to { transform: translateY(0); } }
@keyframes gallery-pulse { from { background-position: 100% 0; } to { background-position: -120% 0; } }

@container (max-width: 760px) {
  .screenshot-body { padding: 24px 18px 40px; }
  .capture-header { align-items: flex-start; flex-direction: column; }
  .capture-actions { width: 100%; }
  .capture-button, .import-button { width: 100%; }
  .density-control span, .density-control > .v-icon, .toolbar-divider { display: none; }
  .filter-chips { overflow-x: auto; }
}
@media (max-width: 880px) {
  .quicklook { width: 98vw; height: 96vh; }
  .ql-topbar { flex-wrap: wrap; height: auto; min-height: 56px; padding: 8px 12px; }
  .ql-actions { width: 100%; justify-content: flex-end; }
  .ql-rail { display: none; }
  .ql-stage.has-ocr { flex-direction: column; overflow: auto; }
  .ql-strip { height: 70px; }
  .strip-thumb { width: 84px; height: 46px; }
}
@media (prefers-reduced-motion: reduce) {
  .capture-card, .skeleton-frame { animation: none; }
  .capture-card.developing .thumbnail img { animation: none; }
  .capture-card.developing::after { display: none; }
}
</style>
