use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::Manager;

#[derive(Serialize)]
pub struct OcrResult {
  pub text: String,
  pub language: String,
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
  tauri::async_runtime::spawn_blocking(move || {
    let path = validate_path(Path::new(&path), &roots)?;
    recognize(&path)
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

#[cfg(not(target_os = "windows"))]
fn recognize(_path: &Path) -> Result<OcrResult, String> {
  Err("unsupported".into())
}

#[cfg(target_os = "windows")]
fn recognize(path: &Path) -> Result<OcrResult, String> {
  use windows::{
    Graphics::Imaging::{BitmapAlphaMode, BitmapDecoder, BitmapPixelFormat},
    Media::Ocr::OcrEngine,
    Storage::Streams::{DataWriter, InMemoryRandomAccessStream},
    Win32::System::WinRT::{RoInitialize, RoUninitialize, RO_INIT_MULTITHREADED},
  };
  // Each blocking-pool thread owns its WinRT apartment for the duration of OCR.
  unsafe { RoInitialize(RO_INIT_MULTITHREADED) }.map_err(|_| "failed")?;
  struct Apartment;
  impl Drop for Apartment {
    fn drop(&mut self) {
      unsafe { RoUninitialize() };
    }
  }
  let _apartment = Apartment;
  let engine = OcrEngine::TryCreateFromUserProfileLanguages().map_err(|_| "languageUnavailable")?;
  let max = OcrEngine::MaxImageDimension().map_err(|_| "failed")?;
  let (w, h) = image::image_dimensions(path).map_err(|_| "unreadable")?;
  if w == 0 || h == 0 || w > max || h > max {
    return Err("tooLarge".into());
  }
  let bytes = std::fs::read(path).map_err(|_| "unreadable")?;
  let run = || -> windows::core::Result<OcrResult> {
    let stream = InMemoryRandomAccessStream::new()?;
    let writer = DataWriter::CreateDataWriter(&stream)?;
    writer.WriteBytes(&bytes)?;
    writer.StoreAsync()?.join()?;
    writer.DetachStream()?;
    stream.Seek(0)?;
    let decoder = BitmapDecoder::CreateAsync(&stream)?.join()?;
    let bitmap = decoder
      .GetSoftwareBitmapConvertedAsync(BitmapPixelFormat::Bgra8, BitmapAlphaMode::Ignore)?
      .join()?;
    let recognized = engine.RecognizeAsync(&bitmap)?.join()?;
    let lines = recognized.Lines()?;
    let mut text = Vec::new();
    for line in lines {
      text.push(line.Text()?.to_string());
    }
    Ok(OcrResult {
      text: text.join("\n"),
      language: engine.RecognizerLanguage()?.LanguageTag()?.to_string(),
    })
  };
  run().map_err(|_| "failed".into())
}

#[cfg(test)]
mod tests {
  use super::*;
  #[cfg(target_os = "windows")]
  #[test]
  #[ignore = "requires a Windows OCR language pack"]
  fn recognizes_text_and_blank_images() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("sample.png");
    std::fs::write(
      &path,
      include_bytes!("../../tests/fixtures/ocr-english.png"),
    )
    .unwrap();
    let result = recognize(&path).unwrap();
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
    assert!(recognize(&path).unwrap().text.trim().is_empty());
    std::fs::write(&path, b"invalid png").unwrap();
    assert!(recognize(&path).is_err());
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
