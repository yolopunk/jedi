<template>
  <div class="wallpaper-manager">
    <!-- Header Area -->
    <v-card class="jedi-card mb-4 px-4 py-3 d-flex flex-column rounded-lg" elevation="0">
      <div class="d-flex align-center justify-space-between w-100 mb-2">
        <div class="d-flex align-center">
          <v-icon :icon="mdiWallpaper" color="primary" class="mr-3" size="32"></v-icon>
          <div>
            <h2 class="text-h6 font-weight-bold">{{ $t('wallpapers.title') }}</h2>
            <div class="text-caption text-secondary">{{ $t('wallpapers.subtitle') }}</div>
          </div>
        </div>
        
        <v-btn
          :prepend-icon="mdiRefresh"
          variant="text"
          size="small"
          @click="loadWallpapers"
          :loading="loading"
        >
          {{ $t('common.refresh') }}
        </v-btn>
      </div>
      
      <!-- Category Filter -->
      <div class="w-100">
        <v-chip-group v-model="selectedCategory" filter mandatory show-arrows>
          <v-chip value="All" filter-icon="mdi-check" variant="outlined" size="small">{{ $t('wallpapers.all') }}</v-chip>
          <v-chip v-for="cat in categories" :key="cat" :value="cat" filter-icon="mdi-check" variant="outlined" size="small">
            {{ cat }}
          </v-chip>
        </v-chip-group>
      </div>
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
    <v-dialog v-model="showPreview" max-width="900">
      <v-card v-if="currentPreview" class="rounded-lg overflow-hidden">
        <v-img :src="currentPreview.url" cover max-height="600"></v-img>
        <v-card-title class="d-flex justify-space-between align-center pt-4">
          <div class="text-truncate mr-2">{{ currentPreview.title }}</div>
          <div class="d-flex flex-wrap gap-1 justify-end">
            <v-chip size="small" color="primary" class="mr-1">{{ currentPreview.category }}</v-chip>
            <v-chip v-for="tag in currentPreview.tags" :key="tag" size="small" variant="outlined" class="mr-1">
              {{ tag }}
            </v-chip>
          </div>
        </v-card-title>
        <v-card-text>
          {{ currentPreview.description }}
        </v-card-text>
        <v-card-actions class="pa-4">
          <v-spacer></v-spacer>
          <v-btn variant="text" @click="showPreview = false">{{ $t('common.close') }}</v-btn>
          <v-btn
            color="primary"
            variant="elevated"
            :prepend-icon="mdiMonitorScreenshot"
            @click="setWallpaper(currentPreview)"
            :loading="settingId === currentPreview.id"
          >
            {{ $t('wallpapers.setDesktop') }}
          </v-btn>
        </v-card-actions>
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
import { ref, computed, onMounted } from 'vue'
import { mdiWallpaper, mdiRefresh, mdiMonitorScreenshot } from '@mdi/js'
import { getWallpapers, setDesktopWallpaper, type WallpaperItem } from '@/api/wallpaper'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
const loading = ref(false)
const settingId = ref<string | null>(null)
const wallpapers = ref<WallpaperItem[]>([])
const selectedCategory = ref('All')
const showPreview = ref(false)
const currentPreview = ref<WallpaperItem | null>(null)
const snackbar = ref({
  show: false,
  text: '',
  color: 'success'
})

const categories = computed(() => {
  const cats = new Set(wallpapers.value.map(w => w.category))
  return Array.from(cats)
})

const filteredWallpapers = computed(() => {
  if (selectedCategory.value === 'All') return wallpapers.value
  return wallpapers.value.filter(w => w.category === selectedCategory.value)
})

async function loadWallpapers() {
  loading.value = true
  console.log('Loading wallpapers...')
  try {
    const data = await getWallpapers()
    console.log('Wallpapers loaded:', data)
    wallpapers.value = data
  } catch (error) {
    console.error(error)
    showSnackbar(t('wallpapers.loadError'), 'error')
  } finally {
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

onMounted(() => {
  loadWallpapers()
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
</style>
