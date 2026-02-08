import { invoke } from '@tauri-apps/api/core';

export interface PodcastEpisode {
  title: string;
  audio_url: string;
  guid: string;
  link: string;
  image_url: string;
  description: string;
  show_notes: string;
  podcast_name: string;
  author: string;
  episode_number?: string;
  pub_date?: string;
  duration?: string;
  chapters: Chapter[];
}

export interface Chapter {
  title: string;
  start_time: number;
  end_time?: number;
}

export interface PodcastSubscription {
  title: string;
  rss_url: string;
  description: string;
  image_url: string;
  website: string;
  categories: string[];
  language: string;
  copyright: string;
  author: string;
  owner_name: string;
  owner_email: string;
  podcast_type: string;
}

export async function refreshSubscription(rssUrl: string): Promise<PodcastSubscription> {
  return await invoke('refresh_subscription', { rssUrl });
}

export async function getSubscriptions(): Promise<PodcastSubscription[]> {
  return await invoke('get_subscriptions');
}

export async function saveSubscription(sub: PodcastSubscription): Promise<PodcastSubscription[]> {
  return await invoke('save_subscription', { sub });
}

export async function removeSubscription(rssUrl: string): Promise<PodcastSubscription[]> {
  return await invoke('remove_subscription', { rssUrl });
}

export async function fetchRssChannel(rssUrl: string): Promise<PodcastSubscription> {
  return await invoke('fetch_rss_channel', { rssUrl });
}

export async function fetchEpisodes(rssUrl: string): Promise<PodcastEpisode[]> {
  return await invoke('fetch_episodes', { rssUrl });
}

export async function importOpml(opmlContent: string): Promise<PodcastSubscription[]> {
  return await invoke('import_opml', { opmlContent });
}
