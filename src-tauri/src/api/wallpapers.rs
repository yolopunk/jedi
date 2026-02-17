use reqwest::Client;
use std::collections::HashMap;
use std::fs;
use tauri::{AppHandle, Emitter, Manager, Runtime};
use regex::Regex;
use wallpaper;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct WallpaperItem {
  pub id: String,
  pub title: String,
  pub url: String,
  pub category: String,
  pub tags: Vec<String>,
  pub description: String,
  pub content: String,
}

const GUIDES_API_URL: &str =
  "https://api.github.com/repos/ByteByteGoHq/system-design-101/contents/data/guides";

#[derive(serde::Deserialize)]
struct GitHubContent {
  name: String,
  download_url: Option<String>,
  sha: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct CacheEntry {
  sha: String,
  item: WallpaperItem,
}

type WallpaperCache = HashMap<String, CacheEntry>;

#[tauri::command]
pub async fn get_wallpapers<R: Runtime>(app: AppHandle<R>) -> Result<Vec<WallpaperItem>, String> {
  println!("Fetching wallpapers from cache...");
  let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;

  let cache_path = app_data_dir.join("wallpapers_cache.json");

  if cache_path.exists() {
    match fs::read_to_string(&cache_path) {
      Ok(content) => match serde_json::from_str::<WallpaperCache>(&content) {
        Ok(cache) => {
          let mut wallpapers: Vec<WallpaperItem> =
            cache.values().map(|entry| entry.item.clone()).collect();
          wallpapers.sort_by(|a, b| a.id.cmp(&b.id));
          wallpapers.dedup_by(|a, b| a.id == b.id);
          println!("Loaded {} wallpapers from cache", wallpapers.len());
          return Ok(wallpapers);
        }
        Err(e) => {
          eprintln!("Failed to parse wallpapers cache: {}", e);
        }
      },
      Err(e) => {
        eprintln!("Failed to read wallpapers cache file: {}", e);
      }
    }
  }

  println!("No cache found or empty cache");
  Ok(Vec::new())
}

#[tauri::command]
pub async fn sync_wallpapers<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
  println!("Syncing wallpapers...");
  let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
  if !app_data_dir.exists() {
    let _ = fs::create_dir_all(&app_data_dir);
  }

  let cache_path = app_data_dir.join("wallpapers_cache.json");

  let cache: WallpaperCache = if cache_path.exists() {
    fs::read_to_string(&cache_path)
      .ok()
      .and_then(|content| serde_json::from_str(&content).ok())
      .unwrap_or_default()
  } else {
    HashMap::new()
  };

  let client = Client::builder()
    .user_agent("Jedi-App")
    .build()
    .map_err(|e| e.to_string())?;

  let resp = client
    .get(GUIDES_API_URL)
    .send()
    .await
    .map_err(|e| e.to_string())?;

  if !resp.status().is_success() {
    return Err(format!("Failed to fetch guides list: {}", resp.status()));
  }

  let contents: Vec<GitHubContent> = resp.json().await.map_err(|e| e.to_string())?;

  let mut items_to_process = Vec::new();
  let mut new_cache = cache.clone();

  for item in contents {
    if item.name.ends_with(".md") {
      if let Some(url) = item.download_url {
        if let Some(entry) = cache.get(&item.name) {
          if entry.sha == item.sha {
            continue;
          }
        }
        items_to_process.push((item.name, item.sha, url));
      }
    }
  }

  println!("Items to process: {}", items_to_process.len());

  let chunk_size = 5;
  for chunk in items_to_process.chunks(chunk_size) {
    let mut tasks = Vec::new();

    for (name, sha, url) in chunk {
      let client_clone = client.clone();
      let name = name.clone();
      let sha = sha.clone();
      let url = url.clone();
      let slug = name.replace(".md", "");

      tasks.push(tokio::spawn(async move {
        match fetch_guide_metadata(client_clone, slug.clone(), url.clone()).await {
          Ok(wallpaper) => Some((name, sha, wallpaper)),
          Err(err) => {
            eprintln!("Failed to fetch guide metadata for {}: {}", slug, err);
            None
          }
        }
      }));
    }

    let mut batch_items = Vec::new();

    for task in tasks {
      if let Ok(Some((name, sha, item))) = task.await {
        new_cache.insert(
          name,
          CacheEntry {
            sha,
            item: item.clone(),
          },
        );
        batch_items.push(item);
      }
    }

    if !batch_items.is_empty() {
      println!("Emitting batch of {} items", batch_items.len());
      let _ = app.emit("wallpaper-batch", batch_items);
    }
  }

  if let Ok(json) = serde_json::to_string(&new_cache) {
    let _ = fs::write(&cache_path, json);
  }

  let _ = app.emit("wallpaper-sync-complete", ());

  Ok(())
}

#[tauri::command]
pub async fn set_desktop_wallpaper<R: Runtime>(
    app: AppHandle<R>,
    url: String,
) -> Result<(), String> {
  let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
  if !app_data_dir.exists() {
    fs::create_dir_all(&app_data_dir).map_err(|e| e.to_string())?;
  }
  let filename = url.split('/').last().unwrap_or("wallpaper.jpg");
  let file_path = app_data_dir.join("wallpapers").join(filename);

  // Create wallpapers dir if not exists
  if let Some(parent) = file_path.parent() {
    if !parent.exists() {
      fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
  }

  // Download file
  let client = Client::new();
  let response = client.get(&url).send().await.map_err(|e| e.to_string())?;

  if !response.status().is_success() {
    return Err(format!("Failed to download image: {}", response.status()));
  }

  let content = response.bytes().await.map_err(|e| e.to_string())?;
  fs::write(&file_path, &content).map_err(|e| e.to_string())?;

  // Set wallpaper using Windows API (for better compatibility)
  #[cfg(target_os = "windows")]
  {
    if let Some(path_str) = file_path.to_str() {
      if let Err(e) = wallpaper::set_from_path(path_str) {
        eprintln!("Failed to set wallpaper on Windows: {}", e);
        return Err(e.to_string());
      }
    } else {
      return Err("Failed to convert wallpaper path to string".to_string());
    }
  }

  Ok(())
}

async fn fetch_guide_metadata(client: Client, slug: String, url: String) -> Result<WallpaperItem, String> {
  println!("Fetching metadata for: {}", slug);
  let resp = client
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

  let (title, description, image, category, tags, content) =
    parse_frontmatter(&text).ok_or_else(|| format!("Failed to parse frontmatter for {}", slug))?;

  let content = fix_relative_paths(&content, &url);

  Ok(WallpaperItem {
    id: slug,
    title,
    url: resolve_image_url(&image, &url),
    category: if category.is_empty() {
      "General".to_string()
    } else {
      category
    },
    tags,
    description,
    content,
  })
}

fn parse_frontmatter(text: &str) -> Option<(String, String, String, String, Vec<String>, String)> {
  let parts: Vec<&str> = text.split("---").collect();
  if parts.len() < 3 {
    return None;
  }
  let fm = parts[1];
  let content = parts[2..].join("---");

  let mut title = String::new();
  let mut description = String::new();
  let mut image = String::new();
  let mut category = String::new();
  let mut tags = Vec::new();

  let mut in_categories = false;
  let mut in_tags = false;

  for line in fm.lines() {
    let line = line.trim();
    if line.is_empty() {
      continue;
    }

    if line.starts_with('-') {
      if in_categories {
        if category.is_empty() {
          let raw_cat = line.trim_start_matches('-').trim();
          let c = clean_value(raw_cat);
          if !c.is_empty() {
            category = c;
          }
        }
      } else if in_tags {
        let raw_tag = line.trim_start_matches('-').trim();
        let tag = clean_value(raw_tag);
        if !tag.is_empty() {
          tags.push(tag);
        }
      }
      continue;
    }

    if let Some((key, val)) = line.split_once(':') {
      let key = key.trim().to_lowercase();
      let val_str = val.trim();
      let val_cleaned = clean_value(val_str);

      match key.as_str() {
        "title" => {
          title = val_cleaned;
          in_categories = false;
          in_tags = false;
        }
        "description" => {
          description = val_cleaned;
          in_categories = false;
          in_tags = false;
        }
        "image" => {
          image = val_cleaned;
          in_categories = false;
          in_tags = false;
        }
        "categories" => {
          in_categories = true;
          in_tags = false;
          if !val_str.is_empty() {
            if val_str.starts_with('[') {
              let cats: Vec<String> = val_str
                .trim_matches(|c| c == '[' || c == ']')
                .split(',')
                .map(|s| clean_value(s))
                .filter(|s| !s.is_empty())
                .collect();
              if let Some(first) = cats.first() {
                category = first.clone();
                in_categories = false;
              }
            } else {
              category = val_cleaned;
              in_categories = false;
            }
          }
        }
        "tags" => {
          in_tags = true;
          in_categories = false;
          if !val_str.is_empty() {
            if val_str.starts_with('[') {
              let ts: Vec<String> = val_str
                .trim_matches(|c| c == '[' || c == ']')
                .split(',')
                .map(|s| clean_value(s))
                .filter(|s| !s.is_empty())
                .collect();
              tags.extend(ts);
              in_tags = false;
            } else {
              let tag = val_cleaned;
              if !tag.is_empty() {
                tags.push(tag);
              }
              in_tags = false;
            }
          }
        }
        _ => {
          in_categories = false;
          in_tags = false;
        }
      }
    }
  }

  if title.is_empty() || image.is_empty() {
    return None;
  }

  Some((title, description, image, category, tags, content))
}

fn clean_value(val: &str) -> String {
  let val = val.trim();
  if (val.starts_with('"') && val.ends_with('"')) || (val.starts_with('\'') && val.ends_with('\''))
  {
    if val.len() >= 2 {
      val[1..val.len() - 1].to_string()
    } else {
      val.to_string()
    }
  } else {
    val.to_string()
  }
}

fn fix_relative_paths(content: &str, base_url: &str) -> String {
  let parent_dir = base_url
        .rsplit_once('/')
        .map(|(h, _)| h)
        .unwrap_or(base_url);

  // Replace relative image paths ![alt](path) with ![alt](parent_dir/path)
  // Only matches paths that don't start with http/https
  if let Ok(re) = Regex::new(r"!\[(.*?)\]\(((?!http).+?)\)") {
        re.replace_all(content, |caps: &regex::Captures| {
                format!("![{}]({}/{})", &caps[1], parent_dir, &caps[2])
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

  // base_url is a full URL to the markdown file
  // We want to directory containing the file
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
                // Prevent popping protocol/domain
                parts.pop();
            }
        } else {
            parts.push(part);
        }
    }

  parts.join("/")
}
