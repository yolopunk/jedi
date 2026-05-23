<template>
  <div class="wallpaper-manager scifi-page">
    <!-- CRT Effects -->
    <div class="scanlines"></div>
    <div class="crt-vignette"></div>
    <!-- Grid Background -->
    <div class="grid-bg-layer"></div>

    <div class="content-wrapper">
      <!-- Top Toolbar -->
      <div class="console-header-bar">
        <div class="header-row">
          <div class="header-left">
            <div class="status-indicators">
              <div class="status-light online"></div>
              <div class="status-light standby"></div>
            </div>
            <div class="console-title">
              <span class="title-prefix">[</span>
              <span class="title-text">WALLPAPER_ARCHIVE</span>
              <span class="title-suffix">]</span>
            </div>
            <div class="header-metrics">
              <span class="metric-item">
                <span class="metric-label">COUNT:</span>
                <span class="metric-value">{{ wallpapers.length }}</span>
              </span>
            </div>
          </div>

          <!-- Search & Filter -->
          <div class="header-right">
            <div class="input-wrapper">
              <span class="input-prompt">>></span>
              <input
                v-model="searchQuery"
                type="text"
                class="console-input"
                :placeholder="$t('common.search') + '...'"
              />
            </div>

            <v-menu location="bottom end">
              <template v-slot:activator="{ props }">
                <button v-bind="props" class="console-btn small">
                  <span class="btn-icon">⬡</span>
                  <span class="btn-text">
                    {{ selectedCategories.includes('All') ? 'ALL' : selectedCategories.join(',') }}
                  </span>
                </button>
              </template>
              <div class="console-menu">
                <div
                  class="menu-item"
                  :class="{ active: selectedCategories.includes('All') }"
                  @click="toggleCategory('All')"
                >
                  <span class="menu-check">{{ selectedCategories.includes('All') ? '▣' : '▢' }}</span>
                  <span class="menu-text">{{ $t('wallpapers.all') }}</span>
                </div>
                <div class="menu-divider"></div>
                <div
                  v-for="cat in categories"
                  :key="cat"
                  class="menu-item"
                  :class="{ active: selectedCategories.includes(cat) }"
                  @click="toggleCategory(cat)"
                >
                  <span class="menu-check">{{ selectedCategories.includes(cat) ? '▣' : '▢' }}</span>
                  <span class="menu-text">{{ cat }}</span>
                </div>
              </div>
            </v-menu>

            <v-menu location="bottom end">
              <template v-slot:activator="{ props }">
                <button v-bind="props" class="console-btn small">
                  <span class="btn-icon">▢</span>
                  <span class="btn-text">{{ currentModeLabel }}</span>
                </button>
              </template>
              <div class="console-menu">
                <div class="menu-header">WALLPAPER_MODE</div>
                <div
                  v-for="mode in modeOptions"
                  :key="mode.value"
                  class="menu-item"
                  :class="{ active: selectedMode === mode.value }"
                  @click="selectedMode = mode.value"
                >
                  <span class="menu-text">{{ mode.label }}</span>
                </div>
              </div>
            </v-menu>

            <button class="console-btn icon-only" @click="checkCurrentWallpaper" :title="$t('wallpapers.getCurrent')">
              <span class="btn-icon">◉</span>
            </button>

            <button class="console-btn icon-only" @click="loadWallpapers" :disabled="loading" :title="$t('common.refresh')">
              <span class="btn-icon" :class="{ spinning: loading }">↻</span>
            </button>
          </div>
        </div>
      </div>

      <!-- Main Content Area -->
      <div class="main-content-area">
        <!-- Wallpapers Grid (Scrollable) -->
        <div class="wallpapers-grid-container console-scroll">
          <div v-if="loading && !wallpapers.length" class="loading-grid">
            <div v-for="i in 8" :key="i" class="loading-card">
              <div class="loading-image"></div>
              <div class="loading-footer">
                <div class="loading-line loading-line--title"></div>
                <div class="loading-line loading-line--meta"></div>
              </div>
            </div>
          </div>

          <div v-else-if="filteredWallpapers.length" class="wallpaper-grid">
            <div
              v-for="wp in filteredWallpapers"
              :key="wp.id"
              class="wallpaper-item"
              @click="previewWallpaper(wp)"
            >
              <div class="wallpaper-card">
                <div class="card-glow"></div>
                <div class="image-wrapper">
                  <v-img
                    :src="wp.url"
                    aspect-ratio="1.6"
                    cover
                    class="wallpaper-image"
                  >
                    <template v-slot:placeholder>
                      <div class="d-flex align-center justify-center fill-height">
                        <v-progress-circular indeterminate color="primary"></v-progress-circular>
                      </div>
                    </template>
                  </v-img>

                  <!-- Overlay Actions -->
                  <div class="overlay">
                    <button
                      class="overlay-btn"
                      @click.stop="setWallpaper(wp)"
                      :disabled="settingId === wp.id"
                      :title="$t('wallpapers.setDesktop')"
                    >
                      <span class="btn-icon">{{ settingId === wp.id ? '⟳' : '⬡' }}</span>
                    </button>
                    <button
                      class="overlay-btn secondary"
                      @click.stop="previewWallpaper(wp)"
                      :title="$t('wallpapers.viewDetails')"
                    >
                      <span class="btn-icon">ℹ</span>
                    </button>
                  </div>
                </div>

                <div class="card-footer">
                  <div class="card-title">{{ wp.title }}</div>
                  <div class="card-meta">
                    <span class="card-category">{{ wp.category }}</span>
                    <span v-if="wp.tags.length" class="card-tag">#{{ wp.tags[0] }}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <div v-else class="empty-state">
            <div class="empty-icon">◇</div>
            <div class="empty-text">NO_WALLPAPERS_FOUND</div>
            <button class="console-btn primary mt-4" @click="loadWallpapers" :disabled="loading">
              <span class="btn-text">{{ loading ? 'LOADING...' : t('common.refresh') }}</span>
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Detail Drawer (Right Side) -->
    <v-navigation-drawer
      v-model="drawer"
      location="right"
      width="400"
      temporary
      class="detail-drawer"
      scrim="rgba(0,0,0,0.5)"
    >
      <div v-if="currentPreview" class="drawer-content">
        <!-- Drawer Header -->
        <div class="drawer-header">
          <v-img
            :src="currentPreview.url"
            height="180"
            cover
            class="header-image"
            @click="showImageViewer = true"
          ></v-img>
          <button class="drawer-close" @click="drawer = false">
            <span class="close-icon">✕</span>
          </button>
          <button class="drawer-zoom" @click="showImageViewer = true">
            <span class="zoom-icon">🔍</span>
          </button>
        </div>

        <!-- Drawer Content -->
        <div class="drawer-body console-scroll">
          <h2 class="drawer-title">[ {{ currentPreview.title }} ]</h2>

          <div class="drawer-tags">
            <span class="tag-chip primary">{{ currentPreview.category }}</span>
            <span v-for="tag in currentPreview.tags" :key="tag" class="tag-chip">#{{ tag }}</span>
          </div>

          <div v-if="currentPreview.description" class="drawer-description">
            <span class="desc-prefix">&gt;</span>
            <span class="desc-text">{{ currentPreview.description }}</span>
          </div>

          <div class="divider-line"></div>

          <div class="markdown-body" v-html="renderMarkdown(currentPreview.content || '')"></div>

          <div class="bottom-spacer"></div>
        </div>

        <!-- Floating Action Button -->
        <div class="drawer-actions">
          <button
            v-if="isCurrentWallpaper"
            class="console-btn icon-only"
            @click="openWallpaperFolder"
            :title="$t('wallpapers.openFolder')"
          >
            <span class="btn-icon">📁</span>
          </button>
          <button
            class="console-btn primary"
            @click="setWallpaper(currentPreview)"
            :disabled="settingId === currentPreview.id"
          >
            <span class="btn-icon">{{ settingId === currentPreview.id ? '⟳' : '⬡' }}</span>
            <span class="btn-text">{{ $t('wallpapers.setDesktop') }}</span>
          </button>
        </div>
      </div>
    </v-navigation-drawer>

    <!-- Image Viewer Dialog -->
    <v-dialog v-model="showImageViewer" fullscreen z-index="2500" class="image-viewer-dialog">
      <div class="image-viewer">
        <button class="viewer-close" @click="showImageViewer = false">
          <span class="close-icon">✕</span>
        </button>
        <v-img
          v-if="currentPreview"
          :src="currentPreview.url"
          max-height="95vh"
          max-width="95vw"
          width="auto"
          height="auto"
          contain
          class="viewer-image"
        ></v-img>
      </div>
    </v-dialog>

    <v-snackbar
      v-model="snackbar.show"
      :color="snackbar.color"
      :scrim="false"
      timeout="3000"
      location="top"
      class="console-snackbar"
    >
      <div class="snackbar-content">
        <span class="snackbar-icon">{{ getSnackbarIcon }}</span>
        <span class="snackbar-text">{{ snackbar.text }}</span>
      </div>
    </v-snackbar>
  </div>
</template>

<script setup lang="ts">
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import MarkdownIt from 'markdown-it'
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  getCurrentWallpaper,
  getWallpapers,
  setDesktopWallpaper,
  showInFolder,
  syncWallpapers,
  type WallpaperItem,
  WallpaperMode,
} from '@/api/wallpaper'

const md = new MarkdownIt({
  html: true,
  linkify: true,
  typographer: true,
})

function renderMarkdown(content: string) {
  if (!content) return ''
  let processedContent = content.trim()
  const mdImageRegex = /^!\[.*?\]\(.*?\)/
  if (mdImageRegex.test(processedContent)) {
    processedContent = processedContent.replace(mdImageRegex, '').trim()
  }
  return md.render(processedContent)
}

const { t } = useI18n()
const loading = ref(false)
const settingId = ref<string | null>(null)
const wallpapers = ref<WallpaperItem[]>([])
const selectedCategories = ref<string[]>(['All'])
const searchQuery = ref('')
const drawer = ref(false)
const currentPreview = ref<WallpaperItem | null>(null)
const currentSystemWallpaperPath = ref<string | null>(null)
const selectedMode = ref<WallpaperMode>(WallpaperMode.Crop)
const snackbar = ref({
  show: false,
  text: '',
  color: 'success',
})

const getSnackbarIcon = computed(() => {
  switch (snackbar.value.color) {
    case 'success':
      return '✓'
    case 'error':
      return '✕'
    case 'warning':
      return '!'
    default:
      return '›'
  }
})

const modeOptions = computed(() => [
  {
    value: WallpaperMode.Center,
    label: t('wallpapers.modes.center.label'),
    description: t('wallpapers.modes.center.description'),
  },
  {
    value: WallpaperMode.Crop,
    label: t('wallpapers.modes.crop.label'),
    description: t('wallpapers.modes.crop.description'),
  },
  {
    value: WallpaperMode.Fit,
    label: t('wallpapers.modes.fit.label'),
    description: t('wallpapers.modes.fit.description'),
  },
  {
    value: WallpaperMode.Span,
    label: t('wallpapers.modes.span.label'),
    description: t('wallpapers.modes.span.description'),
  },
  {
    value: WallpaperMode.Stretch,
    label: t('wallpapers.modes.stretch.label'),
    description: t('wallpapers.modes.stretch.description'),
  },
  {
    value: WallpaperMode.Tile,
    label: t('wallpapers.modes.tile.label'),
    description: t('wallpapers.modes.tile.description'),
  },
])

const currentModeLabel = computed(() => {
  return modeOptions.value.find(m => m.value === selectedMode.value)?.label || t('wallpapers.mode')
})

const showImageViewer = ref(false)
let unlistenBatch: UnlistenFn | null = null
let unlistenComplete: UnlistenFn | null = null

const categories = computed(() => {
  const cats = new Set(wallpapers.value.map(w => w.category))
  return Array.from(cats)
})

const filteredWallpapers = computed(() => {
  let result = wallpapers.value

  if (!selectedCategories.value.includes('All')) {
    result = result.filter(w => selectedCategories.value.includes(w.category))
  }

  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(
      w =>
        w.title.toLowerCase().includes(query) ||
        w.description.toLowerCase().includes(query) ||
        w.tags.some(tag => tag.toLowerCase().includes(query))
    )
  }

  return result
})

function toggleCategory(cat: string) {
  if (cat === 'All') {
    selectedCategories.value = ['All']
  } else {
    const index = selectedCategories.value.indexOf(cat)
    if (index > -1) {
      selectedCategories.value.splice(index, 1)
      if (selectedCategories.value.length === 0) {
        selectedCategories.value = ['All']
      }
    } else {
      const allIndex = selectedCategories.value.indexOf('All')
      if (allIndex > -1) {
        selectedCategories.value.splice(allIndex, 1)
      }
      selectedCategories.value.push(cat)
    }
  }
}

async function loadWallpapers() {
  loading.value = true
  try {
    const cachedData = await getWallpapers()
    if (cachedData && cachedData.length > 0) {
      wallpapers.value = cachedData
      loading.value = false
    }
    await syncWallpapers()
  } catch (error) {
    console.error(error)
    showSnackbar(t('wallpapers.loadError'), 'error')
    loading.value = false
  }
}

async function setWallpaper(wp: WallpaperItem) {
  settingId.value = wp.id
  try {
    await setDesktopWallpaper(wp.url, selectedMode.value)
    showSnackbar(t('wallpapers.setSuccess'), 'success')
  } catch (error) {
    console.error(error)
    showSnackbar(t('wallpapers.setError'), 'error')
  } finally {
    settingId.value = null
  }
}

const isCurrentWallpaper = computed(() => {
  if (!currentPreview.value || !currentSystemWallpaperPath.value) return false
  const filename = currentSystemWallpaperPath.value.split(/[\\/]/).pop()
  return currentPreview.value.url.endsWith(filename || '')
})

async function checkCurrentWallpaper() {
  const path = await getCurrentWallpaper()
  if (path) {
    currentSystemWallpaperPath.value = path
    const filename = path.split(/[\\/]/).pop()
    const matched = wallpapers.value.find(w => w.url.endsWith(filename || ''))

    if (matched) {
      showSnackbar(`${t('wallpapers.currentWallpaper')}: ${matched.title}`, 'success')
      previewWallpaper(matched)
    } else {
      showSnackbar(`${t('wallpapers.currentWallpaper')}: ${path}`, 'info')
    }
  } else {
    showSnackbar(t('wallpapers.getCurrentError'), 'warning')
  }
}

async function openWallpaperFolder() {
  if (currentSystemWallpaperPath.value) {
    try {
      await showInFolder(currentSystemWallpaperPath.value)
    } catch (error) {
      console.error(error)
      showSnackbar(t('wallpapers.openFolderError'), 'error')
    }
  }
}

function previewWallpaper(wp: WallpaperItem) {
  currentPreview.value = wp
  drawer.value = true
}

function showSnackbar(text: string, color: string) {
  snackbar.value = { show: true, text, color }
}

onMounted(async () => {
  unlistenBatch = await listen<WallpaperItem[]>('wallpaper-batch', event => {
    const newItems = event.payload
    const currentIds = new Set(wallpapers.value.map(w => w.id))
    let hasNew = false
    for (const item of newItems) {
      if (!currentIds.has(item.id)) {
        wallpapers.value.push(item)
        currentIds.add(item.id)
        hasNew = true
      }
    }
    if (hasNew && loading.value) {
      loading.value = false
    }
  })

  unlistenComplete = await listen('wallpaper-sync-complete', () => {
    loading.value = false
  })

  loadWallpapers()
})

onUnmounted(() => {
  if (unlistenBatch) unlistenBatch()
  if (unlistenComplete) unlistenComplete()
})
</script>

<style scoped>
.wallpaper-manager {
  position: relative;
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  overflow: hidden;
  background: linear-gradient(180deg, #0a0a0f 0%, #0d0d14 50%, #0a0a0f 100%);
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', monospace;
}

.content-wrapper {
  position: relative;
  z-index: 2;
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

/* Console Header Bar */
.console-header-bar {
  background: linear-gradient(180deg, #0f0f1a 0%, #0a0a12 100%);
  border-bottom: 1px solid rgba(0, 255, 255, 0.15);
  padding: 12px 16px;
  flex-shrink: 0;
  position: relative;
  z-index: 10;
}

.header-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 16px;
  flex-wrap: wrap;
}

.console-title {
  display: flex;
  align-items: center;
  gap: 4px;
}

.title-prefix, .title-suffix {
  color: #00ffff;
  font-size: 11px;
  text-shadow: 0 0 10px rgba(0, 255, 255, 0.5);
}

.title-text {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 2px;
  color: #00ff88;
  text-shadow: 0 0 10px rgba(0, 255, 136, 0.5);
}

.header-metrics {
  display: flex;
  gap: 16px;
}

.metric-item {
  display: flex;
  align-items: center;
  gap: 4px;
}

.metric-label {
  color: #52525b;
  font-size: 10px;
  letter-spacing: 1px;
}

.metric-value {
  color: #00ffff;
  font-size: 12px;
  font-weight: 700;
  text-shadow: 0 0 8px rgba(0, 255, 255, 0.4);
}

.header-right {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
  justify-content: flex-end;
}

.input-wrapper {
  min-width: 220px;
}

/* Console Menu */
.console-menu {
  background: #0d0d12;
  border: 1px solid rgba(0, 255, 255, 0.2);
  border-radius: 4px;
  padding: 8px 0;
  min-width: 200px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
}

.menu-header {
  padding: 8px 12px;
  font-size: 9px;
  color: #52525b;
  letter-spacing: 2px;
  text-transform: uppercase;
  border-bottom: 1px solid rgba(0, 255, 255, 0.1);
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.menu-item:hover {
  background: rgba(0, 255, 255, 0.05);
}

.menu-item.active {
  background: rgba(0, 255, 255, 0.1);
}

.menu-check {
  color: #00ffff;
  font-size: 12px;
  width: 16px;
  text-align: center;
}

.menu-text {
  color: #a1a1aa;
  font-size: 11px;
}

.menu-item.active .menu-text {
  color: #00ffff;
}

.menu-divider {
  height: 1px;
  background: rgba(0, 255, 255, 0.1);
  margin: 4px 0;
}

/* Main Content */
.main-content-area {
  flex: 1;
  overflow: hidden;
  position: relative;
}

.wallpapers-grid-container {
  height: 100%;
  overflow-y: auto;
  padding: 20px;
}

/* Console Scroll */
.console-scroll::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

.console-scroll::-webkit-scrollbar-track {
  background: #0a0a0f;
}

.console-scroll::-webkit-scrollbar-thumb {
  background: rgba(0, 255, 255, 0.2);
  border-radius: 4px;
}

.console-scroll::-webkit-scrollbar-thumb:hover {
  background: rgba(0, 255, 255, 0.33);
}

/* Loading Grid */
.loading-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 24px;
}

.loading-card {
  background: rgba(10, 10, 15, 0.8);
  border: 1px solid #1a1a2e;
  border-radius: 8px;
  overflow: hidden;
}

.loading-image {
  aspect-ratio: 16 / 10;
  background: linear-gradient(90deg, #1a1a2e 25%, #252540 50%, #1a1a2e 75%);
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
}

.loading-footer {
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.loading-line {
  height: 12px;
  border-radius: 2px;
  background: linear-gradient(90deg, #1a1a2e 25%, #252540 50%, #1a1a2e 75%);
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
}

.loading-line--title {
  width: 70%;
}

.loading-line--meta {
  width: 40%;
  animation-delay: 0.15s;
}

@keyframes shimmer {
  0% { background-position: 200% 0; }
  100% { background-position: -200% 0; }
}

/* Wallpaper Grid */
.wallpaper-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 24px;
}

.wallpaper-item {
  transition: all 0.3s ease;
}

.wallpaper-card {
  position: relative;
  background: rgba(10, 10, 15, 0.9);
  border: 1px solid #1a1a2e;
  border-radius: 8px;
  overflow: hidden;
  transition: all 0.3s ease;
}

.wallpaper-card:hover {
  transform: translateY(-4px);
  border-color: rgba(0, 255, 255, 0.3);
}

.card-glow {
  position: absolute;
  inset: -2px;
  border-radius: 10px;
  background: linear-gradient(135deg, rgba(0, 255, 255, 0.3), transparent);
  opacity: 0;
  transition: opacity 0.3s ease;
  z-index: -1;
}

.wallpaper-card:hover .card-glow {
  opacity: 1;
}

.image-wrapper {
  position: relative;
  overflow: hidden;
  aspect-ratio: 16 / 10;
}

.wallpaper-image {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: transform 0.5s ease;
}

.wallpaper-card:hover .wallpaper-image {
  transform: scale(1.05);
}

.overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.6);
  opacity: 0;
  transition: opacity 0.3s ease;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
}

.wallpaper-card:hover .overlay {
  opacity: 1;
}

.overlay-btn {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: rgba(0, 255, 255, 0.2);
  border: 1px solid rgba(0, 255, 255, 0.4);
  color: #00ffff;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}

.overlay-btn:hover {
  background: rgba(0, 255, 255, 0.3);
  box-shadow: 0 0 20px rgba(0, 255, 255, 0.4);
}

.overlay-btn.secondary {
  border-color: rgba(255, 255, 255, 0.3);
  color: #a1a1aa;
}

.overlay-btn.secondary:hover {
  background: rgba(255, 255, 255, 0.1);
  border-color: rgba(255, 255, 255, 0.5);
}

.card-footer {
  padding: 12px;
}

.card-title {
  font-size: 12px;
  font-weight: 600;
  color: #e4e4e7;
  margin-bottom: 6px;
}

.card-meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.card-category {
  font-size: 10px;
  color: #00ffff;
  padding: 2px 8px;
  background: rgba(0, 255, 255, 0.1);
  border-radius: 2px;
  border: 1px solid rgba(0, 255, 255, 0.2);
}

.card-tag {
  font-size: 10px;
  color: #52525b;
}

/* Detail Drawer */
.detail-drawer {
  border-left: 1px solid rgba(0, 255, 255, 0.2);
  background: #0a0a0f !important;
}

.drawer-content {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #0a0a0f;
}

.drawer-header {
  position: relative;
}

.header-image {
  height: 180px;
  cursor: zoom-in;
}

.drawer-close,
.drawer-zoom {
  position: absolute;
  top: 12px;
  width: 32px;
  height: 32px;
  border-radius: 4px;
  background: rgba(0, 0, 0, 0.5);
  border: none;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  transition: all 0.2s ease;
}

.drawer-close {
  right: 12px;
}

.drawer-zoom {
  left: 12px;
}

.drawer-close:hover,
.drawer-zoom:hover {
  background: rgba(0, 0, 0, 0.7);
}

.drawer-body {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
}

.drawer-title {
  font-size: 14px;
  font-weight: 700;
  color: #00ff88;
  margin-bottom: 16px;
  letter-spacing: 1px;
  text-shadow: 0 0 10px rgba(0, 255, 136, 0.5);
}

.drawer-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 20px;
}

.tag-chip {
  font-size: 10px;
  padding: 4px 10px;
  border-radius: 2px;
  border: 1px solid #1a1a2e;
  color: #a1a1aa;
}

.tag-chip.primary {
  color: #00ffff;
  border-color: rgba(0, 255, 255, 0.3);
  background: rgba(0, 255, 255, 0.1);
}

.drawer-description {
  display: flex;
  gap: 8px;
  padding: 12px;
  background: rgba(0, 255, 255, 0.05);
  border-left: 2px solid #00ffff;
  border-radius: 0 4px 4px 0;
  margin-bottom: 20px;
}

.desc-prefix {
  color: #00ffff;
  font-weight: 700;
}

.desc-text {
  color: #a1a1aa;
  font-style: italic;
}

.divider-line {
  height: 1px;
  background: linear-gradient(90deg, transparent, rgba(0, 255, 255, 0.2), transparent);
  margin: 20px 0;
}

.bottom-spacer {
  height: 80px;
}

.drawer-actions {
  position: absolute;
  bottom: 20px;
  right: 20px;
  display: flex;
  gap: 12px;
  z-index: 10;
}

/* Image Viewer */
.image-viewer-dialog {
  background-color: rgba(0, 0, 0, 0.95) !important;
}

.image-viewer {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
}

.viewer-close {
  position: absolute;
  top: 20px;
  right: 20px;
  width: 40px;
  height: 40px;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.2);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  font-size: 18px;
  z-index: 10;
}

.viewer-close:hover {
  background: rgba(255, 255, 255, 0.2);
}

.viewer-image {
  max-width: 95vw;
  max-height: 95vh;
}

/* Console Snackbar */
.console-snackbar {
  background: #0d0d12 !important;
  border: 1px solid rgba(0, 255, 255, 0.3);
  border-radius: 4px !important;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
}

.snackbar-content {
  display: flex;
  align-items: center;
  gap: 10px;
}

.snackbar-icon {
  font-size: 16px;
  color: #00ff88;
}

.snackbar-text {
  font-family: 'JetBrains Mono', monospace;
  font-size: 12px;
  color: #e4e4e7;
}

/* Markdown */
.markdown-body :deep(img) {
  max-width: 100%;
  border-radius: 4px;
  margin: 16px 0;
  border: 1px solid #1a1a2e;
}

.markdown-body :deep(h1),
.markdown-body :deep(h2),
.markdown-body :deep(h3) {
  margin-top: 20px;
  margin-bottom: 10px;
  font-weight: 700;
  color: #00ffff;
  font-family: inherit;
}

.markdown-body :deep(p) {
  margin-bottom: 12px;
  line-height: 1.6;
  color: #a1a1aa;
  font-family: inherit;
}

.markdown-body :deep(pre) {
  background: rgba(0, 0, 0, 0.3);
  padding: 12px;
  border-radius: 4px;
  overflow-x: auto;
  margin-bottom: 12px;
  border: 1px solid #1a1a2e;
}

.markdown-body :deep(code) {
  font-family: 'JetBrains Mono', monospace;
  color: #00ff88;
}

/* =========================================
   Light Theme Styles (Tatooine Outpost)
   ========================================= */
.light-theme .wallpaper-manager {
  background: linear-gradient(180deg, #f5e6d3 0%, #efe0cc 50%, #f5e6d3 100%);
}

.light-theme .console-header-bar {
  background: linear-gradient(180deg, #efe0cc 0%, #e8d4bc 100%);
  border-bottom: 1px solid #b8860b;
}

.light-theme .title-prefix,
.light-theme .title-suffix {
  color: #b8860b;
  text-shadow: 0 0 8px rgba(184, 134, 11, 0.3);
}

.light-theme .title-text {
  color: #cd7f32;
  text-shadow: 0 0 8px rgba(205, 127, 50, 0.3);
}

.light-theme .metric-label {
  color: #9a7b5a;
}

.light-theme .metric-value {
  color: #cd7f32;
  text-shadow: 0 0 6px rgba(205, 127, 50, 0.25);
}

.light-theme .wallpaper-card {
  background: rgba(245, 230, 211, 0.9);
  border: 1px solid #d4a574;
}

.light-theme .wallpaper-card:hover {
  border-color: rgba(205, 127, 50, 0.5);
}

.light-theme .card-glow {
  background: linear-gradient(135deg, rgba(205, 127, 50, 0.3), transparent);
}

.light-theme .card-title {
  color: #3d2914;
}

.light-theme .card-category {
  color: #cd7f32;
  background: rgba(205, 127, 50, 0.1);
  border-color: rgba(205, 127, 50, 0.25);
}

.light-theme .card-tag {
  color: #8b7355;
}

.light-theme .detail-drawer {
  border-left: 1px solid rgba(184, 134, 11, 0.3);
  background: #efe0cc !important;
}

.light-theme .drawer-content {
  background: #efe0cc;
}

.light-theme .empty-title {
  color: #cd7f32;
}

.light-theme .empty-subtitle {
  color: #6b4423;
}

.light-theme .menu-header {
  color: #6b4423;
  border-bottom: 1px solid rgba(184, 134, 11, 0.2);
}

.light-theme .menu-item:hover {
  background: rgba(205, 127, 50, 0.08);
}

.light-theme .menu-item.active {
  background: rgba(205, 127, 50, 0.12);
}

.light-theme .menu-check {
  color: #cd7f32;
}

.light-theme .menu-text {
  color: #6b4423;
}

.light-theme .menu-item.active .menu-text {
  color: #cd7f32;
}

.light-theme .menu-divider {
  background: rgba(184, 134, 11, 0.2);
}

.light-theme .console-snackbar {
  background: #efe0cc !important;
  border: 1px solid rgba(184, 134, 11, 0.4);
}

.light-theme .snackbar-icon {
  color: #cd7f32;
}

.light-theme .snackbar-text {
  color: #3d2914;
}

.light-theme .markdown-body :deep(img) {
  border: 1px solid #d4a574;
}

.light-theme .markdown-body :deep(h1),
.light-theme .markdown-body :deep(h2),
.light-theme .markdown-body :deep(h3) {
  color: #cd7f32;
}

.light-theme .markdown-body :deep(p) {
  color: #6b4423;
}

.light-theme .markdown-body :deep(pre) {
  background: rgba(107, 68, 35, 0.08);
  border: 1px solid #d4a574;
}

.light-theme .markdown-body :deep(code) {
  color: #b8860b;
}

/* Overlay Light Theme */
.light-theme .overlay {
  background: rgba(107, 68, 35, 0.4);
}

.light-theme .overlay-btn {
  background: rgba(205, 127, 50, 0.2);
  border-color: rgba(205, 127, 50, 0.4);
  color: #cd7f32;
}

.light-theme .overlay-btn:hover {
  background: rgba(205, 127, 50, 0.3);
  box-shadow: 0 0 20px rgba(205, 127, 50, 0.3);
}

.light-theme .overlay-btn.secondary {
  border-color: rgba(107, 68, 35, 0.4);
  color: #6b4423;
}

.light-theme .overlay-btn.secondary:hover {
  background: rgba(107, 68, 35, 0.1);
  border-color: rgba(107, 68, 35, 0.5);
}

/* Drawer Light Theme */
.light-theme .drawer-close,
.light-theme .drawer-zoom {
  background: rgba(107, 68, 35, 0.5);
}

.light-theme .drawer-close:hover,
.light-theme .drawer-zoom:hover {
  background: rgba(107, 68, 35, 0.7);
}

/* Drawer Content Light Theme */
.light-theme .drawer-title {
  color: #3d2914;
  text-shadow: none;
}

.light-theme .tag-chip {
  color: #6b4423;
  border-color: rgba(184, 134, 11, 0.3);
}

.light-theme .tag-chip.primary {
  color: #cd7f32;
  border-color: rgba(205, 127, 50, 0.3);
  background: rgba(205, 127, 50, 0.1);
}

.light-theme .drawer-description {
  background: rgba(184, 134, 11, 0.05);
  border-left-color: #cd7f32;
}

.light-theme .desc-prefix {
  color: #cd7f32;
}

.light-theme .desc-text {
  color: #6b4423;
}

.light-theme .divider-line {
  background: linear-gradient(90deg, transparent, rgba(184, 134, 11, 0.2), transparent);
}

.light-theme .console-btn.primary {
  border-color: #cd7f32;
  color: #ffffff;
  background: #cd7f32;
}

.light-theme .console-btn.primary:hover {
  background: #b8860b;
}

.light-theme .loading-card {
  background: rgba(245, 230, 211, 0.9);
  border-color: #d4a574;
}

.light-theme .loading-image,
.light-theme .loading-line {
  background: linear-gradient(90deg, #e8d4bc 25%, #f0e0cc 50%, #e8d4bc 75%);
  background-size: 200% 100%;
}
</style>
