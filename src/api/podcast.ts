import { invoke } from '@tauri-apps/api/core'

export interface PodcastEpisode {
  title: string
  audio_url: string
  guid: string
  link: string
  image_url: string
  description: string
  show_notes: string
  podcast_name: string
  author: string
  episode_number?: string
  pub_date?: string
  duration?: string
  chapters: Chapter[]
  /** 小宇宙付费单集 */
  is_paid?: boolean
}

export interface Chapter {
  title: string
  start_time: number
  end_time?: number
}

export interface PodcastSubscription {
  title: string
  rss_url: string
  description: string
  image_url: string
  website: string
  categories: string[]
  language: string
  copyright: string
  author: string
  owner_name: string
  owner_email: string
  podcast_type: string
  /** FREE / PAY_EPISODE_PODCAST ...（小宇宙来源才有） */
  pay_type?: string
  paid_episode_count?: number
}

/** fetch_episodes 的返回：剧集 + 播客级付费信息 */
export interface EpisodesPayload {
  episodes: PodcastEpisode[]
  pay_type: string
  paid_episode_count: number
}

export async function refreshSubscription(rssUrl: string): Promise<PodcastSubscription> {
  return await invoke<PodcastSubscription>('refresh_subscription', { rssUrl })
}

export async function getSubscriptions(): Promise<PodcastSubscription[]> {
  return await invoke<PodcastSubscription[]>('get_subscriptions')
}

export async function saveSubscription(sub: PodcastSubscription): Promise<PodcastSubscription[]> {
  return await invoke<PodcastSubscription[]>('save_subscription', { sub })
}

export async function removeSubscription(rssUrl: string): Promise<PodcastSubscription[]> {
  return await invoke<PodcastSubscription[]>('remove_subscription', { rssUrl })
}

export async function fetchRssChannel(rssUrl: string): Promise<PodcastSubscription> {
  return await invoke<PodcastSubscription>('fetch_rss_channel', { rssUrl })
}

export async function fetchEpisodes(rssUrl: string): Promise<EpisodesPayload> {
  return await invoke<EpisodesPayload>('fetch_episodes', { rssUrl })
}

export async function importOpml(opmlContent: string): Promise<OpmlImportResult> {
  return await invoke<OpmlImportResult>('import_opml', { opmlContent })
}

/** OPML 导入返回体：完整订阅列表 + 失败明细 */
export interface OpmlImportResult {
  subscriptions: PodcastSubscription[]
  failures: OpmlImportFailure[]
}

export interface OpmlImportFailure {
  title: string
  xml_url: string
  error: string
}

/** 添加订阅：粘贴 RSS 直链或小宇宙节目页链接均可，后端自动解析校验 */
export async function resolveSubscription(url: string): Promise<PodcastSubscription> {
  return await invoke<PodcastSubscription>('fetch_rss_channel', { rssUrl: url })
}
