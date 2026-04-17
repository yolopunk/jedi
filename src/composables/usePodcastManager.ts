import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import {
  fetchEpisodes,
  getSubscriptions,
  importOpml,
  type PodcastEpisode,
  type PodcastSubscription,
  refreshSubscription,
  removeSubscription,
} from '@/api/podcast'
import { useAudioPlayer } from '@/composables/useAudioPlayer'
import { useSnackbar } from '@/composables/useSnackbar'

export function usePodcastManager() {
  const subscriptions = ref<PodcastSubscription[]>([])
  const currentSub = ref<PodcastSubscription | null>(null)
  const episodes = ref<PodcastEpisode[]>([])
  const episodesLoading = ref(false)
  const loading = ref(false)
  const searchQuery = ref('')

  const EpisodesPageSize = 30
  const visibleEpisodeCount = ref(EpisodesPageSize)
  const visibleEpisodes = computed(() => episodes.value.slice(0, visibleEpisodeCount.value))
  const episodesLoadMoreTrigger = ref<HTMLElement | null>(null)
  let episodesObserver: IntersectionObserver | null = null

  const { currentPlaying, currentPlayingSubUrl, playEpisode: globalPlayEpisode } = useAudioPlayer()
  const { show: snackbar, text: snackbarText, color: snackbarColor, notify } = useSnackbar()

  const currentView = ref<'library' | 'detail'>('library')

  const showAddDialog = ref(false)
  const addLoading = ref(false)
  const refreshLoading = ref(false)
  const libraryRefreshLoading = ref(false)

  const showShowNotesDialog = ref(false)
  const currentShowNotesEpisode = ref<PodcastEpisode | null>(null)

  const opmlFile = ref<File | File[] | null>(null)
  const opmlError = ref('')

  async function loadSubscriptions() {
    try {
      subscriptions.value = await getSubscriptions()
    } catch (e) {
      console.error(e)
    }
  }

  async function selectSubscription(sub: PodcastSubscription) {
    currentSub.value = sub
    currentView.value = 'detail'
    episodesLoading.value = true
    episodes.value = []
    visibleEpisodeCount.value = EpisodesPageSize

    try {
      episodes.value = await fetchEpisodes(sub.rss_url)
      await nextTick()
      setupEpisodesObserver()
    } catch (e: unknown) {
      notify(`Failed to load episodes: ${e instanceof Error ? e.message : e}`, 'error')
    } finally {
      episodesLoading.value = false
    }
  }

  function goBack() {
    currentView.value = 'library'
    currentSub.value = null
    if (episodesObserver) {
      episodesObserver.disconnect()
      episodesObserver = null
    }
  }

  function playLatestEpisode() {
    if (episodes.value.length > 0) {
      playEpisode(episodes.value[0])
    }
  }

  function playEpisode(ep: PodcastEpisode) {
    if (!ep.podcast_name && currentSub.value) {
      ep.podcast_name = currentSub.value.title
    }
    if (!ep.image_url && currentSub.value) {
      ep.image_url = currentSub.value.image_url
    }
    globalPlayEpisode(ep, currentSub.value?.rss_url)
  }

  function getEpisodeId(url: string) {
    return `ep-${btoa(url).replace(/[^a-zA-Z0-9]/g, '')}`
  }

  function scrollToPlaying() {
    if (!currentPlaying.value) return
    const el = document.getElementById(getEpisodeId(currentPlaying.value.audio_url))
    if (el) {
      el.scrollIntoView({ behavior: 'smooth', block: 'center' })
      el.classList.add('highlight-flash')
      setTimeout(() => el.classList.remove('highlight-flash'), 2000)
    }
  }

  function openShowNotes(ep: PodcastEpisode) {
    currentShowNotesEpisode.value = ep
    showShowNotesDialog.value = true
  }

  async function refreshSub(sub: PodcastSubscription | null) {
    if (!sub) return
    refreshLoading.value = true
    try {
      const newSub = await refreshSubscription(sub.rss_url)
      currentSub.value = newSub
      const index = subscriptions.value.findIndex(s => s.rss_url === sub.rss_url)
      if (index !== -1) {
        subscriptions.value[index] = newSub
      }
      notify('Subscription refreshed', 'success')
    } catch (e: unknown) {
      notify(`Failed to refresh: ${e instanceof Error ? e.message : e}`, 'error')
    } finally {
      refreshLoading.value = false
    }
  }

  async function refreshLibrary() {
    libraryRefreshLoading.value = true
    try {
      for (let i = 0; i < subscriptions.value.length; i++) {
        const sub = subscriptions.value[i]
        try {
          const newSub = await refreshSubscription(sub.rss_url)
          subscriptions.value[i] = newSub
        } catch (e) {
          console.error(`Failed to refresh ${sub.title}:`, e)
        }
      }
      notify('Library refreshed', 'success')
    } catch (e: unknown) {
      notify(`Failed to refresh library: ${e instanceof Error ? e.message : e}`, 'error')
    } finally {
      libraryRefreshLoading.value = false
    }
  }

  function openAddDialog() {
    resetAddDialog()
    showAddDialog.value = true
  }

  function closeAddDialog() {
    if (addLoading.value) return
    showAddDialog.value = false
    resetAddDialog()
  }

  function resetAddDialog() {
    opmlFile.value = null
    opmlError.value = ''
  }

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
    try {
      const text = await file.text()
      if (text.length === 0) {
        throw new Error('File is empty')
      }
      subscriptions.value = await importOpml(text)
      notify('Import successful', 'success')
      closeAddDialog()
    } catch (e: unknown) {
      console.error('Import failed:', e)
      notify(`Import failed: ${e instanceof Error ? e.message : e}`, 'error')
    } finally {
      addLoading.value = false
    }
  }

  async function unsubscribe(sub: PodcastSubscription) {
    if (!confirm(`Unsubscribe from "${sub.title}"?`)) return
    try {
      subscriptions.value = await removeSubscription(sub.rss_url)
      if (currentSub.value?.rss_url === sub.rss_url) {
        goBack()
      }
    } catch (e: unknown) {
      notify(`Operation failed: ${e instanceof Error ? e.message : e}`, 'error')
    }
  }

  function formatMonth(dateStr?: string): string {
    if (!dateStr) return ''
    const date = new Date(dateStr)
    return date.toLocaleDateString('en-US', { month: 'short' }).toUpperCase()
  }

  function formatDay(dateStr?: string): string {
    if (!dateStr) return ''
    const date = new Date(dateStr)
    return date.getDate().toString()
  }

  function setupEpisodesObserver() {
    if (episodesObserver) {
      episodesObserver.disconnect()
    }

    episodesObserver = new IntersectionObserver(entries => {
      const entry = entries[0]
      if (!entry.isIntersecting) return
      if (visibleEpisodeCount.value >= episodes.value.length) return
      visibleEpisodeCount.value = Math.min(
        visibleEpisodeCount.value + EpisodesPageSize,
        episodes.value.length
      )
    })

    if (episodesLoadMoreTrigger.value) {
      episodesObserver.observe(episodesLoadMoreTrigger.value)
    }
  }

  function cleanup() {
    if (episodesObserver) {
      episodesObserver.disconnect()
      episodesObserver = null
    }
  }

  onMounted(async () => {
    loading.value = true
    await loadSubscriptions()
    loading.value = false
  })

  onBeforeUnmount(cleanup)

  return {
    // State
    subscriptions,
    currentSub,
    episodes,
    episodesLoading,
    loading,
    searchQuery,
    visibleEpisodes,
    visibleEpisodeCount,
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
    // Audio
    currentPlaying,
    currentPlayingSubUrl,
    // Actions
    loadSubscriptions,
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
    notify,
  }
}
