<template>
  <div class="wallpaper-manager fill-height d-flex flex-column bg-background">
    <!-- Top Toolbar -->
    <div
      class="bg-surface px-4 py-2 flex-grow-0 elevation-1 d-flex flex-column border-b"
      style="z-index: 10;"
    >
      <div class="d-flex align-center mb-2">
        <v-icon :icon="mdiWallpaper" color="primary" size="28" class="mr-3"></v-icon>
        <div>
          <div class="text-subtitle-1 font-weight-bold text-no-wrap text-high-emphasis">{{ $t('wallpapers.title') }}</div>
          <div class="text-caption text-medium-emphasis text-no-wrap">{{ $t('wallpapers.subtitle') }}</div>
        </div>
      </div>

      <!-- Search & Filter -->
      <div class="d-flex align-center gap-2 flex-nowrap overflow-x-auto hide-scrollbar" style="max-width: 100%;">
        <v-text-field
          v-model="searchQuery"
          :label="$t('common.search')"
          variant="outlined"
          density="compact"
          hide-details
          :prepend-inner-icon="mdiMagnify"
          class="search-field flex-shrink-1 flex-grow-1"
          style="min-width: 200px; max-width: 400px;"
          rounded="lg"
        ></v-text-field>

        <v-menu location="bottom end" :close-on-content-click="false">
          <template v-slot:activator="{ props }">
             <v-btn
              v-bind="props"
              variant="outlined"
              color="medium-emphasis"
              class="text-none flex-shrink-0"
              rounded="lg"
              border
              :min-width="smAndDown ? 0 : undefined"
              :class="{'px-2': smAndDown}"
            >
              <v-icon :icon="mdiFilterVariant" :class="{'mr-2': !smAndDown}"></v-icon>
              <span v-if="!smAndDown">
                {{ selectedCategories.includes('All') ? $t('wallpapers.all') : selectedCategories.join(', ') }}
              </span>
            </v-btn>
          </template>
          <v-list density="compact" class="py-0" min-width="240" rounded="lg" elevation="4" bg-color="surface">
            <v-list-item
              value="All"
              @click="toggleCategory('All')"
              :active="selectedCategories.includes('All')"
              color="primary"
              class="px-2"
            >
              <div class="d-flex align-center w-100">
                <v-icon :icon="selectedCategories.includes('All') ? mdiCheckboxMarked : mdiCheckboxBlankOutline" size="small" class="mr-2 flex-shrink-0"></v-icon>
                <div class="text-wrap">{{ $t('wallpapers.all') }}</div>
              </div>
            </v-list-item>
            <v-divider class="my-1"></v-divider>
            <v-list-item
              v-for="cat in categories"
              :key="cat"
              :value="cat"
              @click="toggleCategory(cat)"
              :active="selectedCategories.includes(cat)"
              color="primary"
              class="px-2"
            >
              <div class="d-flex align-center w-100">
                <v-icon :icon="selectedCategories.includes(cat) ? mdiCheckboxMarked : mdiCheckboxBlankOutline" size="small" class="mr-2 flex-shrink-0"></v-icon>
                <div class="text-wrap">{{ cat }}</div>
              </div>
            </v-list-item>
          </v-list>
        </v-menu>

        <v-menu location="bottom end">
          <template v-slot:activator="{ props }">
            <v-btn
              v-bind="props"
              variant="outlined"
              color="medium-emphasis"
              class="text-none flex-shrink-0"
              rounded="lg"
              border
              :min-width="smAndDown ? 0 : undefined"
              :class="{'px-2': smAndDown}"
            >
              <v-icon :icon="mdiMonitorDashboard" :class="{'mr-2': !smAndDown}"></v-icon>
              <span v-if="!smAndDown">{{ currentModeLabel }}</span>
            </v-btn>
          </template>
          <v-list density="compact" min-width="240" rounded="lg" elevation="4" bg-color="surface">
            <v-list-subheader class="text-caption font-weight-bold text-uppercase">{{ $t('wallpapers.wallpaperMode') }}</v-list-subheader>
            <v-list-item
              v-for="mode in modeOptions"
              :key="mode.value"
              :value="mode.value"
              @click="selectedMode = mode.value"
              :active="selectedMode === mode.value"
              color="primary"
            >
              <v-list-item-title class="text-body-2 text-wrap">{{ mode.label }}</v-list-item-title>
              <v-list-item-subtitle class="text-caption text-wrap" style="white-space: normal;">{{ mode.description }}</v-list-item-subtitle>
            </v-list-item>
          </v-list>
        </v-menu>

        <v-btn
          icon
          variant="text"
          color="medium-emphasis"
          @click="checkCurrentWallpaper"
          v-tooltip:bottom="$t('wallpapers.getCurrent')"
          class="flex-shrink-0"
        >
          <v-icon :icon="mdiImageSearch"></v-icon>
        </v-btn>

        <v-btn
          icon
          variant="text"
          color="medium-emphasis"
          @click="loadWallpapers"
          :loading="loading"
          class="flex-shrink-0"
        >
          <v-icon :icon="mdiRefresh"></v-icon>
        </v-btn>
      </div>
    </div>

    <!-- Main Content Area -->
    <div class="d-flex flex-grow-1 overflow-hidden position-relative">
      
      <!-- Wallpapers Grid (Scrollable) -->
      <div class="flex-grow-1 overflow-y-auto pa-6 custom-scrollbar">
        
        <v-row v-if="loading && !wallpapers.length">
          <v-col v-for="i in 8" :key="i" cols="12" sm="6" md="4" lg="3" xl="2">
            <v-skeleton-loader type="image, list-item-two-line" class="rounded-lg" elevation="0"></v-skeleton-loader>
          </v-col>
        </v-row>

        <div v-else-if="filteredWallpapers.length" class="wallpaper-grid">
           <div 
            v-for="wp in filteredWallpapers" 
            :key="wp.id" 
            class="wallpaper-item"
            @click="previewWallpaper(wp)"
          >
            <div class="wallpaper-card-inner elevation-1 rounded-lg overflow-hidden bg-surface">
              <div class="image-wrapper position-relative">
                <v-img
                  :src="wp.url"
                  aspect-ratio="1.6"
                  cover
                  class="bg-surface-variant transition-swing"
                >
                  <template v-slot:placeholder>
                    <div class="d-flex align-center justify-center fill-height">
                      <v-progress-circular indeterminate color="medium-emphasis"></v-progress-circular>
                    </div>
                  </template>
                </v-img>
                
                <!-- Overlay Actions -->
                <div class="overlay d-flex align-center justify-center gap-2">
                  <v-btn
                    icon
                    variant="flat"
                    color="surface"
                    size="small"
                    class="elevation-2"
                    @click.stop="setWallpaper(wp)"
                    :loading="settingId === wp.id"
                    v-tooltip:top="$t('wallpapers.setDesktop')"
                  >
                    <v-icon :icon="mdiMonitorScreenshot" size="18" color="primary"></v-icon>
                  </v-btn>
                   <v-btn
                    icon
                    variant="flat"
                    color="surface"
                    size="small"
                    class="elevation-2"
                    @click.stop="previewWallpaper(wp)"
                    v-tooltip:top="$t('wallpapers.viewDetails')"
                  >
                    <v-icon :icon="mdiInformationVariant" size="18" color="medium-emphasis"></v-icon>
                  </v-btn>
                </div>
              </div>

              <div class="pa-3">
                <div class="text-subtitle-2 font-weight-bold text-truncate mb-1 text-high-emphasis">
                  {{ wp.title }}
                </div>
                <div class="d-flex align-center justify-space-between">
                  <v-chip
                    size="x-small"
                    color="primary"
                    variant="tonal"
                    label
                    class="font-weight-medium px-2"
                  >
                    {{ wp.category }}
                  </v-chip>
                  <div class="text-caption text-medium-emphasis" v-if="wp.tags.length">
                    #{{ wp.tags[0] }}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div v-else class="fill-height d-flex flex-column align-center justify-center text-medium-emphasis pb-16">
          <v-icon :icon="mdiWallpaper" size="64" class="mb-4 text-disabled"></v-icon>
          <div class="text-h6 text-medium-emphasis">{{ t('common.noData') }}</div>
          <v-btn
            variant="flat"
            color="primary"
            class="mt-4 text-none px-6"
            rounded="lg"
            @click="loadWallpapers"
            :loading="loading"
          >
            {{ t('common.refresh') }}
          </v-btn>
        </div>
      </div>

      <!-- Detail Drawer (Right Side) -->
      <v-navigation-drawer
        v-model="drawer"
        location="right"
        width="400"
        temporary
        elevation="4"
        class="detail-drawer"
        scrim="rgba(0,0,0,0.3)"
      >
        <div v-if="currentPreview" class="d-flex flex-column fill-height bg-surface">
          <!-- Drawer Header -->
          <div class="position-relative">
            <v-img
              :src="currentPreview.url"
              height="160"
              cover
              class="bg-surface-variant"
              @click="showImageViewer = true"
              style="cursor: zoom-in;"
            >
            </v-img>
            <v-btn
              icon
              variant="flat"
              color="surface"
              size="small"
              class="position-absolute top-0 right-0 ma-2 elevation-2"
              @click="drawer = false"
            >
              <v-icon :icon="mdiClose" size="20"></v-icon>
            </v-btn>
             <v-btn
              icon
              variant="flat"
              color="surface"
              size="small"
              class="position-absolute top-0 left-0 ma-2 elevation-2"
              @click="showImageViewer = true"
            >
              <v-icon :icon="mdiMagnify" size="20"></v-icon>
            </v-btn>
          </div>

          <!-- Drawer Content -->
          <div class="flex-grow-1 overflow-y-auto pa-4 custom-scrollbar pb-16">
            
            <h2 class="text-h6 font-weight-bold mb-2">{{ currentPreview.title }}</h2>

            <div class="d-flex align-center flex-wrap gap-2 mb-3">
              <v-chip
                color="primary"
                variant="flat"
                size="small"
                label
                class="font-weight-bold"
              >
                {{ currentPreview.category }}
              </v-chip>
               <v-chip
                v-for="tag in currentPreview.tags"
                :key="tag"
                variant="tonal"
                size="small"
                color="medium-emphasis"
              >
                #{{ tag }}
              </v-chip>
            </div>

            <div v-if="currentPreview.description" class="text-body-1 text-medium-emphasis mb-6 font-italic border-s-4 pl-4 py-1 border-primary bg-surface-variant rounded-e">
              {{ currentPreview.description }}
            </div>

            <v-divider class="mb-6"></v-divider>

            <div class="markdown-body text-high-emphasis" v-html="renderMarkdown(currentPreview.content || '')"></div>
            
            <!-- Bottom spacing for FAB -->
            <div class="height-64"></div>
          </div>

          <!-- Floating Action Button -->
          <div class="position-absolute bottom-0 right-0 ma-4 d-flex flex-column align-end gap-2">
            <v-btn
              v-if="isCurrentWallpaper"
              icon
              color="secondary"
              class="elevation-3"
              @click="openWallpaperFolder"
              v-tooltip:left="$t('wallpapers.openFolder')"
            >
              <v-icon :icon="mdiFolderOpen"></v-icon>
            </v-btn>

            <v-btn
              icon
              color="primary"
              class="elevation-4"
              @click="setWallpaper(currentPreview)"
              :loading="settingId === currentPreview.id"
            >
              <v-icon :icon="mdiMonitorScreenshot"></v-icon>
              <v-tooltip activator="parent" location="left">
                <div class="d-flex flex-column align-center">
                  <span>{{ $t('wallpapers.setDesktop') }}</span>
                  <span class="text-caption" style="opacity: 0.8">({{ currentModeLabel }})</span>
                </div>
              </v-tooltip>
            </v-btn>
          </div>
        </div>
      </v-navigation-drawer>

    </div>

    <!-- Image Viewer Dialog -->
    <v-dialog v-model="showImageViewer" fullscreen z-index="2500" class="image-viewer-dialog" transition="fade-transition">
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
          max-height="95vh"
          max-width="95vw"
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
      rounded="pill"
      elevation="4"
    >
      <div class="d-flex align-center">
        <v-icon :icon="snackbar.color === 'success' ? mdiCheckCircle : mdiAlertCircle" class="mr-2"></v-icon>
        {{ snackbar.text }}
      </div>
    </v-snackbar>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useDisplay } from 'vuetify'
import { 
  mdiWallpaper, mdiRefresh, mdiMonitorScreenshot, mdiClose, mdiMagnify, 
  mdiFilterVariant, mdiMonitorDashboard,
  mdiInformationVariant, mdiCheckCircle, mdiAlertCircle,
  mdiCheckboxMarked, mdiCheckboxBlankOutline, mdiImageSearch, mdiFolderOpen
} from '@mdi/js'
import { getWallpapers, syncWallpapers, setDesktopWallpaper, getCurrentWallpaper, showInFolder, type WallpaperItem, WallpaperMode } from '@/api/wallpaper'
import { useI18n } from 'vue-i18n'
import MarkdownIt from 'markdown-it'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

const { smAndDown } = useDisplay()

const md = new MarkdownIt({
  html: true,
  linkify: true,
  typographer: true
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
  color: 'success'
})

const modeOptions = computed(() => [
  { value: WallpaperMode.Center, label: t('wallpapers.modes.center.label'), description: t('wallpapers.modes.center.description') },
  { value: WallpaperMode.Crop, label: t('wallpapers.modes.crop.label'), description: t('wallpapers.modes.crop.description') },
  { value: WallpaperMode.Fit, label: t('wallpapers.modes.fit.label'), description: t('wallpapers.modes.fit.description') },
  { value: WallpaperMode.Span, label: t('wallpapers.modes.span.label'), description: t('wallpapers.modes.span.description') },
  { value: WallpaperMode.Stretch, label: t('wallpapers.modes.stretch.label'), description: t('wallpapers.modes.stretch.description') },
  { value: WallpaperMode.Tile, label: t('wallpapers.modes.tile.label'), description: t('wallpapers.modes.tile.description') },
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
    result = result.filter(w => 
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
    // 尝试从路径中匹配已知的壁纸
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
.wallpaper-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 24px;
}

.wallpaper-item {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.wallpaper-card-inner {
  transition: all 0.3s ease;
  border: 1px solid rgba(0,0,0,0.05);
}

.wallpaper-item:hover .wallpaper-card-inner {
  transform: translateY(-4px);
  box-shadow: 0 12px 24px -10px rgba(0,0,0,0.15) !important;
  border-color: rgba(var(--v-theme-primary), 0.2);
}

.image-wrapper {
  overflow: hidden;
}

.image-wrapper .v-img {
  transition: transform 0.5s ease;
}

.wallpaper-item:hover .v-img {
  transform: scale(1.05);
}

.overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0,0,0,0.3);
  opacity: 0;
  transition: opacity 0.3s ease;
}

.wallpaper-item:hover .overlay {
  opacity: 1;
}

.detail-drawer {
  border-left: 1px solid rgba(0,0,0,0.05);
}

.custom-scrollbar {
  scrollbar-width: thin;
  scrollbar-color: rgba(0,0,0,0.2) transparent;
}

.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background-color: rgba(0,0,0,0.2);
  border-radius: 3px;
}

.text-shadow {
  text-shadow: 0 2px 4px rgba(0,0,0,0.5);
}

.markdown-body :deep(img) {
  max-width: 100%;
  border-radius: 8px;
  margin: 16px 0;
  box-shadow: 0 4px 12px rgba(0,0,0,0.1);
}

.markdown-body :deep(h1), 
.markdown-body :deep(h2), 
.markdown-body :deep(h3) {
  margin-top: 24px;
  margin-bottom: 12px;
  font-weight: 700;
  color: rgb(var(--v-theme-on-surface));
}

.markdown-body :deep(p) {
  margin-bottom: 16px;
  line-height: 1.7;
  color: rgb(var(--v-theme-on-surface));
}

.markdown-body :deep(pre) {
  background-color: rgb(var(--v-theme-surface-variant));
  padding: 16px;
  border-radius: 8px;
  overflow-x: auto;
  margin-bottom: 16px;
  border: 1px solid rgba(var(--v-border-color), var(--v-border-opacity));
}

.image-viewer-dialog {
  background-color: rgba(0, 0, 0, 0.95);
  backdrop-filter: blur(10px);
}

.image-viewer-card {
  background-color: transparent !important;
  box-shadow: none !important;
}

.gap-2 { gap: 8px; }
.gap-4 { gap: 16px; }

.hide-scrollbar {
  scrollbar-width: none; /* Firefox */
  -ms-overflow-style: none; /* IE and Edge */
}

.hide-scrollbar::-webkit-scrollbar {
  display: none; /* Chrome, Safari and Opera */
}
</style>
