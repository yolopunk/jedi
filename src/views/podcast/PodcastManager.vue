<template>
  <div class="podcast-manager scifi-page">
    <!-- CRT Effects -->
    <div class="scanlines"></div>
    <div class="crt-vignette"></div>
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

          <!-- Grid View: Library -->
          <div v-if="currentView === 'library'" class="podcasts-grid-container console-scroll">
            <div v-if="loading && !subscriptions.length" class="loading-grid">
              <div v-for="i in 8" :key="i" class="loading-card">
                <div class="loading-image"></div>
                <div class="loading-text"></div>
              </div>
            </div>

            <div v-else-if="subscriptions.length" class="podcast-grid">
              <div
                v-for="sub in subscriptions"
                :key="sub.rss_url"
                class="podcast-item"
                @click="selectSubscription(sub)"
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
              <button class="console-btn primary mt-4" @click="openAddDialog">
                <span class="btn-text">{{ $t('podcast.addShow') }}</span>
              </button>
            </div>
          </div>

          <!-- Detail View: Podcast Info & Episodes -->
          <div v-else-if="currentView === 'detail' && currentSub" class="detail-view-container d-flex flex-column overflow-hidden h-100">
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
                    <button class="console-btn primary" @click="playLatestEpisode">
                      <span class="btn-icon">▶</span>
                      <span class="btn-text">{{ $t('podcast.playLatest') }}</span>
                    </button>
                    <button
                      v-if="currentPlaying && currentPlayingSubUrl === currentSub.rss_url"
                      class="console-btn ml-2"
                      @click="scrollToPlaying"
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
                    <button class="console-btn icon-only small" @click="refreshSub(currentSub)" :disabled="refreshLoading" :title="$t('common.refresh')">
                      <span class="btn-icon" :class="{ spinning: refreshLoading }">↻</span>
                    </button>
                    <button class="console-btn icon-only small danger ml-1" @click="unsubscribe(currentSub)" :title="$t('common.delete')">
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
                  @click="playEpisode(ep)"
                >
                  <!-- Image with Hover Overlay -->
                  <div class="episode-cover mr-3 flex-shrink-0" @click.stop="openShowNotes(ep)">
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
              <div ref="episodesLoadMoreTrigger" class="load-more-trigger"></div>
            </div>
          </div>

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
        <!-- CRT Effects for Dialog -->
        <div class="scanlines"></div>
        <div class="crt-vignette"></div>

        <!-- Fixed Top Bar -->
        <div class="notes-header-bar position-absolute top-0 right-0 w-100 d-flex justify-end pa-4" style="z-index: 20;">
          <button class="console-btn icon-only" @click="showShowNotesDialog = false">
            <span class="btn-icon">✕</span>
          </button>
        </div>

        <!-- Header Area -->
        <div class="notes-header flex-shrink-0 position-relative" style="z-index: 10;">
          <!-- Background SHOW NOTES text -->
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
                <span class="meta-podcast">📻 {{ currentShowNotesEpisode.podcast_name || currentSub?.title }}</span>
                <span class="meta-separator">|</span>
                <span class="meta-date">{{ formatMonth(currentShowNotesEpisode.pub_date) }} {{ formatDay(currentShowNotesEpisode.pub_date) }}</span>
                <span class="meta-separator">|</span>
                <span class="meta-duration">{{ currentShowNotesEpisode.duration }}</span>
                <span v-if="currentShowNotesEpisode.chapters && currentShowNotesEpisode.chapters.length" class="meta-separator">|</span>
                <span v-if="currentShowNotesEpisode.chapters && currentShowNotesEpisode.chapters.length">
                  {{ $t('podcast.chapters') }}{{ currentShowNotesEpisode.chapters.length }}
                </span>
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
        <span class="snackbar-icon">{{ getSnackbarIcon }}</span>
        <span class="snackbar-text">{{ snackbarText }}</span>
      </div>
    </v-snackbar>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, computed, nextTick } from 'vue';
import {
  getSubscriptions,
  removeSubscription,
  fetchEpisodes,
  importOpml,
  refreshSubscription,
  type PodcastEpisode,
  type PodcastSubscription
} from '@/api/podcast';
import EpisodeShowNotes from '@/components/podcast/EpisodeShowNotes.vue';
import OpmlExportGuide from '@/components/podcast/OpmlExportGuide.vue';

import { useAudioPlayer } from '@/composables/useAudioPlayer';

// State
const subscriptions = ref<PodcastSubscription[]>([]);
const currentSub = ref<PodcastSubscription | null>(null);
const episodes = ref<PodcastEpisode[]>([]);
const episodesLoading = ref(false);
const loading = ref(false);
const searchQuery = ref('');

const singleEpisode = ref<PodcastEpisode | null>(null);

const EPISODES_PAGE_SIZE = 30;
const visibleEpisodeCount = ref(EPISODES_PAGE_SIZE);
const visibleEpisodes = computed(() =>
  episodes.value.slice(0, visibleEpisodeCount.value)
);
const episodesLoadMoreTrigger = ref<HTMLElement | null>(null);
let episodesObserver: IntersectionObserver | null = null;

const { currentPlaying, currentPlayingSubUrl, playEpisode: globalPlayEpisode } = useAudioPlayer();

// Navigation State
const currentView = ref<'library' | 'detail'>('library');

// Add Dialog State
const showAddDialog = ref(false);
const addLoading = ref(false);

const refreshLoading = ref(false);
const libraryRefreshLoading = ref(false);

// Show Notes Dialog State
const showShowNotesDialog = ref(false);
const currentShowNotesEpisode = ref<PodcastEpisode | null>(null);

// OPML State
const opmlFile = ref<File | File[] | null>(null);
const opmlError = ref('');

// UI
const snackbar = ref(false);
const snackbarText = ref('');
const snackbarColor = ref<'success' | 'error' | 'info' | 'warning'>('error');

const getSnackbarIcon = computed(() => {
  switch (snackbarColor.value) {
    case 'success':
      return '✓'
    case 'error':
      return '✕'
    case 'warning':
      return '!'
    default:
      return '›'
  }
});

onMounted(async () => {
  loading.value = true;
  await loadSubscriptions();
  loading.value = false;
});

onBeforeUnmount(() => {
  if (episodesObserver) {
    episodesObserver.disconnect();
    episodesObserver = null;
  }
});

async function loadSubscriptions() {
  try {
    subscriptions.value = await getSubscriptions();
  } catch (e) {
    console.error(e);
  }
}

async function selectSubscription(sub: PodcastSubscription) {
  currentSub.value = sub;
  currentView.value = 'detail';
  singleEpisode.value = null;
  episodesLoading.value = true;
  episodes.value = [];
  visibleEpisodeCount.value = EPISODES_PAGE_SIZE;

  try {
    episodes.value = await fetchEpisodes(sub.rss_url);
    await nextTick();
    setupEpisodesObserver();
  } catch (e: any) {
    showMsg('Failed to load episodes: ' + e, 'error');
  } finally {
    episodesLoading.value = false;
  }
}

function goBack() {
  currentView.value = 'library';
  currentSub.value = null;
  if (episodesObserver) {
    episodesObserver.disconnect();
    episodesObserver = null;
  }
}

function playLatestEpisode() {
  if (episodes.value.length > 0) {
    playEpisode(episodes.value[0]);
  }
}

function playEpisode(ep: PodcastEpisode) {
  if (!ep.podcast_name && currentSub.value) {
    ep.podcast_name = currentSub.value.title;
  }
  if (!ep.image_url && currentSub.value) {
    ep.image_url = currentSub.value.image_url;
  }

  globalPlayEpisode(ep, currentSub.value?.rss_url);
}

function getEpisodeId(url: string) {
  return `ep-${btoa(url).replace(/[^a-zA-Z0-9]/g, '')}`;
}

function scrollToPlaying() {
  if (!currentPlaying.value) return;

  const el = document.getElementById(getEpisodeId(currentPlaying.value.audio_url));
  if (el) {
    el.scrollIntoView({ behavior: 'smooth', block: 'center' });
    el.classList.add('highlight-flash');
    setTimeout(() => el.classList.remove('highlight-flash'), 2000);
  }
}

function openShowNotes(ep: PodcastEpisode) {
  currentShowNotesEpisode.value = ep;
  showShowNotesDialog.value = true;
}

async function refreshSub(sub: PodcastSubscription | null) {
  if (!sub) return;
  refreshLoading.value = true;
  try {
    const newSub = await refreshSubscription(sub.rss_url);
    currentSub.value = newSub;
    const index = subscriptions.value.findIndex(s => s.rss_url === sub.rss_url);
    if (index !== -1) {
      subscriptions.value[index] = newSub;
    }
    showMsg('Subscription refreshed', 'success');
  } catch (e: any) {
    showMsg('Failed to refresh: ' + e, 'error');
  } finally {
    refreshLoading.value = false;
  }
}

async function refreshLibrary() {
  libraryRefreshLoading.value = true;
  try {
    for (let i = 0; i < subscriptions.value.length; i++) {
      const sub = subscriptions.value[i];
      try {
        const newSub = await refreshSubscription(sub.rss_url);
        subscriptions.value[i] = newSub;
      } catch (e) {
        console.error(`Failed to refresh ${sub.title}:`, e);
      }
    }
    showMsg('Library refreshed', 'success');
  } catch (e: any) {
    showMsg('Failed to refresh library: ' + e, 'error');
  } finally {
    libraryRefreshLoading.value = false;
  }
}

function openAddDialog() {
  resetAddDialog();
  showAddDialog.value = true;
}

function closeAddDialog() {
  if (addLoading.value) return;
  showAddDialog.value = false;
  resetAddDialog();
}

function resetAddDialog() {
  opmlFile.value = null;
  opmlError.value = '';
}

async function importOpmlFile() {
  opmlError.value = '';

  let file: File | null = null;

  if (Array.isArray(opmlFile.value)) {
    if (opmlFile.value.length > 0) {
      file = opmlFile.value[0];
    }
  } else {
    file = opmlFile.value;
  }

  if (!file) {
    opmlError.value = 'Please select a file';
    return;
  }

  if (!file.name.endsWith('.opml') && !file.name.endsWith('.xml')) {
    opmlError.value = 'Only .opml or .xml files are supported';
    return;
  }

  addLoading.value = true;
  try {
    const text = await file.text();
    console.log('Reading OPML file, length:', text.length);
    if (text.length === 0) {
      throw new Error('File is empty');
    }
    subscriptions.value = await importOpml(text);
    showMsg('Import successful', 'success');
    closeAddDialog();
  } catch (e: any) {
    console.error('Import failed:', e);
    showMsg('Import failed: ' + e, 'error');
  } finally {
    addLoading.value = false;
  }
}

async function unsubscribe(sub: PodcastSubscription) {
  if (!confirm(`Unsubscribe from "${sub.title}"?`)) return;
  try {
    subscriptions.value = await removeSubscription(sub.rss_url);
    if (currentSub.value?.rss_url === sub.rss_url) {
      goBack();
    }
  } catch (e: any) {
    showMsg('Operation failed: ' + e, 'error');
  }
}

function showMsg(text: string, color: 'success' | 'error' | 'info' | 'warning') {
  snackbarText.value = text;
  snackbarColor.value = color;
  snackbar.value = true;
}

function formatMonth(dateStr?: string): string {
  if (!dateStr) return '';
  const date = new Date(dateStr);
  return date.toLocaleDateString('en-US', { month: 'short' }).toUpperCase();
}

function formatDay(dateStr?: string): string {
  if (!dateStr) return '';
  const date = new Date(dateStr);
  return date.getDate().toString();
}

function setupEpisodesObserver() {
  if (episodesObserver) {
    episodesObserver.disconnect();
  }

  episodesObserver = new IntersectionObserver(entries => {
    const entry = entries[0];
    if (!entry.isIntersecting) {
      return;
    }
    if (visibleEpisodeCount.value >= episodes.value.length) {
      return;
    }
    visibleEpisodeCount.value = Math.min(
      visibleEpisodeCount.value + EPISODES_PAGE_SIZE,
      episodes.value.length
    );
  });

  if (episodesLoadMoreTrigger.value) {
    episodesObserver.observe(episodesLoadMoreTrigger.value);
  }
}
</script>

<style scoped>
.podcast-manager {
  position: relative;
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  overflow: hidden;
  background: linear-gradient(180deg, #0a0a0f 0%, #0d0d14 50%, #0a0a0f 100%);
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', monospace;
}

.content-wrapper {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.main-content {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.main-content-area {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

/* Header Styles */
.console-header-bar {
  background: linear-gradient(180deg, #0f0f1a 0%, #0a0a12 100%);
  border-bottom: 1px solid rgba(0, 255, 255, 0.15);
  padding: 12px 16px;
  flex-shrink: 0;
  position: relative;
  z-index: 10;
}

.header-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 16px;
  flex-wrap: wrap;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
  justify-content: flex-end;
}

.header-metrics {
  display: flex;
  gap: 16px;
}

.status-indicators {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-light {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  position: relative;
}

.status-light.online {
  background: #00ff88;
  box-shadow: 0 0 10px #00ff88, 0 0 20px rgba(0, 255, 136, 0.33);
  animation: pulse-online 2s ease-in-out infinite;
}

.status-light.standby {
  background: #ffaa00;
  box-shadow: 0 0 10px #ffaa00, 0 0 20px rgba(255, 170, 0, 0.33);
  animation: pulse-standby 1.5s ease-in-out infinite;
}

@keyframes pulse-online {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

@keyframes pulse-standby {
  0%, 100% { opacity: 0.8; transform: scale(1); }
  50% { opacity: 1; transform: scale(1.2); }
}

.console-title {
  display: flex;
  align-items: center;
  gap: 4px;
}

.title-prefix, .title-suffix {
  color: #00ffff;
  font-size: 11px;
  text-shadow: 0 0 10px rgba(0, 255, 255, 0.5);
}

.title-text {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 2px;
  color: #00ff88;
  text-shadow: 0 0 10px rgba(0, 255, 136, 0.5);
}

.metric-item {
  display: flex;
  align-items: center;
  gap: 4px;
}

.metric-label {
  color: #52525b;
  font-size: 10px;
  letter-spacing: 1px;
}

.metric-value {
  color: #00ffff;
  font-size: 12px;
  font-weight: 700;
  text-shadow: 0 0 8px rgba(0, 255, 255, 0.4);
}

.input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
  background: rgba(5, 5, 8, 0.9);
  border: 1px solid #1a1a3a;
  border-radius: 4px;
  padding: 6px 12px;
  min-width: 200px;
}

.input-prompt {
  color: #00ff88;
  font-size: 12px;
  margin-right: 8px;
  text-shadow: 0 0 8px rgba(0, 255, 136, 0.4);
}

.console-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  color: #00ffff;
  font-family: inherit;
  font-size: 12px;
}

.console-input::placeholder {
  color: #333;
}

.console-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  background: transparent;
  border: 1px solid #00ff88;
  color: #00ff88;
  padding: 6px 12px;
  border-radius: 4px;
  font-family: inherit;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 1px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.console-btn:hover:not(:disabled) {
  background: rgba(0, 255, 136, 0.07);
  box-shadow: 0 0 15px rgba(0, 255, 136, 0.27), inset 0 0 15px rgba(0, 255, 136, 0.07);
}

.console-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.console-btn.primary {
  border-color: #00ffff;
  color: #00ffff;
}

.console-btn.primary:hover:not(:disabled) {
  background: rgba(0, 255, 255, 0.07);
  box-shadow: 0 0 15px rgba(0, 255, 255, 0.27), inset 0 0 15px rgba(0, 255, 255, 0.07);
}

.console-btn.icon-only {
  padding: 6px 10px;
}

.btn-icon {
  font-size: 14px;
}

.btn-icon.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* CRT Effects */
.scanlines {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  pointer-events: none;
  z-index: 100;
  background: repeating-linear-gradient(
    0deg,
    rgba(0, 0, 0, 0.12),
    rgba(0, 0, 0, 0.12) 1px,
    transparent 1px,
    transparent 2px
  );
}

.crt-vignette {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  pointer-events: none;
  z-index: 99;
  background: radial-gradient(ellipse at center, transparent 0%, rgba(0,0,0,0.35) 100%);
}

.grid-bg-layer {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  pointer-events: none;
  z-index: 0;
  background-image:
    linear-gradient(rgba(0, 255, 255, 0.03) 1px, transparent 1px),
    linear-gradient(90deg, rgba(0, 255, 255, 0.03) 1px, transparent 1px);
  background-size: 40px 40px;
}

/* Podcast Grid */
.podcasts-grid-container {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 16px;
}

.podcast-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 16px;
  padding: 8px;
}

.podcast-item {
  cursor: pointer;
}

.podcast-card {
  position: relative;
  border-radius: 8px;
  overflow: hidden;
  background: rgba(15, 15, 26, 0.8);
  border: 1px solid rgba(0, 255, 255, 0.1);
  transition: all 0.2s ease;
}

.podcast-card:hover {
  border-color: rgba(0, 255, 255, 0.4);
  transform: translateY(-2px);
}

.podcast-card:hover .card-glow {
  opacity: 1;
}

.card-glow {
  position: absolute;
  inset: -2px;
  background: radial-gradient(circle at center, rgba(0, 255, 255, 0.15) 0%, transparent 70%);
  opacity: 0;
  transition: opacity 0.3s ease;
  pointer-events: none;
}

.image-wrapper {
  position: relative;
  aspect-ratio: 1;
  overflow: hidden;
}

.podcast-image {
  width: 100%;
  height: 100%;
}

.placeholder-icon {
  font-size: 48px;
  opacity: 0.5;
}

.placeholder-icon.large {
  font-size: 64px;
}

.playing-indicator {
  position: absolute;
  bottom: 8px;
  right: 8px;
  background: rgba(0, 0, 0, 0.7);
  border-radius: 4px;
  padding: 6px 8px;
  display: flex;
  align-items: flex-end;
  justify-content: center;
  gap: 3px;
}

.playing-bar {
  width: 3px;
  border-radius: 2px;
  background: #00ff88;
  box-shadow: 0 0 8px rgba(0, 255, 136, 0.6);
  animation: equalize 1s infinite ease-in-out;
}

.bar-1 { animation-delay: 0s; }
.bar-2 { animation-delay: 0.2s; }
.bar-3 { animation-delay: 0.4s; }

@keyframes equalize {
  0% { height: 6px; }
  50% { height: 16px; }
  100% { height: 6px; }
}

.card-footer {
  padding: 12px;
}

.card-title {
  font-size: 12px;
  font-weight: 700;
  color: #e4e4e7;
  margin-bottom: 4px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.card-meta {
  font-size: 10px;
  color: #71717a;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* Detail View */
.detail-view-container {
  height: 100%;
}

.podcast-header {
  padding: 20px;
  background: rgba(15, 15, 26, 0.6);
  border-bottom: 1px solid rgba(0, 255, 255, 0.15);
}

.podcast-cover {
  width: 140px;
  height: 140px;
  flex-shrink: 0;
}

.cover-image {
  border-radius: 8px;
  box-shadow: 0 0 20px rgba(0, 255, 255, 0.1);
}

.podcast-title {
  font-size: 20px;
  font-weight: 700;
  color: #00ff88;
  text-shadow: 0 0 10px rgba(0, 255, 136, 0.3);
  margin: 0 0 8px 0;
}

.podcast-author {
  font-size: 12px;
  color: #00ffff;
  text-shadow: 0 0 8px rgba(0, 255, 255, 0.3);
}

.podcast-categories {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  margin-top: 8px;
}

.tag-chip {
  font-size: 10px;
  padding: 4px 10px;
  border: 1px solid rgba(0, 255, 255, 0.3);
  border-radius: 4px;
  color: #a1a1aa;
  letter-spacing: 1px;
}

.stats-panel {
  border-left: 1px solid rgba(0, 255, 255, 0.1);
  padding-left: 20px;
  height: 140px;
}

.episodes-count {
  font-size: 36px;
  font-weight: 900;
  color: #00ffff;
  text-shadow: 0 0 15px rgba(0, 255, 255, 0.4);
  line-height: 1;
}

.episodes-label {
  font-size: 10px;
  font-weight: 700;
  color: #52525b;
  letter-spacing: 2px;
  margin-top: 4px;
}

.podcast-description {
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px solid rgba(0, 255, 255, 0.1);
  display: flex;
  gap: 12px;
}

.desc-prefix {
  color: #00ff88;
  font-size: 14px;
  text-shadow: 0 0 8px rgba(0, 255, 136, 0.4);
}

.desc-text {
  color: #71717a;
  font-size: 12px;
  line-height: 1.6;
}

/* Episodes List */
.episodes-container {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 16px 20px;
}

.episodes-header {
  font-size: 14px;
  font-weight: 700;
  color: #a1a1aa;
  letter-spacing: 2px;
  margin-bottom: 12px;
  padding: 0 8px;
}

.loading-episodes {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
}

.loading-spinner {
  width: 40px;
  height: 40px;
  border: 2px solid rgba(0, 255, 255, 0.2);
  border-top-color: #00ffff;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.episodes-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.episode-item {
  display: flex;
  align-items: center;
  padding: 12px;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s ease;
  position: relative;
}

.episode-item::before {
  content: '';
  position: absolute;
  left: 0;
  top: 8px;
  bottom: 8px;
  width: 2px;
  background: #00ffff;
  opacity: 0;
  transform: scaleY(0);
  transition: all 0.15s ease;
  box-shadow: 0 0 10px rgba(0, 255, 255, 0.8);
}

.episode-item:hover {
  background: rgba(0, 255, 255, 0.05);
}

.episode-item.active {
  background: rgba(0, 255, 255, 0.08);
}

.episode-item.active::before {
  opacity: 1;
  transform: scaleY(1);
}

.episode-cover {
  width: 64px;
  height: 64px;
  position: relative;
}

.cover-thumb {
  width: 100%;
  height: 100%;
  border-radius: 4px;
}

.date-thumb {
  background: rgba(5, 5, 8, 0.9);
  border: 1px solid rgba(0, 255, 255, 0.2);
  border-radius: 4px;
}

.date-month {
  font-size: 9px;
  color: #52525b;
  letter-spacing: 1px;
}

.date-day {
  font-size: 18px;
  font-weight: 700;
  color: #00ffff;
}

.episode-overlay {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: opacity 0.2s ease;
  border-radius: 4px;
}

.episode-cover:hover .episode-overlay {
  opacity: 1;
}

.overlay-icon {
  font-size: 20px;
}

.episode-content {
  flex: 1;
  min-width: 0;
}

.episode-title {
  font-size: 12px;
  font-weight: 600;
  color: #e4e4e7;
  margin-bottom: 4px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.episode-item.active .episode-title {
  color: #00ffff;
  text-shadow: 0 0 8px rgba(0, 255, 255, 0.3);
}

.episode-desc {
  font-size: 11px;
  color: #52525b;
  margin-bottom: 6px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.episode-meta {
  display: flex;
  align-items: center;
  gap: 12px;
}

.playing-indicator-inline {
  display: flex;
  align-items: flex-end;
  justify-content: center;
  gap: 2px;
  height: 16px;
}

.meta-item {
  display: flex;
  align-items: center;
  gap: 4px;
}

.meta-icon {
  font-size: 10px;
  opacity: 0.6;
}

.meta-text {
  font-size: 10px;
  color: #71717a;
}

.meta-tag {
  font-size: 10px;
  padding: 2px 6px;
  border: 1px solid rgba(0, 255, 255, 0.2);
  border-radius: 3px;
  color: #a1a1aa;
}

.episode-action {
  padding: 8px;
}

.play-icon {
  font-size: 12px;
  color: #00ff88;
  text-shadow: 0 0 8px rgba(0, 255, 136, 0.4);
  opacity: 0;
  transition: opacity 0.15s ease;
}

.episode-item:hover .play-icon,
.episode-item.active .play-icon {
  opacity: 1;
}

.highlight-flash {
  animation: flash-highlight 2s ease-out;
}

@keyframes flash-highlight {
  0% { background-color: rgba(0, 255, 255, 0.2); }
  100% { background-color: transparent; }
}

.load-more-trigger {
  height: 20px;
}

/* Empty State */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 80px 20px;
}

.empty-icon {
  font-size: 48px;
  color: #52525b;
  margin-bottom: 16px;
}

.empty-text {
  font-size: 12px;
  color: #71717a;
  letter-spacing: 2px;
}

/* Loading Grid */
.loading-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 16px;
  padding: 8px;
}

.loading-card {
  border-radius: 8px;
  overflow: hidden;
  background: rgba(15, 15, 26, 0.5);
  border: 1px solid rgba(0, 255, 255, 0.05);
}

.loading-image {
  aspect-ratio: 1;
  background: linear-gradient(90deg, rgba(0, 255, 255, 0.03) 25%, rgba(0, 255, 255, 0.08) 50%, rgba(0, 255, 255, 0.03) 75%);
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
}

.loading-text {
  height: 40px;
  margin: 12px;
  background: linear-gradient(90deg, rgba(0, 255, 255, 0.03) 25%, rgba(0, 255, 255, 0.08) 50%, rgba(0, 255, 255, 0.03) 75%);
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
  border-radius: 4px;
}

@keyframes shimmer {
  0% { background-position: 200% 0; }
  100% { background-position: -200% 0; }
}

/* Show Notes Dialog */
.show-notes-container {
  background: linear-gradient(180deg, #0a0a0f 0%, #0d0d14 50%, #0a0a0f 100%);
}

.notes-header-bar {
  pointer-events: none;
}

.notes-header-bar > * {
  pointer-events: auto;
}

.notes-header {
  background: rgba(15, 15, 26, 0.8);
  border-bottom: 1px solid rgba(0, 255, 255, 0.15);
}

.show-notes-bg-text {
  position: absolute;
  top: 65%;
  left: 65%;
  transform: translate(-50%, -50%);
  font-size: 120px;
  font-weight: 900;
  color: rgba(0, 255, 255, 0.03);
  white-space: nowrap;
  pointer-events: none;
  z-index: 0;
  letter-spacing: 8px;
}

.notes-header-content {
  position: relative;
  z-index: 1;
}

.notes-cover {
  width: 120px;
  height: 120px;
}

.notes-cover-image {
  border-radius: 8px;
  box-shadow: 0 0 30px rgba(0, 255, 255, 0.2);
}

.notes-title {
  font-size: 20px;
  font-weight: 700;
  color: #00ff88;
  text-shadow: 0 0 10px rgba(0, 255, 136, 0.3);
  margin: 0 0 12px 0;
}

.notes-meta {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
  font-size: 11px;
  color: #a1a1aa;
}

.meta-podcast {
  color: #00ffff;
}

.meta-separator {
  color: #52525b;
}

.meta-date,
.meta-duration {
  color: #71717a;
}

.notes-body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}

.notes-content {
  max-width: 960px;
  margin: 0 auto;
  width: 100%;
}

/* File Input Styling */
.console-file-input {
  flex: 1;
}

.console-file-input :deep(.v-file-input__text) {
  color: #00ffff !important;
  font-family: 'JetBrains Mono', monospace !important;
  font-size: 12px !important;
}

.console-file-input :deep(.v-field__input) {
  padding: 0 !important;
}

.console-file-input :deep(.v-label) {
  color: #52525b !important;
  font-family: 'JetBrains Mono', monospace !important;
  font-size: 10px !important;
  letter-spacing: 1px;
}

/* Console scrollbar */
.console-scroll::-webkit-scrollbar {
  width: 6px;
}

.console-scroll::-webkit-scrollbar-track {
  background: transparent;
}

.console-scroll::-webkit-scrollbar-thumb {
  background: rgba(0, 255, 255, 0.2);
  border-radius: 3px;
}

.console-scroll::-webkit-scrollbar-thumb:hover {
  background: rgba(0, 255, 255, 0.3);
}

/* Spinning animation */
.spinning {
  animation: spin 1s linear infinite;
  display: inline-block;
}

/* =========================================
   Light Theme Styles (Tatooine Outpost)
   ========================================= */
.light-theme .podcast-manager {
  background: #f5e6d3;
}

.light-theme .scanlines {
  background: repeating-linear-gradient(
    0deg,
    rgba(107, 68, 35, 0.03),
    rgba(107, 68, 35, 0.03) 1px,
    transparent 1px,
    transparent 2px
  );
}

.light-theme .crt-vignette {
  background: radial-gradient(ellipse at center, transparent 0%, rgba(107, 68, 35, 0.1) 100%);
}

.light-theme .console-header-bar {
  background: linear-gradient(180deg, #efe0cc 0%, #e8d4bc 100%);
  border-bottom-color: rgba(184, 134, 11, 0.3);
}

.light-theme .title-text {
  color: #cd7f32;
  text-shadow: 0 0 8px rgba(205, 127, 50, 0.3);
}

.light-theme .title-prefix,
.light-theme .title-suffix {
  color: #8b7355;
}

.light-theme .metric-label {
  color: #6b4423;
}

.light-theme .metric-value {
  color: #cd7f32;
}

.light-theme .status-light.online {
  background: #daa520;
  box-shadow: 0 0 8px rgba(218, 165, 32, 0.4);
}

.light-theme .status-light.standby {
  background: #cd853f;
  box-shadow: 0 0 8px rgba(205, 133, 63, 0.4);
}

.light-theme .console-btn {
  border-color: #cd7f32;
  color: #cd7f32;
}

.light-theme .console-btn:hover {
  background: rgba(205, 127, 50, 0.12);
}

.light-theme .console-btn.primary {
  background: #cd7f32;
  color: #ffffff;
}

.light-theme .console-btn.icon-only {
  background: transparent;
}

.light-theme .grid-bg-layer {
  background-image:
    linear-gradient(rgba(184, 134, 11, 0.08) 1px, transparent 1px),
    linear-gradient(90deg, rgba(184, 134, 11, 0.08) 1px, transparent 1px);
}

.light-theme .podcast-card {
  background: linear-gradient(135deg, #efe0cc 0%, #e8d4bc 100%);
  border-color: rgba(184, 134, 11, 0.3);
}

.light-theme .card-glow {
  background: radial-gradient(ellipse at center, rgba(184, 134, 11, 0.15) 0%, transparent 70%);
}

.light-theme .card-title {
  color: #3d2914;
}

.light-theme .card-meta {
  color: #6b4423;
}

.light-theme .empty-state {
  background: #f5e6d3;
}

.light-theme .empty-icon {
  color: #cd7f32;
}

.light-theme .empty-text {
  color: #6b4423;
}

.light-theme .podcast-item:hover .podcast-card {
  border-color: rgba(205, 127, 50, 0.5);
}

.light-theme .playing-indicator {
  background: rgba(218, 165, 32, 0.9);
}

.light-theme .playing-bar {
  background: #ffffff;
}

.light-theme .console-scroll::-webkit-scrollbar-thumb {
  background: rgba(184, 134, 11, 0.3);
}

.light-theme .console-scroll::-webkit-scrollbar-thumb:hover {
  background: rgba(184, 134, 11, 0.5);
}

/* Detail View Light Theme */
.light-theme .podcast-header {
  background: rgba(239, 224, 204, 0.8);
  border-bottom-color: rgba(184, 134, 11, 0.3);
}

.light-theme .cover-image {
  box-shadow: 0 0 20px rgba(184, 134, 11, 0.2);
}

.light-theme .podcast-title {
  color: #cd7f32;
  text-shadow: 0 0 10px rgba(205, 127, 50, 0.2);
}

.light-theme .podcast-author {
  color: #b8860b;
  text-shadow: 0 0 8px rgba(184, 134, 11, 0.2);
}

.light-theme .tag-chip {
  border-color: rgba(184, 134, 11, 0.3);
  color: #6b4423;
}

.light-theme .stats-panel {
  border-left-color: rgba(184, 134, 11, 0.2);
}

.light-theme .episodes-count {
  color: #cd7f32;
  text-shadow: 0 0 15px rgba(205, 127, 50, 0.3);
}

.light-theme .episodes-label {
  color: #8b7355;
}

.light-theme .podcast-description {
  border-top-color: rgba(184, 134, 11, 0.2);
}

.light-theme .playing-bar {
  background: #daa520;
  box-shadow: 0 0 8px rgba(218, 165, 32, 0.5);
}

.light-theme .input-wrapper {
  background: #faf3e8;
  border-color: #d4a574;
}

.light-theme .input-prompt {
  color: #cd7f32;
  text-shadow: 0 0 8px rgba(205, 127, 50, 0.3);
}

.light-theme .console-input {
  color: #3d2914;
}

.light-theme .console-input::placeholder {
  color: #8b7355;
}

.light-theme .placeholder-icon {
  opacity: 0.6;
}

/* Episode List Light Theme */
.light-theme .episodes-header {
  color: #6b4423;
}

.light-theme .episode-item:hover {
  background: rgba(205, 127, 50, 0.08);
}

.light-theme .episode-item.active {
  background: rgba(205, 127, 50, 0.12);
}

.light-theme .episode-item::before {
  background: #cd7f32;
  box-shadow: 0 0 10px rgba(205, 127, 50, 0.6);
}

.light-theme .episode-title {
  color: #3d2914;
}

.light-theme .episode-item.active .episode-title {
  color: #cd7f32;
  text-shadow: 0 0 8px rgba(205, 127, 50, 0.3);
}

.light-theme .episode-desc {
  color: #8b7355;
}

.light-theme .meta-text {
  color: #8b7355;
}

.light-theme .meta-tag {
  border-color: rgba(184, 134, 11, 0.3);
  color: #6b4423;
}

.light-theme .play-icon {
  color: #daa520;
  text-shadow: 0 0 8px rgba(218, 165, 32, 0.4);
}

.light-theme .date-thumb {
  background: rgba(245, 230, 211, 0.9);
  border-color: rgba(184, 134, 11, 0.3);
}

.light-theme .date-month {
  color: #8b7355;
}

.light-theme .date-day {
  color: #cd7f32;
}

.light-theme .episode-overlay {
  background: rgba(107, 68, 35, 0.5);
}

.light-theme .loading-card {
  background: rgba(245, 230, 211, 0.5);
  border-color: rgba(184, 134, 11, 0.1);
}

.light-theme .loading-image {
  background: linear-gradient(90deg, rgba(184, 134, 11, 0.05) 25%, rgba(184, 134, 11, 0.12) 50%, rgba(184, 134, 11, 0.05) 75%);
}

.light-theme .loading-text {
  background: linear-gradient(90deg, rgba(184, 134, 11, 0.05) 25%, rgba(184, 134, 11, 0.12) 50%, rgba(184, 134, 11, 0.05) 75%);
}

.light-theme .loading-spinner {
  border-color: rgba(184, 134, 11, 0.2);
  border-top-color: #cd7f32;
}

/* Show Notes Light Theme */
.light-theme .show-notes-container {
  background: linear-gradient(180deg, #f5e6d3 0%, #efe0cc 50%, #f5e6d3 100%);
}

.light-theme .notes-header {
  background: rgba(239, 224, 204, 0.9);
  border-bottom-color: rgba(184, 134, 11, 0.3);
}

.light-theme .show-notes-bg-text {
  color: rgba(184, 134, 11, 0.06);
}

.light-theme .notes-cover-image {
  box-shadow: 0 0 30px rgba(184, 134, 11, 0.2);
}

.light-theme .notes-title {
  color: #cd7f32;
  text-shadow: 0 0 10px rgba(205, 127, 50, 0.2);
}

.light-theme .notes-meta {
  color: #6b4423;
}

.light-theme .meta-podcast {
  color: #b8860b;
}

.light-theme .meta-separator {
  color: #d4a574;
}

.light-theme .meta-date,
.light-theme .meta-duration {
  color: #8b7355;
}

.light-theme .console-file-input :deep(.v-file-input__text) {
  color: #cd7f32 !important;
}

.light-theme .console-file-input :deep(.v-label) {
  color: #8b7355 !important;
}
</style>
