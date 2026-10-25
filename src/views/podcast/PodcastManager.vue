<template>
  <div class="podcast-manager scifi-page">
    <div class="content-wrapper">
      <!-- Main Content -->
      <div class="main-content d-flex flex-column h-100 overflow-hidden">

        <!-- Page Header: aligned with chat/hosts page header tokens -->
        <div class="podcast-header">
          <div class="header-logo">
            <template v-if="currentView === 'detail'">
              <button
                type="button"
                class="header-back-btn"
                :aria-label="$t('common.back')"
                :title="$t('common.back')"
                @click="goBack"
              >
                <v-icon :icon="mdiArrowLeft" size="18" aria-hidden="true" />
              </button>
              <span class="header-title">{{ currentSub?.title || $t('common.back') }}</span>
            </template>
            <template v-else>
              <v-icon :icon="mdiPodcast" size="18" aria-hidden="true" />
              <span class="header-title">{{ $t('sidebar.podcast') }}</span>
            </template>
          </div>

          <div v-if="currentView === 'library'" class="header-actions">
            <div class="input-wrapper">
              <v-icon :icon="mdiMagnify" size="16" class="search-icon" aria-hidden="true" />
              <input
                v-model="searchQuery"
                type="text"
                class="console-input"
                :placeholder="$t('common.search')"
                :aria-label="$t('common.search')"
              />
            </div>
            <button
              type="button"
              class="console-btn icon-only"
              :disabled="libraryRefreshLoading"
              :title="$t('common.refresh')"
              :aria-label="$t('common.refresh')"
              @click="refreshLibrary"
            >
              <span class="btn-icon" :class="{ spinning: libraryRefreshLoading }">↻</span>
            </button>
            <button type="button" class="header-primary-btn" @click="openAddDialog">
              <v-icon :icon="mdiPlus" size="16" aria-hidden="true" />
              <span>{{ $t('podcast.addShow') }}</span>
            </button>
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
          <span class="dialog-title">{{ $t('podcast.importOpml') }}</span>
        </v-card-title>
        <v-card-text class="console-card-text">
          <div class="dialog-instruction mb-3">{{ $t('podcast.selectOpml') }}</div>
          <div class="input-wrapper">
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
            <span class="btn-text">{{ $t('common.cancel') }}</span>
          </button>
          <button class="console-btn primary" @click="importOpmlFile" :disabled="addLoading">
            <span class="btn-text" v-if="!addLoading">{{ $t('podcast.import') }}</span>
            <span class="btn-text" v-else>{{ $t('podcast.importing') }}</span>
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
        <!-- Fixed Top Bar -->
        <div class="notes-header-bar position-absolute top-0 right-0 w-100 d-flex justify-end pa-4" style="z-index: 20;">
          <button class="console-btn icon-only" @click="showShowNotesDialog = false">
            <span class="btn-icon">✕</span>
          </button>
        </div>

        <!-- Header Area -->
        <div class="notes-header flex-shrink-0 position-relative" style="z-index: 10;">
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

    <AppNotification />
  </div>
</template>

<script setup lang="ts">
import { mdiArrowLeft, mdiMagnify, mdiPlus, mdiPodcast } from '@mdi/js'
import { useI18n } from 'vue-i18n'
import { importOpml } from '@/api/podcast'
import AppNotification from '@/components/common/AppNotification.vue'
import EpisodeShowNotes from '@/components/podcast/EpisodeShowNotes.vue'
import OpmlExportGuide from '@/components/podcast/OpmlExportGuide.vue'
import { useAppNotification } from '@/composables/useAppNotification'
import { usePodcastManager } from '@/composables/usePodcastManager'
import PodcastDetail from './PodcastDetail.vue'
import PodcastLibrary from './PodcastLibrary.vue'

const { t } = useI18n()
const { notify } = useAppNotification()

// biome-ignore lint/correctness/noUnusedVariables: false positive
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
  unsubscribe,
  formatMonth,
  formatDay,
} = usePodcastManager()

async function importOpmlFile() {
  opmlError.value = ''
  let file: File | null = null

  if (Array.isArray(opmlFile.value)) {
    if (opmlFile.value.length > 0) {
      file = opmlFile.value[0]
    }
  } else {
    file = opmlFile.value
  }

  if (!file) {
    opmlError.value = 'Please select a file'
    return
  }

  if (!file.name.endsWith('.opml') && !file.name.endsWith('.xml')) {
    opmlError.value = 'Only .opml or .xml files are supported'
    return
  }

  addLoading.value = true
  const countBefore = subscriptions.value.length
  try {
    const text = await file.text()
    if (text.length === 0) {
      throw new Error('File is empty')
    }
    subscriptions.value = await importOpml(text)
    const added = Math.max(subscriptions.value.length - countBefore, 0)
    notify(t('podcast.importAdded', { n: added }), 'success')
    closeAddDialog()
  } catch (e: unknown) {
    console.error('Import failed:', e)
    notify(
      t('podcast.importFailed', { message: e instanceof Error ? e.message : String(e) }),
      'error'
    )
  } finally {
    addLoading.value = false
  }
}
</script>

<style scoped>
.podcast-manager {
  position: relative;
  height: 100%;
  overflow: hidden;
}

.search-icon {
  color: var(--text-subtle);
  margin-right: 8px;
  flex-shrink: 0;
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

/* Page header: aligned with chat/hosts page header tokens */
.podcast-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-shrink: 0;
  min-height: 60px;
  padding: 12px 24px;
  background: var(--bg);
  border-bottom: 1px solid var(--border);
}

.podcast-header .header-logo {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
  color: var(--text);
}

.podcast-header .header-title {
  font-size: 15px;
  font-weight: 600;
  font-family: var(--jedi-font-ui);
  color: var(--text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.header-back-btn {
  display: grid;
  place-items: center;
  width: 32px;
  height: 32px;
  flex-shrink: 0;
  color: var(--text-muted);
  background: transparent;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  transition: background var(--motion-fast), color var(--motion-fast);
}

.header-back-btn:hover {
  background: var(--bg-elevated-2);
  color: var(--text);
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}

.header-primary-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  min-height: 32px;
  padding: 0 14px;
  font-size: 13px;
  font-weight: 500;
  font-family: var(--jedi-font-ui);
  color: var(--on-accent);
  background: var(--accent);
  border: 1px solid var(--accent);
  border-radius: 8px;
  cursor: pointer;
  transition: background var(--motion-fast), border-color var(--motion-fast);
}

.header-primary-btn:hover {
  background: var(--accent-hover);
  border-color: var(--accent-hover);
}

/* Snackbar */
.console-snackbar :deep(.v-snackbar__wrapper) {
  background:rgb(var(--ink-rgb) / 0.9) !important;
  border: 1px solid rgb(var(--accent-rgb) / 0.2);
}

.snackbar-content {
  display: flex;
  align-items: center;
  gap: 8px;
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
}
</style>
