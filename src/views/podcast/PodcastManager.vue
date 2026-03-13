<template>
  <div class="podcast-manager fill-height d-flex bg-app">
    <!-- Main Content -->
    <div class="flex-grow-1 d-flex flex-column h-100 overflow-hidden bg-app position-relative">
      
      <!-- Top Bar / Header -->
      <div class="px-6 py-4 border-b d-flex align-center">
         <template v-if="currentView === 'detail'">
            <v-btn icon variant="text" size="small" @click="goBack" class="mr-2">
                <v-icon :icon="mdiChevronLeft" size="large"></v-icon>
            </v-btn>
            <span class="text-h6 font-weight-bold">{{ $t('common.back') }}</span>
         </template>
         <template v-else>
            <span class="text-h5 font-weight-bold">{{ $t('podcast.library') }}</span>
            <v-spacer></v-spacer>
            <v-btn 
                icon 
                variant="text" 
                class="mr-2" 
                @click="refreshLibrary" 
                :loading="libraryRefreshLoading"
            >
                <v-icon :icon="mdiRefresh"></v-icon>
                <v-tooltip activator="parent" location="bottom">{{ $t('common.refresh') }}</v-tooltip>
            </v-btn>
            <v-btn 
                color="primary" 
                variant="flat" 
                :prepend-icon="mdiPlus" 
                @click="openAddDialog"
                class="mr-4"
                rounded="pill"
            >
                {{ $t('podcast.addShow') }}
            </v-btn>
            <v-text-field
                density="compact"
                variant="outlined"
                :label="$t('common.search')"
                :prepend-inner-icon="mdiMagnify"
                hide-details
                style="max-width: 300px"
                rounded="lg"
            ></v-text-field>
         </template>
      </div>

      <!-- Content Area -->
      <div class="flex-grow-1 overflow-y-auto scroll-smooth" :class="{ 'pb-16': currentPlaying, 'mb-16': currentPlaying }">
        
        <!-- Grid View: Library -->
        <div v-if="currentView === 'library'" class="pa-6">
            <v-row>
                <v-col 
                    v-for="sub in subscriptions" 
                    :key="sub.rss_url"
                    cols="6" md="4" lg="3" xl="2"
                >
                    <v-card 
                        class="podcast-card bg-transparent" 
                        elevation="0" 
                        @click="selectSubscription(sub)"
                        v-ripple
                    >
                        <v-img 
                        :src="sub.image_url" 
                        aspect-ratio="1" 
                        cover 
                        class="rounded-lg mb-3 elevation-2 transition-swing position-relative"
                    >
                        <!-- Playing Indicator -->
                        <div 
                            v-if="currentPlaying && currentPlayingSubUrl === sub.rss_url"
                            class="position-absolute top-0 right-0 ma-2 bg-black-50 rounded-circle pa-1 d-flex align-center justify-center"
                            style="width: 24px; height: 24px;"
                        >
                            <div class="playing-bar bar-1 bg-primary" style="height: 12px; width: 3px;"></div>
                            <div class="playing-bar bar-2 bg-primary mx-1" style="height: 12px; width: 3px;"></div>
                            <div class="playing-bar bar-3 bg-primary" style="height: 12px; width: 3px;"></div>
                        </div>

                        <template v-slot:placeholder>
                            <div class="d-flex align-center justify-center fill-height bg-grey-lighten-2">
                                <v-icon :icon="mdiPodcast" size="large" color="grey"></v-icon>
                            </div>
                        </template>
                        <template v-slot:error>
                            <div class="d-flex align-center justify-center fill-height bg-grey-lighten-2">
                                <v-icon :icon="mdiPodcast" size="large" color="grey"></v-icon>
                            </div>
                        </template>
                    </v-img>
                        <div class="text-subtitle-1 font-weight-bold text-truncate">{{ sub.title }}</div>
                        <div class="text-caption text-grey text-truncate">{{ sub.author || sub.owner_name || sub.description }}</div>
                    </v-card>
                </v-col>
                
                <!-- Add New Placeholder -->
                <!-- Removed as per request -->
            </v-row>
        </div>

        <!-- Detail View: Podcast Info & Episodes -->
        <div v-else-if="currentView === 'detail' && currentSub" class="fill-height d-flex flex-column overflow-hidden">
            <!-- Podcast Header (Fixed) -->
            <div class="podcast-header position-relative pa-6 flex-shrink-0 bg-surface border-b overflow-hidden">
                <!-- Blurred Background -->
                <div 
                    class="position-absolute top-0 left-0 w-100 h-100" 
                    :style="`background-image: url(${currentSub.image_url}); background-size: cover; background-position: center; filter: blur(60px) saturate(180%); opacity: 0.1; z-index: 0; transform: scale(1.2);`"
                ></div>

                <div class="d-flex flex-column flex-md-row align-start position-relative z-index-1">
                     <v-img 
                        :src="currentSub.image_url" 
                        width="140" 
                        max-width="140" 
                        aspect-ratio="1" 
                        cover 
                        class="rounded-lg elevation-4 mr-md-6 mb-4 mb-md-0 flex-shrink-0"
                    >
                        <template v-slot:placeholder>
                            <div class="d-flex align-center justify-center fill-height bg-grey-lighten-2">
                                <v-icon :icon="mdiPodcast" size="48" color="grey"></v-icon>
                            </div>
                        </template>
                        <template v-slot:error>
                            <div class="d-flex align-center justify-center fill-height bg-grey-lighten-2">
                                <v-icon :icon="mdiPodcast" size="48" color="grey"></v-icon>
                            </div>
                        </template>
                    </v-img>
                    
                    <div class="flex-grow-1 pt-1 overflow-hidden mr-4">
                        <h1 class="text-h5 font-weight-bold mb-1 text-truncate">{{ currentSub.title }}</h1>
                        <div class="text-subtitle-2 text-primary mb-3 text-truncate">{{ currentSub.author || currentSub.owner_name }}</div>
                        
                        <div class="d-flex align-center mb-3">
                            <v-btn color="primary" class="mr-3 px-4" rounded="pill" :prepend-icon="mdiPlay" size="small" @click="playLatestEpisode">
                                {{ $t('podcast.playLatest') }}
                            </v-btn>
                            <v-btn 
                                v-if="currentPlaying && currentPlayingSubUrl === currentSub.rss_url"
                                variant="tonal" 
                                color="primary" 
                                rounded="pill" 
                                class="mr-3" 
                                size="small"
                                @click="scrollToPlaying"
                                :prepend-icon="mdiTarget"
                            >
                                {{ $t('podcast.locatePlaying') }}
                            </v-btn>
                        </div>

                        <div class="d-flex align-center mb-2 flex-wrap">
                            <v-chip 
                                v-for="cat in currentSub.categories.slice(0, 3)" 
                                :key="cat"
                                size="x-small" 
                                variant="outlined" 
                                class="mr-2 mb-1 text-capitalize"
                            >
                                {{ cat }}
                            </v-chip>
                        </div>
                    </div>

                    <!-- Right Side Stats & Actions -->
                    <div class="d-flex flex-column align-end justify-space-between pl-6 ml-auto" style="min-width: 140px; border-left: 1px solid rgba(0,0,0,0.05); height: 140px;">
                        <div class="d-flex align-center">
                            <v-tooltip :text="$t('common.refresh')" location="top">
                                <template v-slot:activator="{ props }">
                                    <v-btn 
                                        v-bind="props" 
                                        icon 
                                        variant="text" 
                                        size="small" 
                                        color="grey-darken-1" 
                                        class="mr-1" 
                                        @click="refreshSub(currentSub)" 
                                        :loading="refreshLoading"
                                    >
                                        <v-icon :icon="mdiRefresh"></v-icon>
                                    </v-btn>
                                </template>
                            </v-tooltip>
                            <v-tooltip :text="$t('common.delete')" location="top">
                                <template v-slot:activator="{ props }">
                                    <v-btn 
                                        v-bind="props" 
                                        icon 
                                        variant="text" 
                                        size="small" 
                                        color="error" 
                                        @click="unsubscribe(currentSub)"
                                    >
                                        <v-icon :icon="mdiDelete"></v-icon>
                                    </v-btn>
                                </template>
                            </v-tooltip>
                        </div>

                        <div class="text-right mt-auto pb-1">
                            <div class="text-h3 font-weight-black text-primary" style="line-height: 1;">
                                {{ episodes.length }}
                            </div>
                            <div class="text-overline font-weight-bold text-grey-darken-1 mt-1" style="letter-spacing: 2px !important; line-height: 1;">
                                EPISODES
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Full Width Description -->
                <div class="mt-4 position-relative z-index-1">
                     <div class="text-body-2 text-grey-darken-1" style="line-height: 1.6;">
                         {{ currentSub.description }}
                     </div>
                </div>
            </div>

            <!-- Episodes List (Scrollable) -->
            <div class="flex-grow-1 overflow-y-auto scroll-smooth pa-4">
                 <div class="text-h5 font-weight-bold px-4 mb-2">{{ $t('podcast.episodes') }}</div>
                 
                 <div v-if="episodesLoading" class="d-flex justify-center pa-8">
                    <v-progress-circular indeterminate color="primary" size="64"></v-progress-circular>
                 </div>

                 <v-list v-else lines="three" bg-color="transparent" class="pa-0">
                    <div 
                        v-for="ep in visibleEpisodes" 
                        :key="ep.audio_url" 
                        :id="getEpisodeId(ep.audio_url)"
                        class="d-flex align-center py-3 px-4 rounded-lg mb-1 episode-item group position-relative"
                        :class="{ 'bg-primary-lighten-5': currentPlaying?.audio_url === ep.audio_url }"
                        @click="playEpisode(ep)"
                        v-ripple
                    >
                        <!-- Image with Hover Overlay -->
                        <div class="mr-4 position-relative flex-shrink-0" style="width: 80px; height: 80px;" @click.stop="openShowNotes(ep)">
                            <v-img 
                                v-if="ep.image_url || currentSub?.image_url" 
                                :src="ep.image_url || currentSub?.image_url" 
                                cover 
                                class="rounded bg-grey-lighten-2 fill-height elevation-1"
                            >
                                <template v-slot:error>
                                    <div class="d-flex flex-column align-center justify-center fill-height bg-grey-lighten-3">
                                        <span class="text-caption font-weight-bold text-grey">{{ formatMonth(ep.pub_date) }}</span>
                                        <span class="text-h6 font-weight-bold">{{ formatDay(ep.pub_date) }}</span>
                                    </div>
                                </template>
                            </v-img>
                            <div v-else class="d-flex flex-column align-center justify-center fill-height bg-grey-lighten-3 rounded">
                                <span class="text-caption font-weight-bold text-grey">{{ formatMonth(ep.pub_date) }}</span>
                                <span class="text-h6 font-weight-bold">{{ formatDay(ep.pub_date) }}</span>
                            </div>

                            <!-- Hover Overlay for Show Notes -->
                            <div class="episode-overlay position-absolute top-0 left-0 w-100 h-100 rounded d-flex align-center justify-center">
                                <div class="overlay-bg position-absolute w-100 h-100 bg-black-50"></div>
                                <v-btn
                                    icon
                                    variant="flat"
                                    color="white"
                                    size="x-small"
                                    class="zoom-icon elevation-2"
                                    density="comfortable"
                                >
                                    <v-icon :icon="mdiArrowExpandAll" color="black" size="small"></v-icon>
                                </v-btn>
                            </div>
                        </div>

                        <!-- Content (Click to Play) -->
                        <div class="flex-grow-1 overflow-hidden" @click="playEpisode(ep)">
                            <div class="text-subtitle-1 font-weight-bold mb-1 text-truncate">{{ ep.title }}</div>
                            <div class="text-body-2 text-grey-darken-1 mb-2 description-text text-truncate">
                                {{ ep.description.replace(/<[^>]*>/g, '').substring(0, 150) }}
                            </div>
                            
                            <div class="d-flex align-center mt-1">
                                <!-- Playing Indicator (Visual Only) -->
                                <div v-if="currentPlaying?.audio_url === ep.audio_url" class="mr-3 d-flex align-end justify-center" style="width: 24px; height: 24px;">
                                    <div class="playing-bar bar-1 bg-primary"></div>
                                    <div class="playing-bar bar-2 bg-primary mx-1"></div>
                                    <div class="playing-bar bar-3 bg-primary"></div>
                                </div>
                                <div v-else class="mr-3" style="width: 24px;"></div> <!-- Spacer -->
 
                                 <v-icon :icon="mdiClockTimeFourOutline" size="x-small" color="grey" class="mr-1"></v-icon>
                                 <span class="text-caption font-weight-bold text-grey-darken-2 mr-3">{{ ep.duration }}</span>
                                 <v-chip size="x-small" v-if="ep.episode_number" variant="outlined" class="text-caption mr-3">Ep {{ ep.episode_number }}</v-chip>
                             </div>
                        </div>
                    </div>
                 </v-list>
                 <div ref="episodesLoadMoreTrigger"></div>
            </div>
        </div>

      </div>

      </div>

    <!-- Add Subscription Dialog (Keep functionality) -->
    <v-dialog v-model="showAddDialog" max-width="500" persistent>
      <v-card class="rounded-xl">
        <v-card-title class="px-6 pt-6 pb-2 d-flex justify-space-between align-center">
            <span class="text-h6 font-weight-bold">{{ $t('podcast.importOpml') }}</span>
            <v-btn icon variant="text" @click="closeAddDialog" :disabled="addLoading">
                <v-icon :icon="mdiClose"></v-icon>
            </v-btn>
        </v-card-title>
        <v-card-text class="px-6 pb-6">
            <div class="text-body-2 text-grey mb-4">{{ $t('podcast.selectOpml') }}</div>
            <v-file-input
                v-model="opmlFile"
                :label="$t('podcast.opmlFile')"
                variant="outlined"
                density="comfortable"
                accept=".opml,.xml"
                :error-messages="opmlError"
                prepend-icon=""
                :prepend-inner-icon="mdiFileXmlBox"
                show-size
                rounded="lg"
            ></v-file-input>
            <OpmlExportGuide />
        </v-card-text>
        <v-card-actions class="px-6 pb-6">
            <v-spacer></v-spacer>
            <v-btn variant="text" @click="closeAddDialog" :disabled="addLoading" rounded="pill" class="px-4">{{ $t('common.cancel') }}</v-btn>
            <v-btn color="primary" variant="flat" @click="importOpmlFile" :loading="addLoading" rounded="pill" class="px-6">
                {{ $t('podcast.import') }}
            </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <v-dialog
      v-model="showShowNotesDialog"
      fullscreen
      persistent
      class="show-notes-dialog"
      transition="dialog-bottom-transition"
    >
      <div
        v-if="currentShowNotesEpisode"
        class="position-relative d-flex flex-column fill-height w-100 overflow-hidden"
      >
        <!-- Background Layer -->
        <div class="position-absolute top-0 left-0 w-100 h-100" style="z-index: 0;">
            <div
                class="w-100 h-100"
                :style="`background-image: url(${currentShowNotesEpisode.image_url || currentSub?.image_url}); background-size: cover; background-position: center; filter: blur(20px) saturate(180%); opacity: 0.6;`"
            ></div>
            <div
                class="position-absolute top-0 left-0 w-100 h-100"
                style="background: linear-gradient(to bottom, rgba(0,0,0,0.3), rgba(var(--v-theme-surface), 0.95)); backdrop-filter: blur(10px);"
            ></div>
        </div>

        <!-- Fixed Top Bar -->
        <div class="position-absolute top-0 right-0 w-100 d-flex justify-end pa-6 pointer-events-none" style="z-index: 20;">
            <v-btn
              icon
              variant="tonal"
              color="white"
              class="glass-btn pointer-events-auto"
              @click="showShowNotesDialog = false"
            >
              <v-icon :icon="mdiClose"></v-icon>
            </v-btn>
        </div>

        <!-- Fixed Header Area -->
        <div class="flex-shrink-0 position-relative" style="z-index: 10;">
          <div class="d-flex align-end px-6 pt-12 pb-4 hero-header show-notes-inner">
              <div
                class="poster-container elevation-10 mr-6 flex-shrink-0"
                style="width: 120px; height: 120px;"
              >
                <v-img
                  :src="currentShowNotesEpisode.image_url || currentSub?.image_url"
                  width="100%"
                  height="100%"
                  cover
                  class="rounded-lg"
                ></v-img>
              </div>

              <div class="flex-grow-1 pb-1 text-white">
                <h2
                  class="text-h5 font-weight-bold mb-2 text-shadow-lg"
                  style="line-height: 1.2;"
                >
                  {{ currentShowNotesEpisode.title }}
                </h2>

                <div class="d-flex align-center flex-wrap text-white-70 mb-2 text-body-2">
                  <v-icon :icon="mdiPodcast" size="small" class="mr-2"></v-icon>
                  <span class="font-weight-bold mr-3">
                    {{ currentShowNotesEpisode.podcast_name || currentSub?.title }}
                  </span>
                  <span class="mr-3 opacity-60">|</span>
                  <span class="mr-3">
                    {{ formatMonth(currentShowNotesEpisode.pub_date) }}
                    {{ formatDay(currentShowNotesEpisode.pub_date) }}
                  </span>
                  <span class="mr-3 opacity-60">|</span>
                  <span class="mr-3">{{ currentShowNotesEpisode.duration }}</span>
                  <span
                    v-if="currentShowNotesEpisode.chapters && currentShowNotesEpisode.chapters.length"
                    class="mr-3 opacity-60"
                  >|</span>
                  <span
                    v-if="currentShowNotesEpisode.chapters && currentShowNotesEpisode.chapters.length"
                  >
                    {{ $t('podcast.chapters') }}{{ currentShowNotesEpisode.chapters.length }}
                  </span>
                </div>

                <div class="d-flex align-center w-100">
                  <v-btn
                    variant="tonal"
                    color="white"
                    rounded="pill"
                    size="small"
                    class="mr-3 text-body-2 font-weight-medium px-4"
                    @click="showShowNotesDialog = false"
                  >
                    <v-icon :icon="mdiChevronLeft" size="small" class="mr-1"></v-icon>
                    {{ $t('common.back') }}
                  </v-btn>
                  <v-btn
                    color="white"
                    variant="flat"
                    rounded="pill"
                    size="small"
                    class="font-weight-bold text-primary px-6"
                    :prepend-icon="mdiPlay"
                    @click="playEpisode(currentShowNotesEpisode)"
                  >
                    {{ $t('podcast.play') }}
                  </v-btn>
                  
                  <v-spacer></v-spacer>

                  <!-- Bold Typographic Show Notes Label -->
                  <div class="position-relative d-flex align-center">
                    <span 
                        class="text-h1 font-weight-black text-uppercase position-absolute right-0 opacity-10 pointer-events-none text-no-wrap"
                        style="font-size: 8rem !important; letter-spacing: -4px !important; color: white; transform: translateY(-5px);"
                    >
                        Show Notes
                    </span>
                  </div>
                </div>
              </div>
          </div>
        </div>

        <!-- Scrollable Content -->
        <div class="d-flex flex-column flex-grow-1 w-100 overflow-hidden position-relative" style="z-index: 10;">
          <div class="show-notes-page h-100">
            <div class="px-6 pt-4 pb-10 show-notes-inner h-100 d-flex flex-column">
              <div
                class="show-notes-container rounded-xl overflow-hidden d-flex flex-column flex-grow-1"
                style="background: rgba(30,30,30,0.6); backdrop-filter: blur(20px); border: 1px solid rgba(255,255,255,0.08);"
              >
                <!-- Show Notes Content -->
                <div class="flex-grow-1 overflow-y-auto px-6 pt-6 pb-16 custom-scrollbar">
                    <EpisodeShowNotes
                      :content="currentShowNotesEpisode.show_notes"
                    />
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </v-dialog>

    <v-snackbar v-model="snackbar" :color="snackbarColor" rounded="pill">
      {{ snackbarText }}
    </v-snackbar>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, computed, nextTick } from 'vue';
import {
    mdiPodcast, mdiPlus, mdiDelete, mdiPlay, mdiClose,
    mdiChevronLeft, mdiMagnify, mdiFileXmlBox, mdiRefresh, mdiClockTimeFourOutline,
    mdiTarget, mdiArrowExpandAll
} from '@mdi/js';
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

// State
const subscriptions = ref<PodcastSubscription[]>([]);
const currentSub = ref<PodcastSubscription | null>(null);
const episodes = ref<PodcastEpisode[]>([]);
const episodesLoading = ref(false);

const singleEpisode = ref<PodcastEpisode | null>(null);

const EPISODES_PAGE_SIZE = 30;
const visibleEpisodeCount = ref(EPISODES_PAGE_SIZE);
const visibleEpisodes = computed(() =>
  episodes.value.slice(0, visibleEpisodeCount.value)
);
const episodesLoadMoreTrigger = ref<HTMLElement | null>(null);
let episodesObserver: IntersectionObserver | null = null;

import { useAudioPlayer } from '@/composables/useAudioPlayer';

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
const snackbarColor = ref('error');

onMounted(async () => {
    await loadSubscriptions();
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
    episodes.value = []; // Clear previous episodes
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
    // Inject podcast name if missing
    if (!ep.podcast_name && currentSub.value) {
        ep.podcast_name = currentSub.value.title;
    }
    // Inject image if missing
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
        // Add a highlight flash
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
        // Update in list
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

// ==================== Add Subscription Dialog ====================

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

// Import OPML file
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

function showMsg(text: string, color: string) {
    snackbarText.value = text;
    snackbarColor.value = color;
    snackbar.value = true;
}

// Helpers
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
.podcast-card {
    transition: transform 0.2s;
    cursor: pointer;
}
.podcast-card:hover {
    transform: scale(1.02);
}

.description-clamp {
    display: -webkit-box;
    -webkit-line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
}

.episode-item {
    transition: background-color 0.2s;
    cursor: pointer;
}
.episode-item:hover {
    background-color: rgba(0,0,0,0.03);
}
.episode-item:hover .play-btn {
    transform: scale(1.1);
}

/* Scrollbar Styling */
::-webkit-scrollbar {
    width: 8px;
}
::-webkit-scrollbar-track {
    background: transparent;
}
::-webkit-scrollbar-thumb {
    background: #e0e0e0;
    border-radius: 4px;
}
::-webkit-scrollbar-thumb:hover {
    background: #bdbdbd;
}

.playing-bar {
    width: 4px;
    border-radius: 2px;
    animation: equalize 1s infinite ease-in-out;
}

.bar-1 { animation-delay: 0s; }
.bar-2 { animation-delay: 0.2s; }
.bar-3 { animation-delay: 0.4s; }

@keyframes equalize {
    0% { height: 6px; }
    50% { height: 18px; }
    100% { height: 6px; }
}

/* New Interaction Styles */
.bg-black-50 {
    background-color: rgba(0, 0, 0, 0.5);
}

.transition-opacity {
    transition: opacity 0.2s ease-in-out;
}

.transition-colors {
    transition: color 0.2s ease-in-out;
}

.hover-text-primary:hover {
    color: rgb(var(--v-theme-primary)) !important;
}

/* Group Hover Logic */
.group:hover .group-hover-opacity-100 {
    opacity: 1 !important;
}

/* Episode Image Hover Effects */
.episode-overlay {
    opacity: 0;
    transition: opacity 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    pointer-events: none;
    z-index: 2;
}

.episode-item:hover .episode-overlay {
    opacity: 1 !important;
}

.overlay-bg {
    backdrop-filter: blur(2px);
    opacity: 0;
    transition: opacity 0.3s ease;
}

.episode-item:hover .overlay-bg {
    opacity: 1;
}

.zoom-icon {
    transform: scale(0.5) translateY(10px);
    opacity: 0;
    transition: all 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);
    z-index: 3;
}

.episode-item:hover .zoom-icon {
    transform: scale(2) translateY(0);
    opacity: 1;
}

/* Highlight Flash Animation */
.highlight-flash {
    animation: flash-highlight 2s ease-out;
}

@keyframes flash-highlight {
    0% { background-color: rgba(var(--v-theme-primary), 0.3); }
    100% { background-color: transparent; }
}

/* Show Notes Glassmorphism */

.pointer-events-none {
    pointer-events: none !important;
}
.pointer-events-auto {
    pointer-events: auto !important;
}

.text-shadow-lg {
    text-shadow: 0 4px 12px rgba(0,0,0,0.5);
}

.glass-btn {
    background: rgba(255,255,255,0.1) !important;
    backdrop-filter: blur(4px);
    border: 1px solid rgba(255,255,255,0.2);
}

.glass-btn:hover {
    background: rgba(255,255,255,0.2) !important;
}

.poster-container {
    transition: transform 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.poster-container:hover {
    transform: scale(1.05) translateY(-5px);
    box-shadow: 0 20px 40px rgba(0,0,0,0.4) !important;
}

/* Ensure play button in overlay scales nicely */
.play-btn-hover {
    transition: transform 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}
.play-btn-hover:hover {
    transform: scale(1.1);
}

/* Remove old hover styles that might conflict */
.episode-item:hover {
    background-color: rgba(0,0,0,0.03);
}

.bg-primary-lighten-5 {
    background-color: rgba(var(--v-theme-primary), 0.08) !important;
}

/* Ensure content readability in the dark glass drawer */
.highlight-flash {
    animation: flash-highlight 2s ease-out;
}

@keyframes flash-highlight {
    0% { background-color: rgba(var(--v-theme-primary), 0.3); }
    100% { background-color: rgba(var(--v-theme-primary), 0.08); }
}

.bg-gradient-to-t {
    background: linear-gradient(to top, rgba(0,0,0,0.8) 0%, rgba(0,0,0,0) 100%);
}

.text-shadow {
    text-shadow: 0 2px 4px rgba(0,0,0,0.5);
}
.spin-record {
    animation: spin 10s linear infinite;
}
.spin-record.paused {
    animation-play-state: paused;
}
@keyframes spin {
    100% { transform: rotate(360deg); }
}

.show-notes-inner {
    max-width: 960px;
    margin: 0 auto;
    width: 100%;
}



.show-notes-container {
    background-clip: padding-box;
}
</style>

<style>
/* Unscoped styles for dialog content to ensure scrolling works */
.show-notes-dialog .v-overlay__content {
    max-height: 100vh;
    overflow: hidden !important; /* Disable dialog scroll, handle internally */
    width: 100%;
    margin: 0;
    pointer-events: auto;
    display: flex; /* Flex layout */
    flex-direction: column;
}

.show-notes-dialog {
    background: transparent !important;
}

.text-white-70 {
    color: rgba(255, 255, 255, 0.7);
}

.text-white-90 {
    color: rgba(255, 255, 255, 0.9);
}

.letter-spacing-1 {
    letter-spacing: 1px;
}

.custom-scrollbar::-webkit-scrollbar {
    width: 6px;
}

.custom-scrollbar::-webkit-scrollbar-track {
    background: rgba(255, 255, 255, 0.02);
    margin: 8px 0;
    border-radius: 4px;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.15);
    border-radius: 4px;
    border: 1px solid rgba(255, 255, 255, 0.05);
    transition: background 0.2s ease;
}

.custom-scrollbar::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.3);
}
</style>
