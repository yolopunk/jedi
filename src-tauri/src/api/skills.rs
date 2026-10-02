// Skills: SKILL.md instruction packages (Claude Code Agent Skills style).
//
// A skill is a directory under ~/.jedi/skills/<name>/ containing SKILL.md
// (YAML frontmatter + markdown instructions) and optional bundled files
// (references/ scripts/ assets/). Skills carry no executable body — the
// model loads the body via the `skill` tool and acts with the regular
// tools, reading bundled files on demand.
//
// Legacy flat files ~/.jedi/skills/<id>.md are migrated once into
// <name>/SKILL.md (id normalized to kebab-case) when listing.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// SKILL.md 正文大小上限，防止误写超大文件
const MAX_BODY_BYTES: usize = 512 * 1024;
/// 单个附属文件读取上限
const MAX_FILE_BYTES: usize = 1024 * 1024;
/// 附属文件目录遍历深度上限
const MAX_WALK_DEPTH: u8 = 3;

/// 用户技能根目录（~/.jedi/skills/），必要时创建
fn skills_dir() -> Result<PathBuf, String> {
  let home = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
  let dir = home.join(".jedi").join("skills");
  if !dir.exists() {
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建 skills 目录失败: {}", e))?;
  }
  Ok(dir)
}

/// 校验技能名：kebab-case（小写字母/数字/-，1-64 字符，不以 - 开头结尾）
fn validate_name(name: &str) -> Result<(), String> {
  if name.is_empty() || name.len() > 64 {
    return Err("技能名长度需在 1-64 之间".to_string());
  }
  let ok = name
    .chars()
    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
  if !ok {
    return Err("技能名只能包含小写字母、数字和 -".to_string());
  }
  if name.starts_with('-') || name.ends_with('-') || name.contains("--") {
    return Err("技能名不能以 - 开头/结尾或包含连续 -".to_string());
  }
  Ok(())
}

/// 旧版扁平 id 归一化为 kebab-case（小写、_ 转 -、合并连续 -、去首尾 -）
fn normalize_legacy_id(id: &str) -> String {
  let mut out = String::with_capacity(id.len());
  let mut prev_dash = false;
  for c in id.chars() {
    let normalized = if c.is_ascii_uppercase() {
      c.to_ascii_lowercase()
    } else {
      c
    };
    if normalized == '_' {
      if prev_dash {
        continue;
      }
      out.push('-');
      prev_dash = true;
    } else if normalized.is_ascii_lowercase() || normalized.is_ascii_digit() {
      out.push(normalized);
      prev_dash = false;
    } else if normalized == '-' {
      if prev_dash {
        continue;
      }
      out.push('-');
      prev_dash = true;
    }
    // 其他字符丢弃
  }
  out.trim_matches('-').to_string()
}

/// frontmatter 元数据（name/description 必填，icon 可选）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillManifest {
  pub name: String,
  pub description: String,
  pub icon: String,
  /// 磁盘技能固定为 "user"；内置技能由前端打包提供
  pub source: String,
}

/// 技能完整内容（正文 + 附属文件清单）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillDetail {
  pub name: String,
  pub description: String,
  pub icon: String,
  pub source: String,
  pub body: String,
  /// 附属文件相对路径（相对于技能目录），按字典序
  pub files: Vec<String>,
  /// 技能目录绝对路径（供模型用文件工具读取附属文件）
  pub dir: String,
}

/// 编辑器保存载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillSaveDef {
  pub name: String,
  pub description: String,
  pub icon: String,
  pub body: String,
}

/// 解析 SKILL.md 内容为（元数据, 正文）。
/// frontmatter 的 name 若存在必须与目录名一致；缺省时用目录名。
fn parse_skill_content(
  dir_name: &str,
  content: &str,
) -> Result<(SkillManifest, String), String> {
  let trimmed = content.trim_start_matches('\u{feff}');
  let rest = match trimmed.strip_prefix("---\n") {
    Some(r) => r,
    None => return Err(format!("技能 {} 缺少 frontmatter", dir_name)),
  };
  let end = rest
    .find("\n---")
    .ok_or_else(|| format!("技能 {} 的 frontmatter 未闭合", dir_name))?;
  let header = &rest[..end];
  let mut body = &rest[end + 4..]; // 跳过 "\n---"
  // 去掉闭合标记后紧跟的换行，正文从下一行开始
  if let Some(b) = body.strip_prefix('\n') {
    body = b;
  } else if body.starts_with('\r') {
    body = body.trim_start_matches('\r').trim_start_matches('\n');
  }

  let mut name: Option<String> = None;
  let mut description: Option<String> = None;
  let mut icon = "🧩".to_string();

  for line in header.lines() {
    if line.trim().is_empty() {
      continue;
    }
    let Some((key, value)) = line.split_once(':') else {
      continue; // 容错：跳过无法解析的行
    };
    let (key, value) = (key.trim(), value.trim());
    match key {
      "name" if !value.is_empty() => name = Some(value.to_string()),
      "description" if !value.is_empty() => description = Some(value.to_string()),
      "icon" if !value.is_empty() => icon = value.to_string(),
      _ => {}
    }
  }

  if let Some(front_name) = &name {
    if front_name != dir_name {
      return Err(format!(
        "技能 {} 的 frontmatter name ({}) 与目录名不一致",
        dir_name, front_name
      ));
    }
  }
  if body.len() > MAX_BODY_BYTES {
    return Err(format!("技能 {} 正文超过大小上限", dir_name));
  }

  Ok((
    SkillManifest {
      name: dir_name.to_string(),
      description: description
        .ok_or_else(|| format!("技能 {} 缺少 description", dir_name))?,
      icon,
      source: "user".to_string(),
    },
    body.to_string(),
  ))
}

/// 启动/列表时的一次性迁移：把旧版扁平 `<id>.md` 移入 `<name>/SKILL.md`。
/// 返回迁移条数；同名技能目录已存在时跳过（保留旧文件不动）。
fn migrate_legacy_flat_files(dir: &Path) -> usize {
  let Ok(entries) = std::fs::read_dir(dir) else {
    return 0;
  };
  let mut migrated = 0;
  for entry in entries.flatten() {
    let path = entry.path();
    if path.extension().and_then(|e| e.to_str()) != Some("md") {
      continue;
    }
    let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
      continue;
    };
    let name = normalize_legacy_id(stem);
    if name.is_empty() || validate_name(&name).is_err() {
      continue;
    }
    let target_dir = dir.join(&name);
    let target = target_dir.join("SKILL.md");
    if target.exists() {
      continue;
    }
    let Ok(content) = std::fs::read_to_string(&path) else {
      continue;
    };
    // 旧格式 frontmatter 含 name/description/icon/risk；直接搬正文风险小，
    // 但为了通过新解析器校验，重写为仅含必需字段的 frontmatter。
    let Ok((manifest, body)) = parse_legacy_content(&content) else {
      continue;
    };
    if std::fs::create_dir_all(&target_dir).is_err() {
      continue;
    }
    let serialized = format!(
      "---\nname: {}\ndescription: {}\nicon: {}\n---\n{}",
      name, manifest.description, manifest.icon, body
    );
    if std::fs::write(&target, serialized).is_ok() {
      let _ = std::fs::remove_file(&path);
      migrated += 1;
    }
  }
  migrated
}

/// 解析旧版扁平技能文件（frontmatter 至少要有 description，name 用归一化 id 补齐）
fn parse_legacy_content(content: &str) -> Result<(SkillManifest, String), String> {
  let trimmed = content.trim_start_matches('\u{feff}');
  let rest = trimmed
    .strip_prefix("---\n")
    .ok_or_else(|| "缺少 frontmatter".to_string())?;
  let end = rest.find("\n---").ok_or_else(|| "frontmatter 未闭合".to_string())?;
  let header = &rest[..end];
  let mut body = &rest[end + 4..];
  if let Some(b) = body.strip_prefix('\n') {
    body = b;
  }
  let mut description: Option<String> = None;
  let mut icon = "🧩".to_string();
  for line in header.lines() {
    let Some((key, value)) = line.split_once(':') else {
      continue;
    };
    let (key, value) = (key.trim(), value.trim());
    match key {
      "description" if !value.is_empty() => description = Some(value.to_string()),
      "icon" if !value.is_empty() => icon = value.to_string(),
      _ => {}
    }
  }
  Ok((
    SkillManifest {
      name: String::new(), // 迁移时由归一化 id 决定
      description: description.ok_or_else(|| "缺少 description".to_string())?,
      icon,
      source: "user".to_string(),
    },
    body.to_string(),
  ))
}

/// 递归收集技能目录下的附属文件相对路径（跳过 SKILL.md 与隐藏目录）。
/// `base` 恒为技能根目录，用于把深层路径还原为相对路径。
fn collect_bundled_files(dir: &Path, base: &Path, out: &mut Vec<String>, depth: u8) {
  if depth > MAX_WALK_DEPTH {
    return;
  }
  let Ok(entries) = std::fs::read_dir(dir) else {
    return;
  };
  let mut names: Vec<(String, PathBuf)> = entries
    .flatten()
    .filter_map(|e| {
      let name = e.file_name().to_str()?.to_string();
      if name.starts_with('.') {
        return None;
      }
      Some((name, e.path()))
    })
    .collect();
  names.sort_by(|a, b| a.0.cmp(&b.0));
  for (name, path) in names {
    if path.is_dir() {
      collect_bundled_files(&path, base, out, depth + 1);
    } else if name != "SKILL.md" {
      if let Ok(rel) = path.strip_prefix(base) {
        out.push(rel.to_string_lossy().replace('\\', "/"));
      }
    }
  }
}

/// 列出全部用户技能（目录格式；格式非法的目录跳过）
#[tauri::command]
pub fn skills_list() -> Result<Vec<SkillManifest>, String> {
  let dir = skills_dir()?;
  migrate_legacy_flat_files(&dir);
  let mut defs = Vec::new();
  let entries = match std::fs::read_dir(&dir) {
    Ok(e) => e,
    Err(_) => return Ok(defs), // 目录不可读时按空处理
  };
  for entry in entries.flatten() {
    let path = entry.path();
    if !path.is_dir() {
      continue;
    }
    let Some(dir_name) = path.file_name().and_then(|s| s.to_str()) else {
      continue;
    };
    if validate_name(dir_name).is_err() {
      continue;
    }
    let skill_md = path.join("SKILL.md");
    if !skill_md.exists() {
      continue;
    }
    if let Ok(content) = std::fs::read_to_string(&skill_md) {
      if let Ok((manifest, _)) = parse_skill_content(dir_name, &content) {
        defs.push(manifest);
      }
    }
  }
  defs.sort_by(|a, b| a.name.cmp(&b.name));
  Ok(defs)
}

/// 读取一个技能的完整内容（正文 + 附属文件清单）
#[tauri::command]
pub fn skills_read(name: String) -> Result<SkillDetail, String> {
  validate_name(&name)?;
  let skill_dir = skills_dir()?.join(&name);
  let content = std::fs::read_to_string(skill_dir.join("SKILL.md"))
    .map_err(|_| format!("技能 {} 不存在或缺少 SKILL.md", name))?;
  let (manifest, body) = parse_skill_content(&name, &content)?;
  let mut files = Vec::new();
  collect_bundled_files(&skill_dir, &skill_dir, &mut files, 0);
  Ok(SkillDetail {
    name: manifest.name,
    description: manifest.description,
    icon: manifest.icon,
    source: manifest.source,
    body,
    files,
    dir: skill_dir.to_string_lossy().to_string(),
  })
}

/// 读取技能的附属文件（rel_path 相对技能目录，禁止穿越）
#[tauri::command]
pub fn skills_read_file(name: String, rel_path: String) -> Result<String, String> {
  validate_name(&name)?;
  if rel_path.is_empty() || rel_path.len() > 512 {
    return Err("文件路径长度非法".to_string());
  }
  let skill_root = skills_dir()?.join(&name);
  let canonical_root = std::fs::canonicalize(&skill_root)
    .map_err(|_| format!("技能 {} 不存在", name))?;
  let target = skill_root.join(&rel_path);
  let canonical_target = std::fs::canonicalize(&target)
    .map_err(|_| format!("文件 {} 不存在", rel_path))?;
  if !canonical_target.starts_with(&canonical_root) {
    return Err("文件路径越界".to_string());
  }
  if !canonical_target.is_file() {
    return Err("目标不是文件".to_string());
  }
  let meta = std::fs::metadata(&canonical_target).map_err(|e| e.to_string())?;
  if meta.len() > MAX_FILE_BYTES as u64 {
    return Err("文件超过 1MB 读取上限".to_string());
  }
  std::fs::read_to_string(&canonical_target).map_err(|e| format!("读取文件失败: {}", e))
}

/// 保存（新建或覆盖）一个用户技能（目录制）
#[tauri::command]
pub fn skills_save(def: SkillSaveDef) -> Result<SkillManifest, String> {
  let name = def.name.trim().to_string();
  let description = def.description.trim().to_string();
  let icon = if def.icon.trim().is_empty() {
    "🧩".to_string()
  } else {
    def.icon.trim().to_string()
  };
  validate_name(&name)?;
  if description.is_empty() {
    return Err("技能描述不能为空".to_string());
  }
  if def.body.len() > MAX_BODY_BYTES {
    return Err("正文超过大小上限".to_string());
  }
  let skill_dir = skills_dir()?.join(&name);
  std::fs::create_dir_all(&skill_dir).map_err(|e| format!("创建技能目录失败: {}", e))?;
  let serialized = format!("---\nname: {}\ndescription: {}\nicon: {}\n---\n{}", name, description, icon, def.body);
  std::fs::write(skill_dir.join("SKILL.md"), serialized)
    .map_err(|e| format!("写入 SKILL.md 失败: {}", e))?;
  Ok(SkillManifest {
    name,
    description,
    icon,
    source: "user".to_string(),
  })
}

/// 删除一个用户技能目录；不存在时返回 false 而非报错
#[tauri::command]
pub fn skills_delete(name: String) -> Result<bool, String> {
  validate_name(&name)?;
  let dir = skills_dir()?.join(&name);
  match std::fs::remove_dir_all(&dir) {
    Ok(_) => Ok(true),
    Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
    Err(e) => Err(format!("删除技能目录失败: {}", e)),
  }
}

// ---------------------------------------------------------------------------
// Skill execution stats
//
// Aggregated in the frontend (stores/skillStats.ts) and flushed in batches;
// the backend only persists the map and enforces hard caps (recent_errors
// kept to the newest 10, message truncated to 200 chars) so a hand-edited
// file cannot grow unbounded.

const MAX_RECENT_ERRORS: usize = 10;
const MAX_ERROR_MSG_CHARS: usize = 200;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SkillStatError {
  pub ts: u64,
  pub msg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SkillStatEntry {
  pub calls: u64,
  pub failures: u64,
  pub total_ms: u64,
  pub last_used_at: u64,
  #[serde(default)]
  pub recent_errors: Vec<SkillStatError>,
}

/// 统计文件路径（~/.jedi/skill_stats.json），必要时创建目录
fn stats_path() -> Result<PathBuf, String> {
  let home = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
  let dir = home.join(".jedi");
  if !dir.exists() {
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建 .jedi 目录失败: {}", e))?;
  }
  Ok(dir.join("skill_stats.json"))
}

fn load_stats() -> Result<BTreeMap<String, SkillStatEntry>, String> {
  let path = stats_path()?;
  Ok(
    std::fs::read_to_string(&path)
      .ok()
      .and_then(|s| serde_json::from_str(&s).ok())
      .unwrap_or_default(),
  )
}

/// 落盘前强制截断：错误只保留最近 N 条、单条消息截断到上限字符
fn clamp_entry(entry: &mut SkillStatEntry) {
  if entry.recent_errors.len() > MAX_RECENT_ERRORS {
    let keep_from = entry.recent_errors.len() - MAX_RECENT_ERRORS;
    entry.recent_errors.drain(0..keep_from);
  }
  for err in &mut entry.recent_errors {
    if err.msg.chars().count() > MAX_ERROR_MSG_CHARS {
      err.msg = err.msg.chars().take(MAX_ERROR_MSG_CHARS).collect();
    }
  }
}

/// 列出全部技能执行统计
#[tauri::command]
pub fn skill_stats_list() -> Result<BTreeMap<String, SkillStatEntry>, String> {
  load_stats()
}

/// 整表保存（前端防抖批量写），保存前强制截断
#[tauri::command]
pub fn skill_stats_save(stats: BTreeMap<String, SkillStatEntry>) -> Result<(), String> {
  let mut stats = stats;
  for entry in stats.values_mut() {
    clamp_entry(entry);
  }
  let path = stats_path()?;
  let s = serde_json::to_string_pretty(&stats).map_err(|e| e.to_string())?;
  std::fs::write(&path, s).map_err(|e| format!("写入统计文件失败: {}", e))
}

/// 清零全部统计
#[tauri::command]
pub fn skill_stats_clear() -> Result<(), String> {
  let path = stats_path()?;
  std::fs::write(&path, "{}").map_err(|e| format!("清空统计文件失败: {}", e))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_roundtrip() {
    let content = "---\nname: deploy-frontend\ndescription: 部署前端项目\nicon: 🚀\n---\n# 步骤\n1. 构建";
    let (manifest, body) = parse_skill_content("deploy-frontend", content).unwrap();
    assert_eq!(manifest.name, "deploy-frontend");
    assert_eq!(manifest.description, "部署前端项目");
    assert_eq!(manifest.icon, "🚀");
    assert_eq!(manifest.source, "user");
    assert_eq!(body, "# 步骤\n1. 构建");
  }

  #[test]
  fn parse_defaults_icon_and_name() {
    // name 缺省时用目录名；icon 用默认值
    let content = "---\ndescription: d\n---\nbody";
    let (manifest, body) = parse_skill_content("my-skill", content).unwrap();
    assert_eq!(manifest.name, "my-skill");
    assert_eq!(manifest.icon, "🧩");
    assert_eq!(body, "body");
  }

  #[test]
  fn parse_rejects_name_mismatch() {
    let content = "---\nname: other-name\ndescription: d\n---\nbody";
    assert!(parse_skill_content("my-skill", content).is_err());
  }

  #[test]
  fn parse_rejects_missing_frontmatter() {
    assert!(parse_skill_content("a", "只有正文没有 frontmatter").is_err());
  }

  #[test]
  fn parse_rejects_unclosed_frontmatter() {
    assert!(parse_skill_content("a", "---\nname: a\n").is_err());
  }

  #[test]
  fn parse_rejects_missing_description() {
    assert!(parse_skill_content("a", "---\nname: a\n---\nbody").is_err());
  }

  #[test]
  fn parse_description_with_colon() {
    let content = "---\ndescription: 注意: 冒号测试\n---\nbody";
    let (manifest, _) = parse_skill_content("a", content).unwrap();
    assert_eq!(manifest.description, "注意: 冒号测试");
  }

  #[test]
  fn parse_skips_legacy_risk_field() {
    let content = "---\nname: a\ndescription: d\nrisk: write\n---\nbody";
    let (manifest, body) = parse_skill_content("a", content).unwrap();
    assert_eq!(manifest.description, "d");
    assert_eq!(body, "body");
  }

  #[test]
  fn validate_name_accepts_kebab() {
    assert!(validate_name("deploy-frontend").is_ok());
    assert!(validate_name("my-skill-2").is_ok());
    assert!(validate_name("a").is_ok());
  }

  #[test]
  fn validate_name_rejects_bad_names() {
    assert!(validate_name("../evil").is_err());
    assert!(validate_name("a/b").is_err());
    assert!(validate_name("a\\b").is_err());
    assert!(validate_name("").is_err());
    assert!(validate_name("UPPER").is_err());
    assert!(validate_name("中文").is_err());
    assert!(validate_name("-lead").is_err());
    assert!(validate_name("trail-").is_err());
    assert!(validate_name("double--dash").is_err());
    let long = "a".repeat(65);
    assert!(validate_name(&long).is_err());
  }

  #[test]
  fn normalize_legacy_id_converts() {
    assert_eq!(normalize_legacy_id("DEPLOY_FRONTEND"), "deploy-frontend");
    assert_eq!(normalize_legacy_id("my__skill"), "my-skill");
    assert_eq!(normalize_legacy_id("-weird_id-"), "weird-id");
    // CJK 字符丢弃，其后合法片段保留
    assert_eq!(normalize_legacy_id("中文_id"), "id");
    assert_eq!(normalize_legacy_id("中文"), "");
  }

  fn temp_dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!(
      "jedi-skills-test-{}-{}",
      tag,
      std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
    ));
    std::fs::create_dir_all(&d).unwrap();
    d
  }

  #[test]
  fn migration_moves_flat_file_into_directory() {
    let dir = temp_dir("migrate");
    std::fs::write(
      dir.join("deploy_old.md"),
      "---\nname: whatever\ndescription: 旧技能\nicon: 🚀\nrisk: read\n---\n旧正文",
    )
    .unwrap();
    assert_eq!(migrate_legacy_flat_files(&dir), 1);
    assert!(!dir.join("deploy_old.md").exists());
    let migrated = std::fs::read_to_string(dir.join("deploy-old").join("SKILL.md")).unwrap();
    assert!(migrated.contains("name: deploy-old"));
    assert!(migrated.contains("旧正文"));
    assert!(!migrated.contains("risk:"));
    std::fs::remove_dir_all(&dir).ok();
  }

  #[test]
  fn migration_skips_existing_target() {
    let dir = temp_dir("migrate-skip");
    let skill_dir = dir.join("deploy-old");
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(skill_dir.join("SKILL.md"), "---\ndescription: new\n---\nnew").unwrap();
    std::fs::write(
      dir.join("deploy_old.md"),
      "---\ndescription: old\n---\nold",
    )
    .unwrap();
    assert_eq!(migrate_legacy_flat_files(&dir), 0);
    assert!(dir.join("deploy_old.md").exists());
    assert_eq!(
      std::fs::read_to_string(skill_dir.join("SKILL.md")).unwrap(),
      "---\ndescription: new\n---\nnew"
    );
    std::fs::remove_dir_all(&dir).ok();
  }

  #[test]
  fn migration_ignores_files_without_description() {
    let dir = temp_dir("migrate-invalid");
    std::fs::write(dir.join("no_desc.md"), "---\nname: x\n---\nbody").unwrap();
    std::fs::write(dir.join("plain.md"), "没有 frontmatter").unwrap();
    assert_eq!(migrate_legacy_flat_files(&dir), 0);
    assert!(dir.join("no_desc.md").exists());
    assert!(dir.join("plain.md").exists());
    std::fs::remove_dir_all(&dir).ok();
  }

  #[test]
  fn collect_bundled_files_walks_and_sorts() {
    let dir = temp_dir("collect");
    let refs = dir.join("references");
    let scripts = dir.join("scripts");
    std::fs::create_dir_all(&refs).unwrap();
    std::fs::create_dir_all(&scripts).unwrap();
    std::fs::write(dir.join("SKILL.md"), "x").unwrap();
    std::fs::write(refs.join("aws.md"), "x").unwrap();
    std::fs::write(refs.join("gcp.md"), "x").unwrap();
    std::fs::write(scripts.join("run.sh"), "x").unwrap();
    std::fs::create_dir_all(dir.join(".hidden")).unwrap();
    std::fs::write(dir.join(".hidden").join("x.md"), "x").unwrap();
    let mut out = Vec::new();
    collect_bundled_files(&dir, &dir, &mut out, 0);
    assert_eq!(out, vec!["references/aws.md", "references/gcp.md", "scripts/run.sh"]);
    std::fs::remove_dir_all(&dir).ok();
  }

  fn stat_entry(errors: Vec<SkillStatError>) -> SkillStatEntry {
    SkillStatEntry {
      calls: 5,
      failures: errors.len() as u64,
      total_ms: 100,
      last_used_at: 1,
      recent_errors: errors,
    }
  }

  #[test]
  fn stats_clamp_keeps_newest_ten_errors() {
    let errors: Vec<SkillStatError> = (0..15)
      .map(|i| SkillStatError { ts: i, msg: format!("err-{}", i) })
      .collect();
    let mut entry = stat_entry(errors);
    clamp_entry(&mut entry);
    assert_eq!(entry.recent_errors.len(), 10);
    // 保留最近 10 条（ts 5..=14），丢最早的
    assert_eq!(entry.recent_errors[0].ts, 5);
    assert_eq!(entry.recent_errors[9].ts, 14);
  }

  #[test]
  fn stats_clamp_truncates_long_messages() {
    let errors = vec![SkillStatError { ts: 1, msg: "x".repeat(500) }];
    let mut entry = stat_entry(errors);
    clamp_entry(&mut entry);
    assert_eq!(entry.recent_errors[0].msg.chars().count(), 200);
  }

  #[test]
  fn stats_clamp_noop_on_small_entries() {
    let errors = vec![SkillStatError { ts: 1, msg: "boom".to_string() }];
    let mut entry = stat_entry(errors);
    let before = entry.clone();
    clamp_entry(&mut entry);
    assert_eq!(entry, before);
  }
}
