use futures::future::join_all;
use regex::Regex;
use reqwest::Client;
use rss::Channel;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
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
  /// 小宇宙付费单集（payType != FREE 或 isPrivateMedia）
  #[serde(default)]
  pub is_paid: bool,
}

/// 小宇宙播客的付费信息（来自播客网页的 __NEXT_DATA__）
#[derive(Debug, Default, Clone)]
struct PayInfo {
  pay_type: String,
  paid_episode_count: u32,
  paid_eids: HashSet<String>,
}

/// fetch_episodes 的返回体：剧集 + 播客级付费信息
#[derive(Debug, Serialize)]
pub struct EpisodesPayload {
  pub episodes: Vec<PodcastEpisode>,
  pub pay_type: String,
  pub paid_episode_count: u32,
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
  /// FREE / PAY_EPISODE_PODCAST / ...
  #[serde(default)]
  pub pay_type: String,
  #[serde(default)]
  pub paid_episode_count: u32,
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
    pay_type: "FREE".to_string(),
    paid_episode_count: 0,
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
        is_paid: false,
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

// ==================== Paid Episode Support ====================

/// 从频道 link 中取出小宇宙播客 id
fn xiaoyuzhou_podcast_id(channel_link: &str) -> Option<String> {
  let re = Regex::new(r"xiaoyuzhoufm\.com/podcast/([0-9a-f]+)").ok()?;
  re.captures(channel_link)
    .and_then(|cap| cap.get(1))
    .map(|m| m.as_str().to_string())
}

/// 抓取小宇宙播客网页并解析付费信息（网页内嵌的 __NEXT_DATA__）。
/// 只有小宇宙来源的播客才有，其它来源返回 None。
async fn fetch_pay_info(channel_link: &str) -> Option<PayInfo> {
  let pid = xiaoyuzhou_podcast_id(channel_link)?;
  let client = Client::builder()
    .timeout(Duration::from_secs(12))
    .build()
    .ok()?;

  let html = client
    .get(format!("https://www.xiaoyuzhoufm.com/podcast/{pid}"))
    .header(
      "User-Agent",
      "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
       (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    )
    .send()
    .await
    .ok()?
    .text()
    .await
    .ok()?;

  let re = Regex::new(
    r#"(?s)<script id="__NEXT_DATA__" type="application/json">(.*?)</script>"#,
  )
  .ok()?;
  let json: serde_json::Value = serde_json::from_str(re.captures(&html)?.get(1)?.as_str()).ok()?;
  let podcast = json.pointer("/props/pageProps/podcast")?;

  let pay_type = podcast
    .get("payType")
    .and_then(|v| v.as_str())
    .unwrap_or("FREE")
    .to_string();
  let paid_episode_count = podcast
    .get("payEpisodeCount")
    .and_then(|v| v.as_u64())
    .unwrap_or(0) as u32;

  // 播客页只内嵌最近若干集，能确认付费的就先标记出来
  let mut paid_eids = HashSet::new();
  if let Some(episodes) = podcast.get("episodes").and_then(|v| v.as_array()) {
    for ep in episodes {
      let ep_pay = ep.get("payType").and_then(|v| v.as_str()).unwrap_or("FREE");
      let private = ep
        .get("isPrivateMedia")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
      if ep_pay != "FREE" || private {
        if let Some(eid) = ep.get("eid").and_then(|v| v.as_str()) {
          paid_eids.insert(eid.to_string());
        }
      }
    }
  }

  Some(PayInfo {
    pay_type,
    paid_episode_count,
    paid_eids,
  })
}

/// 把付费信息补充到订阅对象上
async fn apply_pay_info(sub: &mut PodcastSubscription, channel_link: &str) {
  if let Some(info) = fetch_pay_info(channel_link).await {
    sub.pay_type = info.pay_type;
    sub.paid_episode_count = info.paid_episode_count;
  }
}

// ==================== Tauri Commands ====================

#[tauri::command]
pub async fn refresh_subscription(
  app: AppHandle,
  rss_url: String,
) -> Result<PodcastSubscription, String> {
  // 1. Fetch latest info
  let channel = fetch_rss_content(&rss_url).await?;
  let mut new_sub = parse_subscription_from_channel(&channel, &rss_url);
  apply_pay_info(&mut new_sub, &channel.link).await;

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
  let mut sub = parse_subscription_from_channel(&channel, &rss_url);
  apply_pay_info(&mut sub, &channel.link).await;
  Ok(sub)
}

#[tauri::command]
pub async fn fetch_rss_channel(rss_url: String) -> Result<PodcastSubscription, String> {
  let resolved_url = resolve_rss_url(&rss_url).await?;
  let channel = fetch_rss_content(&resolved_url).await?;
  let mut sub = parse_subscription_from_channel(&channel, &resolved_url);
  apply_pay_info(&mut sub, &channel.link).await;
  Ok(sub)
}

#[tauri::command]
pub async fn fetch_episodes(rss_url: String) -> Result<EpisodesPayload, String> {
  let channel = fetch_rss_content(&rss_url).await?;
  let mut episodes = parse_episodes_from_channel(&channel);

  let mut pay_type = "FREE".to_string();
  let mut paid_episode_count = 0u32;
  if let Some(info) = fetch_pay_info(&channel.link).await {
    pay_type = info.pay_type;
    paid_episode_count = info.paid_episode_count;
    // RSS 的 guid 与接口的 eid 一致，可直接对齐
    for ep in &mut episodes {
      ep.is_paid = info.paid_eids.contains(&ep.guid);
    }
  }

  Ok(EpisodesPayload {
    episodes,
    pay_type,
    paid_episode_count,
  })
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
