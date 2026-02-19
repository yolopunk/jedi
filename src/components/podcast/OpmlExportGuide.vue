<template>
  <v-dialog v-model="dialog" max-width="800">
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

    <v-card class="rounded-xl">
      <v-card-title class="d-flex justify-space-between align-center px-6 pt-6">
        <span class="text-h6 font-weight-bold">{{ $t('podcast.exportGuide') }}</span>
        <v-btn icon variant="text" @click="dialog = false">
          <v-icon :icon="mdiClose"></v-icon>
        </v-btn>
      </v-card-title>
      
      <v-card-text class="px-6 pb-6">
        <v-carousel
          v-model="currentSlide"
          hide-delimiter-background
          show-arrows="hover"
          height="260"
          class="rounded-lg elevation-2 mt-4"
        >
          <v-carousel-item
            v-for="(_, i) in images"
            :key="i"
          >
            <div class="d-flex flex-column align-center justify-center h-100 bg-grey-lighten-4 pa-4">
              <v-img
                :src="images[i].src"
                height="100%"
                width="100%"
                contain
                class="rounded-lg bg-white"
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
              <div class="mt-2 text-subtitle-1 font-weight-medium text-center">
                {{ $t(`podcast.exportGuideStep${i + 1}`) }}
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
import { mdiHelpCircleOutline, mdiClose } from '@mdi/js'

const dialog = ref(false)
const currentSlide = ref(0)

const images = [
  { src: '/images/opml-export-guide/1.png' },
  { src: '/images/opml-export-guide/2.png' },
  { src: '/images/opml-export-guide/3.png' },
  { src: '/images/opml-export-guide/4.png' }
]
</script>
