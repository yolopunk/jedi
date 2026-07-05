// Shell execution tool exposed to the AI agent.
//
// Backs the `TERMINAL` skill: runs a command through the platform shell, captures
// stdout/stderr, enforces a timeout, and returns combined output. This is the
// general "run a command" ability that lets the agent inspect the system, build,
// or fetch data — the same primitive Claude Code / Codex expose as their Bash /
// shell tool.
//
// SECURITY: this executes arbitrary commands with the app's privileges and no
// confirmation prompt. It is gated only by the skill being enabled. If you ship
// this to untrusted users, add an allowlist or a per-call confirmation UI.

use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

const COMMAND_TIMEOUT_SECS: u64 = 30;
const MAX_OUTPUT_CHARS: usize = 16_000;

/// Execute a shell command and return its combined stdout/stderr output.
///
/// `cwd` is the optional working directory. A non-zero exit status is returned as
/// `Err` (with the captured output) so the agent can see the failure and recover.
#[tauri::command]
pub async fn execute_command(command: String, cwd: Option<String>) -> Result<String, String> {
  let trimmed = command.trim();
  if trimmed.is_empty() {
    return Err("command is empty".into());
  }

  let mut cmd = build_shell_command(trimmed);
  if let Some(dir) = cwd.as_deref().filter(|d| !d.is_empty()) {
    cmd.current_dir(dir);
  }
  cmd
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    // Ensure the child is reaped if we drop it on timeout, rather than orphaning it.
    .kill_on_drop(true);

  let child = cmd
    .spawn()
    .map_err(|e| format!("failed to spawn command: {e}"))?;

  let output = match timeout(
    Duration::from_secs(COMMAND_TIMEOUT_SECS),
    child.wait_with_output(),
  )
  .await
  {
    Ok(res) => res.map_err(|e| format!("command failed: {e}"))?,
    Err(_) => return Err(format!("command timed out after {COMMAND_TIMEOUT_SECS}s")),
  };

  let stdout = String::from_utf8_lossy(&output.stdout);
  let stderr = String::from_utf8_lossy(&output.stderr);

  let mut combined = String::new();
  if !stdout.trim().is_empty() {
    combined.push_str(stdout.trim_end());
  }
  if !stderr.trim().is_empty() {
    if !combined.is_empty() {
      combined.push('\n');
    }
    combined.push_str("[stderr] ");
    combined.push_str(stderr.trim_end());
  }

  let code = output.status.code();
  if combined.is_empty() {
    combined = match code {
      Some(0) => "(command produced no output)".into(),
      Some(c) => format!("(no output; exit code {c})"),
      None => "(no output; process terminated by signal)".into(),
    };
  }

  let combined = truncate(&combined);

  if output.status.success() {
    Ok(combined)
  } else {
    let status = code
      .map(|c| c.to_string())
      .unwrap_or_else(|| "signal".into());
    Err(format!("exit code {status}: {combined}"))
  }
}

fn build_shell_command(command: &str) -> Command {
  #[cfg(target_os = "windows")]
  {
    let mut c = Command::new("cmd");
    c.arg("/C").arg(command);
    c
  }
  #[cfg(not(target_os = "windows"))]
  {
    let mut c = Command::new("sh");
    c.arg("-c").arg(command);
    c
  }
}

fn truncate(text: &str) -> String {
  if text.chars().count() <= MAX_OUTPUT_CHARS {
    return text.to_string();
  }
  let head: String = text.chars().take(MAX_OUTPUT_CHARS).collect();
  format!("{head}\n...(output truncated)")
}
