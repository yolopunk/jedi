// Agent process manager (docs §4.1, D1/D2/D3).
//
// Generic stdio process plumbing for external agent engines. This module
// knows NOTHING about any agent protocol: it spawns per SpawnSpec, frames
// stdout strictly on b'\n' (pi RPC warns that generic line readers break on
// U+2028/U+2029 inside JSON strings — a byte-level split is safe), forwards
// lines to the renderer over a Tauri IPC Channel (ordered, per-session), and
// owns the child lifecycle. Protocol parsing lives in the TS driver layer.

use crate::agent::detector::SpawnSpec;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::ipc::Channel;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::Mutex;

pub struct ProcManState(pub Arc<Mutex<HashMap<String, ProcHandle>>>);

pub struct ProcHandle {
  stdin: Mutex<ChildStdin>,
  /// Aborting the supervisor drops the Child; kill_on_drop then terminates
  /// the process tree (including the wsl.exe side of a WSL engine).
  supervisor: tokio::task::JoinHandle<()>,
}

fn build_command(spec: &SpawnSpec, args: &[String]) -> Command {
  #[cfg(target_os = "windows")]
  let resolved = if spec.kind == "windows-exe" || spec.kind == "windows-cmd" {
    crate::agent::detector::windows_launcher(&spec.exe)
  } else {
    None
  };
  #[cfg(target_os = "windows")]
  let spec = resolved.as_ref().unwrap_or(spec);
  let mut cmd = match spec.kind.as_str() {
    "wsl" => {
      // Quote exe+args into one login-shell command line so the WSL side
      // receives them verbatim.
      let mut line = shell_quote(&spec.exe);
      for a in args {
        line.push(' ');
        line.push_str(&shell_quote(a));
      }
      let mut c = Command::new("wsl.exe");
      c.arg("-d")
        .arg(spec.distro.as_deref().unwrap_or_default())
        .arg("--exec")
        .arg("bash")
        .arg("-lc")
        .arg(line);
      c
    }
    "windows-cmd" => {
      let mut c = Command::new(std::env::var_os("COMSPEC").unwrap_or_else(|| "cmd.exe".into()));
      c.arg("/D").arg("/S").arg("/C");
      #[cfg(target_os = "windows")]
      {
        use std::os::windows::process::CommandExt;
        // /S strips the outer quote pair; preserve the quoted executable
        // path and each argument inside it (notably paths with spaces).
        c.as_std_mut().raw_arg("\"");
        c.arg(&spec.exe).args(args);
        c.as_std_mut().raw_arg("\"");
      }
      #[cfg(not(target_os = "windows"))]
      c.arg(&spec.exe).args(args);
      c
    }
    _ => {
      let mut c = Command::new(&spec.exe);
      c.args(args);
      c
    }
  };
  #[cfg(target_os = "windows")]
  cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
  cmd
}

fn shell_quote(s: &str) -> String {
  format!("'{}'", s.replace('\'', "'\\''"))
}

#[tauri::command]
pub async fn agent_proc_spawn(
  id: String,
  spec: SpawnSpec,
  args: Option<Vec<String>>,
  cwd: Option<String>,
  env: Option<HashMap<String, String>>,
  on_event: Channel<Value>,
  state: tauri::State<'_, ProcManState>,
) -> Result<(), String> {
  let mut procs = state.0.lock().await;
  // Re-spawning an existing id replaces (and kills) the old process.
  if let Some(old) = procs.remove(&id) {
    old.supervisor.abort();
  }

  let args = args.unwrap_or_default();
  let mut cmd = build_command(&spec, &args);
  if let Some(dir) = cwd.as_deref().filter(|d| !d.is_empty()) {
    cmd.current_dir(dir);
  }
  if let Some(vars) = env {
    if spec.kind == "wsl" {
      // wsl.exe does NOT pass the Win32 environment into the distro: vars
      // must be listed in WSLENV to be forwarded (verified on a real box).
      let names: Vec<String> = vars.keys().map(|k| format!("{k}/u")).collect();
      cmd.env("WSLENV", names.join(":"));
    }
    cmd.envs(vars);
  }
  cmd
    .stdin(std::process::Stdio::piped())
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped())
    .kill_on_drop(true);

  let mut child: Child = cmd.spawn().map_err(|e| format!("failed to spawn agent {} ({}): {e}", spec.exe, spec.kind))?;
  let stdin = child.stdin.take().ok_or("no stdin pipe")?;
  let stdout = child.stdout.take().ok_or("no stdout pipe")?;
  let stderr = child.stderr.take().ok_or("no stderr pipe")?;

  // Strict byte-level framing: split on b'\n' only, strip a trailing b'\r'.
  async fn pump<R: tokio::io::AsyncRead + Unpin>(
    mut reader: BufReader<R>,
    chan: Channel<Value>,
    is_stderr: bool,
  ) {
    let mut buf: Vec<u8> = Vec::new();
    loop {
      buf.clear();
      match reader.read_until(b'\n', &mut buf).await {
        Ok(0) | Err(_) => break,
        Ok(_) => {
          if buf.last() == Some(&b'\n') {
            buf.pop();
          }
          if buf.last() == Some(&b'\r') {
            buf.pop();
          }
          if buf.is_empty() {
            continue;
          }
          let line = String::from_utf8_lossy(&buf).into_owned();
          let event = if is_stderr {
            json!({ "type": "stderr", "line": line })
          } else {
            json!({ "type": "line", "line": line })
          };
          let _ = chan.send(event);
        }
      }
    }
  }

  let stdout_task = tokio::spawn(pump(BufReader::new(stdout), on_event.clone(), false));
  let stderr_task = tokio::spawn(pump(BufReader::new(stderr), on_event.clone(), true));

  // Supervisor: wait for both pipes to close, reap the child, emit exit.
  let supervisor = tokio::spawn(async move {
    let _ = stdout_task.await;
    let _ = stderr_task.await;
    let code = child.wait().await.ok().and_then(|s| s.code());
    let _ = on_event.send(json!({ "type": "exit", "code": code }));
  });

  procs.insert(id, ProcHandle { stdin: Mutex::new(stdin), supervisor });
  Ok(())
}

#[tauri::command]
pub async fn agent_proc_stdin(
  id: String,
  line: String,
  state: tauri::State<'_, ProcManState>,
) -> Result<(), String> {
  let procs = state.0.lock().await;
  let handle = procs.get(&id).ok_or_else(|| format!("no such process: {id}"))?;
  let mut stdin = handle.stdin.lock().await;
  let map = |e: std::io::Error| format!("failed to write to agent stdin: {e}");
  stdin.write_all(line.as_bytes()).await.map_err(map)?;
  stdin.write_all(b"\n").await.map_err(map)?;
  stdin.flush().await.map_err(map)?;
  Ok(())
}

#[tauri::command]
pub async fn agent_proc_kill(
  id: String,
  state: tauri::State<'_, ProcManState>,
) -> Result<(), String> {
  let handle = state.0.lock().await.remove(&id);
  if let Some(h) = handle {
    h.supervisor.abort(); // drops Child -> kill_on_drop terminates the tree
  }
  Ok(())
}

#[cfg(all(test, target_os = "windows"))]
mod launcher_tests {
  use super::*;
  #[tokio::test]
  async fn installed_opencode_launcher_starts() {
    let dir = std::path::PathBuf::from(std::env::var("USERPROFILE").unwrap())
      .join("scoop/apps/fnm/current/aliases/default");
    let spec =
      match crate::agent::detector::windows_launcher(&dir.join("opencode").to_string_lossy()) {
        Some(spec) => spec,
        None => return, // Optional smoke test on machines without OpenCode.
      };
    let output = build_command(&spec, &["--version".into()])
      .output()
      .await
      .unwrap();
    assert!(
      output.status.success(),
      "{}",
      String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout)
      .to_ascii_lowercase()
      .contains("opencode"));
  }
  #[tokio::test]
  async fn cached_unix_shim_uses_windows_launcher_with_spaces() {
    let dir = tempfile::Builder::new()
      .prefix("agent launcher ")
      .tempdir()
      .unwrap();
    let shim = dir.path().join("opencode");
    std::fs::write(&shim, "#!/bin/sh\nexit 1\n").unwrap();
    std::fs::write(dir.path().join("opencode.cmd"), "@echo off\r\necho %~1\r\n").unwrap();
    let spec = SpawnSpec {
      kind: "windows-exe".into(),
      distro: None,
      exe: shim.to_string_lossy().into_owned(),
    };
    let output = build_command(&spec, &["hello world".into()])
      .output()
      .await
      .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(
      String::from_utf8_lossy(&output.stdout).trim(),
      "hello world"
    );
    assert_eq!(
      crate::agent::detector::windows_launcher(&spec.exe)
        .unwrap()
        .kind,
      "windows-cmd"
    );
  }
}
