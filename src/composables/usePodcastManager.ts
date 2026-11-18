import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  fetchEpisodes,
  getSubscriptions,
  type OpmlImportFailure,
  type OpmlImportResult,
  type PodcastEpisode,
  type PodcastSubscription,
  refreshSubscription,
  removeSubscription,
  saveSubscription,
} from '@/api/podcast'
import { useAudioPlayer } from '@/composables/useAudioPlayer'
import { useSnackbar } from '@/composables/useSnackbar'

export function usePodcastManager() {
  const { t } = useI18n()
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

  const {
    currentPlaying,
    currentPlayingSubUrl,
    playbackError,
    playEpisode: globalPlayEpisode,
  } = useAudioPlayer()
  const { show: snackbar, text: snackbarText, color: snackbarColor, notify } = useSnackbar()

  // 播放失败（无音频地址 / 网络错误 / 被拦截）时给用户明确提示，而不是静默无声
  watch(playbackError, err => {
    if (err) {
      notify(err, 'error')
    }
  })

  const currentView = ref<'library' | 'detail'>('library')

  const showAddDialog = ref(false)
  const addLoading = ref(false)
  const refreshLoading = ref(false)
  const libraryRefreshLoading = ref(false)

  const showShowNotesDialog = ref(false)
  const currentShowNotesEpisode = ref<PodcastEpisode | null>(null)

  // OPML 导入结果反馈
  const showImportReport = ref(false)
  const importFailures = ref<OpmlImportFailure[]>([])
  const importAddedCount = ref(0)
  // 手动补订：失败条目带回填 URL 重新打开添加对话框
  const retryUrl = ref('')

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
      const payload = await fetchEpisodes(sub.rss_url)
      episodes.value = payload.episodes

      // 小宇宙付费信息回填到订阅并持久化，节目库/详情页才能显示付费标记
      if (payload.pay_type && payload.pay_type !== sub.pay_type) {
        const updated: PodcastSubscription = {
          ...sub,
          pay_type: payload.pay_type,
          paid_episode_count: payload.paid_episode_count,
        }
        currentSub.value = updated
        const index = subscriptions.value.findIndex(s => s.rss_url === sub.rss_url)
        if (index !== -1) {
          subscriptions.value[index] = updated
        }
        saveSubscription(updated).catch(err => console.error('保存付费信息失败:', err))
      }

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
    retryUrl.value = ''
  }

  /** 添加对话框确认了一个解析成功的订阅：入库并选中 */
  async function confirmAddSubscription(sub: PodcastSubscription) {
    try {
      subscriptions.value = await saveSubscription(sub)
      notify(t('podcast.subscribed', { title: sub.title }), 'success')
      selectSubscription(sub)
    } catch (e: unknown) {
      notify(`Failed to subscribe: ${e instanceof Error ? e.message : e}`, 'error')
    }
  }

  /** OPML 导入完成（AddSubscriptionDialog 的 imported 事件） */
  function handleImportResult(result: OpmlImportResult) {
    // 先算新增数，再覆盖列表
    const before = new Set(subscriptions.value.map(s => s.rss_url))
    importAddedCount.value = result.subscriptions.filter(s => !before.has(s.rss_url)).length
    subscriptions.value = result.subscriptions
    importFailures.value = result.failures

    if (importFailures.value.length > 0) {
      showImportReport.value = true
    } else if (importAddedCount.value > 0) {
      notify(t('podcast.importAdded', { n: importAddedCount.value }), 'success')
    } else {
      notify(t('podcast.importNothingNew'), 'info')
    }
  }

  /** 失败条目「手动添加」：关报告、开添加框并回填链接 */
  function retryFailure(failure: OpmlImportFailure) {
    showImportReport.value = false
    retryUrl.value = failure.xml_url
    showAddDialog.value = true
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
    snackbar,
    snackbarText,
    snackbarColor,
    // Audio
    currentPlaying,
    currentPlayingSubUrl,
    showImportReport,
    importFailures,
    importAddedCount,
    retryUrl,
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
    confirmAddSubscription,
    handleImportResult,
    retryFailure,
    unsubscribe,
    formatMonth,
    formatDay,
    notify,
  }
}
