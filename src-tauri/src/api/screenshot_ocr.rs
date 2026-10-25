use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::Manager;

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
      engine: "Windows OCR".into(),
    })
  };
  run().map_err(|_| "failed".into())
}

#[cfg(target_os = "macos")]
fn recognize(path: &Path) -> Result<OcrResult, String> {
  use objc2::rc::Retained;
  use objc2::runtime::AnyObject;
  use objc2::ClassType;
  use objc2_foundation::{NSArray, NSData, NSDictionary};
  use objc2_vision::{
    VNImageOption, VNImageRequestHandler, VNRecognizeTextRequest, VNRequest,
    VNRequestTextRecognitionLevel,
  };

  let version = std::process::Command::new("sw_vers")
    .arg("-productVersion")
    .output()
    .map_err(|_| "engineUnavailable")?;
  let version_text = String::from_utf8_lossy(&version.stdout);
  let mut parts = version_text.trim().split('.');
  let major = parts
    .next()
    .and_then(|part| part.parse::<u32>().ok())
    .unwrap_or(0);
  let minor = parts
    .next()
    .and_then(|part| part.parse::<u32>().ok())
    .unwrap_or(0);
  if !version.status.success() || major < 10 || (major == 10 && minor < 15) {
    return Err("engineUnavailable".into());
  }

  let bytes = std::fs::read(path).map_err(|_| "unreadable")?;
  let data = NSData::with_bytes(&bytes);
  let options = NSDictionary::<VNImageOption, AnyObject>::new();
  let handler =
    VNImageRequestHandler::initWithData_options(VNImageRequestHandler::alloc(), &data, &options);
  let request = VNRecognizeTextRequest::new();
  request.setRecognitionLevel(VNRequestTextRecognitionLevel::Accurate);
  request.setUsesLanguageCorrection(true);
  if major >= 13 {
    request.setAutomaticallyDetectsLanguage(true);
  }
  let base: Retained<VNRequest> = request.clone().into_super().into_super();
  handler
    .performRequests_error(&NSArray::from_retained_slice(&[base]))
    .map_err(|_| "failed")?;

  let mut lines = Vec::new();
  if let Some(results) = request.results() {
    for observation in results.iter() {
      if let Some(candidate) = observation.topCandidates(1).iter().next() {
        lines.push(candidate.string().to_string());
      }
    }
  }
  Ok(OcrResult {
    text: lines.join("\n"),
    language: if major >= 13 { "auto" } else { "system" }.into(),
    engine: "Apple Vision".into(),
  })
}

#[cfg(target_os = "linux")]
fn recognize(path: &Path) -> Result<OcrResult, String> {
  use std::io::ErrorKind;
  use std::process::Command;

  let available = Command::new("tesseract").arg("--list-langs").output();
  let available = match available {
    Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout).into_owned(),
    Ok(_) => return Err("engineUnavailable".into()),
    Err(error) if error.kind() == ErrorKind::NotFound => return Err("engineUnavailable".into()),
    Err(_) => return Err("failed".into()),
  };
  let languages = choose_tesseract_languages(&available).ok_or("languageUnavailable")?;
  let output = Command::new("tesseract")
    .arg(path)
    .arg("stdout")
    .arg("--psm")
    .arg("6")
    .arg("-l")
    .arg(&languages)
    .output()
    .map_err(|_| "failed")?;
  if !output.status.success() {
    return Err("failed".into());
  }
  Ok(OcrResult {
    text: String::from_utf8_lossy(&output.stdout).trim().to_string(),
    language: languages,
    engine: "Tesseract".into(),
  })
}

#[cfg(any(test, target_os = "linux"))]
fn choose_tesseract_languages(output: &str) -> Option<String> {
  let languages: Vec<&str> = output
    .lines()
    .map(str::trim)
    .filter(|line| !line.is_empty() && !line.starts_with("List of available languages"))
    .collect();
  let preferred: Vec<&str> = ["chi_sim", "chi_tra", "eng"]
    .into_iter()
    .filter(|candidate| languages.contains(candidate))
    .collect();
  if preferred.is_empty() {
    languages.first().map(|language| (*language).to_string())
  } else {
    Some(preferred.join("+"))
  }
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn recognize(_path: &Path) -> Result<OcrResult, String> {
  Err("engineUnavailable".into())
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

  #[test]
  fn prefers_chinese_and_english_tesseract_languages() {
    let output =
      "List of available languages in /usr/share/tessdata/ (4):\nosd\neng\nchi_tra\nchi_sim\n";
    assert_eq!(
      choose_tesseract_languages(output).as_deref(),
      Some("chi_sim+chi_tra+eng")
    );
    assert_eq!(choose_tesseract_languages("deu\n").as_deref(), Some("deu"));
    assert_eq!(choose_tesseract_languages(""), None);
  }
}
