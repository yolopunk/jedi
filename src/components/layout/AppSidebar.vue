<template>
  <v-navigation-drawer permanent :border="0" :width="200" class="jedi-nav-drawer text-center">
    <div class="app-logo-container py-3">
      <v-img src="/icon.png" width="80" class="mx-auto jedi-hover-scale"
             style="filter: drop-shadow(0 4px 8px rgba(0,0,0,0.1));"></v-img>
    </div>
    <div class="sidebar-banner px-4 py-3 mb-4 lightsaber-glow"
         style="background: linear-gradient(135deg, #1A2530 0%, #2C3E50 100%);
                border-bottom: 1px solid rgba(52, 152, 219, 0.3);">
      <h2 class="text-h5 font-weight-bold mb-0 jedi-title">
        <span class="jedi-text">JEDI</span>
      </h2>
      <div class="lightsaber-border">
        <div class="electric-beam beam-1"></div>
        <div class="electric-beam beam-2"></div>
        <div class="electric-beam beam-3"></div>
      </div>
    </div>
    <v-container class="py-0 pb-32">
      <v-list nav class="jedi-nav-list pa-2">
        <v-list-item
          :active="true"
          color="var(--jedi-accent)"
          rounded="sm"
          title="Hosts 管理"
          :prepend-icon="mdiDns"
          class="jedi-nav-item mb-1"
        >
        </v-list-item>
      </v-list>
    </v-container>
    <app-footer
      @open-github="$emit('open-github')"
      @open-website="$emit('open-website')"
      @open-email="$emit('open-email')"
      @show-help="$emit('show-help')"
      @show-settings="$emit('show-settings')"
      @show-about="$emit('show-about')"
    />
  </v-navigation-drawer>
</template>

<script setup lang="ts">
import { mdiDns } from '@mdi/js'
import AppFooter from './AppFooter.vue'

// 定义组件事件
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
.jedi-nav-drawer {
  background-color: #f8f9fa !important;
  border-right: 1px solid rgba(0, 0, 0, 0.05) !important;
  overflow-y: auto !important;
}

.sidebar-banner {
  position: relative;
  overflow: hidden;
  transition: all 0.3s ease;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
  border-radius: 12px;
  margin-left: 8px;
  margin-right: 8px;
}

.sidebar-banner::after {
  content: '';
  position: absolute;
  top: -50%;
  left: -50%;
  width: 200%;
  height: 200%;
  background: radial-gradient(circle, rgba(52, 152, 219, 0.2) 0%, rgba(26, 37, 48, 0) 70%);
  opacity: 0;
  transition: opacity 0.3s ease;
  pointer-events: none;
}

.sidebar-banner:hover::after {
  opacity: 1;
}

.app-logo-container {
  transition: all 0.3s ease;
  padding-bottom: 0 !important;
}

.jedi-title {
  letter-spacing: 1px;
  transition: all 0.3s ease;
  position: relative;
  z-index: 1;
}

.jedi-title:hover {
  transform: translateY(-1px);
}

.jedi-text {
  color: white;
  text-shadow: 0 0 10px rgba(255,255,255,0.5);
  animation: lightsaber-text-glow 2s infinite alternate;
}

@keyframes lightsaber-text-glow {
  0% {
    text-shadow: 0 0 10px rgba(255,255,255,0.5), 0 0 20px rgba(52, 152, 219, 0.3);
  }
  100% {
    text-shadow: 0 0 15px rgba(255,255,255,0.7), 0 0 30px rgba(52, 152, 219, 0.7);
  }
}

.jedi-nav-list {
  background-color: transparent !important;
}

.jedi-nav-item {
  margin-bottom: 4px;
  transition: all 0.2s ease;
  border-radius: 8px;
  overflow: hidden;
}

.jedi-nav-item:hover {
  background-color: rgba(66, 165, 245, 0.08) !important;
  transform: translateY(-1px);
  box-shadow: 0 2px 5px rgba(0, 0, 0, 0.05);
}

/* 光剑电流边框效果 */
.lightsaber-border {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  border-radius: 12px;
  pointer-events: none;
  overflow: hidden;
}

.lightsaber-border::before,
.lightsaber-border::after {
  content: '';
  position: absolute;
  top: -50%;
  left: -50%;
  width: 200%;
  height: 200%;
  background-repeat: no-repeat;
  opacity: 0.7;
  mix-blend-mode: screen;
}

.lightsaber-border::before {
  background-image: conic-gradient(
    transparent 0deg,
    transparent 30deg,
    rgba(52, 152, 219, 0.8) 40deg,
    rgba(52, 152, 219, 0.8) 60deg,
    transparent 70deg,
    transparent 150deg,
    transparent 160deg,
    rgba(52, 152, 219, 0.6) 170deg,
    rgba(52, 152, 219, 0.6) 180deg,
    transparent 190deg,
    transparent 360deg
  );
  animation: rotate-clockwise 4s infinite linear;
}

.lightsaber-border::after {
  background-image: conic-gradient(
    transparent 0deg,
    transparent 190deg,
    rgba(52, 152, 219, 0.8) 200deg,
    rgba(52, 152, 219, 0.8) 220deg,
    transparent 230deg,
    transparent 300deg,
    rgba(52, 152, 219, 0.7) 310deg,
    rgba(52, 152, 219, 0.7) 320deg,
    transparent 330deg,
    transparent 360deg
  );
  animation: rotate-clockwise 4s infinite linear;
  animation-delay: -2s;
}

@keyframes rotate-clockwise {
  0% {
    transform: rotate(0deg);
  }
  100% {
    transform: rotate(360deg);
  }
}

/* 电流光束效果 */
.electric-beam {
  position: absolute;
  width: 100%;
  height: 100%;
  border-radius: 12px;
  pointer-events: none;
}

.beam-1 {
  background: linear-gradient(90deg, transparent, transparent, rgba(52, 152, 219, 0.7), transparent, transparent);
  background-size: 200% 100%;
  animation: electric-flow 3s infinite linear;
  box-shadow: 0 0 10px rgba(52, 152, 219, 0.3);
}

.beam-2 {
  background: linear-gradient(180deg, transparent, transparent, rgba(52, 152, 219, 0.7), transparent, transparent);
  background-size: 100% 200%;
  animation: electric-flow-vertical 4s infinite linear;
  animation-delay: -1s;
  box-shadow: 0 0 10px rgba(52, 152, 219, 0.3);
}

.beam-3 {
  background: radial-gradient(circle at 50% 50%, rgba(52, 152, 219, 0.2), transparent 70%);
  animation: pulse 2s infinite alternate;
  box-shadow: inset 0 0 15px rgba(52, 152, 219, 0.5);
}

@keyframes electric-flow {
  0% {
    background-position: 200% 0;
  }
  100% {
    background-position: -200% 0;
  }
}

@keyframes electric-flow-vertical {
  0% {
    background-position: 0 200%;
  }
  100% {
    background-position: 0 -200%;
  }
}

@keyframes pulse {
  0% {
    opacity: 0.3;
  }
  100% {
    opacity: 0.7;
  }
}
</style>
