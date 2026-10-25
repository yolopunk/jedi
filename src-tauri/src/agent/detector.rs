// Agent hub detection (see docs/ai-chat/05-multi-agent-rd-plan.md §5).
//
// Discovers installed coding agents across hosts, aligned with Magpie's
// mainstream agent list (https://github.com/yetone/magpie):
//  - Windows host: known-dir scan (npm/scoop/cargo/volta/...) + `where.exe`
//  - WSL: enumerate distros (`wsl.exe -l -q`, UTF-16LE output!), filter
//    virtual distros, then one probe script per distro (PATH + login shell
//    + known dirs — PATH alone misses agents installed under ~/.local/bin,
//    verified on a real machine).
//  - macOS/Linux host: the same probe script through the user's login shell.
//
// Version probing is best-effort (short timeout, failure => None) so a slow
// agent CLI can never stall startup detection.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::time::timeout;

pub struct AgentDef {
  pub id: &'static str,
  pub display_name: &'static str,
  pub binaries: &'static [&'static str],
}

/// Mainstream subset of Magpie's supported agents (docs §3.2).
pub const AGENTS: &[AgentDef] = &[
  AgentDef { id: "claude-code", display_name: "Claude Code", binaries: &["claude"] },
  AgentDef { id: "codex", display_name: "Codex", binaries: &["codex"] },
  AgentDef { id: "pi", display_name: "pi", binaries: &["pi"] },
  AgentDef { id: "gemini-cli", display_name: "Gemini CLI", binaries: &["gemini"] },
  AgentDef { id: "qwen-code", display_name: "Qwen Code", binaries: &["qwen"] },
  AgentDef { id: "cursor-cli", display_name: "Cursor CLI", binaries: &["cursor-agent"] },
  AgentDef { id: "opencode", display_name: "OpenCode", binaries: &["opencode"] },
  AgentDef { id: "crush", display_name: "Crush", binaries: &["crush"] },
  AgentDef { id: "goose", display_name: "Goose", binaries: &["goose"] },
  AgentDef { id: "droid", display_name: "Droid", binaries: &["droid"] },
  AgentDef { id: "kimi-code", display_name: "Kimi Code", binaries: &["kimi"] },
  // Antigravity CLI's binary is `agy`. NOTE: an `antigravity` binary on PATH
  // is the Antigravity IDE launcher (VS Code fork: diff/merge/goto flags) —
  // probing that name yields a false positive, verified on a real machine.
  AgentDef { id: "antigravity", display_name: "Antigravity CLI", binaries: &["agy"] },
];

const CACHE_TTL: Duration = Duration::from_secs(24 * 3600);
const WSL_ENUM_TIMEOUT: Duration = Duration::from_secs(10);
const HOST_PROBE_TIMEOUT: Duration = Duration::from_secs(20);
const VERSION_TIMEOUT: Duration = Duration::from_secs(6);

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SpawnSpec {
  /// "windows-exe" | "windows-cmd" | "wsl" | "unix"
  pub kind: String,
  pub distro: Option<String>,
  pub exe: String,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AgentInstallation {
  pub agent_id: String,
  /// "windows" | "wsl:<distro>" | "macos" | "linux"
  pub host: String,
  pub exe: String,
  pub version: Option<String>,
  pub spawn: SpawnSpec,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AgentReport {
  pub id: String,
  pub name: String,
  /// Empty when the agent is not installed anywhere.
  pub installations: Vec<AgentInstallation>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DetectionResult {
  pub reports: Vec<AgentReport>,
  pub duration_ms: u64,
}

pub struct DetectorState {
  cache: Mutex<Option<(Instant, DetectionResult)>>,
}

impl DetectorState {
  pub fn new() -> Self {
    Self { cache: Mutex::new(None) }
  }
}

/// wsl.exe prints UTF-16LE; most other console tools print UTF-8. Detect by
/// NUL-byte density and decode accordingly.
fn decode_console_output(bytes: &[u8]) -> String {
  let nul_count = bytes.iter().filter(|&&b| b == 0).count();
  if bytes.len() > 4 && nul_count > bytes.len() / 8 {
    let units: Vec<u16> =
      bytes.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
    String::from_utf16_lossy(&units)
  } else {
    String::from_utf8_lossy(bytes).into_owned()
  }
}

async fn run_capture(cmd: &mut Command, t: Duration) -> Result<String, String> {
  let out = timeout(t, cmd.output())
    .await
    .map_err(|_| "timed out".to_string())?
    .map_err(|e| e.to_string())?;
  let text = decode_console_output(&out.stdout);
  let err = decode_console_output(&out.stderr);
  // Some CLIs print --version to stderr; accept either.
  Ok(if text.trim().is_empty() { err } else { text })
}

fn first_line(text: &str) -> String {
  text.lines().map(str::trim).find(|l| !l.is_empty()).unwrap_or("").to_string()
}

fn shquote(s: &str) -> String {
  format!("'{}'", s.replace('\'', "'\\''"))
}

/// POSIX probe script: per agent, resolve the first binary visible on PATH or
/// in a known install dir, printing `agent-id|/abs/path` lines. Runs once per
/// host (login shell), so one spawn covers all 12 agents.
fn unix_probe_script() -> String {
  let mut script = String::from(
    "DIRS=\"$HOME/.local/bin $HOME/.npm-global/bin $HOME/.local/share/pnpm $HOME/.bun/bin $HOME/.volta/bin $HOME/.cargo/bin $HOME/go/bin $HOME/.deno/bin /usr/local/bin /opt/homebrew/bin /usr/bin /usr/sbin\"\n",
  );
  script.push_str("p() {\n");
  script.push_str("  for b in \"$@\"; do x=$(command -v \"$b\" 2>/dev/null) && { printf '%s\\n' \"$x\"; return 0; }; done\n");
  script.push_str("  for b in \"$@\"; do for d in $DIRS; do [ -x \"$d/$b\" ] && { printf '%s\\n' \"$d/$b\"; return 0; }; done; done\n");
  script.push_str("  return 1\n}\n");
  for a in AGENTS {
    script.push_str(&format!(
      "x=$(p {}) && echo \"{}|$x\" || true\n",
      a.binaries.join(" "),
      a.id
    ));
  }
  script
}

/// Parse `agent-id|/path` lines into (agent, host, spawn-spec) triples.
fn parse_probe_output(host: &str, text: &str) -> Vec<(String, String, SpawnSpec)> {
  let mut found = Vec::new();
  for line in text.lines() {
    let line = line.trim();
    if let Some((agent_id, path)) = line.split_once('|') {
      if path.is_empty() {
        continue;
      }
      // WSL inherits the Windows PATH, so Windows installs leak in as
      // /mnt/<drive>/... entries. Skip them: the native Windows probe finds
      // those (and spawning them natively beats going through WSL interop).
      if host.starts_with("wsl:") && path.starts_with("/mnt/") {
        let rest = &path[5..];
        if rest.len() >= 2 && rest.as_bytes()[1] == b'/' {
          continue;
        }
      }
      let spec = SpawnSpec {
        kind: if host.starts_with("wsl:") { "wsl".into() } else { "unix".into() },
        distro: host.strip_prefix("wsl:").map(str::to_string),
        exe: path.to_string(),
      };
      found.push((agent_id.to_string(), host.to_string(), spec));
    }
  }
  found
}

async fn probe_unix_host(host: &str) -> Vec<(String, String, SpawnSpec)> {
  // GUI processes do not inherit the interactive shell PATH, so run the probe
  // through the user's login shell (macOS zsh / Linux bash).
  let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
  let script = unix_probe_script();
  let mut cmd = Command::new(&shell);
  cmd.arg("-lc").arg(&script);
  let text = run_capture(&mut cmd, HOST_PROBE_TIMEOUT).await.unwrap_or_default();
  parse_probe_output(host, &text)
}

async fn list_wsl_distros() -> Vec<String> {
  let mut cmd = Command::new("wsl.exe");
  cmd.args(["-l", "-q"]);
  let text = run_capture(&mut cmd, WSL_ENUM_TIMEOUT).await.unwrap_or_default();
  text
    .lines()
    .map(|l| l.trim().trim_end_matches('\u{0}').trim())
    .filter(|l| !l.is_empty())
    // Skip WSL's boilerplate and virtual distros used by Docker Desktop.
    .filter(|l| !l.starts_with("docker-desktop"))
    .map(str::to_string)
    .collect()
}

async fn probe_wsl_distro(distro: &str) -> Vec<(String, String, SpawnSpec)> {
  let host = format!("wsl:{distro}");
  // --exec bypasses WSL's default shell so the script reaches bash verbatim.
  let mut cmd = Command::new("wsl.exe");
  cmd
    .arg("-d")
    .arg(distro)
    .arg("--exec")
    .arg("bash")
    .arg("-lc")
    .arg(unix_probe_script());
  let text = run_capture(&mut cmd, HOST_PROBE_TIMEOUT).await.unwrap_or_default();
  parse_probe_output(&host, &text)
}

#[cfg(target_os = "windows")]
fn windows_candidate_dirs() -> Vec<std::path::PathBuf> {
  let home = std::path::PathBuf::from(std::env::var("USERPROFILE").unwrap_or_default());
  [
    "AppData\\Roaming\\npm",
    "scoop\\shims",
    ".cargo\\bin",
    ".volta\\bin",
    ".local\\bin",
    "AppData\\Local\\pnpm",
  ]
  .iter()
  .map(|rel| home.join(rel))
  .collect()
}

#[cfg(target_os = "windows")]
async fn probe_windows_host() -> Vec<(String, String, SpawnSpec)> {
  let dirs = windows_candidate_dirs();
  let exts = ["", ".cmd", ".exe", ".bat"];
  let mut found: Vec<(String, String, SpawnSpec)> = Vec::new();
  for agent in AGENTS {
    let mut hit: Option<String> = None;
    'outer: for bin in agent.binaries {
      // Known install dirs first: cheap, no process spawn, immune to the
      // GUI-process PATH problem.
      for dir in &dirs {
        for ext in exts {
          let candidate = dir.join(format!("{bin}{ext}"));
          if candidate.is_file() {
            hit = Some(candidate.to_string_lossy().into_owned());
            break 'outer;
          }
        }
      }
      // PATH fallback via where.exe (first result line).
      let mut cmd = Command::new("where.exe");
      cmd.arg(bin);
      if let Ok(text) = run_capture(&mut cmd, Duration::from_secs(3)).await {
        if let Some(line) = text.lines().map(str::trim).find(|l| !l.is_empty()) {
          hit = Some(line.to_string());
          break;
        }
      }
    }
    if let Some(exe) = hit {
      let kind =
        if exe.ends_with(".cmd") || exe.ends_with(".bat") { "windows-cmd" } else { "windows-exe" };
      found.push((
        agent.id.to_string(),
        "windows".to_string(),
        SpawnSpec { kind: kind.into(), distro: None, exe },
      ));
    }
  }
  found
}

#[cfg(not(target_os = "windows"))]
async fn probe_windows_host() -> Vec<(String, String, SpawnSpec)> {
  Vec::new()
}

fn current_host() -> &'static str {
  if cfg!(target_os = "windows") {
    "windows"
  } else if cfg!(target_os = "macos") {
    "macos"
  } else {
    "linux"
  }
}

async fn probe_version(spec: SpawnSpec) -> Option<String> {
  let mut cmd = match spec.kind.as_str() {
    "wsl" => {
      let mut c = Command::new("wsl.exe");
      c
        .arg("-d")
        .arg(spec.distro.as_deref().unwrap_or_default())
        .arg("--exec")
        .arg("bash")
        .arg("-lc")
        .arg(format!("{} --version 2>&1 | head -1", shquote(&spec.exe)));
      c
    }
    "windows-cmd" => {
      let mut c = Command::new("cmd");
      c.args(["/C", &format!("\"{}\" --version", spec.exe)]);
      c
    }
    _ => {
      let mut c = Command::new(&spec.exe);
      c.arg("--version");
      c
    }
  };
  #[cfg(target_os = "windows")]
  cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW: no console flash
  let text = run_capture(&mut cmd, VERSION_TIMEOUT).await.ok()?;
  let line = first_line(&text);
  if line.is_empty() { None } else { Some(line.chars().take(120).collect()) }
}

/// Detect every mainstream agent across all hosts. Results are cached for
/// CACHE_TTL; `force` bypasses the cache.
#[tauri::command]
pub async fn detect_agents(
  force: Option<bool>,
  state: tauri::State<'_, DetectorState>,
) -> Result<DetectionResult, String> {
  {
    let cache = state.cache.lock().await;
    if !force.unwrap_or(false) {
      if let Some((at, result)) = cache.as_ref() {
        if at.elapsed() < CACHE_TTL {
          return Ok(result.clone());
        }
      }
    }
  }

  let started = Instant::now();

  // Host probes run concurrently: windows (or current unix host) + every WSL
  // distro (WSL cold start is 1-3s per distro, so parallelize).
  type RawHit = (String, String, SpawnSpec);
  let mut probes: Vec<tokio::task::JoinHandle<Vec<RawHit>>> = Vec::new();
  if cfg!(target_os = "windows") {
    probes.push(tokio::spawn(probe_windows_host()));
    for distro in list_wsl_distros().await {
      probes.push(tokio::spawn(async move { probe_wsl_distro(&distro).await }));
    }
  } else {
    probes.push(tokio::spawn(probe_unix_host(current_host())));
  }
  let mut raw: Vec<RawHit> = Vec::new();
  for handle in probes {
    if let Ok(list) = handle.await {
      raw.extend(list);
    }
  }

  // Version probes: best-effort, all in parallel.
  let mut versions: HashMap<(String, String), Option<String>> = HashMap::new();
  let mut version_tasks = Vec::new();
  for (_agent_id, _host, spec) in &raw {
    version_tasks.push(tokio::spawn(probe_version(spec.clone())));
  }
  for (i, handle) in version_tasks.into_iter().enumerate() {
    let v = handle.await.ok().flatten();
    versions.insert((raw[i].0.clone(), raw[i].2.exe.clone()), v);
  }

  let mut reports: Vec<AgentReport> = AGENTS
    .iter()
    .map(|a| AgentReport {
      id: a.id.to_string(),
      name: a.display_name.to_string(),
      installations: Vec::new(),
    })
    .collect();
  for (agent_id, host, spec) in raw {
    let version = versions.get(&(agent_id.clone(), spec.exe.clone())).cloned().flatten();
    if let Some(report) = reports.iter_mut().find(|r| r.id == agent_id) {
      report.installations.push(AgentInstallation {
        agent_id: agent_id.clone(),
        host,
        exe: spec.exe.clone(),
        version,
        spawn: spec,
      });
    }
  }

  let result =
    DetectionResult { reports, duration_ms: started.elapsed().as_millis() as u64 };
  *state.cache.lock().await = Some((Instant::now(), result.clone()));
  Ok(result)
}
