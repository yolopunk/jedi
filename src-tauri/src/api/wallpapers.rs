use futures::stream::{self, StreamExt};
use gray_matter::engine::YAML;
use gray_matter::{Matter, Pod};
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tokio::task;

const GUIDES_API_URL: &str =
  "https://api.github.com/repos/ByteByteGoHq/system-design-101/contents/data/guides";
const CONCURRENT_REQUESTS: usize = 5;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WallpaperItem {
  pub id: String,
  pub title: String,
  pub url: String,
  pub category: String,
  pub tags: Vec<String>,
  pub description: String,
  pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct GitHubContent {
  name: String,
  download_url: Option<String>,
  sha: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheEntry {
  sha: String,
  item: WallpaperItem,
}

type WallpaperCache = HashMap<String, CacheEntry>;

#[derive(Debug, Deserialize, Clone)]
struct FrontMatter {
  title: Option<String>,
  description: Option<String>,
  image: Option<String>,
  categories: Option<Vec<String>>,
  tags: Option<Vec<String>>,
  // For single string fields that might be used instead of arrays
  category: Option<String>,
  tag: Option<String>,
}

pub struct WallpaperManager {
  client: Client,
  app_data_dir: PathBuf,
}

impl WallpaperManager {
  pub fn new(app_data_dir: PathBuf) -> Self {
    let client = Client::builder()
      .user_agent("Jedi-App")
      .build()
      .unwrap_or_default();

    Self {
      client,
      app_data_dir,
    }
  }

  fn cache_path(&self) -> PathBuf {
    self.app_data_dir.join("wallpapers_cache.json")
  }

  fn wallpapers_dir(&self) -> PathBuf {
    self.app_data_dir.join("wallpapers")
  }

  pub fn load_cache(&self) -> WallpaperCache {
    let path = self.cache_path();
    if path.exists() {
      fs::read_to_string(&path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
    } else {
      HashMap::new()
    }
  }

  pub fn save_cache(&self, cache: &WallpaperCache) -> Result<(), String> {
    let json = serde_json::to_string(cache).map_err(|e| e.to_string())?;
    fs::write(self.cache_path(), json).map_err(|e| e.to_string())?;
    Ok(())
  }

  pub async fn fetch_remote_list(&self) -> Result<Vec<GitHubContent>, String> {
    let resp = self
      .client
      .get(GUIDES_API_URL)
      .send()
      .await
      .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
      return Err(format!("Failed to fetch guides list: {}", resp.status()));
    }

    resp.json().await.map_err(|e| e.to_string())
  }

  pub async fn process_item(&self, name: String, url: String) -> Result<WallpaperItem, String> {
    let slug = name.replace(".md", "");
    println!("Fetching metadata for: {}", slug);

    let resp = self
      .client
      .get(&url)
      .send()
      .await
      .map_err(|e| format!("Network error fetching {}: {}", url, e))?;

    if !resp.status().is_success() {
      return Err(format!("Failed to fetch URL {}: {}", url, resp.status()));
    }

    let text = resp
      .text()
      .await
      .map_err(|e| format!("Failed to get text content for {}: {}", slug, e))?;

    let matter = Matter::<YAML>::new();
    let result = matter
      .parse(&text)
      .map_err(|e| format!("Failed to parse frontmatter for {}: {}", slug, e))?;

    let front_matter: FrontMatter = result
      .data
      .ok_or_else(|| format!("No frontmatter found for {}", slug))
      .and_then(|data: Pod| {
        data
          .deserialize::<FrontMatter>()
          .map_err(|e| format!("Failed to parse frontmatter: {}", e))
      })?;

    let title = front_matter
      .title
      .ok_or_else(|| format!("Missing title for {}", slug))?;
    let image = front_matter
      .image
      .ok_or_else(|| format!("Missing image for {}", slug))?;

    let category = front_matter
      .categories
      .as_ref()
      .and_then(|c| c.first())
      .cloned()
      .or(front_matter.category.clone())
      .unwrap_or_else(|| "General".to_string());

    let tags = front_matter
      .tags
      .clone()
      .or_else(|| front_matter.tag.clone().map(|t| vec![t]))
      .unwrap_or_default();

    let description = front_matter.description.unwrap_or_default();
    let content = fix_relative_paths(&result.content, &url);

    Ok(WallpaperItem {
      id: slug,
      title,
      url: resolve_image_url(&image, &url),
      category,
      tags,
      description,
      content,
    })
  }
}

#[tauri::command]
pub async fn get_wallpapers<R: Runtime>(app: AppHandle<R>) -> Result<Vec<WallpaperItem>, String> {
  let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
  let manager = WallpaperManager::new(app_data_dir);

  let cache = manager.load_cache();
  let mut wallpapers: Vec<WallpaperItem> = cache.values().map(|entry| entry.item.clone()).collect();

  wallpapers.sort_by(|a, b| a.id.cmp(&b.id));
  println!("Loaded {} wallpapers from cache", wallpapers.len());

  Ok(wallpapers)
}

#[tauri::command]
pub async fn sync_wallpapers<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
  println!("Syncing wallpapers...");
  let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;

  if !app_data_dir.exists() {
    fs::create_dir_all(&app_data_dir).map_err(|e| e.to_string())?;
  }

  let manager = WallpaperManager::new(app_data_dir.clone());
  let mut cache = manager.load_cache();

  let contents = manager.fetch_remote_list().await?;

  // Filter items that need processing
  let items_to_process: Vec<_> = contents
    .into_iter()
    .filter(|item| item.name.ends_with(".md") && item.download_url.is_some())
    .filter(|item| {
      if let Some(entry) = cache.get(&item.name) {
        entry.sha != item.sha
      } else {
        true
      }
    })
    .collect();

  println!("Items to process: {}", items_to_process.len());

  if items_to_process.is_empty() {
    return Ok(());
  }

  // Process items concurrently
  let manager_ref = &manager;
  let mut stream = stream::iter(items_to_process)
    .map(|item| async move {
      let url = item.download_url.unwrap();
      match manager_ref.process_item(item.name.clone(), url).await {
        Ok(wallpaper) => Some((item.name, item.sha, wallpaper)),
        Err(e) => {
          eprintln!("Error processing {}: {}", item.name, e);
          None
        }
      }
    })
    .buffer_unordered(CONCURRENT_REQUESTS);

  let mut batch_items = Vec::new();
  let mut processed_count = 0;

  while let Some(result) = stream.next().await {
    if let Some((name, sha, item)) = result {
      cache.insert(
        name,
        CacheEntry {
          sha,
          item: item.clone(),
        },
      );
      batch_items.push(item);
      processed_count += 1;

      // Emit batch updates every 5 items or so to keep UI responsive
      if batch_items.len() >= 5 {
        let _ = app.emit("wallpaper-batch", &batch_items);
        batch_items.clear();
      }
    }
  }

  // Emit remaining items
  if !batch_items.is_empty() {
    let _ = app.emit("wallpaper-batch", &batch_items);
  }

  manager.save_cache(&cache)?;
  let _ = app.emit("wallpaper-sync-complete", ());

  println!("Sync complete. Processed {} items.", processed_count);
  Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum WallpaperMode {
  Center,
  Crop,
  Fit,
  Span,
  Stretch,
  Tile,
}

impl From<WallpaperMode> for wallpaper::Mode {
  fn from(mode: WallpaperMode) -> Self {
    match mode {
      WallpaperMode::Center => wallpaper::Mode::Center,
      WallpaperMode::Crop => wallpaper::Mode::Crop,
      WallpaperMode::Fit => wallpaper::Mode::Fit,
      WallpaperMode::Span => wallpaper::Mode::Span,
      WallpaperMode::Stretch => wallpaper::Mode::Stretch,
      WallpaperMode::Tile => wallpaper::Mode::Tile,
    }
  }
}

#[tauri::command]
pub async fn set_desktop_wallpaper<R: Runtime>(
  app: AppHandle<R>,
  url: String,
  mode: Option<WallpaperMode>,
) -> Result<(), String> {
  let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
  let manager = WallpaperManager::new(app_data_dir);
  let wallpapers_dir = manager.wallpapers_dir();

  if !wallpapers_dir.exists() {
    fs::create_dir_all(&wallpapers_dir).map_err(|e| e.to_string())?;
  }

  let filename = Path::new(&url)
    .file_name()
    .and_then(|s| s.to_str())
    .unwrap_or("wallpaper.jpg");

  let file_path = wallpapers_dir.join(filename);

  // Download file if it doesn't exist or we want to overwrite
  // Always download for now to ensure fresh content or handling url changes
  let response = manager
    .client
    .get(&url)
    .send()
    .await
    .map_err(|e| e.to_string())?;

  if !response.status().is_success() {
    return Err(format!("Failed to download image: {}", response.status()));
  }

  let content = response.bytes().await.map_err(|e| e.to_string())?;
  fs::write(&file_path, &content).map_err(|e| e.to_string())?;

  // Set wallpaper in a blocking task to avoid blocking the async runtime
  let path_str = file_path
    .to_str()
    .ok_or_else(|| "Failed to convert path to string".to_string())?
    .to_string();

  let mode_val = mode.unwrap_or(WallpaperMode::Crop);

  let _ = task::spawn_blocking(move || {
    wallpaper::set_from_path(&path_str).map_err(|e| e.to_string())?;
    // set_mode is unsupported on macOS via AppleScript, ignore the error
    let _ = wallpaper::set_mode(mode_val.into());
    Ok::<(), String>(())
  })
  .await
  .map_err(|e| format!("Task join error: {}", e))?;
  Ok(())
}

#[tauri::command]
pub async fn get_current_wallpaper() -> Result<String, String> {
  task::spawn_blocking(|| wallpaper::get().map_err(|e| e.to_string()))
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
pub async fn show_in_folder(path: String) -> Result<(), String> {
  #[cfg(target_os = "windows")]
  {
    Command::new("explorer")
      .args(["/select,", &path])
      .spawn()
      .map_err(|e| e.to_string())?;
  }

  #[cfg(target_os = "macos")]
  {
    Command::new("open")
      .args(["-R", &path])
      .spawn()
      .map_err(|e| e.to_string())?;
  }

  #[cfg(target_os = "linux")]
  {
    let path_buf = PathBuf::from(&path);
    let parent = path_buf.parent().unwrap_or(&path_buf);
    Command::new("xdg-open")
      .arg(parent)
      .spawn()
      .map_err(|e| e.to_string())?;
  }

  Ok(())
}

fn fix_relative_paths(content: &str, base_url: &str) -> String {
  let parent_dir = base_url
    .rsplit_once('/')
    .map(|(h, _)| h)
    .unwrap_or(base_url);

  // Replace relative image paths ![alt](path) with ![alt](parent_dir/path)
  // Only matches paths that don't start with http/https
  if let Ok(re) = Regex::new(r"!\[(.*?)\]\((.+?)\)") {
    re.replace_all(content, |caps: &regex::Captures| {
      let url = &caps[2];
      if url.starts_with("http") {
        caps[0].to_string()
      } else {
        format!("![{}]({}/{})", &caps[1], parent_dir, url)
      }
    })
    .to_string()
  } else {
    content.to_string()
  }
}

fn resolve_image_url(image: &str, base_url: &str) -> String {
  if image.starts_with("http://") || image.starts_with("https://") {
    return image.to_string();
  }

  let parent_dir = base_url
    .rsplit_once('/')
    .map(|(h, _)| h)
    .unwrap_or(base_url);

  let mut parts: Vec<&str> = parent_dir.split('/').collect();
  let image_parts: Vec<&str> = image.split('/').collect();

  for part in image_parts {
    if part == "." {
      continue;
    } else if part == ".." {
      if parts.len() > 3 {
        parts.pop();
      }
    } else {
      parts.push(part);
    }
  }

  parts.join("/")
}
