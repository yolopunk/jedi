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
        <div v-else-if="currentView === 'detail' && currentSub" class="fill-height">
            <!-- Podcast Header -->
            <div class="podcast-header pa-4 pa-md-8 d-flex align-start">
                <div class="d-flex flex-column mr-4 mr-md-8 flex-shrink-0" :style="$vuetify.display.smAndDown ? 'width: 100px' : ''">
                    <v-img 
                        :src="currentSub.image_url" 
                        :width="$vuetify.display.smAndDown ? 100 : 240" 
                        :max-width="$vuetify.display.smAndDown ? 100 : 240" 
                        aspect-ratio="1" 
                        cover 
                        class="rounded-lg elevation-6 mb-2"
                    >
                        <template v-slot:placeholder>
                            <div class="d-flex align-center justify-center fill-height bg-grey-lighten-2">
                                <v-icon :icon="mdiPodcast" size="64" color="grey"></v-icon>
                            </div>
                        </template>
                        <template v-slot:error>
                            <div class="d-flex align-center justify-center fill-height bg-grey-lighten-2">
                                <v-icon :icon="mdiPodcast" size="64" color="grey"></v-icon>
                            </div>
                        </template>
                    </v-img>

                    <!-- Episodes Count for Small Screen -->
                    <div v-if="$vuetify.display.smAndDown && episodes.length > 0" class="text-caption text-center text-grey-darken-1 mt-1 font-weight-medium">
                        {{ $t('podcast.episodesCount', { count: episodes.length }) }}
                    </div>
                </div>
                
                <div class="flex-grow-1 pt-1 pt-md-2" style="min-width: 0">
                    <h1 :class="$vuetify.display.smAndDown ? 'text-h6' : 'text-h3'" class="font-weight-bold mb-1 mb-md-2 text-truncate">{{ currentSub.title }}</h1>
                    <div :class="$vuetify.display.smAndDown ? 'text-subtitle-2' : 'text-h6'" class="text-primary mb-2 mb-md-4 text-truncate">{{ currentSub.author || currentSub.owner_name }}</div>
                    
                    <div class="d-flex align-center mb-3 mb-md-6 flex-wrap gap-2">
                        <v-btn 
                            color="primary" 
                            class="mr-2 mr-md-4 mb-2 mb-md-0" 
                            rounded="pill" 
                            :prepend-icon="mdiPlay" 
                            :size="$vuetify.display.smAndDown ? 'small' : 'large'"
                            @click="playLatest"
                            :disabled="episodes.length === 0"
                        >
                            {{ $t('podcast.playLatest') }}
                        </v-btn>
                        <v-btn 
                            v-if="currentPlaying && currentPlayingSubUrl === currentSub.rss_url"
                            variant="tonal" 
                            color="primary" 
                            rounded="pill" 
                            class="mr-2 mr-md-4 mb-2 mb-md-0" 
                            @click="scrollToPlaying"
                            :prepend-icon="mdiTarget"
                            :size="$vuetify.display.smAndDown ? 'small' : 'default'"
                        >
                            {{ $t('podcast.locatePlaying') }}
                        </v-btn>
                         <v-btn 
                            variant="outlined" 
                            color="grey-darken-1" 
                            rounded="pill" 
                            class="mr-2 mb-2 mb-md-0" 
                            @click="refreshSub(currentSub)" 
                            :loading="refreshLoading"
                            :size="$vuetify.display.smAndDown ? 'small' : 'default'"
                         >
                            {{ $t('common.refresh') }}
                        </v-btn>
                        <v-btn 
                            icon 
                            variant="text" 
                            color="error" 
                            @click="unsubscribe(currentSub)"
                            :size="$vuetify.display.smAndDown ? 'small' : 'default'"
                            class="mb-2 mb-md-0"
                        >
                            <v-icon :icon="mdiDelete"></v-icon>
                        </v-btn>
                    </div>

                    <!-- Categories for Small Screen (Moved here for better layout) -->
                    <div class="d-flex flex-wrap align-center mb-2 gap-2" v-if="$vuetify.display.smAndDown">
                        <v-chip 
                            v-for="cat in currentSub.categories" 
                            :key="cat"
                            size="x-small" 
                            variant="tonal" 
                            class="mr-1 mb-1 text-capitalize"
                            color="secondary"
                        >
                            {{ cat }}
                        </v-chip>
                        <v-chip 
                            size="x-small" 
                            variant="tonal" 
                            class="mr-1 mb-1 text-capitalize" 
                            color="secondary"
                            v-if="currentSub.podcast_type"
                        >
                            {{ currentSub.podcast_type }}
                        </v-chip>
                    </div>

                    <div class="d-flex align-center mb-2 mb-md-4 flex-wrap" v-if="!$vuetify.display.smAndDown">
                        <v-chip 
                            v-for="cat in currentSub.categories" 
                            :key="cat"
                            size="small" 
                            variant="outlined" 
                            class="mr-2 mb-2 text-capitalize"
                        >
                            {{ cat }}
                        </v-chip>
                        <v-chip 
                            size="small" 
                            variant="outlined" 
                            class="mr-2 mb-2 text-capitalize" 
                            v-if="currentSub.podcast_type"
                        >
                            {{ currentSub.podcast_type }}
                        </v-chip>
                        <v-chip 
                            size="small" 
                            variant="text" 
                            class="text-grey-darken-1 mb-2" 
                            v-if="episodes.length > 0"
                        >
                            {{ $t('podcast.episodesCount', { count: episodes.length }) }}
                        </v-chip>
                    </div>

                    <div class="text-body-2 text-md-body-1 text-grey-darken-1 description-clamp">
                        {{ currentSub.description }}
                    </div>
                </div>
            </div>

            <v-divider></v-divider>

            <!-- Episodes List -->
            <div class="pa-4">
                 <div class="text-h5 font-weight-bold px-4 mb-2">{{ $t('podcast.episodes') }}</div>
                 
                 <div v-if="episodesLoading" class="d-flex justify-center pa-8">
                    <v-progress-circular indeterminate color="primary" size="64"></v-progress-circular>
                 </div>

                 <v-list v-else lines="three" bg-color="transparent" class="pa-0">
                    <div 
                        v-for="ep in episodes" 
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
                            <div class="episode-overlay position-absolute top-0 left-0 w-100 h-100 rounded d-flex align-center justify-center bg-black-50 opacity-0 transition-opacity">
                                <div class="d-flex align-center justify-center expand-arrows position-relative w-100 h-100">
                                    <v-icon :icon="mdiArrowBottomLeft" color="white" size="40" class="arrow-bl position-absolute" style="bottom: 0; left: 0;"></v-icon>
                                    <v-icon :icon="mdiArrowTopRight" color="white" size="40" class="arrow-tr position-absolute" style="top: 0; right: 0;"></v-icon>
                                </div>
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

    <v-navigation-drawer
        v-model="showShowNotesDialog"
        location="right"
        width="900"
        temporary
        class="elevation-10 show-notes-drawer"
        :style="{ 'max-width': $vuetify.display.smAndDown ? '100vw' : '95vw', 'z-index': 9999 }"
    >
        <div v-if="currentShowNotesEpisode" class="fill-height position-relative">
            <!-- Blurred Background -->
            <div 
                class="absolute-background"
                :style="`background-image: url(${currentShowNotesEpisode.image_url || currentSub?.image_url});`"
            ></div>
            <div class="glass-overlay"></div>

            <!-- Content Container -->
            <div class="position-relative d-flex flex-column fill-height w-100" style="z-index: 2;">
                
                <!-- Close Button (Fixed) -->
                <div class="position-absolute top-0 right-0 ma-6" style="z-index: 10;">
                    <v-btn icon variant="tonal" color="white" class="glass-btn" @click="showShowNotesDialog = false">
                        <v-icon :icon="mdiClose"></v-icon>
                    </v-btn>
                </div>

                <!-- Scrollable Area -->
                <div class="flex-grow-1 overflow-y-auto">
                    <!-- Hero Header -->
                    <div class="d-flex align-end pa-6 pa-md-8 pt-16 pb-6 pb-md-8 hero-header" :class="{'flex-column align-start': $vuetify.display.smAndDown}">
                        <!-- Poster -->
                        <div class="poster-container elevation-10 mr-0 mr-md-8 mb-4 mb-md-0 flex-shrink-0" :style="$vuetify.display.smAndDown ? 'width: 140px; height: 140px;' : 'width: 200px; height: 200px;'">
                            <v-img
                                :src="currentShowNotesEpisode.image_url || currentSub?.image_url"
                                width="100%"
                                height="100%"
                                cover
                                class="rounded-lg"
                            ></v-img>
                        </div>

                        <!-- Info -->
                        <div class="flex-grow-1 pb-2">
                            <h2 :class="$vuetify.display.smAndDown ? 'text-h5' : 'text-h4'" class="font-weight-bold text-white mb-2 mb-md-4 text-shadow-lg leading-tight" style="line-height: 1.2;">
                                {{ currentShowNotesEpisode.title }}
                            </h2>
                            
                            <div class="d-flex align-center text-white-70 mb-4 mb-md-6 flex-wrap">
                                <v-icon :icon="mdiPodcast" size="small" class="mr-2"></v-icon>
                                <span class="font-weight-bold mr-4 text-truncate" style="max-width: 200px;">{{ currentShowNotesEpisode.podcast_name || currentSub?.title }}</span>
                                
                                <span class="mr-4 hidden-sm-and-down">•</span>
                                <span class="d-none d-md-inline">{{ formatMonth(currentShowNotesEpisode.pub_date) }} {{ formatDay(currentShowNotesEpisode.pub_date) }}</span>
                                <span class="mx-2 d-none d-md-inline">•</span>
                                <span class="d-none d-md-inline">{{ currentShowNotesEpisode.duration }}</span>

                                <!-- Mobile Meta -->
                                <div v-if="$vuetify.display.smAndDown" class="w-100 mt-2 text-caption opacity-80">
                                    <span>{{ formatMonth(currentShowNotesEpisode.pub_date) }} {{ formatDay(currentShowNotesEpisode.pub_date) }}</span>
                                    <span class="mx-2">•</span>
                                    <span>{{ currentShowNotesEpisode.duration }}</span>
                                </div>
                            </div>

                            <v-btn 
                                color="white" 
                                variant="flat"
                                rounded="pill" 
                                size="large"
                                class="font-weight-bold text-primary px-8"
                                :prepend-icon="mdiPlay"
                                @click="playEpisode(currentShowNotesEpisode)"
                            >
                                {{ $t('podcast.play') }}
                            </v-btn>
                        </div>
                    </div>

                    <!-- Show Notes Content -->
                    <div class="pa-4 pa-md-8 pt-0 pt-md-4">
                        <div class="show-notes-container pa-4 pa-md-8 rounded-xl" style="background: rgba(30,30,30,0.6); backdrop-filter: blur(20px);">
                            <EpisodeShowNotes 
                                :content="currentShowNotesEpisode.show_notes"
                                @seek="seekToTime"
                            />
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </v-navigation-drawer>

    <v-snackbar v-model="snackbar" :color="snackbarColor" rounded="pill">
      {{ snackbarText }}
    </v-snackbar>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import {
    mdiPodcast, mdiPlus, mdiDelete, mdiPlay, mdiClose,
    mdiChevronLeft, mdiMagnify, mdiFileXmlBox, mdiRefresh, mdiClockTimeFourOutline,
    mdiTarget, mdiArrowTopRight, mdiArrowBottomLeft
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

// State
const subscriptions = ref<PodcastSubscription[]>([]);
const currentSub = ref<PodcastSubscription | null>(null);
const episodes = ref<PodcastEpisode[]>([]);
const episodesLoading = ref(false);

const singleEpisode = ref<PodcastEpisode | null>(null);

import { useAudioPlayer } from '@/composables/useAudioPlayer';

const { currentPlaying, currentPlayingSubUrl, isPaused, playEpisode: globalPlayEpisode, audioRef } = useAudioPlayer();

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
    
    try {
        episodes.value = await fetchEpisodes(sub.rss_url);
    } catch (e: any) {
        showMsg('Failed to load episodes: ' + e, 'error');
    } finally {
        episodesLoading.value = false;
    }
}

function goBack() {
    currentView.value = 'library';
    currentSub.value = null;
}

function playLatest() {
    if (episodes.value.length > 0) {
        playEpisode(episodes.value[0]);
    }
}

function seekToTime(seconds: number) {
    if (!currentShowNotesEpisode.value) return;

    // If currently playing this episode, just seek
    if (currentPlaying.value?.audio_url === currentShowNotesEpisode.value.audio_url) {
        if (audioRef.value) {
            audioRef.value.currentTime = seconds;
            if (isPaused.value) {
                 audioRef.value.play().catch(e => console.error(e));
            }
        }
    } else {
        // If not playing, play it first
        playEpisode(currentShowNotesEpisode.value);
        // Attempt to seek after a short delay (once metadata likely loaded)
        setTimeout(() => {
            if (audioRef.value) {
                audioRef.value.currentTime = seconds;
            }
        }, 500);
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
.episode-item:hover .episode-overlay {
    opacity: 1 !important;
}

.expand-arrows .arrow-bl,
.expand-arrows .arrow-tr {
    opacity: 0.9;
}

.episode-item:hover .expand-arrows .arrow-bl {
    animation: expand-bl 1.5s infinite ease-in-out;
}

.episode-item:hover .expand-arrows .arrow-tr {
    animation: expand-tr 1.5s infinite ease-in-out;
}

@keyframes expand-bl {
    0%, 100% { transform: translate(6px, -6px); }
    50% { transform: translate(-2px, 2px); }
}

@keyframes expand-tr {
    0%, 100% { transform: translate(-6px, 6px); }
    50% { transform: translate(2px, -2px); }
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
.show-notes-drawer {
    background: transparent !important;
    border: none !important;
    box-shadow: none !important;
}

.absolute-background {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-size: cover;
    background-position: center;
    filter: blur(20px) saturate(180%);
    opacity: 0.6;
    z-index: 0;
}

.glass-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: linear-gradient(to bottom, rgba(0,0,0,0.3), rgba(var(--v-theme-surface), 0.95));
    z-index: 1;
    backdrop-filter: blur(10px);
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
.show-notes-drawer .show-notes-content {
    color: rgba(255, 255, 255, 0.9) !important;
}

.show-notes-drawer .show-notes-content a {
    color: #90caf9 !important; /* Light Blue 200 for better contrast on dark */
}

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

</style>
