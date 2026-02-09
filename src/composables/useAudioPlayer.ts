import { ref } from 'vue';
import type { PodcastEpisode } from '@/api/podcast';

// Global state
const currentPlaying = ref<PodcastEpisode | null>(null);
const currentPlayingSubUrl = ref<string | null>(null); // To track which subscription the episode belongs to
const isPaused = ref(false);
const currentTime = ref(0);
const duration = ref(0);
const audioRef = ref<HTMLAudioElement | null>(null);

// Actions
function playEpisode(ep: PodcastEpisode, subUrl?: string) {
    if (currentPlaying.value?.audio_url === ep.audio_url) {
        togglePlay();
        return;
    }

    currentPlaying.value = ep;
    if (subUrl) {
        currentPlayingSubUrl.value = subUrl;
    }
    isPaused.value = false;
    currentTime.value = 0;
    duration.value = 0; // Reset duration until metadata loads
}

function togglePlay() {
    if (!audioRef.value) return;
    
    if (audioRef.value.paused) {
        audioRef.value.play().catch(e => console.error("Play error:", e));
        isPaused.value = false;
    } else {
        audioRef.value.pause();
        isPaused.value = true;
    }
}

function stop() {
    if (audioRef.value) {
        audioRef.value.pause();
        audioRef.value.currentTime = 0;
    }
    currentPlaying.value = null;
    currentPlayingSubUrl.value = null;
    isPaused.value = false;
}

function seek(seconds: number) {
    if (audioRef.value) {
        audioRef.value.currentTime = Math.min(Math.max(audioRef.value.currentTime + seconds, 0), audioRef.value.duration);
    }
}

function setAudioRef(el: HTMLAudioElement) {
    audioRef.value = el;
    
    // Attach listeners
    el.addEventListener('pause', () => isPaused.value = true);
    el.addEventListener('play', () => isPaused.value = false);
    el.addEventListener('ended', () => isPaused.value = true);
    el.addEventListener('timeupdate', () => {
        currentTime.value = el.currentTime;
    });
    el.addEventListener('loadedmetadata', () => {
        duration.value = el.duration;
    });
}

export function useAudioPlayer() {
    return {
        currentPlaying,
        currentPlayingSubUrl,
        isPaused,
        currentTime,
        duration,
        audioRef,
        playEpisode,
        togglePlay,
        stop,
        seek,
        setAudioRef
    };
}
