// Generic stdio process plumbing for external engines. Protocol parsing lives
// in the TS drivers. Frame only on b'\n' (U+2028/U+2029 may occur inside JSON).

use crate::agent::detector::SpawnSpec;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::ipc::Channel;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::{oneshot, Mutex, Notify};
use tokio::task::JoinHandle;

pub struct ProcManState(pub Arc<Mutex<HashMap<String, ProcHandle>>>);

pub struct ProcHandle {
  stdin: Arc<Mutex<ChildStdin>>,
  stop: Option<oneshot::Sender<()>>,
  supervisor: Option<JoinHandle<Result<(), String>>>,
  generation: Arc<()>,
}

impl Drop for ProcHandle {
  fn drop(&mut self) {
    // Dropping the application state also asks supervisors to terminate/reap.
    // Dropping a JoinHandle detaches it; aborting it would skip tree cleanup.
    if let Some(stop) = self.stop.take() {
      let _ = stop.send(());
    }
  }
}

async fn stop_handle(mut handle: ProcHandle) -> Result<(), String> {
  if let Some(stop) = handle.stop.take() {
    let _ = stop.send(());
  }
  if let Some(supervisor) = handle.supervisor.take() {
    supervisor
      .await
      .map_err(|e| format!("agent supervisor failed: {e}"))??;
  }
  Ok(())
}

fn shell_quote(s: &str) -> String {
  format!("'{}'", s.replace('\'', "'\\''"))
}

fn validate_process_id(id: &str) -> Result<(), String> {
  if id.is_empty() || id.len() > 512 || id.chars().any(char::is_control) {
    return Err("invalid agent process id".into());
  }
  Ok(())
}

fn wsl_command(spec: &SpawnSpec) -> Command {
  let mut command = Command::new("wsl.exe");
  if let Some(distro) = spec.distro.as_deref().filter(|d| !d.is_empty()) {
    command.arg("-d").arg(distro);
  }
  #[cfg(target_os = "windows")]
  command.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
  command
}

// Keep the original helper signature for existing launcher tests.
#[cfg(test)]
fn build_command(spec: &SpawnSpec, args: &[String]) -> Command {
  build_command_at(spec, args, None, None)
}

fn build_command_at(
  spec: &SpawnSpec,
  args: &[String],
  cwd: Option<&str>,
  wsl_marker: Option<&str>,
) -> Command {
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
      let mut line = String::new();
      // Login startup files may cd. Restore the validated cwd after loading
      // PATH as well as passing --cd to WSL before --exec.
      if let Some(dir) = cwd {
        line.push_str("cd -- ");
        line.push_str(&shell_quote(dir));
        line.push_str(" && ");
      }
      line.push_str("exec ");
      line.push_str(&shell_quote(&spec.exe));
      for arg in args {
        line.push(' ');
        line.push_str(&shell_quote(arg));
      }
      let mut command = wsl_command(spec);
      if let Some(dir) = cwd {
        command.arg("--cd").arg(dir);
      }
      command.arg("--exec");
      if let Some(marker) = wsl_marker {
        // wsl.exe is a Windows relay: killing it alone does NOT reliably kill
        // the Linux children. Give the agent its own process group and report
        // the group ID before login startup files can delay initialization.
        command
          .args(["setsid", "--wait", "bash", "--noprofile", "--norc", "-c"])
          .arg(format!(
            "printf '%s%s\\n' {} \"$$\" >&2; exec bash -lc \"$1\"",
            shell_quote(marker)
          ))
          .arg("jedi-agent")
          .arg(line);
      } else {
        command.args(["bash", "-lc"]).arg(line);
      }
      command
    }
    "windows-cmd" => {
      let mut command =
        Command::new(std::env::var_os("COMSPEC").unwrap_or_else(|| "cmd.exe".into()));
      command.arg("/D").arg("/S").arg("/C");
      #[cfg(target_os = "windows")]
      {
        use std::os::windows::process::CommandExt;
        // /S strips the outer pair, preserving the quoted executable and args.
        command.as_std_mut().raw_arg("\"");
        command.arg(&spec.exe).args(args);
        command.as_std_mut().raw_arg("\"");
      }
      #[cfg(not(target_os = "windows"))]
      command.arg(&spec.exe).args(args);
      command
    }
    _ => {
      let mut command = Command::new(&spec.exe);
      command.args(args);
      command
    }
  };
  if spec.kind != "wsl" {
    if let Some(dir) = cwd {
      cmd.current_dir(dir);
    }
    #[cfg(unix)]
    cmd.process_group(0);
  }
  #[cfg(target_os = "windows")]
  cmd.creation_flags(0x0800_0000);
  cmd
}

fn windows_absolute(path: &str) -> bool {
  let bytes = path.as_bytes();
  (bytes.len() >= 3
    && bytes[0].is_ascii_alphabetic()
    && bytes[1] == b':'
    && (bytes[2] == b'\\' || bytes[2] == b'/'))
    || path.starts_with("\\\\")
    || path.starts_with("//")
}

/// Win32 canonicalization adds \\?\; most agent CLIs expect conventional paths.
fn agent_path(path: PathBuf) -> String {
  let text = path.to_string_lossy().into_owned();
  #[cfg(target_os = "windows")]
  {
    if let Some(rest) = text.strip_prefix("\\\\?\\UNC\\") {
      return format!("\\\\{rest}");
    }
    if let Some(rest) = text.strip_prefix("\\\\?\\") {
      return rest.to_string();
    }
  }
  text
}

async fn native_cwd(dir: &str) -> Result<String, String> {
  if !Path::new(dir).is_absolute() {
    return Err("agent working directory must be absolute".into());
  }
  let canonical = tokio::fs::canonicalize(dir)
    .await
    .map_err(|e| format!("invalid agent working directory {dir}: {e}"))?;
  if !tokio::fs::metadata(&canonical)
    .await
    .map_err(|e| format!("invalid agent working directory {dir}: {e}"))?
    .is_dir()
  {
    return Err(format!("agent working directory is not a directory: {dir}"));
  }
  Ok(agent_path(canonical))
}

async fn wsl_capture(spec: &SpawnSpec, args: &[&str]) -> Result<String, String> {
  let mut command = wsl_command(spec);
  command
    .arg("--exec")
    .args(args)
    .stdin(std::process::Stdio::null())
    .kill_on_drop(true);
  let output = tokio::time::timeout(Duration::from_secs(5), command.output())
    .await
    .map_err(|_| "WSL working directory validation timed out".to_string())?
    .map_err(|e| format!("WSL working directory validation failed: {e}"))?;
  if !output.status.success() {
    return Err(format!(
      "invalid WSL working directory: {}",
      String::from_utf8_lossy(&output.stderr).trim()
    ));
  }
  let text = String::from_utf8(output.stdout)
    .map_err(|_| "WSL working directory is not valid UTF-8".to_string())?;
  let path = text.trim_end_matches(['\r', '\n']);
  if !path.starts_with('/') || path.contains(['\0', '\r', '\n']) {
    return Err("WSL did not return a valid absolute working directory".into());
  }
  Ok(path.to_string())
}

async fn resolve_cwd(spec: &SpawnSpec, cwd: Option<&str>) -> Result<Option<String>, String> {
  let Some(dir) = cwd else {
    return Ok(None); // Legacy callers keep their inherited cwd unchanged.
  };
  if dir.trim().is_empty() || dir.contains(['\0', '\r', '\n']) {
    return Err("agent working directory must be a nonempty absolute path".into());
  }
  if spec.kind != "wsl" {
    return native_cwd(dir).await.map(Some);
  }
  let converted;
  let wsl_dir = if windows_absolute(dir) {
    // Resolve junctions/relative components on Windows before asking the
    // selected distro about its actual drvfs mounts (not always /mnt/c).
    let canonical = native_cwd(dir).await?;
    converted = wsl_capture(spec, &["wslpath", "-a", "-u", &canonical]).await?;
    converted.as_str()
  } else if dir.starts_with('/') {
    dir
  } else {
    return Err("WSL working directory must be absolute".into());
  };
  // All user paths are argv values, never interpolated into shell source.
  wsl_capture(
    spec,
    &[
      "bash",
      "--noprofile",
      "--norc",
      "-c",
      "cd -P -- \"$1\" && test -d . && pwd -P",
      "jedi-cwd",
      wsl_dir,
    ],
  )
  .await
  .map(Some)
}

struct WslGroup {
  marker: String,
  pid: AtomicU32,
  ready: Notify,
}

async fn pump<R: tokio::io::AsyncRead + Unpin>(
  mut reader: BufReader<R>,
  channel: Channel<Value>,
  is_stderr: bool,
  wsl_group: Option<Arc<WslGroup>>,
) {
  let mut buf = Vec::new();
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
        if let Some(group) = wsl_group.as_ref().filter(|_| is_stderr) {
          if let Some(pid) = line
            .strip_prefix(&group.marker)
            .and_then(|pid| pid.parse::<u32>().ok())
            .filter(|pid| *pid > 1)
          {
            // Accept only the first bootstrap line, not agent output later.
            if group
              .pid
              .compare_exchange(0, pid, Ordering::AcqRel, Ordering::Acquire)
              .is_ok()
            {
              group.ready.notify_one();
              continue;
            }
          }
        }
        let event = if is_stderr {
          json!({ "type": "stderr", "line": line })
        } else {
          json!({ "type": "line", "line": line })
        };
        if channel.send(event).is_err() {
          break;
        }
      }
    }
  }
}

async fn terminate_wsl_group(spec: &SpawnSpec, group: &WslGroup) -> Result<(), String> {
  if group.pid.load(Ordering::Acquire) == 0 {
    // The bootstrap reports before login startup. Allow a short window when
    // Stop raced with process creation before the pipes were first polled.
    let _ = tokio::time::timeout(Duration::from_secs(1), group.ready.notified()).await;
  }
  let pid = group.pid.load(Ordering::Acquire);
  if pid <= 1 {
    return Err("WSL agent process group was not reported before termination".into());
  }
  let mut command = wsl_command(spec);
  command
    .args(["--exec", "/bin/kill", "-KILL", "--"])
    .arg(format!("-{pid}"))
    .stdin(std::process::Stdio::null())
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::null())
    .kill_on_drop(true);
  // A nonzero exit also means the group already exited; either is settled.
  tokio::time::timeout(Duration::from_secs(3), command.status())
    .await
    .map_err(|_| "WSL agent group termination timed out".to_string())?
    .map_err(|e| format!("failed to terminate WSL agent group: {e}"))?;
  Ok(())
}

async fn terminate_child_tree(child: &mut Child) -> Result<(), String> {
  if let Some(pid) = child.id() {
    #[cfg(target_os = "windows")]
    {
      // kill_on_drop and Child::kill kill only the launcher, NOT its tree.
      let mut kill = Command::new("taskkill.exe");
      kill
        .args(["/PID", &pid.to_string(), "/T", "/F"])
        .creation_flags(0x0800_0000)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true);
      let result = tokio::time::timeout(Duration::from_secs(5), kill.status()).await;
      // Always reap the direct child, even when the tree helper fails.
      let _ = child.start_kill();
      child
        .wait()
        .await
        .map_err(|e| format!("failed to reap agent: {e}"))?;
      return match result {
        Ok(Ok(status)) if status.success() => Ok(()),
        // taskkill may race a normal exit. The reaped child is gone, but do
        // not claim descendant cleanup when the OS helper could not verify it.
        Ok(Ok(_)) => Err("Windows could not confirm termination of the agent process tree".into()),
        Ok(Err(e)) => Err(format!(
          "failed to terminate Windows agent process tree: {e}"
        )),
        Err(_) => Err("Windows agent process tree termination timed out".into()),
      };
    }
    #[cfg(unix)]
    {
      let _ = tokio::time::timeout(
        Duration::from_secs(3),
        Command::new("/bin/kill")
          .args(["-KILL", "--", &format!("-{pid}")])
          .stdin(std::process::Stdio::null())
          .stdout(std::process::Stdio::null())
          .stderr(std::process::Stdio::null())
          .kill_on_drop(true)
          .status(),
      )
      .await;
    }
    #[cfg(not(any(target_os = "windows", unix)))]
    let _ = pid;
  }
  let _ = child.start_kill();
  child
    .wait()
    .await
    .map_err(|e| format!("failed to reap agent: {e}"))?;
  Ok(())
}

async fn drain_pump(mut task: JoinHandle<()>) {
  // A descendant may hold a pipe after its launcher exited. Never strand a
  // detached pump indefinitely or wait for pipe EOF before reaping the child.
  if tokio::time::timeout(Duration::from_millis(500), &mut task)
    .await
    .is_err()
  {
    task.abort();
    let _ = task.await;
  }
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
) -> Result<Option<String>, String> {
  validate_process_id(&id)?;
  let cwd = resolve_cwd(&spec, cwd.as_deref()).await?;
  let args = args.unwrap_or_default();
  let group = (spec.kind == "wsl").then(|| {
    Arc::new(WslGroup {
      marker: format!("JEDI_PROC_GROUP_{}:", uuid::Uuid::new_v4()),
      pid: AtomicU32::new(0),
      ready: Notify::new(),
    })
  });
  let mut cmd = build_command_at(
    &spec,
    &args,
    cwd.as_deref(),
    group.as_ref().map(|g| g.marker.as_str()),
  );
  if let Some(vars) = env {
    if spec.kind == "wsl" {
      let mut names = Vec::new();
      if let Ok(inherited) = std::env::var("WSLENV") {
        if !inherited.is_empty() {
          names.push(inherited);
        }
      }
      names.extend(vars.keys().map(|key| format!("{key}/u")));
      cmd.env("WSLENV", names.join(":"));
    }
    cmd.envs(vars);
  }
  cmd
    .stdin(std::process::Stdio::piped())
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped())
    .kill_on_drop(true);

  let mut procs = state.0.lock().await;
  // Replacements signal their old supervisor, which still owns/reaps the
  // child. A generation token prevents it from removing the replacement.
  drop(procs.remove(&id));
  let mut child: Child = cmd
    .spawn()
    .map_err(|e| format!("failed to spawn agent {} ({}): {e}", spec.exe, spec.kind))?;
  let stdin = child.stdin.take().ok_or("no stdin pipe")?;
  let stdout = child.stdout.take().ok_or("no stdout pipe")?;
  let stderr = child.stderr.take().ok_or("no stderr pipe")?;
  let stdout_task = tokio::spawn(pump(BufReader::new(stdout), on_event.clone(), false, None));
  let stderr_task = tokio::spawn(pump(
    BufReader::new(stderr),
    on_event.clone(),
    true,
    group.clone(),
  ));
  let (stop_tx, stop_rx) = oneshot::channel();
  let weak_state = Arc::downgrade(&state.0);
  let generation = Arc::new(());
  let supervisor_generation = generation.clone();
  let supervisor_id = id.clone();
  let supervisor = tokio::spawn(async move {
    let (code, stopped) = tokio::select! {
      status = child.wait() => (status.ok().and_then(|s| s.code()), false),
      _ = stop_rx => (None, true),
    };
    let mut cleanup = Ok(());
    if let Some(group) = group {
      cleanup = terminate_wsl_group(&spec, &group).await;
    }
    if stopped {
      let tree_cleanup = terminate_child_tree(&mut child).await;
      cleanup = cleanup.and(tree_cleanup);
    }
    drain_pump(stdout_task).await;
    drain_pump(stderr_task).await;
    let _ = on_event.send(json!({ "type": "exit", "code": code }));
    if let Some(state) = weak_state.upgrade() {
      let mut procs = state.lock().await;
      if procs
        .get(&supervisor_id)
        .map(|handle| Arc::ptr_eq(&handle.generation, &supervisor_generation))
        .unwrap_or(false)
      {
        procs.remove(&supervisor_id);
      }
    }
    cleanup
  });
  procs.insert(
    id,
    ProcHandle {
      stdin: Arc::new(Mutex::new(stdin)),
      stop: Some(stop_tx),
      supervisor: Some(supervisor),
      generation,
    },
  );
  // Existing callers ignore this optional value. Group drivers use it to
  // apply exactly the same canonical cwd to native protocol overrides.
  Ok(cwd)
}

#[tauri::command]
pub async fn agent_proc_stdin(
  id: String,
  line: String,
  state: tauri::State<'_, ProcManState>,
) -> Result<(), String> {
  let stdin = state
    .0
    .lock()
    .await
    .get(&id)
    .map(|handle| handle.stdin.clone())
    .ok_or_else(|| format!("no such process: {id}"))?;
  // Do not retain the global process map lock across a blocked pipe write;
  // Stop must be able to find/remove the process while stdin is backpressured.
  let mut stdin = stdin.lock().await;
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
  if let Some(handle) = handle {
    stop_handle(handle).await?;
  }
  Ok(())
}

#[cfg(test)]
mod cwd_tests {
  use super::*;

  fn spec(kind: &str, exe: &str) -> SpawnSpec {
    SpawnSpec {
      kind: kind.into(),
      distro: Some("ArchLinux".into()),
      exe: exe.into(),
    }
  }

  #[test]
  fn shell_paths_are_quoted_as_one_argument() {
    assert_eq!(
      shell_quote("/home/a b/it's $x;pwd"),
      "'/home/a b/it'\\''s $x;pwd'"
    );
  }

  #[test]
  fn wsl_cd_precedes_exec_and_login_shell_restores_it() {
    let dir = "/mnt/e/work space/it's $(touch nope)";
    let command = build_command_at(
      &spec("wsl", "/home/my tools/codex"),
      &["app-server".into(), "a' b;$HOME".into()],
      Some(dir),
      None,
    );
    let args: Vec<_> = command
      .as_std()
      .get_args()
      .map(|arg| arg.to_string_lossy().into_owned())
      .collect();
    assert_eq!(&args[..5], &["-d", "ArchLinux", "--cd", dir, "--exec"]);
    assert_eq!(
      args.last().unwrap(),
      &format!(
        "cd -- {} && exec {} 'app-server' {}",
        shell_quote(dir),
        shell_quote("/home/my tools/codex"),
        shell_quote("a' b;$HOME")
      )
    );
    assert!(command.as_std().get_current_dir().is_none());
  }

  #[test]
  fn legacy_wsl_launch_has_no_cd_override() {
    let command = build_command(&spec("wsl", "/usr/bin/agent"), &[]);
    assert!(!command.as_std().get_args().any(|arg| arg == "--cd"));
    assert!(command.as_std().get_current_dir().is_none());
  }

  #[test]
  fn native_cwd_uses_current_dir_without_shell_interpolation() {
    let command = build_command_at(&spec("unix", "agent"), &[], Some("/a b/$x"), None);
    assert_eq!(
      command.as_std().get_current_dir(),
      Some(Path::new("/a b/$x"))
    );
  }

  #[tokio::test]
  async fn cwd_is_canonical_and_must_exist_as_directory() {
    let temp = tempfile::tempdir().unwrap();
    let child = temp.path().join("child");
    tokio::fs::create_dir(&child).await.unwrap();
    let path = child.join("..");
    assert_eq!(
      native_cwd(path.to_str().unwrap()).await.unwrap(),
      agent_path(tokio::fs::canonicalize(temp.path()).await.unwrap())
    );
    assert!(native_cwd(temp.path().join("missing").to_str().unwrap())
      .await
      .is_err());
    let file = temp.path().join("file");
    tokio::fs::write(&file, "data").await.unwrap();
    assert!(native_cwd(file.to_str().unwrap()).await.is_err());
    assert!(native_cwd("relative").await.is_err());
    assert_eq!(
      resolve_cwd(&spec("unix", "agent"), None).await.unwrap(),
      None
    );
  }
}

#[cfg(all(test, target_os = "windows"))]
mod launcher_tests {
  use super::*;

  #[tokio::test]
  async fn stopping_windows_tree_reaps_launcher_and_descendant() {
    let mut child = Command::new("powershell.exe")
      .args([
        "-NoLogo",
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        "$child = Start-Process -FilePath (Join-Path $PSHOME 'powershell.exe') -ArgumentList '-NoLogo -NoProfile -NonInteractive -Command Start-Sleep -Seconds 30' -WindowStyle Hidden -PassThru; Write-Output $child.Id; $child.WaitForExit()",
      ])
      .creation_flags(0x0800_0000)
      .stdin(std::process::Stdio::null())
      .stdout(std::process::Stdio::piped())
      .stderr(std::process::Stdio::null())
      .kill_on_drop(true)
      .spawn()
      .unwrap();
    let mut output = BufReader::new(child.stdout.take().unwrap()).lines();
    let pid_line = tokio::time::timeout(Duration::from_secs(10), output.next_line()).await;
    // Clean up even if the fixture could not report its descendant.
    let stopped = terminate_child_tree(&mut child).await;
    let descendant: u32 = pid_line.unwrap().unwrap().unwrap().trim().parse().unwrap();
    stopped.unwrap();
    assert!(child.id().is_none(), "launcher must be reaped");
    let status = Command::new("powershell.exe")
      .args([
        "-NoLogo",
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        &format!("if (Get-Process -Id {descendant} -ErrorAction SilentlyContinue) {{ exit 1 }}"),
      ])
      .creation_flags(0x0800_0000)
      .stdin(std::process::Stdio::null())
      .stdout(std::process::Stdio::null())
      .stderr(std::process::Stdio::null())
      .kill_on_drop(true)
      .status()
      .await
      .unwrap();
    assert!(status.success(), "descendant must be terminated");
  }

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
    assert!(
      output.status.success(),
      "{}",
      String::from_utf8_lossy(&output.stderr)
    );
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
