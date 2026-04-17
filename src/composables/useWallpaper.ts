import { onUnmounted, ref } from 'vue'
import { getWallpapers, setDesktopWallpaper, WallpaperMode } from '@/api/wallpaper'
import { useStorage } from '@/composables/useStorage'

const { getItem, setItem } = useStorage()

export interface WallpaperSettings {
  autoUpdate: boolean
  frequencyDays: number
  selectedCategories: string[]
  lastUpdate: number
  mode: WallpaperMode
}

const defaultSettings: WallpaperSettings = {
  autoUpdate: false,
  frequencyDays: 1,
  selectedCategories: [],
  lastUpdate: 0,
  mode: WallpaperMode.Crop,
}

// Global state
const settings = ref<WallpaperSettings>({ ...defaultSettings })
const loaded = ref(false)
let timer: ReturnType<typeof setInterval> | null = null

export function useWallpaper() {
  onUnmounted(() => {
    stopAutoUpdateCheck()
  })

  async function loadSettings() {
    if (loaded.value) return // Already loaded

    const saved = await getItem<WallpaperSettings>('wallpaper_settings')
    if (saved) {
      // Merge with defaults to handle new fields
      settings.value = { ...defaultSettings, ...saved }
    }
    loaded.value = true
  }

  async function saveSettings(newSettings: WallpaperSettings) {
    settings.value = newSettings
    await setItem('wallpaper_settings', newSettings)
  }

  async function checkAndRunAutoUpdate() {
    if (!loaded.value) await loadSettings()

    if (!settings.value.autoUpdate) return

    const now = Date.now()
    const msPerDay = 24 * 60 * 60 * 1000
    const nextUpdate = settings.value.lastUpdate + settings.value.frequencyDays * msPerDay

    if (now >= nextUpdate) {
      console.log('Auto-updating wallpaper...')
      try {
        const wallpapers = await getWallpapers()
        let candidates = wallpapers

        // Filter by category if selected
        if (settings.value.selectedCategories.length > 0) {
          candidates = wallpapers.filter(w =>
            settings.value.selectedCategories.includes(w.category)
          )
        }

        if (candidates.length === 0 && wallpapers.length > 0) {
          // Fallback to all if filter result is empty
          candidates = wallpapers
        }

        if (candidates.length > 0) {
          const random = candidates[Math.floor(Math.random() * candidates.length)]
          await setDesktopWallpaper(random.url, settings.value.mode)

          // Update timestamp
          settings.value.lastUpdate = now
          await saveSettings(settings.value)
          console.log('Wallpaper auto-updated to:', random.title)
        }
      } catch (error) {
        console.error('Failed to auto-update wallpaper:', error)
      }
    }
  }

  function startAutoUpdateCheck() {
    checkAndRunAutoUpdate()
    if (timer) clearInterval(timer)
    // Check every hour
    timer = setInterval(checkAndRunAutoUpdate, 60 * 60 * 1000)
  }

  function stopAutoUpdateCheck() {
    if (timer) clearInterval(timer)
    timer = null
  }

  return {
    settings,
    loadSettings,
    saveSettings,
    checkAndRunAutoUpdate,
    startAutoUpdateCheck,
    stopAutoUpdateCheck,
  }
}
