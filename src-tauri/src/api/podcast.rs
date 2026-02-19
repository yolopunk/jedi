use futures::future::join_all;
use regex::Regex;
use reqwest::Client;
use rss::Channel;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize)]
pub struct PodcastEpisode {
  pub title: String,
  pub audio_url: String,
  pub guid: String,
  pub link: String,
  pub image_url: String,
  pub description: String,
  pub show_notes: String,   // content:encoded or description
  pub podcast_name: String, // Optional, can be filled by frontend
  pub author: String,
  pub episode_number: Option<String>,
  pub pub_date: Option<String>,
  pub duration: Option<String>,
  pub chapters: Vec<String>, // Placeholder for now
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PodcastSubscription {
  pub title: String,
  pub rss_url: String,
  pub description: String,
  pub image_url: String,
  pub website: String, // link
  #[serde(default)]
  pub categories: Vec<String>,
  #[serde(default)]
  pub language: String,
  #[serde(default)]
  pub copyright: String,
  #[serde(default)]
  pub author: String,
  #[serde(default)]
  pub owner_name: String,
  #[serde(default)]
  pub owner_email: String,
  #[serde(default)]
  pub podcast_type: String,
}

// ==================== Helper Functions ====================

fn parse_subscription_from_channel(channel: &Channel, rss_url: &str) -> PodcastSubscription {
  // Extract iTunes extension
  let itunes = channel.itunes_ext.as_ref();

  // Image: iTunes > Channel Image > Default
  let image_url = itunes
    .and_then(|ext| ext.image.clone())
    .or_else(|| channel.image.as_ref().map(|i| i.url.clone()))
    .unwrap_or_default();

  // Categories
  let categories = itunes
    .map(|ext| ext.categories.iter().map(|c| c.text.clone()).collect())
    .unwrap_or_default();

  // Author
  let author = itunes
    .and_then(|ext| ext.author.clone())
    .unwrap_or_default();

  // Owner
  let (owner_name, owner_email) = itunes
    .and_then(|ext| ext.owner.as_ref())
    .map(|o| {
      (
        o.name.clone().unwrap_or_default(),
        o.email.clone().unwrap_or_default(),
      )
    })
    .unwrap_or_default();

  // Type
  let podcast_type = itunes
    .and_then(|ext| ext.r#type.clone())
    .unwrap_or_else(|| "episodic".to_string());

  PodcastSubscription {
    title: channel.title.clone(),
    rss_url: rss_url.to_string(),
    description: channel.description.clone(),
    image_url,
    website: channel.link.clone(),
    categories,
    language: channel.language.clone().unwrap_or_default(),
    copyright: channel.copyright.clone().unwrap_or_default(),
    author,
    owner_name,
    owner_email,
    podcast_type,
  }
}

fn parse_episodes_from_channel(channel: &Channel) -> Vec<PodcastEpisode> {
  let channel_image = channel
    .itunes_ext
    .as_ref()
    .and_then(|ext| ext.image.clone())
    .or_else(|| channel.image.as_ref().map(|i| i.url.clone()));

  channel
    .items
    .iter()
    .map(|item| {
      // Audio URL
      let audio_url = item
        .enclosure
        .as_ref()
        .map(|e| e.url.clone())
        .unwrap_or_default();

      // GUID
      let guid = item
        .guid
        .as_ref()
        .map(|g| g.value.clone())
        .unwrap_or_default();

      // Image: Item iTunes > Item Media > Channel Image
      let image_url = item
        .itunes_ext
        .as_ref()
        .and_then(|i| i.image.clone())
        .or_else(|| channel_image.clone())
        .unwrap_or_default();

      // Duration
      let duration = item.itunes_ext.as_ref().and_then(|i| i.duration.clone());

      // Episode Number
      let episode_number = item.itunes_ext.as_ref().and_then(|i| i.episode.clone());

      // Author
      let author = item
        .author
        .clone()
        .or_else(|| item.itunes_ext.as_ref().and_then(|i| i.author.clone()))
        .unwrap_or_default();

      // Show Notes / Content
      // Priority: content:encoded > description
      let show_notes = item
        .content
        .clone()
        .or_else(|| item.description.clone())
        .unwrap_or_default();

      PodcastEpisode {
        title: item.title.clone().unwrap_or_default(),
        audio_url,
        guid,
        link: item.link.clone().unwrap_or_default(),
        image_url,
        description: item.description.clone().unwrap_or_default(),
        show_notes,
        podcast_name: channel.title.clone(),
        author,
        episode_number,
        pub_date: item.pub_date.clone(),
        duration,
        chapters: Vec::new(), // TODO: Parse chapters if available
      }
    })
    .collect()
}

async fn fetch_rss_content(url: &str) -> Result<Channel, String> {
  let client = Client::new();
  let res = client
    .get(url)
    .header("User-Agent", "Jedi/1.0")
    .send()
    .await
    .map_err(|e| e.to_string())?;

  if !res.status().is_success() {
    return Err(format!("Failed to fetch RSS: HTTP {}", res.status()));
  }

  let content = res.bytes().await.map_err(|e| e.to_string())?;
  Channel::read_from(&content[..]).map_err(|e| e.to_string())
}

// ==================== Tauri Commands ====================

#[tauri::command]
pub async fn refresh_subscription(
  app: AppHandle,
  rss_url: String,
) -> Result<PodcastSubscription, String> {
  // 1. Fetch latest info
  let channel = fetch_rss_content(&rss_url).await?;
  let new_sub = parse_subscription_from_channel(&channel, &rss_url);

  // 2. Update storage
  save_subscription(app, new_sub.clone()).await?;

  Ok(new_sub)
}

#[tauri::command]
pub async fn resolve_xiaoyuzhou_podcast(url: String) -> Result<PodcastSubscription, String> {
  // This function now only acts as a URL resolver, then fetches RSS
  let rss_url = if url.contains("xiaoyuzhoufm.com") {
    resolve_xiaoyuzhou_rss_url(&url).await?
  } else {
    url
  };

  let channel = fetch_rss_content(&rss_url).await?;
  Ok(parse_subscription_from_channel(&channel, &rss_url))
}

#[tauri::command]
pub async fn fetch_rss_channel(rss_url: String) -> Result<PodcastSubscription, String> {
  let resolved_url = resolve_rss_url(&rss_url).await?;
  let channel = fetch_rss_content(&resolved_url).await?;
  Ok(parse_subscription_from_channel(&channel, &resolved_url))
}

#[tauri::command]
pub async fn fetch_episodes(rss_url: String) -> Result<Vec<PodcastEpisode>, String> {
  let channel = fetch_rss_content(&rss_url).await?;
  Ok(parse_episodes_from_channel(&channel))
}

#[tauri::command]
pub async fn get_subscriptions(app: AppHandle) -> Result<Vec<PodcastSubscription>, String> {
  let path = get_subscriptions_path(&app)?;
  if !path.exists() {
    return Ok(Vec::new());
  }

  let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
  let subs: Vec<PodcastSubscription> = serde_json::from_str(&content).unwrap_or_default();
  Ok(subs)
}

#[tauri::command]
pub async fn save_subscription(
  app: AppHandle,
  sub: PodcastSubscription,
) -> Result<Vec<PodcastSubscription>, String> {
  let mut subs = get_subscriptions(app.clone()).await?;

  if let Some(index) = subs.iter().position(|s| s.rss_url == sub.rss_url) {
    subs[index] = sub;
  } else {
    subs.push(sub);
  }

  let path = get_subscriptions_path(&app)?;
  let content = serde_json::to_string_pretty(&subs).map_err(|e| e.to_string())?;
  fs::write(path, content).map_err(|e| e.to_string())?;

  Ok(subs)
}

#[tauri::command]
pub async fn remove_subscription(
  app: AppHandle,
  rss_url: String,
) -> Result<Vec<PodcastSubscription>, String> {
  let mut subs = get_subscriptions(app.clone()).await?;
  subs.retain(|s| s.rss_url != rss_url);

  let path = get_subscriptions_path(&app)?;
  let content = serde_json::to_string_pretty(&subs).map_err(|e| e.to_string())?;
  fs::write(path, content).map_err(|e| e.to_string())?;

  Ok(subs)
}

#[tauri::command]
pub async fn import_opml(
  app: AppHandle,
  opml_content: String,
) -> Result<Vec<PodcastSubscription>, String> {
  println!("Importing OPML content length: {}", opml_content.len());

  let outline_re = Regex::new(r#"<outline[^>]+>"#).map_err(|e| e.to_string())?;
  let xml_url_re = Regex::new(r#"xmlUrl=(?:"([^"]*)"|'([^']*)')"#).unwrap();

  let mut current_subs = get_subscriptions(app.clone()).await?;
  let mut pending_fetches = Vec::new();

  for cap in outline_re.captures_iter(&opml_content) {
    let tag = &cap[0];

    let mut rss_url = String::new();
    if let Some(url_cap) = xml_url_re.captures(tag) {
      if let Some(m) = url_cap.get(1).or(url_cap.get(2)) {
        rss_url = m.as_str().to_string();
      }
    }

    rss_url = rss_url.trim().replace('`', "");

    if rss_url.is_empty() {
      continue;
    }

    if current_subs.iter().any(|s| s.rss_url == rss_url) {
      continue;
    }

    let rss_url_clone = rss_url.clone();

    pending_fetches.push(async move {
      match fetch_rss_channel(rss_url_clone.clone()).await {
        Ok(s) => Some(s),
        Err(e) => {
          println!("Failed to import {}: {}", rss_url_clone, e);
          None
        }
      }
    });
  }

  if !pending_fetches.is_empty() {
    let new_subs = join_all(pending_fetches).await;
    for sub in new_subs.into_iter().flatten() {
      current_subs.push(sub);
    }

    let path = get_subscriptions_path(&app)?;
    let content = serde_json::to_string_pretty(&current_subs).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())?;
  }

  Ok(current_subs)
}

// ==================== Internal Utils ====================

fn get_subscriptions_path(app: &AppHandle) -> Result<PathBuf, String> {
  let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
  if !app_data_dir.exists() {
    fs::create_dir_all(&app_data_dir).map_err(|e| e.to_string())?;
  }
  Ok(app_data_dir.join("subscriptions.json"))
}

async fn resolve_rss_url(url: &str) -> Result<String, String> {
  // If it's a Xiaoyuzhou URL, try to extract RSS from it
  if url.contains("xiaoyuzhoufm.com") {
    return resolve_xiaoyuzhou_rss_url(url).await;
  }

  let client = Client::new();
  let res = client
    .get(url)
    .header("User-Agent", "Jedi/1.0")
    .send()
    .await
    .map_err(|e| e.to_string())?;

  if !res.status().is_success() {
    return Ok(url.to_string());
  }

  let content_type = res
    .headers()
    .get("content-type")
    .and_then(|v| v.to_str().ok())
    .unwrap_or("");

  if content_type.contains("html") {
    let html = res.text().await.map_err(|e| e.to_string())?;
    let re = Regex::new(r#""rssUrl"\s*:\s*"([^"]+)""#).map_err(|e| e.to_string())?;
    if let Some(cap) = re.captures(&html) {
      if let Some(rss) = cap.get(1) {
        return Ok(rss.as_str().to_string());
      }
    }
  }

  Ok(url.to_string())
}

async fn resolve_xiaoyuzhou_rss_url(url: &str) -> Result<String, String> {
  let client = Client::new();
  let res = client.get(url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

  if !res.status().is_success() {
    return Err(format!("HTTP {}", res.status()));
  }

  let html = res
    .text()
    .await
    .map_err(|e| format!("Read failed: {}", e))?;

  let rss_patterns = [
    r#""rssUrl"\s*:\s*"([^"]+)""#,
    r#"<link[^>]+type="application/rss\+xml"[^>]+href="([^"]+)""#,
  ];

  for pattern in &rss_patterns {
    let re = Regex::new(pattern).map_err(|e| e.to_string())?;
    if let Some(cap) = re.captures(&html) {
      if let Some(matched) = cap.get(1) {
        return Ok(matched.as_str().to_string().replace("\\u002F", "/"));
      }
    }
  }

  Err("Could not find RSS URL in page".to_string())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_fetch_rss_info_complete() {
    let url = "https://feed.xyzfm.space/dk4yh3pkpjp3";
    match fetch_rss_channel(url.to_string()).await {
      Ok(sub) => {
        println!("Title: {}", sub.title);
        println!("Author: {}", sub.author);
        println!("Owner: {} <{}>", sub.owner_name, sub.owner_email);
        println!("Categories: {:?}", sub.categories);
        println!("Image: {}", sub.image_url);
        assert!(!sub.categories.is_empty());
        assert!(!sub.image_url.is_empty());
      }
      Err(e) => panic!("Failed: {}", e),
    }
  }
}
