<template>
  <div class="detail-view-container d-flex flex-column overflow-hidden h-100">
    <!-- Podcast Header -->
    <div class="podcast-header flex-shrink-0">
      <div class="d-flex flex-column flex-md-row align-start">
        <div class="podcast-cover mr-md-4 mb-4 mb-md-0 flex-shrink-0">
          <v-img
            :src="currentSub.image_url"
            width="140"
            height="140"
            cover
            class="cover-image"
          >
            <template v-slot:placeholder>
              <div class="d-flex align-center justify-center fill-height">
                <span class="placeholder-icon large">📻</span>
              </div>
            </template>
          </v-img>
        </div>

        <div class="flex-grow-1 pt-1 overflow-hidden mr-4">
          <h1 class="podcast-title">[ {{ currentSub.title }} ]</h1>
          <div class="podcast-author">&gt; {{ currentSub.author || currentSub.owner_name }}</div>

          <div class="d-flex align-center mb-3 mt-3">
            <button class="console-btn primary" @click="$emit('play-latest')">
              <span class="btn-icon">▶</span>
              <span class="btn-text">{{ $t('podcast.playLatest') }}</span>
            </button>
            <button
              v-if="isPlayingThisSub"
              class="console-btn ml-2"
              @click="$emit('scroll-to-playing')"
            >
              <span class="btn-icon">◎</span>
              <span class="btn-text">{{ $t('podcast.locatePlaying') }}</span>
            </button>
          </div>

          <div class="podcast-categories">
            <span
              v-for="cat in currentSub.categories.slice(0, 3)"
              :key="cat"
              class="tag-chip"
            >
              {{ cat }}
            </span>
          </div>
        </div>

        <!-- Right Side Stats & Actions -->
        <div class="d-flex flex-column align-end justify-space-between pl-4 ml-auto stats-panel">
          <div class="d-flex align-center">
            <button class="console-btn icon-only small" @click="$emit('refresh')" :disabled="refreshLoading" :title="$t('common.refresh')">
              <span class="btn-icon" :class="{ spinning: refreshLoading }">↻</span>
            </button>
            <button class="console-btn icon-only small danger ml-1" @click="$emit('unsubscribe')" :title="$t('common.delete')">
              <span class="btn-icon">✕</span>
            </button>
          </div>

          <div class="text-right mt-auto pb-1">
            <div class="episodes-count">{{ episodes.length }}</div>
            <div class="episodes-label">EPISODES</div>
          </div>
        </div>
      </div>

      <div class="podcast-description mt-3">
        <span class="desc-prefix">&gt;</span>
        <span class="desc-text">{{ currentSub.description }}</span>
      </div>
    </div>

    <!-- Episodes List -->
    <div class="episodes-container flex-grow-1 console-scroll">
      <div class="episodes-header">[ {{ $t('podcast.episodes') }} ]</div>

      <div v-if="episodesLoading" class="loading-episodes">
        <div class="loading-spinner"></div>
        <div class="loading-text">LOADING_FEED...</div>
      </div>

      <div v-else class="episodes-list">
        <div
          v-for="ep in visibleEpisodes"
          :key="ep.audio_url"
          :id="getEpisodeId(ep.audio_url)"
          class="episode-item"
          :class="{ active: currentPlaying?.audio_url === ep.audio_url }"
          @click="$emit('play-episode', ep)"
        >
          <!-- Image with Hover Overlay -->
          <div class="episode-cover mr-3 flex-shrink-0" @click.stop="$emit('show-notes', ep)">
            <v-img
              v-if="ep.image_url || currentSub?.image_url"
              :src="ep.image_url || currentSub?.image_url"
              cover
              class="cover-thumb"
            >
              <template v-slot:error>
                <div class="d-flex flex-column align-center justify-center fill-height date-thumb">
                  <span class="date-month">{{ formatMonth(ep.pub_date) }}</span>
                  <span class="date-day">{{ formatDay(ep.pub_date) }}</span>
                </div>
              </template>
            </v-img>
            <div v-else class="d-flex flex-column align-center justify-center fill-height date-thumb">
              <span class="date-month">{{ formatMonth(ep.pub_date) }}</span>
              <span class="date-day">{{ formatDay(ep.pub_date) }}</span>
            </div>

            <div class="episode-overlay">
              <span class="overlay-icon">ℹ</span>
            </div>
          </div>

          <div class="episode-content flex-grow-1 overflow-hidden">
            <div class="episode-title">{{ ep.title }}</div>
            <div class="episode-desc">
              {{ ep.description.replace(/<[^>]*>/g, '').substring(0, 120) }}
            </div>

            <div class="episode-meta">
              <div v-if="currentPlaying?.audio_url === ep.audio_url" class="playing-indicator-inline">
                <div class="playing-bar bar-1"></div>
                <div class="playing-bar bar-2"></div>
                <div class="playing-bar bar-3"></div>
              </div>
              <span class="meta-item">
                <span class="meta-icon">⏱</span>
                <span class="meta-text">{{ ep.duration }}</span>
              </span>
              <span v-if="ep.episode_number" class="meta-item">
                <span class="meta-tag">EP {{ ep.episode_number }}</span>
              </span>
            </div>
          </div>

          <div class="episode-action">
            <span class="play-icon">{{ currentPlaying?.audio_url === ep.audio_url ? '❚❚' : '▶' }}</span>
          </div>
        </div>
      </div>
      <div ref="loadMoreTrigger" class="load-more-trigger"></div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { PodcastEpisode, PodcastSubscription } from '@/api/podcast'

defineProps<{
  currentSub: PodcastSubscription
  episodes: PodcastEpisode[]
  visibleEpisodes: PodcastEpisode[]
  episodesLoading: boolean
  refreshLoading: boolean
  currentPlaying: { audio_url: string } | null
  currentPlayingSubUrl: string | null
  isPlayingThisSub: boolean
  getEpisodeId: (url: string) => string
  formatMonth: (dateStr?: string) => string
  formatDay: (dateStr?: string) => string
}>()

defineEmits<{
  (e: 'play-episode', ep: PodcastEpisode): void
  (e: 'play-latest'): void
  (e: 'show-notes', ep: PodcastEpisode): void
  (e: 'refresh'): void
  (e: 'unsubscribe'): void
  (e: 'scroll-to-playing'): void
}>()

const _loadMoreTrigger = defineModel<HTMLElement | null>('loadMoreTrigger')
</script>
