<template>
  <v-dialog v-model="dialogModel" max-width="700" class="jedi-dialog-card">
    <v-card class="jedi-dialog-card">
      <v-toolbar color="surface" class="px-4 jedi-dialog-header border-b">
        <v-icon :icon="mdiCog" color="primary" class="mr-2"></v-icon>
        <v-toolbar-title class="font-weight-medium">应用设置</v-toolbar-title>
        <v-spacer></v-spacer>
        <v-btn :icon="mdiClose" variant="text" color="medium-emphasis" @click="dialogModel = false"></v-btn>
      </v-toolbar>
      <v-card-text class="pa-6">
        <v-tabs v-model="settingsTab" color="var(--jedi-accent)">
          <v-tab value="general">常规设置</v-tab>
          <v-tab value="wallpaper">{{ $t('settings.wallpaper') }}</v-tab>
          <v-tab value="advanced">{{ $t('settings.advanced') }}</v-tab>
        </v-tabs>

        <v-window v-model="settingsTab" class="mt-4">
          <!-- 常规设置 -->
          <v-window-item value="general">
            <v-list>
              <v-list-item>
                <template v-slot:prepend>
                  <v-icon :icon="mdiTranslate" color="var(--jedi-primary)" class="mr-3"></v-icon>
                </template>
                <v-list-item-title>语言 / Language</v-list-item-title>
                <template v-slot:append>
                  <v-menu>
                    <template v-slot:activator="{ props }">
                      <v-btn
                        color="var(--jedi-accent)"
                        variant="tonal"
                        size="small"
                        v-bind="props"
                        rounded="sm"
                      >
                        {{ currentLangLabel }}
                      </v-btn>
                    </template>
                    <v-list density="compact">
                      <v-list-item
                        v-for="lang in languages"
                        :key="lang.value"
                        :value="lang.value"
                        @click="changeLanguage(lang.value)"
                        :active="locale === lang.value"
                        color="primary"
                      >
                        <v-list-item-title>{{ lang.label }}</v-list-item-title>
                      </v-list-item>
                    </v-list>
                  </v-menu>
                </template>
              </v-list-item>

              <v-list-item>
                <template v-slot:prepend>
                  <v-icon :icon="mdiLaunch" color="var(--jedi-primary)" class="mr-3"></v-icon>
                </template>
                <v-list-item-title>开机自启动</v-list-item-title>
                <template v-slot:append>
                  <v-switch
                    v-model="autostartEnabled"
                    color="var(--jedi-accent)"
                    hide-details
                    :loading="autostartLoading"
                    @update:model-value="toggleAutostart"
                  ></v-switch>
                </template>
              </v-list-item>

              <v-list-item>
                <template v-slot:prepend>
                  <v-icon :icon="mdiTrayArrowDown" color="var(--jedi-primary)" class="mr-3"></v-icon>
                </template>
                <v-list-item-title>最小化到托盘</v-list-item-title>
                <template v-slot:append>
                  <v-switch color="var(--jedi-accent)" hide-details></v-switch>
                </template>
              </v-list-item>

              <v-list-item>
                <template v-slot:prepend>
                  <v-icon :icon="mdiUpdate" color="var(--jedi-primary)" class="mr-3"></v-icon>
                </template>
                <v-list-item-title>自动检查更新</v-list-item-title>
                <template v-slot:append>
                  <v-switch color="var(--jedi-accent)" hide-details></v-switch>
                </template>
              </v-list-item>
            </v-list>
          </v-window-item>

          <!-- 壁纸设置 -->
          <v-window-item value="wallpaper">
            <v-list>
              <v-list-item>
                <template v-slot:prepend>
                  <v-icon :icon="mdiWallpaper" color="var(--jedi-primary)" class="mr-3"></v-icon>
                </template>
                <v-list-item-title>{{ $t('settings.wpAutoUpdate') }}</v-list-item-title>
                <template v-slot:append>
                  <v-switch
                    v-model="wallpaperSettings.autoUpdate"
                    color="var(--jedi-accent)"
                    hide-details
                    @update:model-value="saveWallpaperSettings(wallpaperSettings)"
                  ></v-switch>
                </template>
              </v-list-item>

              <v-list-item v-if="wallpaperSettings.autoUpdate">
                 <template v-slot:prepend>
                  <v-icon icon="" class="mr-3"></v-icon>
                </template>
                <v-list-item-title>{{ $t('settings.wpFrequency') }}</v-list-item-title>
                <template v-slot:append>
                  <v-text-field
                    v-model.number="wallpaperSettings.frequencyDays"
                    type="number"
                    min="1"
                    variant="outlined"
                    density="compact"
                    hide-details
                    style="width: 100px"
                    @update:model-value="saveWallpaperSettings(wallpaperSettings)"
                  ></v-text-field>
                </template>
              </v-list-item>

              <v-list-item>
                <template v-slot:prepend>
                  <v-icon icon="" class="mr-3"></v-icon>
                </template>
                <v-list-item-title>{{ $t('settings.wpCategories') }}</v-list-item-title>
                <template v-slot:append>
                  <v-select
                    v-model="wallpaperSettings.selectedCategories"
                    :items="allCategories"
                    multiple
                    chips
                    variant="outlined"
                    density="compact"
                    hide-details
                    style="width: 250px"
                    @update:model-value="saveWallpaperSettings(wallpaperSettings)"
                  ></v-select>
                </template>
              </v-list-item>
              
               <v-list-item>
                <v-list-item-subtitle class="text-caption text-right">
                  {{ $t('settings.wpLastUpdate', { time: wallpaperSettings.lastUpdate ? new Date(wallpaperSettings.lastUpdate).toLocaleString() : 'N/A' }) }}
                </v-list-item-subtitle>
              </v-list-item>
            </v-list>
          </v-window-item>

          <!-- 高级设置 -->
          <v-window-item value="advanced">
            <v-list>
              <v-list-item>
                <template v-slot:prepend>
                  <v-icon :icon="mdiFileDocument" color="var(--jedi-primary)" class="mr-3"></v-icon>
                </template>
                <v-list-item-title>Hosts 文件路径</v-list-item-title>
                <template v-slot:append>
                  <v-text-field
                    variant="outlined"
                    density="compact"
                    hide-details
                    readonly
                    value="/etc/hosts"
                    style="width: 250px"
                  ></v-text-field>
                </template>
              </v-list-item>

              <v-list-item>
                <template v-slot:prepend>
                  <v-icon :icon="mdiBackupRestore" color="var(--jedi-primary)" class="mr-3"></v-icon>
                </template>
                <v-list-item-title>备份设置</v-list-item-title>
                <template v-slot:append>
                  <v-btn color="var(--jedi-accent)" variant="tonal" size="small" rounded="sm">备份</v-btn>
                </template>
              </v-list-item>

              <v-list-item>
                <template v-slot:prepend>
                  <v-icon :icon="mdiRefresh" color="var(--jedi-primary)" class="mr-3"></v-icon>
                </template>
                <v-list-item-title>重置应用</v-list-item-title>
                <template v-slot:append>
                  <v-btn color="var(--jedi-danger)" variant="tonal" size="small" rounded="sm">重置</v-btn>
                </template>
              </v-list-item>
            </v-list>
          </v-window-item>
        </v-window>
      </v-card-text>
      <v-card-actions class="pa-4 pt-0">
        <v-spacer></v-spacer>
        <v-btn variant="text" @click="dialogModel = false" rounded="sm" class="mr-2">
          取消
        </v-btn>
        <v-btn color="var(--jedi-accent)" variant="elevated" @click="dialogModel = false" rounded="sm">
          保存
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useStorage } from '@/composables/useStorage'
import {
  mdiCog,
  mdiClose,
  mdiLaunch,
  mdiTrayArrowDown,
  mdiUpdate,
  mdiFileDocument,
  mdiBackupRestore,
  mdiRefresh,
  mdiTranslate,
  mdiWallpaper
} from '@mdi/js'
import { enableAutostart, disableAutostart, isAutostartEnabled } from '@/api/app'
import { useWallpaper } from '@/composables/useWallpaper'
import { getWallpapers } from '@/api/wallpaper'

// 定义组件属性
const props = defineProps<{
  modelValue: boolean;
}>()

// 定义组件事件
const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
}>()

const { settings: wallpaperSettings, saveSettings: saveWallpaperSettings } = useWallpaper()
const allCategories = ref<string[]>([])

const { locale } = useI18n()
const { setItem } = useStorage()

// 语言选项
const languages = [
  { label: '简体中文', value: 'zh' },
  { label: 'English', value: 'en' }
]

const currentLangLabel = computed(() => {
  return languages.find(l => l.value === locale.value)?.label || '简体中文'
})

const changeLanguage = async (lang: string) => {
  locale.value = lang
  await setItem('language', lang)
}

// 对话框状态
const dialogModel = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

// 设置选项卡
const settingsTab = ref('general')

// 自启动相关状态
const autostartEnabled = ref(false)
const autostartLoading = ref(false)

// 切换自启动状态
async function toggleAutostart(value: boolean | null) {
  if (value === null) return
  try {
    autostartLoading.value = true
    if (value) {
      await enableAutostart()
    } else {
      await disableAutostart()
    }
  } catch (error) {
    console.error('切换自启动状态失败:', error)
    // 恢复原来的状态
    autostartEnabled.value = !value
  } finally {
    autostartLoading.value = false
  }
}

// 检查自启动状态
async function checkAutostartStatus() {
  try {
    autostartLoading.value = true
    const enabled = await isAutostartEnabled()
    autostartEnabled.value = enabled
  } catch (error) {
    console.error('检查自启动状态失败:', error)
  } finally {
    autostartLoading.value = false
  }
}

// 组件挂载时检查自启动状态
onMounted(async () => {
  await checkAutostartStatus()

  // Load wallpaper settings and categories
  const { loadSettings } = useWallpaper()
  await loadSettings()

  try {
    const wallpapers = await getWallpapers()
    const categories = new Set(wallpapers.map(w => w.category))
    allCategories.value = Array.from(categories).sort()
  } catch (e) {
    console.error('Failed to load wallpaper categories', e)
  }
})
</script>
