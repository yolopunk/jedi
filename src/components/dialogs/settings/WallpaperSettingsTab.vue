<template>
  <div class="settings-section">
    <div class="setting-item">
      <div class="setting-icon">🖼</div>
      <div class="setting-info">
        <div class="setting-label">{{ t('settings.wpAutoUpdate') }}</div>
      </div>
      <div class="setting-action">
        <div
          class="toggle-switch"
          :class="{ active: wallpaperSettings.autoUpdate }"
          @click="wallpaperSettings.autoUpdate = !wallpaperSettings.autoUpdate; saveWallpaperSettings(wallpaperSettings)"
        >
          <div class="toggle-handle"></div>
        </div>
      </div>
    </div>

    <div class="setting-item" v-if="wallpaperSettings.autoUpdate">
      <div class="setting-icon"></div>
      <div class="setting-info">
        <div class="setting-label">{{ t('settings.wpFrequency') }}</div>
      </div>
      <div class="setting-action">
        <div class="input-wrapper small">
          <input
            v-model.number="wallpaperSettings.frequencyDays"
            type="number"
            min="1"
            class="console-input"
            @input="saveWallpaperSettings(wallpaperSettings)"
          />
        </div>
      </div>
    </div>

    <div class="setting-item">
      <div class="setting-icon"></div>
      <div class="setting-info">
        <div class="setting-label">{{ t('settings.wpCategories') }}</div>
      </div>
      <div class="setting-action">
        <v-menu location="bottom end">
          <template v-slot:activator="{ props }">
            <button v-bind="props" class="console-btn small">
              {{ t('settings.categories', { n: wallpaperSettings.selectedCategories?.length || 0 }) }}
            </button>
          </template>
          <div class="console-menu">
            <div
              v-for="cat in allCategories"
              :key="cat"
              class="menu-item"
              :class="{ active: wallpaperSettings.selectedCategories?.includes(cat) }"
              @click="toggleCategory(cat)"
            >
              <span class="menu-check">{{ wallpaperSettings.selectedCategories?.includes(cat) ? '▣' : '▢' }}</span>
              <span class="menu-text">{{ cat }}</span>
            </div>
          </div>
        </v-menu>
      </div>
    </div>

    <div class="setting-item no-hover">
      <div class="setting-icon"></div>
      <div class="setting-info">
        <div class="setting-subtitle text-right">
          {{ t('settings.wpLastUpdate', { time: wallpaperSettings.lastUpdate ? new Date(wallpaperSettings.lastUpdate).toLocaleString() : 'N/A' }) }}
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useWallpaper } from '@/composables/useWallpaper'
import { getWallpapers } from '@/api/wallpaper'

const { t } = useI18n()
const { settings: wallpaperSettings, saveSettings: saveWallpaperSettings, loadSettings } = useWallpaper()
const allCategories = ref<string[]>([])

async function toggleCategory(cat: string) {
  if (!wallpaperSettings.value.selectedCategories) {
    wallpaperSettings.value.selectedCategories = []
  }
  const index = wallpaperSettings.value.selectedCategories.indexOf(cat)
  if (index > -1) {
    wallpaperSettings.value.selectedCategories.splice(index, 1)
  } else {
    wallpaperSettings.value.selectedCategories.push(cat)
  }
  saveWallpaperSettings(wallpaperSettings.value)
}

onMounted(async () => {
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

<style scoped>
.setting-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border-radius: 4px;
  transition: background-color 0.15s ease;
}

.setting-item:not(.no-hover):hover {
  background: rgba(0, 255, 255, 0.03);
}

.setting-icon {
  width: 24px;
  text-align: center;
  font-size: 16px;
}

.setting-info {
  flex: 1;
  min-width: 0;
}

.setting-label {
  font-size: 12px;
  font-weight: 600;
  color: #e4e4e7;
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', monospace;
}

.setting-subtitle {
  font-size: 10px;
  color: #52525b;
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', monospace;
}

.setting-action {
  display: flex;
  align-items: center;
  gap: 8px;
}

.toggle-switch {
  width: 40px;
  height: 22px;
  background: rgba(82, 82, 91, 0.5);
  border-radius: 12px;
  position: relative;
  cursor: pointer;
  transition: all 0.2s ease;
  border: 1px solid rgba(82, 82, 91, 0.5);
}

.toggle-switch.active {
  background: rgba(0, 255, 136, 0.15);
  border-color: rgba(0, 255, 136, 0.5);
}

.toggle-handle {
  position: absolute;
  width: 16px;
  height: 16px;
  background: #52525b;
  border-radius: 50%;
  top: 2px;
  left: 2px;
  transition: all 0.2s ease;
}

.toggle-switch.active .toggle-handle {
  background: #00ff88;
  box-shadow: 0 0 10px rgba(0, 255, 136, 0.5);
  left: 20px;
}

.console-menu {
  background: rgba(20, 20, 25, 0.95);
  border: 1px solid rgba(0, 255, 255, 0.2);
  border-radius: 6px;
  padding: 4px;
  min-width: 150px;
  backdrop-filter: blur(8px);
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.15s ease;
  color: #a1a1aa;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 11px;
}

.menu-item:hover {
  background: rgba(0, 255, 255, 0.1);
  color: #00ffff;
}

.menu-item.active {
  background: rgba(0, 255, 255, 0.15);
  color: #00ffff;
}

.menu-check {
  color: #00ffff;
  font-size: 10px;
  width: 14px;
  text-align: center;
}

.menu-text {
  flex: 1;
}

.input-wrapper.small {
  padding: 4px 8px;
}

.input-wrapper.small .console-input {
  font-size: 11px;
  padding: 4px 8px;
}

.text-right {
  text-align: right;
}
</style>