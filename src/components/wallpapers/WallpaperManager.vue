<template>
  <div class="wallpaper-manager">
    <!-- Header Area -->
    <v-card class="jedi-card mb-4 rounded-lg" elevation="0">
      <div class="d-flex align-center justify-space-between px-4 pt-4 pb-2">
        <div class="d-flex align-center">
          <v-icon :icon="mdiWallpaper" color="primary" class="mr-3" size="32"></v-icon>
          <div>
            <h2 class="text-h6 font-weight-bold">{{ $t('wallpapers.title') }}</h2>
            <div class="text-caption text-secondary">{{ $t('wallpapers.subtitle') }}</div>
          </div>
        </div>
        
        <div class="d-flex align-center" style="gap: 12px; width: 300px;">
          <v-text-field
            v-model="searchQuery"
            placeholder="搜索..."
            density="compact"
            variant="outlined"
            hide-details
            :prepend-inner-icon="mdiMagnify"
            bg-color="surface"
            single-line
          ></v-text-field>
          
          <v-btn
            icon
            variant="text"
            @click="loadWallpapers"
            :loading="loading"
            color="primary"
          >
            <v-icon :icon="mdiRefresh"></v-icon>
          </v-btn>
        </div>
      </div>
      
      <v-divider class="mx-4"></v-divider>
      
      <!-- Category Tabs -->
      <v-tabs
        v-model="selectedCategory"
        show-arrows
        color="primary"
        align-tabs="start"
        class="px-2"
        density="compact"
      >
        <v-tab value="All" class="text-body-2 text-capitalize">{{ $t('wallpapers.all') }}</v-tab>
        <v-tab 
          v-for="cat in categories" 
          :key="cat" 
          :value="cat"
          class="text-body-2 text-capitalize"
        >
          {{ cat }}
        </v-tab>
      </v-tabs>
    </v-card>

    <!-- Wallpapers Grid -->
    <v-row v-if="loading && !wallpapers.length">
      <v-col v-for="i in 6" :key="i" cols="12" md="4" lg="3">
        <v-skeleton-loader type="image, article" height="250"></v-skeleton-loader>
      </v-col>
    </v-row>

    <v-row v-else-if="filteredWallpapers.length">
      <v-col v-for="wp in filteredWallpapers" :key="wp.id" cols="12" md="4" lg="3">
        <v-card class="wallpaper-card h-100 d-flex flex-column" hover @click="previewWallpaper(wp)">
          <v-img
            :src="wp.url"
            height="180"
            cover
            class="align-end"
            gradient="to bottom, rgba(0,0,0,.1), rgba(0,0,0,.7)"
          >
            <v-card-title class="text-white text-body-2 font-weight-bold pb-2 pl-3">
              {{ wp.title }}
            </v-card-title>
          </v-img>
          
          <v-card-text class="pt-3 pb-0 flex-grow-1">
            <div class="d-flex flex-wrap gap-1 mb-2">
              <v-chip size="x-small" color="primary" variant="flat" class="mr-1">
                {{ wp.category }}
              </v-chip>
              <v-chip 
                v-for="tag in wp.tags.slice(0, 2)" 
                :key="tag"
                size="x-small" 
                variant="outlined" 
                class="mr-1"
              >
                {{ tag }}
              </v-chip>
              <v-chip v-if="wp.tags.length > 2" size="x-small" variant="text" class="px-1">
                +{{ wp.tags.length - 2 }}
              </v-chip>
            </div>
            <div class="text-caption text-medium-emphasis text-truncate-2">
              {{ wp.description }}
            </div>
          </v-card-text>
          
          <v-card-actions>
            <v-spacer></v-spacer>
            <v-btn
              size="small"
              variant="text"
              color="primary"
              :prepend-icon="mdiMonitorScreenshot"
              @click.stop="setWallpaper(wp)"
              :loading="settingId === wp.id"
            >
              {{ $t('wallpapers.setDesktop') }}
            </v-btn>
          </v-card-actions>
        </v-card>
      </v-col>
    </v-row>

    <div v-else class="d-flex flex-column align-center justify-center py-12 text-medium-emphasis">
      <v-icon :icon="mdiWallpaper" size="64" class="mb-4" color="grey-lighten-1"></v-icon>
      <div class="text-h6">{{ t('common.noData') }}</div>
      <v-btn variant="text" color="primary" class="mt-2" @click="loadWallpapers" :loading="loading">
        {{ t('common.refresh') }}
      </v-btn>
    </div>

    <!-- Preview Dialog -->
    <v-dialog v-model="showPreview" fullscreen transition="dialog-bottom-transition">
      <v-card v-if="currentPreview" class="rounded-0">
        <v-toolbar color="surface" density="compact" class="border-b">
          <v-toolbar-title class="text-subtitle-1 font-weight-bold ml-4">{{ currentPreview.title }}</v-toolbar-title>
          <v-spacer></v-spacer>
          <v-toolbar-items>
            <v-btn
              variant="text"
              color="primary"
              :prepend-icon="mdiMonitorScreenshot"
              @click="setWallpaper(currentPreview)"
              :loading="settingId === currentPreview.id"
            >
              {{ $t('wallpapers.setDesktop') }}
            </v-btn>
          </v-toolbar-items>
          <v-btn icon @click="showPreview = false" class="mr-2">
            <v-icon :icon="mdiClose"></v-icon>
          </v-btn>
        </v-toolbar>
        
        <v-container class="fill-height align-start overflow-y-auto" style="max-width: 900px">
          <v-row>
            <v-col cols="12">
              <v-card class="mb-6 rounded-lg elevation-2 position-relative overflow-hidden group">
                <v-img 
                  :src="currentPreview.url" 
                  cover 
                  max-height="500"
                  class="bg-grey-lighten-2"
                ></v-img>
                <div class="position-absolute top-0 right-0 pa-4">
                  <v-btn
                    icon
                    color="surface"
                    variant="flat"
                    @click="showImageViewer = true"
                  >
                    <v-icon :icon="mdiMagnify"></v-icon>
                  </v-btn>
                </div>
              </v-card>
              
              <div class="d-flex align-center flex-wrap gap-2 mb-6">
                <v-chip color="primary" label variant="flat" size="small" class="font-weight-bold">{{ currentPreview.category }}</v-chip>
                <v-chip 
                  v-for="tag in currentPreview.tags" 
                  :key="tag" 
                  variant="tonal" 
                  size="small"
                  color="secondary"
                  class="text-body-2"
                >
                  #{{ tag }}
                </v-chip>
              </div>

              <div class="text-h4 font-weight-bold mb-4">{{ currentPreview.title }}</div>
              
              <div v-if="currentPreview.description" class="text-body-1 text-medium-emphasis mb-6 font-italic">
                {{ currentPreview.description }}
              </div>
              
              <v-divider class="mb-6"></v-divider>
              
              <div class="markdown-body" v-html="renderMarkdown(currentPreview.content || '')"></div>
            </v-col>
          </v-row>
        </v-container>
      </v-card>
    </v-dialog>

    <!-- Image Viewer Dialog -->
    <v-dialog v-model="showImageViewer" fullscreen z-index="2500" class="image-viewer-dialog">
      <v-card class="d-flex align-center justify-center rounded-0 image-viewer-card" width="100%" height="100%">
        <v-btn
          icon
          variant="text"
          color="white"
          class="position-absolute top-0 right-0 ma-4"
          style="z-index: 10"
          @click="showImageViewer = false"
        >
          <v-icon :icon="mdiClose" size="32"></v-icon>
        </v-btn>
        
        <v-img
          v-if="currentPreview"
          :src="currentPreview.url"
          max-height="90vh"
          max-width="90vw"
          width="auto"
          height="auto"
          contain
        ></v-img>
      </v-card>
    </v-dialog>

    <v-snackbar
      v-model="snackbar.show"
      :color="snackbar.color"
      timeout="3000"
      location="top"
    >
      {{ snackbar.text }}
    </v-snackbar>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { mdiWallpaper, mdiRefresh, mdiMonitorScreenshot, mdiClose, mdiMagnify } from '@mdi/js'
import { getWallpapers, syncWallpapers, setDesktopWallpaper, type WallpaperItem } from '@/api/wallpaper'
import { useI18n } from 'vue-i18n'
import MarkdownIt from 'markdown-it'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

const md = new MarkdownIt({
  html: true,
  linkify: true,
  typographer: true
})

// Custom render rule to hide the first image if it duplicates the main image
// Since we can't easily check equality in the render rule, we'll just use a CSS class approach or
// preprocess the content. Preprocessing is safer.
function renderMarkdown(content: string) {
  if (!content) return ''
  
  // Simple heuristic: if the content starts with an image tag, remove it
  // Regex for ![alt](url) or <img src="url">
  // We'll strip the first image if it appears within the first few lines
  let processedContent = content.trim()
  
  // Check if it starts with an image
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
const selectedCategory = ref('All')
const searchQuery = ref('')
const showPreview = ref(false)
const currentPreview = ref<WallpaperItem | null>(null)
const snackbar = ref({
  show: false,
  text: '',
  color: 'success'
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
  
  if (selectedCategory.value !== 'All') {
    result = result.filter(w => w.category === selectedCategory.value)
  }
  
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(w => 
      w.title.toLowerCase().includes(query) || 
      w.description.toLowerCase().includes(query) ||
      w.tags.some(tag => tag.toLowerCase().includes(query))
    )
  }
  
  return result
})

async function loadWallpapers() {
  loading.value = true
  console.log('Loading wallpapers...')
  try {
    // 1. Load from cache first (fast)
    const cachedData = await getWallpapers()
    if (cachedData && cachedData.length > 0) {
      console.log('Loaded from cache:', cachedData.length)
      wallpapers.value = cachedData
      loading.value = false // Show immediately
    }
    
    // 2. Trigger background sync
    console.log('Starting background sync...')
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
    await setDesktopWallpaper(wp.url)
    showSnackbar(t('wallpapers.setSuccess'), 'success')
  } catch (error) {
    console.error(error)
    showSnackbar(t('wallpapers.setError'), 'error')
  } finally {
    settingId.value = null
    showPreview.value = false // Close dialog if setting from there
  }
}

function previewWallpaper(wp: WallpaperItem) {
  currentPreview.value = wp
  showPreview.value = true
}

function showSnackbar(text: string, color: string) {
  snackbar.value = { show: true, text, color }
}

onMounted(async () => {
  // Setup event listeners
  unlistenBatch = await listen<WallpaperItem[]>('wallpaper-batch', (event) => {
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
.text-truncate-2 {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
.wallpaper-card {
  transition: transform 0.2s, box-shadow 0.2s;
  border-radius: 12px;
  overflow: hidden;
}
.wallpaper-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 6px 16px rgba(0,0,0,0.12) !important;
}

.image-viewer-card {
  background-color: rgba(0, 0, 0, 0.9) !important;
  backdrop-filter: blur(10px);
}

.markdown-body {
  font-size: 16px;
  line-height: 1.6;
  color: rgba(var(--v-theme-on-surface), 0.87);
}

.markdown-body :deep(h1),
.markdown-body :deep(h2),
.markdown-body :deep(h3) {
  margin-top: 24px;
  margin-bottom: 16px;
  font-weight: 600;
  line-height: 1.25;
}

.markdown-body :deep(h1) {
  font-size: 2em;
  padding-bottom: 0.3em;
  border-bottom: 1px solid rgba(var(--v-border-color), 0.2);
}

.markdown-body :deep(h2) {
  font-size: 1.5em;
  padding-bottom: 0.3em;
  border-bottom: 1px solid rgba(var(--v-border-color), 0.2);
}

.markdown-body :deep(p) {
  margin-bottom: 16px;
}

.markdown-body :deep(ul),
.markdown-body :deep(ol) {
  padding-left: 2em;
  margin-bottom: 16px;
}

.markdown-body :deep(img) {
  max-width: 100%;
  border-radius: 8px;
  margin: 16px 0;
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
}

.markdown-body :deep(pre) {
  background-color: rgba(var(--v-theme-surface-variant), 0.5);
  padding: 16px;
  border-radius: 8px;
  overflow: auto;
  margin-bottom: 16px;
}

.markdown-body :deep(code) {
  background-color: rgba(var(--v-theme-surface-variant), 0.5);
  padding: 0.2em 0.4em;
  border-radius: 4px;
}

.markdown-body :deep(pre) :deep(code) {
  background-color: transparent;
  padding: 0;
}

.markdown-body :deep(blockquote) {
  margin: 0 0 16px;
  padding: 0 1em;
  color: rgba(var(--v-theme-on-surface), 0.6);
  border-left: 0.25em solid rgba(var(--v-border-color), 0.5);
}

.markdown-body :deep(a) {
  color: rgb(var(--v-theme-primary));
  text-decoration: none;
}

.markdown-body :deep(a:hover) {
  text-decoration: underline;
}

.markdown-body :deep(table) {
  border-spacing: 0;
  border-collapse: collapse;
  margin-bottom: 16px;
  width: 100%;
  overflow: auto;
  display: block;
}

.markdown-body :deep(table) th,
.markdown-body :deep(table) td {
  padding: 6px 13px;
  border: 1px solid rgba(var(--v-border-color), 0.2);
}

.markdown-body :deep(table) tr:nth-child(2n) {
  background-color: rgba(var(--v-theme-surface-variant), 0.1);
}
</style>
