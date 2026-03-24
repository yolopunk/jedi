<template>
  <v-dialog v-model="dialog" max-width="800" persistent>
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

    <v-card class="scifi-card dialog-with-glow">
      <div class="dialog-decorator"></div>
      <v-card-title class="console-title-bar">
        <span class="dialog-icon">⬡</span>
        <span class="dialog-title">{{ $t('podcast.exportGuide') }}</span>
        <v-spacer></v-spacer>
        <button class="close-btn" @click="dialog = false">✕</button>
      </v-card-title>
      
      <v-card-text class="console-card-text">
        <v-carousel
          v-model="currentSlide"
          hide-delimiter-background
          show-arrows="hover"
          height="280"
          class="opml-carousel mt-2"
        >
          <v-carousel-item
            v-for="(_, i) in images"
            :key="i"
          >
            <div class="carousel-slide">
              <v-img
                :src="images[i].src"
                height="200"
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
                        Image Load Failed
                    </div>
                </template>
              </v-img>
              <div class="slide-label">
                <span class="step-number">{{ i + 1 }}</span>
                <span class="step-text">{{ $t(`podcast.exportGuideStep${i + 1}`) }}</span>
              </div>
            </div>
          </v-carousel-item>
        </v-carousel>
        
        <div class="d-flex justify-center mt-4">
            <v-btn-toggle v-model="currentSlide" mandatory rounded="xl" color="primary" variant="outlined" divided>
                <v-btn v-for="(_, i) in images" :key="i" :value="i" size="small">
                    {{ i + 1 }}
                </v-btn>
            </v-btn-toggle>
        </div>
      </v-card-text>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { mdiHelpCircleOutline } from '@mdi/js'

const dialog = ref(false)
const currentSlide = ref(0)

const images = [
  { src: '/images/opml-export-guide/1.png' },
  { src: '/images/opml-export-guide/2.png' },
  { src: '/images/opml-export-guide/3.png' },
  { src: '/images/opml-export-guide/4.png' }
]
</script>

<style scoped>
.dialog-with-glow {
  position: relative;
  box-shadow: 0 0 40px rgba(0, 255, 255, 0.15);
}

.dialog-decorator {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: linear-gradient(90deg, transparent, #00ffff, transparent);
  opacity: 0.6;
}

.dialog-icon {
  color: #00ffff;
  margin-right: 8px;
  font-size: 14px;
}

.dialog-title {
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 1px;
  color: #00ff88;
  text-shadow: 0 0 10px rgba(0, 255, 136, 0.5);
}

.close-btn {
  background: none;
  border: none;
  color: rgba(255, 255, 255, 0.5);
  cursor: pointer;
  font-size: 14px;
  padding: 4px 8px;
  transition: color 0.2s;
}

.close-btn:hover {
  color: #ff4444;
}

.opml-carousel {
  border: 1px solid rgba(0, 255, 255, 0.15);
  border-radius: 8px;
  overflow: hidden;
}

.carousel-slide {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: 16px;
  background: rgba(0, 0, 0, 0.3);
}

.slide-image {
  border-radius: 8px;
  border: 1px solid rgba(0, 255, 255, 0.1);
}

.slide-label {
  display: flex;
  align-items: center;
  margin-top: 12px;
  color: rgba(255, 255, 255, 0.8);
  font-size: 14px;
}

.step-number {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  background: rgba(0, 255, 255, 0.2);
  border: 1px solid rgba(0, 255, 255, 0.3);
  border-radius: 4px;
  font-size: 12px;
  font-weight: 600;
  color: #00ffff;
  margin-right: 8px;
}

.step-text {
  letter-spacing: 0.5px;
}

/* =========================================
   Light Theme Styles (Tatooine Outpost)
   ========================================= */
.light-theme .dialog-with-glow {
  box-shadow: 0 0 30px rgba(184, 134, 11, 0.2);
}

.light-theme .dialog-decorator {
  background: linear-gradient(90deg, transparent, #cd7f32, transparent);
}

.light-theme .dialog-icon {
  color: #cd7f32;
}

.light-theme .dialog-title {
  color: #daa520;
  text-shadow: 0 0 10px rgba(218, 165, 32, 0.4);
}

.light-theme .close-btn {
  color: rgba(107, 68, 35, 0.6);
}

.light-theme .close-btn:hover {
  color: #b22222;
}

.light-theme .opml-carousel {
  border-color: rgba(184, 134, 11, 0.3);
}

.light-theme .carousel-slide {
  background: rgba(245, 230, 211, 0.5);
}

.light-theme .slide-image {
  border-color: rgba(184, 134, 11, 0.2);
}

.light-theme .slide-label {
  color: #3d2914;
}

.light-theme .step-number {
  background: rgba(205, 127, 50, 0.15);
  border-color: rgba(205, 127, 50, 0.3);
  color: #cd7f32;
}
</style>
