<template>
  <v-dialog v-model="dialogModel" max-width="700" persistent>
    <template v-slot:activator="{ props }">
      <div class="d-flex justify-end">
        <v-btn
          v-bind="props"
          variant="text"
          size="small"
          color="primary"
          class="mt-1"
          :prepend-icon="mdiHelpCircleOutline"
        >
          {{ $t('podcast.howToExportOpml') }}
        </v-btn>
      </div>
    </template>

    <v-card class="model-settings-card">
      <!-- Header -->
      <div class="card-header">
        <div class="header-brand">
          <div class="brand-icon">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" stroke="currentColor" stroke-width="1.5"/>
              <polyline points="14 2 14 8 20 8" stroke="currentColor" stroke-width="1.5"/>
              <line x1="16" y1="13" x2="8" y2="13" stroke="currentColor" stroke-width="1.5"/>
              <line x1="16" y1="17" x2="8" y2="17" stroke="currentColor" stroke-width="1.5"/>
              <polyline points="10 9 9 9 8 9" stroke="currentColor" stroke-width="1.5"/>
            </svg>
          </div>
          <div class="brand-text">
            <h2>Export OPML Guide</h2>
            <p>How to export your podcast subscriptions</p>
          </div>
        </div>
        <button class="close-btn" @click="dialogModel = false">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
            <line x1="18" y1="6" x2="6" y2="18" stroke="currentColor" stroke-width="2"/>
            <line x1="6" y1="6" x2="18" y2="18" stroke="currentColor" stroke-width="2"/>
          </svg>
        </button>
      </div>

      <!-- Content -->
      <div class="card-body">
        <v-carousel
          v-model="currentSlide"
          hide-delimiter-background
          show-arrows="hover"
          height="280"
          class="opml-carousel"
        >
          <v-carousel-item
            v-for="(_, i) in images"
            :key="i"
          >
            <div class="carousel-slide">
              <v-img
                :src="images[i].src"
                height="180"
                width="100%"
                contain
                class="slide-image"
              >
                <template v-slot:placeholder>
                  <div class="d-flex align-center justify-center h-100">
                    <v-progress-circular indeterminate color="primary"></v-progress-circular>
                  </div>
                </template>
                <template v-slot:error>
                  <div class="d-flex align-center justify-center h-100 text-error">
                    Image not found
                  </div>
                </template>
              </v-img>
              <div class="slide-label">
                <span class="step-badge">{{ i + 1 }}</span>
                <span class="step-text">{{ $t(`podcast.exportGuideStep${i + 1}`) }}</span>
              </div>
            </div>
          </v-carousel-item>
        </v-carousel>

        <div class="step-dots">
          <button
            v-for="(_, i) in images"
            :key="i"
            class="step-dot"
            :class="{ active: currentSlide === i }"
            @click="currentSlide = i"
          ></button>
        </div>
      </div>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { mdiHelpCircleOutline } from '@mdi/js'
import { computed, ref } from 'vue'

const dialog = ref(false)
const currentSlide = ref(0)

const dialogModel = computed({
  get: () => dialog.value,
  set: val => {
    dialog.value = val
  },
})

const images = [
  { src: '/images/opml-export-guide/1.png' },
  { src: '/images/opml-export-guide/2.png' },
  { src: '/images/opml-export-guide/3.png' },
  { src: '/images/opml-export-guide/4.png' },
]
</script>

<style scoped>
.model-settings-card {
  background: #0a0e14 !important;
  border-radius: 16px !important;
  overflow: hidden;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: rgba(20, 30, 40, 0.6);
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  flex-shrink: 0;
}

.header-brand {
  display: flex;
  align-items: center;
  gap: 12px;
}

.brand-icon {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, rgba(0, 255, 255, 0.15) 0%, rgba(0, 255, 136, 0.05) 100%);
  border: 1px solid rgba(0, 255, 255, 0.25);
  border-radius: 10px;
  color: #00ffff;
}

.brand-text h2 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: #ffffff;
}

.brand-text p {
  margin: 2px 0 0;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.4);
}

.close-btn {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  color: rgba(255, 255, 255, 0.5);
  cursor: pointer;
  transition: all 0.2s ease;
}

.close-btn:hover {
  background: rgba(255, 107, 107, 0.1);
  border-color: rgba(255, 107, 107, 0.3);
  color: #ff6b6b;
}

.card-body {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.opml-carousel {
  border: 1px solid rgba(0, 255, 255, 0.1);
  border-radius: 12px;
  overflow: hidden;
}

.carousel-slide {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: 16px;
  background: rgba(0, 0, 0, 0.2);
}

.slide-image {
  border-radius: 8px;
  border: 1px solid rgba(0, 255, 255, 0.08);
  overflow: hidden;
}

.slide-label {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 12px;
  color: rgba(255, 255, 255, 0.8);
  font-size: 13px;
}

.step-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  background: rgba(0, 255, 255, 0.15);
  border: 1px solid rgba(0, 255, 255, 0.3);
  border-radius: 6px;
  font-size: 12px;
  font-weight: 700;
  color: #00ffff;
}

.step-text {
  letter-spacing: 0.3px;
}

.step-dots {
  display: flex;
  justify-content: center;
  gap: 8px;
}

.step-dot {
  width: 8px;
  height: 8px;
  background: rgba(255, 255, 255, 0.15);
  border: none;
  border-radius: 50%;
  cursor: pointer;
  transition: all 0.2s ease;
}

.step-dot:hover {
  background: rgba(255, 255, 255, 0.3);
}

.step-dot.active {
  background: #00ffff;
  box-shadow: 0 0 8px rgba(0, 255, 255, 0.5);
}

::-webkit-scrollbar {
  width: 4px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 2px;
}

::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.15);
}
</style>
