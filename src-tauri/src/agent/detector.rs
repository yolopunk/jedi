// Agent hub detection (see docs/ai-chat/05-multi-agent-rd-plan.md §5).
//
// Discovers installed coding agents across hosts, aligned with Magpie's
// mainstream agent list (https://github.com/yetone/magpie) plus ZCode:
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
  AgentDef { id: "zcode", display_name: "ZCode", binaries: &["zcode"] },
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
  pub default_model: Option<String>,
  /// Where `default_model` came from: "env:<NAME>" | "config:<file>" |
  /// "profile:<name>" | "agent-default" (the agent decides; confirmed only
  /// once a run reports the model it actually used).
  pub model_source: String,
  /// "ready" | "needs_auth" | "invalid_path" | "unknown" (cannot be checked
  /// statically; confirmed by the first run).
  pub status: String,
  pub status_reason: Option<String>,
  pub status_hint: Option<String>,
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

/// A config file the readiness probe reads. Its content stays inside this
/// process: only derived facts (model name, auth *kind*) are ever returned.
struct ConfFile {
  key: &'static str,
  /// Env var that relocates the agent's config dir (e.g. CODEX_HOME).
  dir_env: Option<&'static str>,
  /// Default dir relative to $HOME.
  dir: &'static str,
  file: &'static str,
}

struct ProbePlan {
  /// Files whose content is parsed (model / auth-type keys).
  read: &'static [ConfFile],
  /// Credential files: only existence + size is checked, never read.
  sized: &'static [ConfFile],
  /// Env vars whose presence (not value) means "credentials configured".
  auth_env: &'static [&'static str],
  /// Env vars that name the model; their value is a model id, not a secret.
  model_env: &'static [&'static str],
}

// A macro (not a fn) so `&[cf!(..)]` is promoted to a `'static` slice.
macro_rules! cf {
  ($key:expr, $dir_env:expr, $dir:expr, $file:expr) => {
    ConfFile { key: $key, dir_env: $dir_env, dir: $dir, file: $file }
  };
}

fn probe_plan(agent: &str) -> Option<ProbePlan> {
  Some(match agent {
    "claude-code" => ProbePlan {
      read: &[cf!("settings", Some("CLAUDE_CONFIG_DIR"), ".claude", "settings.json")],
      sized: &[cf!("credentials", Some("CLAUDE_CONFIG_DIR"), ".claude", ".credentials.json")],
      auth_env: &[
        "ANTHROPIC_API_KEY",
        "ANTHROPIC_AUTH_TOKEN",
        "CLAUDE_CODE_OAUTH_TOKEN",
        "CLAUDE_CODE_USE_BEDROCK",
        "CLAUDE_CODE_USE_VERTEX",
      ],
      model_env: &["ANTHROPIC_MODEL"],
    },
    "codex" => ProbePlan {
      read: &[cf!("config", Some("CODEX_HOME"), ".codex", "config.toml")],
      sized: &[cf!("auth", Some("CODEX_HOME"), ".codex", "auth.json")],
      auth_env: &["OPENAI_API_KEY", "CODEX_API_KEY"],
      model_env: &[],
    },
    "pi" => ProbePlan {
      read: &[
        cf!("settings", Some("PI_CODING_AGENT_DIR"), ".pi/agent", "settings.json"),
        // auth.json is parsed only to count its provider entries.
        cf!("auth", Some("PI_CODING_AGENT_DIR"), ".pi/agent", "auth.json"),
      ],
      sized: &[cf!("models", Some("PI_CODING_AGENT_DIR"), ".pi/agent", "models.json")],
      auth_env: &[
        "ANTHROPIC_API_KEY",
        "OPENAI_API_KEY",
        "GEMINI_API_KEY",
        "OPENROUTER_API_KEY",
        "GROQ_API_KEY",
        "XAI_API_KEY",
        "MISTRAL_API_KEY",
        "DEEPSEEK_API_KEY",
      ],
      model_env: &[],
    },
    "gemini-cli" => ProbePlan {
      read: &[cf!("settings", None, ".gemini", "settings.json")],
      sized: &[],
      auth_env: &[
        "GEMINI_API_KEY",
        "GOOGLE_API_KEY",
        "GOOGLE_GENAI_USE_VERTEXAI",
        "GOOGLE_GENAI_USE_GCA",
      ],
      model_env: &["GEMINI_MODEL"],
    },
    "qwen-code" => ProbePlan {
      read: &[cf!("settings", None, ".qwen", "settings.json")],
      sized: &[],
      auth_env: &["OPENAI_API_KEY", "DASHSCOPE_API_KEY"],
      model_env: &["OPENAI_MODEL"],
    },
    "cursor-cli" => ProbePlan {
      read: &[cf!("config", None, ".cursor", "cli-config.json")],
      sized: &[],
      auth_env: &["CURSOR_API_KEY"],
      model_env: &[],
    },
    "opencode" => ProbePlan {
      read: &[
        cf!("config", Some("XDG_CONFIG_HOME"), ".config", "opencode/opencode.json"),
        cf!("configc", Some("XDG_CONFIG_HOME"), ".config", "opencode/opencode.jsonc"),
      ],
      sized: &[],
      auth_env: &[],
      model_env: &["OPENCODE_MODEL"],
    },
    "antigravity" => ProbePlan { read: &[], sized: &[], auth_env: &[], model_env: &[] },
    _ => return None,
  })
}

/// Everything the readiness probe learned about one installation.
#[derive(Default, Debug)]
struct Facts {
  exe_exists: bool,
  env_set: Vec<String>,
  env_vals: HashMap<String, String>,
  sizes: HashMap<String, u64>,
  files: HashMap<String, String>,
}

impl Facts {
  fn any_env(&self, names: &[&str]) -> Option<String> {
    names.iter().find(|n| self.env_set.iter().any(|s| s == *n)).map(|s| s.to_string())
  }
  fn json(&self, key: &str) -> Option<serde_json::Value> {
    let text = self.files.get(key)?;
    serde_json::from_str(text).ok().or_else(|| serde_json::from_str(&strip_jsonc(text)).ok())
  }
}

/// Drop `//` line comments so opencode.jsonc parses (strings kept intact).
fn strip_jsonc(text: &str) -> String {
  let mut out = String::with_capacity(text.len());
  for line in text.lines() {
    let mut in_str = false;
    let mut prev = '\0';
    let mut cut = line.len();
    for (i, ch) in line.char_indices() {
      if ch == '"' && prev != '\\' {
        in_str = !in_str;
      } else if !in_str && ch == '/' && prev == '/' {
        cut = i - 1;
        break;
      }
      prev = ch;
    }
    out.push_str(&line[..cut]);
    out.push('\n');
  }
  out
}

fn clean_model(name: Option<&str>) -> Option<String> {
  let name = name?.trim();
  if name.is_empty() || name == "unknown" { None } else { Some(name.to_string()) }
}

/// Top-level TOML string value (before the first table header).
fn toml_top_string(content: &str, key: &str) -> Option<String> {
  content
    .lines()
    .take_while(|line| !line.trim_start().starts_with('['))
    .find_map(|line| toml_kv(line, key))
}

fn toml_kv(line: &str, key: &str) -> Option<String> {
  let line = line.split(" #").next()?;
  let (k, v) = line.split_once('=')?;
  if k.trim() != key {
    return None;
  }
  let v = v.trim();
  serde_json::from_str::<String>(v)
    .ok()
    .or_else(|| v.strip_prefix('\'')?.strip_suffix('\'').map(str::to_string))
}

/// `[profiles.<name>]` (or `[profiles."<name>"]`) table value.
fn toml_profile_string(content: &str, profile: &str, key: &str) -> Option<String> {
  let headers = [format!("[profiles.{profile}]"), format!("[profiles.\"{profile}\"]")];
  let mut inside = false;
  for line in content.lines() {
    let t = line.trim();
    if t.starts_with('[') {
      inside = headers.iter().any(|h| h == t);
      continue;
    }
    if inside {
      if let Some(v) = toml_kv(t, key) {
        return Some(v);
      }
    }
  }
  None
}

/// Only return the configured model name; never expose configuration or credentials.
fn configured_model(agent: &str, content: &str) -> Option<String> {
  if agent == "codex" {
    if let Some(profile) = toml_top_string(content, "profile") {
      if let Some(m) = toml_profile_string(content, &profile, "model") {
        return clean_model(Some(&m));
      }
    }
    return clean_model(toml_top_string(content, "model").as_deref());
  }
  let json: serde_json::Value = serde_json::from_str(content)
    .ok()
    .or_else(|| serde_json::from_str(&strip_jsonc(content)).ok())?;
  let pick = json
    .get("defaultModel")
    .and_then(|v| v.as_str())
    .or_else(|| json.get("model").and_then(|v| v.as_str()))
    .or_else(|| json.pointer("/model/name").and_then(|v| v.as_str()))
    .or_else(|| json.pointer("/env/ANTHROPIC_MODEL").and_then(|v| v.as_str()))
    .map(str::to_owned);
  // pi stores provider and model separately; show "provider/model".
  let pick = match (agent, pick, json.get("defaultProvider").and_then(|v| v.as_str())) {
    ("pi", Some(m), Some(p)) if !m.contains('/') && !p.is_empty() => Some(format!("{p}/{m}")),
    (_, m, _) => m,
  };
  clean_model(pick.as_deref())
}

struct Readiness {
  status: &'static str,
  reason: Option<String>,
  hint: Option<String>,
  default_model: Option<String>,
  model_source: String,
}

fn conf_file_label(plan: &ProbePlan, key: &str) -> String {
  plan
    .read
    .iter()
    .find(|f| f.key == key)
    .map(|f| format!("{}/{}", f.dir.trim_start_matches('.').trim_start_matches('/'), f.file))
    .map(|p| format!("~/.{p}"))
    .unwrap_or_else(|| key.to_string())
}

/// Pure assessment from probe facts (unit-tested below).
fn assess(agent: &str, plan: &ProbePlan, facts: &Facts) -> Readiness {
  // --- default model: env beats config file (matches the agents' own order)
  let mut default_model = None;
  let mut model_source = "agent-default".to_string();
  for name in plan.model_env {
    if let Some(v) = clean_model(facts.env_vals.get(*name).map(String::as_str)) {
      default_model = Some(v);
      model_source = format!("env:{name}");
      break;
    }
  }
  if default_model.is_none() {
    for f in plan.read {
      if f.key == "auth" {
        continue;
      }
      let Some(text) = facts.files.get(f.key) else { continue };
      if let Some(m) = configured_model(agent, text) {
        default_model = Some(m);
        model_source = match (agent, toml_top_string(text, "profile")) {
          ("codex", Some(p)) => format!("profile:{p}"),
          _ => format!("config:{}", conf_file_label(plan, f.key)),
        };
        break;
      }
    }
  }

  let ready = |r: Option<&str>| Readiness {
    status: "ready",
    reason: r.map(str::to_string),
    hint: None,
    default_model: default_model.clone(),
    model_source: model_source.clone(),
  };
  let needs = |reason: &str, hint: &str| Readiness {
    status: "needs_auth",
    reason: Some(reason.to_string()),
    hint: Some(hint.to_string()),
    default_model: default_model.clone(),
    model_source: model_source.clone(),
  };
  let unknown = |reason: &str, hint: &str| Readiness {
    status: "unknown",
    reason: Some(reason.to_string()),
    hint: Some(hint.to_string()),
    default_model: default_model.clone(),
    model_source: model_source.clone(),
  };

  if !facts.exe_exists {
    return Readiness {
      status: "invalid_path",
      reason: Some("安装路径已失效（可执行文件不存在）".into()),
      hint: Some("重新安装该 Agent 后点击“重新检测”".into()),
      default_model,
      model_source,
    };
  }

  let size = |k: &str| facts.sizes.get(k).copied().unwrap_or(0);
  let settings = facts.json("settings");
  let selected_auth = settings.as_ref().and_then(|s| {
    s.pointer("/security/auth/selectedType")
      .or_else(|| s.get("selectedAuthType"))
      .and_then(|v| v.as_str())
      .map(str::to_string)
  });

  match agent {
    "claude-code" => {
      let settings_env = settings
        .as_ref()
        .and_then(|s| s.get("env"))
        .map(|e| ["ANTHROPIC_API_KEY", "ANTHROPIC_AUTH_TOKEN"].iter().any(|k| e.get(*k).is_some()))
        .unwrap_or(false);
      let helper = settings.as_ref().map(|s| s.get("apiKeyHelper").is_some()).unwrap_or(false);
      if size("credentials") > 2 || settings_env || helper || facts.any_env(plan.auth_env).is_some()
      {
        ready(None)
      } else if cfg!(target_os = "macos") {
        // macOS keeps the OAuth token in the Keychain: not checkable here.
        unknown("登录状态保存在系统钥匙串中，将在首次运行时确认", "如未登录，请在终端运行 claude 并执行 /login")
      } else {
        needs("未找到 Claude Code 登录凭据", "在终端运行 claude 并执行 /login，或设置 ANTHROPIC_API_KEY")
      }
    }
    "codex" => {
      let custom_provider = facts
        .files
        .get("config")
        .and_then(|t| toml_top_string(t, "model_provider"))
        .map(|p| p != "openai")
        .unwrap_or(false);
      if size("auth") > 2 || custom_provider || facts.any_env(plan.auth_env).is_some() {
        ready(None)
      } else {
        needs("未找到 Codex 登录凭据", "在终端运行 codex login，或设置 OPENAI_API_KEY")
      }
    }
    "pi" => {
      let providers = facts
        .json("auth")
        .and_then(|v| v.as_object().map(|o| o.len()))
        .unwrap_or(0);
      if providers > 0 || size("models") > 2 || facts.any_env(plan.auth_env).is_some() {
        ready(None)
      } else {
        needs(
          "pi 尚未登录任何模型提供方（auth.json 为空）",
          "在终端运行 pi 并输入 /login 登录，或设置对应提供方的 API Key 环境变量",
        )
      }
    }
    "gemini-cli" => {
      if selected_auth.is_some() || facts.any_env(plan.auth_env).is_some() {
        ready(None)
      } else {
        needs(
          "Gemini CLI 未选择认证方式（~/.gemini/settings.json 缺少 auth 配置）",
          "在终端运行 gemini 完成登录并选择认证方式，或设置 GEMINI_API_KEY",
        )
      }
    }
    "qwen-code" => {
      if selected_auth.is_some() || facts.any_env(plan.auth_env).is_some() {
        ready(None)
      } else {
        needs(
          "Qwen Code 未选择认证方式（~/.qwen/settings.json 缺少 auth 配置）",
          "在终端运行 qwen 完成认证，或设置 OPENAI_API_KEY",
        )
      }
    }
    // OpenCode ships free models, so it runs without any login.
    "opencode" => ready(None),
    "antigravity" => unknown(
      "Antigravity 使用浏览器 OAuth 登录，无法离线确认登录状态",
      "如未登录，请先在终端运行 agy 完成浏览器登录",
    ),
    "cursor-cli" => {
      if facts.any_env(plan.auth_env).is_some() {
        ready(None)
      } else {
        unknown("无法离线确认 Cursor CLI 登录状态", "如未登录，请在终端运行 cursor-agent login")
      }
    }
    _ => unknown("该 Agent 暂未适配", "请选择其他 Agent"),
  }
}

/// bash expression for a config path, honoring the relocation env var.
fn bash_path(f: &ConfFile) -> String {
  let base = match f.dir_env {
    Some(var) => format!("${{{var}:-$HOME/{}}}", f.dir),
    None => format!("$HOME/{}", f.dir),
  };
  // XDG_CONFIG_HOME replaces ~/.config itself; other vars replace the
  // agent dir. Both resolve to "<base>/<file>".
  format!("\"{base}/{}\"", f.file)
}

fn native_path(f: &ConfFile) -> Option<std::path::PathBuf> {
  let base = f
    .dir_env
    .and_then(|v| std::env::var_os(v))
    .filter(|v| !v.is_empty())
    .map(std::path::PathBuf::from)
    .or_else(|| dirs::home_dir().map(|h| h.join(f.dir)))?;
  Some(base.join(f.file))
}

/// One WSL round-trip per installation. Output lines (never secrets):
/// `X|1` exe present, `E|NAME` auth env set, `V|NAME|model`, `S|key|size`,
/// `R|key|<base64 config>` (parsed in-process, never forwarded).
fn wsl_readiness_script(exe: &str, plan: &ProbePlan) -> String {
  let mut s = format!("[ -x {} ] && echo 'X|1'\n", shquote(exe));
  for v in plan.auth_env {
    s.push_str(&format!("[ -n \"${{{v}:-}}\" ] && echo 'E|{v}'\n"));
  }
  for v in plan.model_env {
    s.push_str(&format!("[ -n \"${{{v}:-}}\" ] && printf 'V|{v}|%s\\n' \"${{{v}}}\"\n"));
  }
  for f in plan.sized {
    let p = bash_path(f);
    s.push_str(&format!("[ -f {p} ] && printf 'S|{}|%s\\n' \"$(wc -c < {p})\"\n", f.key));
  }
  for f in plan.read {
    let p = bash_path(f);
    s.push_str(&format!(
      "[ -f {p} ] && {{ printf 'R|{}|'; base64 -w0 {p}; echo; }}\n",
      f.key
    ));
  }
  s.push_str("true\n");
  s
}

fn parse_readiness_output(text: &str) -> Facts {
  use base64::Engine;
  let mut facts = Facts::default();
  for line in text.lines() {
    let mut parts = line.trim_end_matches('\r').splitn(3, '|');
    match (parts.next(), parts.next(), parts.next()) {
      (Some("X"), _, _) => facts.exe_exists = true,
      (Some("E"), Some(name), _) => facts.env_set.push(name.to_string()),
      (Some("V"), Some(name), Some(v)) => {
        facts.env_vals.insert(name.to_string(), v.to_string());
      }
      (Some("S"), Some(key), Some(n)) => {
        facts.sizes.insert(key.to_string(), n.trim().parse().unwrap_or(0));
      }
      (Some("R"), Some(key), Some(b64)) => {
        if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(b64.trim()) {
          facts.files.insert(key.to_string(), String::from_utf8_lossy(&bytes).into_owned());
        }
      }
      _ => {}
    }
  }
  facts
}

async fn gather_facts(plan: &ProbePlan, spec: &SpawnSpec) -> Facts {
  if spec.kind == "wsl" {
    let Some(distro) = spec.distro.as_deref() else { return Facts::default() };
    let mut cmd = Command::new("wsl.exe");
    cmd.args(["-d", distro, "--exec", "bash", "-lc"]);
    cmd.arg(wsl_readiness_script(&spec.exe, plan));
    return match run_capture(&mut cmd, HOST_PROBE_TIMEOUT).await {
      Ok(text) => parse_readiness_output(&text),
      // A probe failure is not proof the path is gone; keep it usable.
      Err(_) => Facts { exe_exists: true, ..Facts::default() },
    };
  }
  let mut facts = Facts {
    exe_exists: std::path::Path::new(&spec.exe).is_file(),
    ..Facts::default()
  };
  for v in plan.auth_env {
    if std::env::var_os(v).is_some_and(|x| !x.is_empty()) {
      facts.env_set.push(v.to_string());
    }
  }
  for v in plan.model_env {
    if let Ok(x) = std::env::var(v) {
      facts.env_vals.insert(v.to_string(), x);
    }
  }
  for f in plan.sized {
    if let Some(meta) = native_path(f).and_then(|p| std::fs::metadata(p).ok()) {
      facts.sizes.insert(f.key.to_string(), meta.len());
    }
  }
  for f in plan.read {
    if let Some(p) = native_path(f) {
      if let Ok(text) = tokio::fs::read_to_string(p).await {
        facts.files.insert(f.key.to_string(), text);
      }
    }
  }
  facts
}

async fn probe_readiness(agent: String, spec: SpawnSpec) -> Readiness {
  let Some(plan) = probe_plan(&agent) else {
    let exists = spec.kind == "wsl" || std::path::Path::new(&spec.exe).is_file();
    return Readiness {
      status: if exists { "unknown" } else { "invalid_path" },
      reason: Some("该 Agent 暂未适配".into()),
      hint: None,
      default_model: None,
      model_source: "agent-default".into(),
    };
  };
  let facts = gather_facts(&plan, &spec).await;
  assess(&agent, &plan, &facts)
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
  // Every detector probe targets a console binary (where.exe, reg.exe,
  // wsl.exe, login shells). Without CREATE_NO_WINDOW each spawn flashes a
  // console window over the app on Windows; stdio stays piped either way.
  #[cfg(target_os = "windows")]
  cmd.creation_flags(0x0800_0000);
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
/// host (login shell), so one spawn covers every supported agent.
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

/// npm installs both a Unix shell shim and a Windows launcher. Never pass
/// the extensionless shim to CreateProcess, even for cached installations.
#[cfg(target_os = "windows")]
pub(crate) fn windows_launcher(exe: &str) -> Option<SpawnSpec> {
  let path = std::path::Path::new(exe);
  let candidates = if path.extension().is_none() {
    vec![
      path.with_extension("exe"),
      path.with_extension("cmd"),
      path.with_extension("bat"),
    ]
  } else {
    vec![path.to_path_buf()]
  };
  for candidate in candidates {
    if !candidate.is_file() {
      continue;
    }
    let ext = candidate.extension()?.to_str()?.to_ascii_lowercase();
    if !matches!(ext.as_str(), "exe" | "cmd" | "bat") {
      continue;
    }
    // Resolve fnm's per-shell junction before the owning shell disappears.
    let resolved = candidate.canonicalize().unwrap_or(candidate);
    let resolved = resolved.to_string_lossy();
    let exe = resolved
      .strip_prefix(r"\\?\")
      .unwrap_or(&resolved)
      .to_string();
    return Some(SpawnSpec {
      kind: if ext == "exe" {
        "windows-exe"
      } else {
        "windows-cmd"
      }
      .into(),
      distro: None,
      exe,
    });
  }
  None
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
    "scoop\\apps\\fnm\\current\\aliases\\default",
  ]
  .iter()
  .map(|rel| home.join(rel))
  .collect()
}

#[cfg(target_os = "windows")]
async fn probe_windows_zcode_desktop() -> Option<(String, String, SpawnSpec)> {
  // The desktop installer does not add `zcode` to PATH, but it ships the same
  // Agent CLI runtime under resources/glm/zcode.cjs. Resolve the install from
  // its per-user uninstall entry so custom installation directories work.
  let mut cmd = Command::new("reg.exe");
  cmd.args([
    "query",
    "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
    "/s",
    "/f",
    "ZCode",
    "/d",
  ]);
  let output = run_capture(&mut cmd, Duration::from_secs(3)).await.ok()?;
  let icon_path = output.lines().find_map(|line| {
    let line = line.trim();
    let value = line.strip_prefix("DisplayIcon")?.trim_start();
    value.strip_prefix("REG_SZ").map(str::trim)
  })?;
  let install_dir = std::path::Path::new(icon_path.trim_matches('"')).parent()?;
  let runtime = install_dir.join("resources").join("glm").join("zcode.cjs");
  if !runtime.is_file() {
    return None;
  }
  Some((
    "zcode".into(),
    "windows".into(),
    SpawnSpec {
      kind: "windows-app".into(),
      distro: None,
      exe: runtime.to_string_lossy().into_owned(),
    },
  ))
}

#[cfg(target_os = "windows")]
async fn probe_windows_host() -> Vec<(String, String, SpawnSpec)> {
  let dirs = windows_candidate_dirs();
  let exts = [".exe", ".cmd", ".bat"];
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
        if let Some(line) = text
          .lines()
          .map(str::trim)
          .find(|l| windows_launcher(l).is_some())
        {
          hit = Some(line.to_string());
          break;
        }
      }
    }
    if let Some(spec) = hit.and_then(|exe| windows_launcher(&exe)) {
      found.push((agent.id.to_string(), "windows".to_string(), spec));
    }
  }
  if !found.iter().any(|(agent_id, _, _)| agent_id == "zcode") {
    if let Some(zcode) = probe_windows_zcode_desktop().await {
      found.push(zcode);
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
  if spec.kind == "windows-app" {
    return None;
  }
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
  // Readiness probes (auth + default model): one per installation, parallel.
  let readiness_tasks: Vec<_> = raw
    .iter()
    .map(|(agent_id, _, spec)| tokio::spawn(probe_readiness(agent_id.clone(), spec.clone())))
    .collect();
  let mut readiness = Vec::new();
  for handle in readiness_tasks {
    readiness.push(handle.await.ok());
  }
  for ((agent_id, host, spec), ready) in raw.into_iter().zip(readiness) {
    let version = versions.get(&(agent_id.clone(), spec.exe.clone())).cloned().flatten();
    let ready = ready.unwrap_or(Readiness {
      status: "unknown",
      reason: None,
      hint: None,
      default_model: None,
      model_source: "agent-default".into(),
    });
    if let Some(report) = reports.iter_mut().find(|r| r.id == agent_id) {
      report.installations.push(AgentInstallation {
        agent_id: agent_id.clone(),
        host,
        exe: spec.exe.clone(),
        version,
        default_model: ready.default_model,
        model_source: ready.model_source,
        status: ready.status.to_string(),
        status_reason: ready.reason,
        status_hint: ready.hint,
        spawn: spec,
      });
    }
  }

  let result =
    DetectionResult { reports, duration_ms: started.elapsed().as_millis() as u64 };
  *state.cache.lock().await = Some((Instant::now(), result.clone()));
  Ok(result)
}

#[cfg(test)]
mod model_tests {
  use super::*;

  #[test]
  fn reads_model_without_returning_other_settings() {
    assert_eq!(
      configured_model("pi", r#"{"defaultModel":"my-model","apiKey":"secret"}"#),
      Some("my-model".into())
    );
    assert_eq!(
      configured_model("pi", r#"{"defaultProvider":"anthropic","defaultModel":"m"}"#),
      Some("anthropic/m".into())
    );
    assert_eq!(
      configured_model("codex", "model = \"gpt-test\"
[profiles.other]
model = \"other\""),
      Some("gpt-test".into())
    );
    assert_eq!(
      configured_model(
        "codex",
        "profile = \"fast\"
model = \"base\"
[profiles.fast]
model = \"gpt-fast\""
      ),
      Some("gpt-fast".into())
    );
    assert_eq!(
      configured_model("gemini-cli", r#"{"model":{"name":"gemini-test"}}"#),
      Some("gemini-test".into())
    );
    assert_eq!(
      configured_model("opencode", "{
  // comment
  \"model\": \"anthropic/x\"
}"),
      Some("anthropic/x".into())
    );
    assert_eq!(configured_model("pi", "{}"), None);
    assert_eq!(configured_model("pi", "invalid"), None);
  }

  fn facts() -> Facts {
    Facts { exe_exists: true, ..Facts::default() }
  }

  #[test]
  fn env_model_beats_config() {
    let plan = probe_plan("claude-code").unwrap();
    let mut f = facts();
    f.files.insert("settings".into(), r#"{"model":"opus"}"#.into());
    f.env_vals.insert("ANTHROPIC_MODEL".into(), "sonnet".into());
    let r = assess("claude-code", &plan, &f);
    assert_eq!(r.default_model.as_deref(), Some("sonnet"));
    assert_eq!(r.model_source, "env:ANTHROPIC_MODEL");
    f.env_vals.clear();
    let r = assess("claude-code", &plan, &f);
    assert_eq!(r.default_model.as_deref(), Some("opus"));
    assert_eq!(r.model_source, "config:~/.claude/settings.json");
  }

  #[test]
  fn unset_model_is_agent_default() {
    let plan = probe_plan("opencode").unwrap();
    let r = assess("opencode", &plan, &facts());
    assert_eq!(r.default_model, None);
    assert_eq!(r.model_source, "agent-default");
    assert_eq!(r.status, "ready");
  }

  #[test]
  fn auth_states() {
    let pi = probe_plan("pi").unwrap();
    let mut f = facts();
    f.files.insert("auth".into(), "{}".into());
    assert_eq!(assess("pi", &pi, &f).status, "needs_auth");
    f.files.insert("auth".into(), r#"{"anthropic":{}}"#.into());
    assert_eq!(assess("pi", &pi, &f).status, "ready");

    let gem = probe_plan("gemini-cli").unwrap();
    assert_eq!(assess("gemini-cli", &gem, &facts()).status, "needs_auth");
    let mut f = facts();
    f.files.insert(
      "settings".into(),
      r#"{"security":{"auth":{"selectedType":"oauth-personal"}}}"#.into(),
    );
    assert_eq!(assess("gemini-cli", &gem, &f).status, "ready");
    let mut f = facts();
    f.env_set.push("GEMINI_API_KEY".into());
    assert_eq!(assess("gemini-cli", &gem, &f).status, "ready");

    let codex = probe_plan("codex").unwrap();
    let mut f = facts();
    f.sizes.insert("auth".into(), 400);
    assert_eq!(assess("codex", &codex, &f).status, "ready");

    let mut gone = facts();
    gone.exe_exists = false;
    assert_eq!(assess("codex", &codex, &gone).status, "invalid_path");
  }

  #[test]
  fn readiness_output_never_needs_secret_values() {
    let script = wsl_readiness_script("/usr/bin/claude", &probe_plan("claude-code").unwrap());
    // Credential env vars are tested for presence only.
    assert!(script.contains("echo 'E|ANTHROPIC_API_KEY'"));
    assert!(!script.contains("${ANTHROPIC_API_KEY}\""));
    // Credential files are sized, never read.
    assert!(script.contains("wc -c < \"${CLAUDE_CONFIG_DIR:-$HOME/.claude}/.credentials.json\""));
    assert!(!script.contains("base64 -w0 \"${CLAUDE_CONFIG_DIR:-$HOME/.claude}/.credentials.json"));
    let f = parse_readiness_output("X|1
E|ANTHROPIC_API_KEY
S|credentials|0
R|settings|e30=
");
    assert!(f.exe_exists);
    assert_eq!(f.files.get("settings").map(String::as_str), Some("{}"));
  }
}

#[cfg(test)]
mod live_tests {
  use super::*;

  /// Real-machine readiness dump (status + model only, never secrets).
  /// Run: cargo test live_readiness -- --ignored --nocapture
  #[tokio::test]
  #[ignore]
  async fn live_readiness() {
    let mut raw = probe_windows_host().await;
    for d in list_wsl_distros().await {
      raw.extend(probe_wsl_distro(&d).await);
    }
    for (agent, host, spec) in raw {
      let r = probe_readiness(agent.clone(), spec.clone()).await;
      println!(
        "{agent:12} {host:16} {:12} model={:?} src={} reason={:?}",
        r.status, r.default_model, r.model_source, r.reason
      );
    }
  }
}
