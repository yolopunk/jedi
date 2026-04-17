import dayjs from 'dayjs'
import { type Ref, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { checkUpdate, downloadAndInstallUpdate, type UpdateInfo } from '@/api/update'
import { useStorage } from '@/composables/useStorage'

// Global state
const hasUpdate: Ref<boolean> = ref(false)
const updateInfo: Ref<UpdateInfo | null> = ref(null)
const isChecking: Ref<boolean> = ref(false)
const isInstalling: Ref<boolean> = ref(false)
const autoUpdateEnabled: Ref<boolean> = ref(true)
const lastCheckTime: Ref<number> = ref(0)
let checkInterval: ReturnType<typeof setInterval> | null = null
let isInitialized = false

export function useUpdate() {
  const { t } = useI18n()
  const { getItem, setItem } = useStorage()

  const AutoCheckInterval = 24 * 60 * 60 * 1000 // 24 hours

  // Initialize settings from storage
  const initSettings = async () => {
    if (isInitialized) return

    const savedAutoUpdate = await getItem<boolean>('settings.autoUpdate')
    if (savedAutoUpdate !== null) {
      autoUpdateEnabled.value = savedAutoUpdate
    }

    isInitialized = true
  }

  // Watch for changes and save to storage
  watch(autoUpdateEnabled, async newValue => {
    await setItem('settings.autoUpdate', newValue)
    if (newValue) {
      startAutoCheck()
    } else {
      stopAutoCheck()
    }
  })

  const checkForUpdate = async () => {
    if (isChecking.value) return

    isChecking.value = true
    try {
      const result = await checkUpdate()
      if (result) {
        hasUpdate.value = true
        updateInfo.value = result
      } else {
        hasUpdate.value = false
        updateInfo.value = null
      }
      lastCheckTime.value = Date.now()
    } catch (error) {
      console.error('Failed to check for update:', error)
    } finally {
      isChecking.value = false
    }
  }

  const installUpdate = async () => {
    if (isInstalling.value) return

    isInstalling.value = true
    try {
      // Using progress callback to show download progress
      await downloadAndInstallUpdate((total, current) => {
        console.log(`Download progress: ${current}/${total}`)
      })
    } catch (error) {
      console.error('Failed to install update:', error)
      throw error
    } finally {
      isInstalling.value = false
    }
  }

  const startAutoCheck = async () => {
    await initSettings()

    if (!autoUpdateEnabled.value) return
    if (checkInterval) return // Already running

    // Immediate check on start
    checkForUpdate()

    // Set up periodic checks
    checkInterval = setInterval(() => {
      if (autoUpdateEnabled.value) {
        checkForUpdate()
      }
    }, AutoCheckInterval)
  }

  const stopAutoCheck = () => {
    if (checkInterval) {
      clearInterval(checkInterval)
      checkInterval = null
    }
  }

  const formatLastCheckTime = () => {
    if (lastCheckTime.value === 0) {
      return t('settings.lastCheckTime', { time: t('settings.never') })
    }
    return t('settings.lastCheckTime', {
      time: dayjs(lastCheckTime.value).format('YYYY-MM-DD HH:mm:ss'),
    })
  }

  return {
    hasUpdate,
    updateInfo,
    isChecking,
    isInstalling,
    autoUpdateEnabled,
    lastCheckTime,
    checkForUpdate,
    installUpdate,
    startAutoCheck,
    stopAutoCheck,
    formatLastCheckTime,
  }
}
