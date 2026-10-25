use rusto::{DetectTextResult, ImageSource, InitializeConfig, OcrRunOptions, RustO};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::path::BaseDirectory;
use tauri::Manager;

static OCR_ENGINE: Mutex<Option<RustO>> = Mutex::new(None);

struct OcrModelPaths {
  detector: PathBuf,
  recognizer: PathBuf,
  dictionary: PathBuf,
}

#[derive(Serialize)]
pub struct OcrResult {
  pub text: String,
  pub language: String,
  pub engine: String,
}

fn validate_path(path: &Path, roots: &[PathBuf]) -> Result<PathBuf, String> {
  let path = path.canonicalize().map_err(|_| "unreadable".to_string())?;
  let allowed = roots
    .iter()
    .any(|root| root.canonicalize().is_ok_and(|root| path.starts_with(root)));
  if !allowed || !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("png") {
    return Err("invalidPath".into());
  }
  if std::fs::metadata(&path).map_err(|_| "unreadable")?.len() > 32 * 1024 * 1024 {
    return Err("tooLarge".into());
  }
  Ok(path)
}

#[tauri::command]
pub async fn screenshot_ocr(app: tauri::AppHandle, path: String) -> Result<OcrResult, String> {
  let roots = vec![
    app
      .path()
      .picture_dir()
      .map_err(|_| "unreadable")?
      .join("jedi"),
    app
      .path()
      .app_data_dir()
      .map_err(|_| "unreadable")?
      .join("screenshots"),
  ];
  let models = resolve_model_paths(&app)?;
  tauri::async_runtime::spawn_blocking(move || {
    let path = validate_path(Path::new(&path), &roots)?;
    recognize(&path, &models)
  })
  .await
  .map_err(|_| "failed".to_string())?
}

#[tauri::command]
pub fn screenshot_copy_text(app: tauri::AppHandle, text: String) -> Result<(), String> {
  use tauri_plugin_clipboard_manager::ClipboardExt;
  if text.len() > 4 * 1024 * 1024 {
    return Err("tooLarge".into());
  }
  app
    .clipboard()
    .write_text(text)
    .map_err(|_| "copyFailed".into())
}

fn resolve_model_paths(app: &tauri::AppHandle) -> Result<OcrModelPaths, String> {
  let resolve = |file: &str| {
    app
      .path()
      .resolve(format!("ocr/{file}"), BaseDirectory::Resource)
      .map_err(|_| "engineUnavailable".to_string())
  };
  Ok(OcrModelPaths {
    detector: resolve("det.rten")?,
    recognizer: resolve("rec.rten")?,
    dictionary: resolve("dict.txt")?,
  })
}

fn recognize(path: &Path, models: &OcrModelPaths) -> Result<OcrResult, String> {
  let (width, height) = image::image_dimensions(path).map_err(|_| "unreadable")?;
  let pixels = u64::from(width) * u64::from(height);
  if width == 0 || height == 0 || width > 16_384 || height > 16_384 || pixels > 64_000_000 {
    return Err("tooLarge".into());
  }
  if [&models.detector, &models.recognizer, &models.dictionary]
    .iter()
    .any(|path| !path.is_file())
  {
    return Err("engineUnavailable".into());
  }
  let mut engine = OCR_ENGINE.lock().map_err(|_| "failed")?;
  if engine.is_none() {
    *engine = Some(
      RustO::initialize(InitializeConfig::ppv6(
        &models.detector,
        &models.recognizer,
        &models.dictionary,
      ))
      .map_err(|_| "engineUnavailable")?,
    );
  }
  let result = engine
    .as_mut()
    .ok_or("engineUnavailable")?
    .detect_text(
      &ImageSource::Path(path.to_path_buf()),
      &OcrRunOptions::default(),
    )
    .map_err(|_| "failed")?;
  let text = match result {
    DetectTextResult::Structured(lines) => lines
      .into_iter()
      .map(|line| line.text)
      .collect::<Vec<_>>()
      .join("\n"),
    DetectTextResult::Spatial(text) => text,
  };
  Ok(OcrResult {
    text: text.trim().to_string(),
    language: "auto · multilingual".into(),
    engine: "RustO · PP-OCRv6 Tiny".into(),
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn recognizes_text_and_blank_images() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("sample.png");
    std::fs::write(
      &path,
      include_bytes!("../../tests/fixtures/ocr-english.png"),
    )
    .unwrap();
    let models = OcrModelPaths {
      detector: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../node_modules/@rustors/model-ppocrv6-tiny/models/det.rten"),
      recognizer: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../node_modules/@rustors/model-ppocrv6-tiny/models/rec.rten"),
      dictionary: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../node_modules/@rustors/model-ppocrv6-tiny/models/dict.txt"),
    };
    let result = recognize(&path, &models).unwrap();
    assert!(
      result.text.contains("JEDI"),
      "expected JEDI, got {}",
      result.text
    );
    assert!(result.text.contains("12345"));
    assert!(!result.language.is_empty());
    image::RgbaImage::from_pixel(400, 100, image::Rgba([255, 255, 255, 255]))
      .save(&path)
      .unwrap();
    assert!(recognize(&path, &models).unwrap().text.trim().is_empty());
    std::fs::write(&path, b"invalid png").unwrap();
    assert!(recognize(&path, &models).is_err());
  }

  #[test]
  fn only_gallery_png_files_are_allowed() {
    let root = tempfile::tempdir().unwrap();
    let gallery = root.path().join("jedi");
    std::fs::create_dir(&gallery).unwrap();
    let valid = gallery.join("capture.png");
    std::fs::write(&valid, b"test").unwrap();
    assert!(validate_path(&valid, &[gallery.clone()]).is_ok());
    let outside = root.path().join("private.png");
    std::fs::write(&outside, b"test").unwrap();
    assert_eq!(
      validate_path(&outside, &[gallery.clone()]).unwrap_err(),
      "invalidPath"
    );
    assert_eq!(
      validate_path(&gallery, &[gallery.clone()]).unwrap_err(),
      "invalidPath"
    );
    assert_eq!(
      validate_path(&gallery.join("missing.png"), &[gallery]).unwrap_err(),
      "unreadable"
    );
  }
}
