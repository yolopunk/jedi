// Filesystem primitives exposed to the AI agent's FILESYS tool.
//
// Backs `src/agent/tools/filesystem.ts`: read / write / exists on absolute
// paths. Every mutating call goes through the tool risk gate (write-level
// confirmation) on the frontend; the commands themselves stay deliberately
// dumb so the confirmation policy lives in one place.

use std::fs;
use std::path::Path;

/// Cap file reads so a huge file cannot blow up the model context or the
/// IPC payload. Matches the spirit of the terminal tool's output cap.
const MAX_READ_CHARS: usize = 200_000;

#[tauri::command]
pub fn read_file(path: String) -> Result<String, String> {
  let p = Path::new(&path);
  if !p.is_file() {
    return Err(format!("file not found: {path}"));
  }
  let bytes = fs::read(p).map_err(|e| format!("failed to read {path}: {e}"))?;
  let mut text = String::from_utf8_lossy(&bytes).into_owned();
  if text.chars().count() > MAX_READ_CHARS {
    let head: String = text.chars().take(MAX_READ_CHARS).collect();
    text = format!("{head}\n...(truncated)");
  }
  Ok(text)
}

#[tauri::command]
pub fn write_file(path: String, content: String) -> Result<(), String> {
  let p = Path::new(&path);
  if let Some(parent) = p.parent() {
    fs::create_dir_all(parent).map_err(|e| format!("failed to create parent dir: {e}"))?;
  }
  fs::write(p, content).map_err(|e| format!("failed to write {path}: {e}"))
}

#[tauri::command]
pub fn file_exists(path: String) -> bool {
  Path::new(&path).exists()
}
