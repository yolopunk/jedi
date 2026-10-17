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
use tracing::{debug, error, info, warn};

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
    atomic_write(&self.cache_path(), json.as_bytes())
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
    debug!("Fetching metadata for: {}", slug);

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
  info!("Loaded {} wallpapers from cache", wallpapers.len());

  Ok(wallpapers)
}

#[tauri::command]
pub async fn sync_wallpapers<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
  info!("Syncing wallpapers...");
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

  info!("Items to process: {}", items_to_process.len());

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
          error!("Error processing {}: {}", item.name, e);
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

  info!("Sync complete. Processed {} items.", processed_count);
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
  force_refresh: Option<bool>,
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

  // 本地副本优先：Windows 注册表会长期引用这份本地文件，文件丢失 = 桌面
  // 黑屏。壁纸源是静态仓库的图片，同 URL 内容基本不变，能复用就复用，
  // force_refresh 才强制重新下载。
  let bytes: Vec<u8> = if force_refresh.unwrap_or(false) {
    download_image(&manager, &url).await?
  } else {
    match fs::read(&file_path) {
      Ok(bytes) if sniff_image_format(&bytes).is_some() => {
        debug!("reusing cached wallpaper: {}", file_path.display());
        bytes
      }
      Ok(bytes) => {
        warn!(
          "cached wallpaper {} is not a valid image ({} bytes), re-downloading",
          file_path.display(),
          bytes.len()
        );
        download_image(&manager, &url).await?
      }
      Err(_) => download_image(&manager, &url).await?,
    }
  };

  atomic_write(&file_path, &bytes)?;

  // Set wallpaper in a blocking task to avoid blocking the async runtime
  let path_str = file_path
    .to_str()
    .ok_or_else(|| "Failed to convert path to string".to_string())?
    .to_string();

  let mode_val = mode.unwrap_or(WallpaperMode::Crop);

  task::spawn_blocking(move || {
    wallpaper::set_from_path(&path_str).map_err(|e| e.to_string())?;
    // set_mode is unsupported on macOS via AppleScript, ignore the error
    let _ = wallpaper::set_mode(mode_val.into());
    // 壁纸确认应用后写 last-known-good 备份，供启动自愈兜底
    if let Some(ext) = sniff_image_format(&bytes) {
      if let Some(dir) = Path::new(&path_str).parent() {
        backup_wallpaper(dir, &bytes, ext);
      }
    }
    Ok::<(), String>(())
  })
  .await
  .map_err(|e| format!("Task join error: {}", e))??;
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

/// 按头部魔数识别 PNG/JPEG/GIF/BMP/WEBP。
/// 只能挡住"明显不是图片"的响应（HTML 错误页、空文件、头部损坏），
/// 头部完整但尾部截断的文件识别不了，由启动自愈兜底。
fn sniff_image_format(bytes: &[u8]) -> Option<&'static str> {
  if bytes.len() >= 8 && bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
    Some("png")
  } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
    Some("jpg")
  } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
    Some("gif")
  } else if bytes.len() >= 54 && bytes.starts_with(b"BM") {
    Some("bmp")
  } else if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == &b"WEBP"[..] {
    Some("webp")
  } else {
    None
  }
}

/// std::fs::rename 在 Windows 上不能覆盖已存在的目标，先挪开再试
fn replace_via_rename(tmp: &Path, dest: &Path) -> Result<(), String> {
  if fs::rename(tmp, dest).is_ok() {
    return Ok(());
  }
  let _ = fs::remove_file(dest);
  fs::rename(tmp, dest).map_err(|e| e.to_string())
}

/// 原子写文件：先写同目录 .tmp，再 rename 到位，读取方永远看不到半截文件
fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
  let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("file");
  let tmp = path.with_file_name(format!(".{file_name}.tmp"));
  fs::write(&tmp, bytes).map_err(|e| format!("failed to write {}: {e}", tmp.display()))?;
  replace_via_rename(&tmp, path).inspect_err(|_| {
    let _ = fs::remove_file(&tmp);
  })
}

/// 写 last-known-good 备份到 wallpapers/.backup/current.<ext>。
/// 备份只是额外兜底，失败只告警，不影响主流程。
fn backup_wallpaper(wallpapers_dir: &Path, bytes: &[u8], ext: &str) {
  let write = || -> Result<(), String> {
    let dir = wallpapers_dir.join(".backup");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let dest = dir.join(format!("current.{ext}"));
    let tmp = dir.join(format!("current.{ext}.tmp"));
    fs::write(&tmp, bytes).map_err(|e| e.to_string())?;
    replace_via_rename(&tmp, &dest)
  };
  if let Err(e) = write() {
    warn!("failed to write wallpaper backup: {e}");
  }
}

/// 取 last-known-good 备份（current.* 中 mtime 最新且能通过图像嗅探的）
fn find_backup(wallpapers_dir: &Path) -> Option<(Vec<u8>, &'static str)> {
  let dir = wallpapers_dir.join(".backup");
  let mut best: Option<(std::time::SystemTime, PathBuf, &'static str)> = None;
  for entry in fs::read_dir(&dir).ok()?.flatten() {
    let path = entry.path();
    let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
      continue;
    };
    if !name.starts_with("current.") {
      continue;
    }
    let Ok(bytes) = fs::read(&path) else {
      continue;
    };
    let Some(fmt) = sniff_image_format(&bytes) else {
      continue;
    };
    let Ok(modified) = entry.metadata().and_then(|m| m.modified()) else {
      continue;
    };
    if best.as_ref().map_or(true, |(t, _, _)| modified > *t) {
      best = Some((modified, path, fmt));
    }
  }
  let (_, path, fmt) = best?;
  fs::read(&path).ok().map(|bytes| (bytes, fmt))
}

/// 生成 24 位未压缩 BMP 纯色图，作为零依赖的最后兜底壁纸。
/// 颜色是占位的中性蓝灰，后续可换成品牌视觉资产。
const FALLBACK_RGB: [u8; 3] = [0x2B, 0x3A, 0x55];

fn solid_color_bmp(width: u32, height: u32, rgb: [u8; 3]) -> Vec<u8> {
  let row_len = (width as usize * 3).div_ceil(4) * 4;
  let pixel_size = row_len * height as usize;
  let file_size = 54 + pixel_size;
  let mut out = Vec::with_capacity(file_size);
  // BITMAPFILEHEADER
  out.extend_from_slice(b"BM");
  out.extend_from_slice(&(file_size as u32).to_le_bytes());
  out.extend_from_slice(&0u32.to_le_bytes());
  out.extend_from_slice(&54u32.to_le_bytes());
  // BITMAPINFOHEADER
  out.extend_from_slice(&40u32.to_le_bytes());
  out.extend_from_slice(&(width as i32).to_le_bytes());
  out.extend_from_slice(&(height as i32).to_le_bytes());
  out.extend_from_slice(&1u16.to_le_bytes());
  out.extend_from_slice(&24u16.to_le_bytes());
  out.extend_from_slice(&0u32.to_le_bytes());
  out.extend_from_slice(&(pixel_size as u32).to_le_bytes());
  out.extend_from_slice(&2835u32.to_le_bytes());
  out.extend_from_slice(&2835u32.to_le_bytes());
  out.extend_from_slice(&0u32.to_le_bytes());
  out.extend_from_slice(&0u32.to_le_bytes());
  // 像素数据按 BMP 约定自底向上、BGR 排列
  for _ in 0..height {
    let mut row = vec![0u8; row_len];
    for x in 0..width as usize {
      row[x * 3] = rgb[2];
      row[x * 3 + 1] = rgb[1];
      row[x * 3 + 2] = rgb[0];
    }
    out.extend_from_slice(&row);
  }
  out
}

/// 下载壁纸并做图像嗅探，明确拒绝把 HTML 错误页/截断响应写进缓存
async fn download_image(manager: &WallpaperManager, url: &str) -> Result<Vec<u8>, String> {
  let response = manager
    .client
    .get(url)
    .send()
    .await
    .map_err(|e| e.to_string())?;

  if !response.status().is_success() {
    return Err(format!("Failed to download image: {}", response.status()));
  }

  let bytes = response.bytes().await.map_err(|e| e.to_string())?.to_vec();
  if sniff_image_format(&bytes).is_none() {
    return Err(format!(
      "content from {url} ({} bytes) is not a recognizable image",
      bytes.len()
    ));
  }
  Ok(bytes)
}

/// 启动自愈：确保系统记录的壁纸路径指向真实存在的图片文件。
/// 数据目录被外部清空（清理工具/重装）后，注册表会长期指向已消失的
/// 文件，explorer 重读时就把桌面落成纯黑。恢复顺序：文件完好→不动；
/// .backup 备份→缓存里的原始 URL 重下→内置纯色图。
/// 在 main.rs 的 setup 里 spawn 执行。
pub async fn ensure_wallpaper_health<R: Runtime>(app: AppHandle<R>) -> Result<String, String> {
  let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
  let manager = WallpaperManager::new(app_data_dir);
  let wallpapers_dir = manager.wallpapers_dir();

  let current = task::spawn_blocking(|| wallpaper::get().map_err(|e| e.to_string()))
    .await
    .map_err(|e| format!("Task join error: {e}"))?;
  let current_path = match current {
    Ok(path) => PathBuf::from(path.trim()),
    // 部分平台读不到当前壁纸路径：保守起见不做任何恢复动作
    Err(e) => {
      info!("cannot read current wallpaper path, skipping health check: {e}");
      return Ok("skipped".to_string());
    }
  };

  if !current_path.as_os_str().is_empty() {
    if let Ok(bytes) = fs::read(&current_path) {
      if sniff_image_format(&bytes).is_some() {
        debug!("wallpaper intact: {}", current_path.display());
        return Ok("intact".to_string());
      }
      warn!(
        "wallpaper {} exists but is not a valid image, restoring",
        current_path.display()
      );
    } else {
      warn!("wallpaper {} is missing, restoring", current_path.display());
    }
  } else {
    warn!("no wallpaper path recorded by system, applying default");
  }

  // 源 1：last-known-good 备份
  let mut restore = find_backup(&wallpapers_dir).map(|(bytes, ext)| (bytes, ext, "backup"));

  // 源 2：元数据缓存里记录的原始 URL（按注册表里的文件名匹配）
  if restore.is_none() {
    let wanted = current_path
      .file_name()
      .and_then(|s| s.to_str())
      .map(str::to_string);
    if let Some(wanted) = wanted {
      let url = manager.load_cache().values().find_map(|entry| {
        let name = Path::new(&entry.item.url)
          .file_name()?
          .to_str()?
          .to_string();
        (name == wanted).then(|| entry.item.url.clone())
      });
      if let Some(url) = url {
        match download_image(&manager, &url).await {
          Ok(bytes) => {
            restore = sniff_image_format(&bytes).map(|ext| (bytes, ext, "re-download"));
          }
          Err(e) => warn!("re-download from cached url failed: {e}"),
        }
      }
    }
  }

  // 终极兜底：内置纯色图，恢复流程不允许以"没有可设置的图"告终
  let (bytes, ext, source) = restore.unwrap_or_else(|| {
    warn!("all restore sources failed, using built-in fallback color");
    (solid_color_bmp(64, 64, FALLBACK_RGB), "bmp", "built-in")
  });

  // 恢复文件写回系统记录的原路径（注册表条目保持不变）；注册表为空时写新文件
  let target = if current_path.as_os_str().is_empty() {
    wallpapers_dir.join(format!("jedi-default.{ext}"))
  } else {
    current_path.clone()
  };
  if let Some(parent) = target.parent() {
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
  }
  atomic_write(&target, &bytes)?;

  let target_str = target
    .to_str()
    .ok_or_else(|| "Failed to convert path to string".to_string())?
    .to_string();
  task::spawn_blocking(move || wallpaper::set_from_path(&target_str).map_err(|e| e.to_string()))
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

  backup_wallpaper(&wallpapers_dir, &bytes, ext);
  info!("wallpaper restored from {source}: {}", target.display());
  Ok(format!("restored from {source}"))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn sniffs_common_image_formats() {
    assert_eq!(
      sniff_image_format(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0]),
      Some("png")
    );
    assert_eq!(sniff_image_format(&[0xFF, 0xD8, 0xFF, 0xE0]), Some("jpg"));
    assert_eq!(sniff_image_format(b"GIF89a......"), Some("gif"));
    let bmp = vec![b'B', b'M']
      .into_iter()
      .chain(std::iter::repeat(0u8).take(52))
      .collect::<Vec<u8>>();
    assert_eq!(sniff_image_format(&bmp), Some("bmp"));
    assert_eq!(sniff_image_format(b"<html>404</html>"), None);
    assert_eq!(sniff_image_format(&[]), None);
  }

  #[test]
  fn sniffs_webp_riff() {
    let mut bytes = b"RIFF".to_vec();
    bytes.extend_from_slice(&[0, 0, 0, 0]);
    bytes.extend_from_slice(b"WEBP");
    assert_eq!(sniff_image_format(&bytes), Some("webp"));
  }

  #[test]
  fn solid_bmp_has_valid_header() {
    let bmp = solid_color_bmp(64, 64, [1, 2, 3]);
    assert_eq!(&bmp[0..2], &b"BM"[..]);
    let size = u32::from_le_bytes(bmp[2..6].try_into().unwrap()) as usize;
    assert_eq!(size, bmp.len());
    assert_eq!(bmp.len(), 54 + 64 * 192);
    // 首个像素在数据区开头，BMP 是 BGR 排列
    assert_eq!(&bmp[54..57], &[3u8, 2, 1][..]);
  }

  #[test]
  fn atomic_write_replaces_existing_content() {
    let dir = std::env::temp_dir();
    let path = dir.join(format!("jedi-atomic-test-{}.txt", std::process::id()));
    let _ = fs::remove_file(&path);

    atomic_write(&path, b"first").unwrap();
    atomic_write(&path, b"second-much-longer").unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"second-much-longer");

    let tmp = path.with_file_name(format!(
      ".{}.tmp",
      path.file_name().unwrap().to_str().unwrap()
    ));
    assert!(!tmp.exists());

    let _ = fs::remove_file(&path);
  }
}
