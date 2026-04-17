<template>
  <div class="podcast-manager scifi-page">
    <!-- CRT Effects -->
    <CrtOverlay />
    <!-- Grid Background -->
    <div class="grid-bg-layer"></div>

    <div class="content-wrapper">
      <!-- Main Content -->
      <div class="main-content d-flex flex-column h-100 overflow-hidden">

        <!-- Top Bar / Header -->
        <div class="console-header-bar">
          <div class="header-row">
            <div class="header-left">
              <template v-if="currentView === 'detail'">
                <button class="console-btn icon-only" @click="goBack">
                  <span class="btn-icon">◀</span>
                </button>
                <div class="console-title">
                  <span class="title-prefix">[</span>
                  <span class="title-text">{{ currentSub?.title || 'BACK' }}</span>
                  <span class="title-suffix">]</span>
                </div>
              </template>
              <template v-else>
                <div class="status-indicators">
                  <div class="status-light online"></div>
                  <div class="status-light standby"></div>
                </div>
                <div class="console-title">
                  <span class="title-prefix">[</span>
                  <span class="title-text">PODCAST_ARCHIVE</span>
                  <span class="title-suffix">]</span>
                </div>
                <div class="header-metrics">
                  <span class="metric-item">
                    <span class="metric-label">FEEDS:</span>
                    <span class="metric-value">{{ subscriptions.length }}</span>
                  </span>
                </div>
              </template>
            </div>

            <div class="header-right" v-if="currentView === 'library'">
              <div class="input-wrapper">
                <span class="input-prompt">>></span>
                <input
                  v-model="searchQuery"
                  type="text"
                  class="console-input"
                  :placeholder="$t('common.search') + '...'"
                />
              </div>
              <button class="console-btn icon-only" @click="refreshLibrary" :disabled="libraryRefreshLoading" :title="$t('common.refresh')">
                <span class="btn-icon" :class="{ spinning: libraryRefreshLoading }">↻</span>
              </button>
              <button class="console-btn primary" @click="openAddDialog">
                <span class="btn-icon">＋</span>
                <span class="btn-text">{{ $t('podcast.addShow') }}</span>
              </button>
            </div>
          </div>
        </div>

        <!-- Content Area -->
        <div class="main-content-area">
          <PodcastLibrary
            v-if="currentView === 'library'"
            :subscriptions="subscriptions"
            :search-query="searchQuery"
            :loading="loading"
            :current-playing="currentPlaying"
            :current-playing-sub-url="currentPlayingSubUrl"
            @select="selectSubscription"
            @add="openAddDialog"
          />

          <PodcastDetail
            v-else-if="currentView === 'detail' && currentSub"
            :current-sub="currentSub"
            :episodes="episodes"
            :visible-episodes="visibleEpisodes"
            :episodes-loading="episodesLoading"
            :refresh-loading="refreshLoading"
            :current-playing="currentPlaying"
            :current-playing-sub-url="currentPlayingSubUrl"
            :is-playing-this-sub="!!(currentPlaying && currentPlayingSubUrl === currentSub.rss_url)"
            :get-episode-id="getEpisodeId"
            :format-month="formatMonth"
            :format-day="formatDay"
            v-model:load-more-trigger="episodesLoadMoreTrigger"
            @play-episode="playEpisode"
            @play-latest="playLatestEpisode"
            @show-notes="openShowNotes"
            @refresh="refreshSub(currentSub)"
            @unsubscribe="unsubscribe(currentSub)"
            @scroll-to-playing="scrollToPlaying"
          />
        </div>
      </div>
    </div>

    <!-- Add Subscription Dialog -->
    <v-dialog v-model="showAddDialog" max-width="500" persistent>
      <v-card class="scifi-card">
        <v-card-title class="console-title-bar">
          <span class="dialog-title">[ IMPORT_OPML ]</span>
        </v-card-title>
        <v-card-text class="console-card-text">
          <div class="dialog-instruction mb-3">{{ $t('podcast.selectOpml') }}</div>
          <div class="input-wrapper">
            <span class="input-prompt">>></span>
            <v-file-input
              v-model="opmlFile"
              class="console-file-input"
              :label="$t('podcast.opmlFile')"
              variant="plain"
              density="compact"
              accept=".opml,.xml"
              :error-messages="opmlError"
              hide-details
            ></v-file-input>
          </div>
          <OpmlExportGuide />
        </v-card-text>
        <v-card-actions class="console-card-actions">
          <v-spacer></v-spacer>
          <button class="console-btn" @click="closeAddDialog" :disabled="addLoading">
            <span class="btn-text">CANCEL</span>
          </button>
          <button class="console-btn primary" @click="importOpmlFile" :disabled="addLoading">
            <span class="btn-text" v-if="!addLoading">{{ $t('podcast.import') }}</span>
            <span class="btn-text" v-else>IMPORTING...</span>
          </button>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Show Notes Dialog -->
    <v-dialog
      v-model="showShowNotesDialog"
      fullscreen
      persistent
      class="show-notes-dialog"
      transition="dialog-bottom-transition"
    >
      <div
        v-if="currentShowNotesEpisode"
        class="position-relative d-flex flex-column fill-height w-100 overflow-hidden show-notes-container"
      >
        <CrtOverlay />

        <!-- Fixed Top Bar -->
        <div class="notes-header-bar position-absolute top-0 right-0 w-100 d-flex justify-end pa-4" style="z-index: 20;">
          <button class="console-btn icon-only" @click="showShowNotesDialog = false">
            <span class="btn-icon">✕</span>
          </button>
        </div>

        <!-- Header Area -->
        <div class="notes-header flex-shrink-0 position-relative" style="z-index: 10;">
          <div class="show-notes-bg-text">SHOW NOTES</div>

          <div class="d-flex align-end px-6 pt-12 pb-4 notes-header-content">
            <div class="notes-cover mr-4 flex-shrink-0">
              <v-img
                :src="currentShowNotesEpisode.image_url || currentSub?.image_url"
                width="120"
                height="120"
                cover
                class="notes-cover-image"
              ></v-img>
            </div>

            <div class="flex-grow-1 pb-1">
              <h2 class="notes-title">[ {{ currentShowNotesEpisode.title }} ]</h2>

              <div class="notes-meta">
                <span class="meta-podcast">{{ currentShowNotesEpisode.podcast_name || currentSub?.title }}</span>
                <span class="meta-separator">|</span>
                <span class="meta-date">{{ formatMonth(currentShowNotesEpisode.pub_date) }} {{ formatDay(currentShowNotesEpisode.pub_date) }}</span>
                <span class="meta-separator">|</span>
                <span class="meta-duration">{{ currentShowNotesEpisode.duration }}</span>
              </div>

              <div class="d-flex align-center mt-3">
                <button class="console-btn" @click="showShowNotesDialog = false">
                  <span class="btn-icon">◀</span>
                  <span class="btn-text">{{ $t('common.back') }}</span>
                </button>
                <button class="console-btn primary ml-2" @click="playEpisode(currentShowNotesEpisode)">
                  <span class="btn-icon">▶</span>
                  <span class="btn-text">{{ $t('podcast.play') }}</span>
                </button>
              </div>
            </div>
          </div>
        </div>

        <!-- Scrollable Content -->
        <div class="notes-body flex-grow-1 console-scroll" style="z-index: 10;">
          <div class="notes-content px-6 pt-4 pb-10">
            <EpisodeShowNotes :content="currentShowNotesEpisode.show_notes" />
          </div>
        </div>
      </div>
    </v-dialog>

    <v-snackbar
      v-model="snackbar"
      :color="snackbarColor"
      :scrim="false"
      timeout="3000"
      location="top"
      class="console-snackbar"
    >
      <div class="snackbar-content">
        <span class="snackbar-text">{{ snackbarText }}</span>
      </div>
    </v-snackbar>
  </div>
</template>

<script setup lang="ts">
import CrtOverlay from '@/components/common/CrtOverlay.vue'
import EpisodeShowNotes from '@/components/podcast/EpisodeShowNotes.vue'
import OpmlExportGuide from '@/components/podcast/OpmlExportGuide.vue'
import { usePodcastManager } from '@/composables/usePodcastManager'
import PodcastDetail from './PodcastDetail.vue'
import PodcastLibrary from './PodcastLibrary.vue'

const {
  subscriptions,
  currentSub,
  episodes,
  episodesLoading,
  loading,
  searchQuery,
  visibleEpisodes,
  episodesLoadMoreTrigger,
  currentView,
  showAddDialog,
  addLoading,
  refreshLoading,
  libraryRefreshLoading,
  showShowNotesDialog,
  currentShowNotesEpisode,
  opmlFile,
  opmlError,
  snackbar,
  snackbarText,
  snackbarColor,
  currentPlaying,
  currentPlayingSubUrl,
  selectSubscription,
  goBack,
  playLatestEpisode,
  playEpisode,
  getEpisodeId,
  scrollToPlaying,
  openShowNotes,
  refreshSub,
  refreshLibrary,
  openAddDialog,
  closeAddDialog,
  importOpmlFile,
  unsubscribe,
  formatMonth,
  formatDay,
} = usePodcastManager()
</script>

<style scoped>
.podcast-manager {
  position: relative;
  height: 100%;
  overflow: hidden;
}

.content-wrapper {
  position: relative;
  z-index: 2;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.main-content {
  height: 100%;
}

.main-content-area {
  flex: 1;
  overflow: hidden;
}

/* Header */
.console-header-bar {
  padding: 12px 16px;
  border-bottom: 1px solid rgba(0, 255, 255, 0.15);
  background: rgba(0, 0, 0, 0.2);
  flex-shrink: 0;
}

.header-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.header-left, .header-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.header-metrics {
  display: flex;
  gap: 16px;
}

.metric-item {
  display: flex;
  align-items: center;
  gap: 4px;
}

.metric-label {
  font-size: 9px;
  color: #52525b;
  font-family: 'JetBrains Mono', monospace;
  letter-spacing: 1px;
}

.metric-value {
  font-size: 12px;
  font-weight: 700;
  color: #00ff88;
  font-family: 'JetBrains Mono', monospace;
}

/* Snackbar */
.console-snackbar :deep(.v-snackbar__wrapper) {
  background: rgba(0, 0, 0, 0.9) !important;
  border: 1px solid rgba(0, 255, 255, 0.2);
}

.snackbar-content {
  display: flex;
  align-items: center;
  gap: 8px;
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
}

/* Light Theme */
.light-theme .console-header-bar {
  background: rgba(184, 134, 11, 0.05);
  border-bottom-color: rgba(184, 134, 11, 0.2);
}

.light-theme .metric-label {
  color: #6b4423;
}

.light-theme .metric-value {
  color: #cd7f32;
}

.light-theme .console-snackbar :deep(.v-snackbar__wrapper) {
  background: #efe0cc !important;
  border-color: rgba(184, 134, 11, 0.3);
}

.light-theme .snackbar-content {
  color: #3d2914;
}
</style>
