use reqwest::Client;
use std::fs;
use tauri::{AppHandle, Manager, Runtime};

use std::collections::HashMap;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct WallpaperItem {
  pub id: String,
  pub title: String,
  pub url: String,
  pub category: String,
  pub tags: Vec<String>,
  pub description: String,
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
  println!("Fetching wallpapers...");
  let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
  if !app_data_dir.exists() {
    let _ = fs::create_dir_all(&app_data_dir);
  }
  let cache_path = app_data_dir.join("wallpapers_cache.json");

  // Load cache
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
    println!("API request failed: {}", resp.status());
    return Err(format!("Failed to fetch guides list: {}", resp.status()));
  }

  let contents: Vec<GitHubContent> = resp.json().await.map_err(|e| e.to_string())?;
  println!("Found {} guides from GitHub", contents.len());

  let mut tasks = Vec::new();
  let mut new_cache: WallpaperCache = HashMap::new();

  for item in contents {
    if item.name.ends_with(".md") {
      if let Some(url) = item.download_url {
        // Check if we have valid cache
        if let Some(entry) = cache.get(&item.name) {
          if entry.sha == item.sha {
            new_cache.insert(item.name.clone(), entry.clone());
            continue;
          }
        }

        let client_clone = client.clone();
        let slug = item.name.replace(".md", "");
        let name = item.name.clone();
        let sha = item.sha.clone();
        tasks.push(tokio::spawn(async move {
          match fetch_guide_metadata(client_clone, slug.clone(), url.clone()).await {
            Some(wallpaper) => {
              println!("Successfully processed: {}", slug);
              Some((name, sha, wallpaper))
            }
            None => {
              println!("Failed to process: {}", slug);
              None
            }
          }
        }));
      }
    }
  }

  // Wait for all tasks
  for task in tasks {
    if let Ok(Some((name, sha, item))) = task.await {
      new_cache.insert(
        name,
        CacheEntry {
          sha,
          item: item.clone(),
        },
      );
    }
  }

  // Save updated cache
  if let Ok(json) = serde_json::to_string(&new_cache) {
    let _ = fs::write(&cache_path, json);
  }

  let mut wallpapers: Vec<WallpaperItem> =
    new_cache.values().map(|entry| entry.item.clone()).collect();
  println!("Total wallpapers processed: {}", wallpapers.len());

  // Remove duplicates by ID
  wallpapers.sort_by(|a, b| a.id.cmp(&b.id));
  wallpapers.dedup_by(|a, b| a.id == b.id);

  Ok(wallpapers)
}

async fn fetch_guide_metadata(client: Client, slug: String, url: String) -> Option<WallpaperItem> {
  println!("Fetching metadata for: {}", slug);
  match client.get(&url).send().await {
    Ok(resp) => {
      if resp.status().is_success() {
        if let Ok(text) = resp.text().await {
          if let Some((title, description, image, category, tags)) = parse_frontmatter(&text) {
            // Trust the image URL from frontmatter, let frontend handle load errors
            return Some(WallpaperItem {
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
            });
          } else {
            println!(
              "Failed to parse frontmatter for {}. Content start: {:.100}",
              slug, text
            );
          }
        } else {
          println!("Failed to get text content for {}", slug);
        }
      } else {
        println!("Failed to fetch URL {}: {}", url, resp.status());
      }
    }
    Err(e) => {
      println!("Network error fetching {}: {}", url, e);
    }
  }
  None
}

fn parse_frontmatter(text: &str) -> Option<(String, String, String, String, Vec<String>)> {
  let parts: Vec<&str> = text.split("---").collect();
  if parts.len() < 3 {
    return None;
  }
  let fm = parts[1];

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

    if line.starts_with("-") {
      if in_categories {
        if category.is_empty() {
          let raw_cat = line.trim_start_matches('-').trim();
          category = clean_value(raw_cat);
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
              // Simple inline array parsing
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

  Some((title, description, image, category, tags))
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

fn resolve_image_url(image: &str, base_url: &str) -> String {
  if image.starts_with("http://") || image.starts_with("https://") {
    return image.to_string();
  }

  // base_url is the full URL to the markdown file
  // We want the directory containing the file
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

  // Set wallpaper
  wallpaper::set_from_path(file_path.to_str().unwrap()).map_err(|e| e.to_string())?;
  wallpaper::set_mode(wallpaper::Mode::Crop).map_err(|e| e.to_string())?;

  Ok(())
}
