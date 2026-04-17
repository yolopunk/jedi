<template>
  <div class="podcasts-grid-container console-scroll">
    <div v-if="loading && !subscriptions.length" class="loading-grid">
      <div v-for="i in 8" :key="i" class="loading-card">
        <div class="loading-image"></div>
        <div class="loading-text"></div>
      </div>
    </div>

    <div v-else-if="filteredSubscriptions.length" class="podcast-grid">
      <div
        v-for="sub in filteredSubscriptions"
        :key="sub.rss_url"
        class="podcast-item"
        @click="$emit('select', sub)"
      >
        <div class="podcast-card">
          <div class="card-glow"></div>
          <div class="image-wrapper">
            <v-img
              :src="sub.image_url"
              aspect-ratio="1"
              cover
              class="podcast-image"
            >
              <template v-slot:placeholder>
                <div class="d-flex align-center justify-center fill-height">
                  <span class="placeholder-icon">📻</span>
                </div>
              </template>
            </v-img>

            <!-- Playing Indicator -->
            <div
              v-if="currentPlaying && currentPlayingSubUrl === sub.rss_url"
              class="playing-indicator"
            >
              <div class="playing-bar bar-1"></div>
              <div class="playing-bar bar-2"></div>
              <div class="playing-bar bar-3"></div>
            </div>
          </div>

          <div class="card-footer">
            <div class="card-title">{{ sub.title }}</div>
            <div class="card-meta">{{ sub.author || sub.owner_name || sub.description }}</div>
          </div>
        </div>
      </div>
    </div>

    <div v-else class="empty-state">
      <div class="empty-icon">◇</div>
      <div class="empty-text">NO_PODCASTS_FOUND</div>
      <button class="console-btn primary mt-4" @click="$emit('add')">
        <span class="btn-text">{{ $t('podcast.addShow') }}</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { PodcastSubscription } from '@/api/podcast'

const props = defineProps<{
  subscriptions: PodcastSubscription[]
  searchQuery: string
  loading: boolean
  currentPlaying: { audio_url: string } | null
  currentPlayingSubUrl: string | null
}>()

defineEmits<{
  (e: 'select', sub: PodcastSubscription): void
  (e: 'add'): void
}>()

const filteredSubscriptions = computed(() => {
  if (!props.searchQuery) return props.subscriptions
  const q = props.searchQuery.toLowerCase()
  return props.subscriptions.filter(
    sub =>
      sub.title.toLowerCase().includes(q) ||
      sub.author?.toLowerCase().includes(q) ||
      sub.description?.toLowerCase().includes(q)
  )
})
</script>
