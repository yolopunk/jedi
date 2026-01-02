<template>
  <v-navigation-drawer permanent :width="220" class="jedi-sidebar">
    <!-- Logo Area -->
    <div class="d-flex flex-column align-center py-6 border-bottom">
      <v-img src="/icon.png" width="64" class="mb-3 jedi-logo-glow"></v-img>
      <h2 class="text-h6 font-weight-bold text-primary">{{ $t('sidebar.title') }}</h2>
      <div class="text-caption text-secondary">{{ $t('sidebar.subtitle') }}</div>
    </div>

    <!-- Navigation -->
    <v-list nav class="pa-2 mt-2">
      <v-list-item
        :active="true"
        rounded="lg"
        class="mb-1 sidebar-item"
        color="primary"
      >
        <template v-slot:prepend>
          <v-icon :icon="mdiDns" size="small" class="mr-2"></v-icon>
        </template>
        <v-list-item-title>{{ $t('sidebar.hostsManager') }}</v-list-item-title>
      </v-list-item>
    </v-list>

    <!-- Footer Area: Jedi Control Deck -->
    <template v-slot:append>
      <div class="pa-4">
        <div class="jedi-control-deck d-flex align-center justify-space-between px-1 py-1">
          
          <!-- Power Core (Theme) -->
          <div class="power-core-wrapper">
            <v-btn 
              icon 
              density="compact" 
              variant="text" 
              @click="toggleTheme" 
              class="power-core-btn"
              :class="{ 'core-active': isDark }"
            >
              <v-icon :icon="themeIcon" size="20" class="core-icon"></v-icon>
              <div class="core-glow"></div>
              <v-tooltip activator="parent" location="top">{{ themeTooltip }}</v-tooltip>
            </v-btn>
          </div>
          
          <div class="deck-divider mx-1"></div>
          
          <!-- Control Module (Settings & Github) -->
          <div class="d-flex align-center control-module">
            <v-btn 
              icon 
              density="compact" 
              variant="text" 
              size="small" 
              @click="$emit('show-settings')" 
              class="deck-btn"
            >
              <v-icon :icon="mdiCog" size="18"></v-icon>
              <v-tooltip activator="parent" location="top">{{ $t('settings.title') }}</v-tooltip>
            </v-btn>
            
            <v-btn 
              icon 
              density="compact" 
              variant="text" 
              size="small" 
              @click="$emit('open-github')" 
              class="deck-btn"
            >
              <v-icon :icon="mdiGithub" size="18"></v-icon>
              <v-tooltip activator="parent" location="top">{{ $t('sidebar.github') }}</v-tooltip>
            </v-btn>
          </div>
        </div>
        
        <div class="text-center mt-3 text-caption jedi-status-text">
          <span class="status-dot"></span> {{ $t('sidebar.status_connected') }}
        </div>
      </div>
    </template>
  </v-navigation-drawer>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { mdiDns, mdiCog, mdiGithub, mdiWeatherNight, mdiWeatherSunny, mdiThemeLightDark } from '@mdi/js'
import { useTheme } from '@/composables/useTheme'

const { t } = useI18n()
const { isDark, themeMode, setTheme } = useTheme()

const themeIcon = computed(() => {
  if (themeMode.value === 'dark') return mdiWeatherNight
  if (themeMode.value === 'light') return mdiWeatherSunny
  return mdiThemeLightDark
})

const themeTooltip = computed(() => {
  if (themeMode.value === 'dark') return t('theme.dark')
  if (themeMode.value === 'light') return t('theme.light')
  return t('theme.system')
})

function toggleTheme() {
  if (themeMode.value === 'light') {
    setTheme('dark')
  } else if (themeMode.value === 'dark') {
    setTheme('system')
  } else {
    setTheme('light')
  }
}

defineEmits<{
  (e: 'open-github'): void;
  (e: 'open-website'): void;
  (e: 'open-email'): void;
  (e: 'show-help'): void;
  (e: 'show-settings'): void;
  (e: 'show-about'): void;
}>()
</script>

<style scoped>
/* Minimal scoped styles, rely on global theme */
.jedi-sidebar {
  /* Ensure border is visible */
  border-right: 1px solid var(--jedi-border) !important;
}

.border-bottom {
  border-bottom: 1px solid var(--jedi-border);
}

.jedi-logo-glow {
  filter: drop-shadow(0 0 10px rgba(59, 130, 246, 0.3));
}

/* Sidebar Item Active State Override for Light Theme */
:global(.light-theme) .sidebar-item.v-list-item--active {
  background-color: #e0f2fe !important; /* Light Blue-50 */
  color: #0369a1 !important; /* Sky-700 */
}

/* Use a subtle gradient for dark mode active state */
:global(.dark-theme) .sidebar-item.v-list-item--active {
  background: linear-gradient(90deg, rgba(59, 130, 246, 0.15), transparent) !important;
  color: #60a5fa !important; /* Blue-400 */
  border-left: 2px solid #60a5fa;
}

/* Jedi Control Deck - Bolder Design */
.jedi-control-deck {
  background: linear-gradient(135deg, rgba(30, 41, 59, 0.9), rgba(15, 23, 42, 0.95));
  border: 1px solid rgba(59, 130, 246, 0.3);
  border-radius: 12px;
  position: relative;
  overflow: hidden;
  box-shadow: 
    0 4px 6px -1px rgba(0, 0, 0, 0.1), 
    0 2px 4px -1px rgba(0, 0, 0, 0.06),
    inset 0 1px 0 rgba(255, 255, 255, 0.1);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

/* Holographic sheen */
.jedi-control-deck::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 1px;
  background: linear-gradient(90deg, transparent, rgba(59, 130, 246, 0.8), transparent);
  opacity: 0.5;
}

.jedi-control-deck:hover {
  border-color: rgba(59, 130, 246, 0.6);
  box-shadow: 
    0 0 15px rgba(59, 130, 246, 0.2),
    inset 0 0 20px rgba(59, 130, 246, 0.05);
  transform: translateY(-2px);
}

/* Power Core (Theme Toggle) */
.power-core-wrapper {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 4px;
  border-radius: 8px;
  background: rgba(0, 0, 0, 0.2);
  margin: 2px;
}

.power-core-btn {
  color: rgba(255, 255, 255, 0.7);
  transition: all 0.3s ease;
  position: relative;
  z-index: 2;
}

.power-core-btn:hover {
  color: #fff;
  text-shadow: 0 0 8px rgba(255, 255, 255, 0.8);
}

.core-glow {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 0;
  height: 0;
  background: radial-gradient(circle, rgba(59, 130, 246, 0.8) 0%, transparent 70%);
  transition: all 0.4s ease;
  border-radius: 50%;
  opacity: 0;
  z-index: 1;
}

.power-core-btn:hover .core-glow {
  width: 40px;
  height: 40px;
  opacity: 0.6;
}

.core-active .core-icon {
  color: #fbbf24; /* Amber for light/sun */
  filter: drop-shadow(0 0 5px rgba(251, 191, 36, 0.5));
}

/* Deck Divider */
.deck-divider {
  width: 1px;
  height: 24px;
  background: linear-gradient(to bottom, transparent, rgba(255, 255, 255, 0.2), transparent);
}

/* Deck Buttons */
.deck-btn {
  color: rgba(255, 255, 255, 0.6);
  transition: all 0.2s;
  margin: 0 2px;
}

.deck-btn:hover {
  color: #60a5fa; /* Blue-400 */
  background: rgba(59, 130, 246, 0.1);
  transform: scale(1.1);
}

/* Status Text */
.jedi-status-text {
  font-family: 'JetBrains Mono', monospace;
  font-size: 9px !important;
  letter-spacing: 2px;
  color: rgba(var(--v-theme-on-surface), 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  text-transform: uppercase;
}

.status-dot {
  width: 4px;
  height: 4px;
  background-color: #10b981; /* Emerald-500 */
  border-radius: 50%;
  margin-right: 6px;
  box-shadow: 0 0 8px #10b981;
  animation: pulse-status 3s infinite;
}

@keyframes pulse-status {
  0%, 100% { opacity: 0.5; transform: scale(1); }
  50% { opacity: 1; transform: scale(1.5); }
}
</style>
