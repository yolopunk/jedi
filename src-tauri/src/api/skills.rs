// User-defined custom skills for the AI agent.
//
// Custom skills are prompt-style skills authored by the user as markdown
// files (frontmatter + prompt body) under ~/.jedi/skills/. The frontend
// lists them via skills_list_custom and bridges each into the in-memory
// skill registry; executing one renders {{placeholder}} args into the
// prompt and returns it as the tool result for the model to act on.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// 单个提示词正文的大小上限（64KB），防止误写超大文件
const MAX_PROMPT_BYTES: usize = 64 * 1024;

/// 自定义技能目录（~/.jedi/skills/），必要时创建
fn skills_dir() -> Result<PathBuf, String> {
  let home = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
  let dir = home.join(".jedi").join("skills");
  if !dir.exists() {
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建 skills 目录失败: {}", e))?;
  }
  Ok(dir)
}

/// 校验技能 id：仅允许小写字母/数字/-/_，从根上杜绝路径穿越
fn validate_id(id: &str) -> Result<(), String> {
  if id.is_empty() || id.len() > 64 {
    return Err("技能 id 长度需在 1-64 之间".to_string());
  }
  let ok = id
    .chars()
    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_');
  if ok {
    Ok(())
  } else {
    Err("技能 id 只能包含小写字母、数字、- 和 _".to_string())
  }
}

/// 校验风险级别取值
fn validate_risk(risk: &str) -> Result<(), String> {
  if matches!(risk, "read" | "write" | "system") {
    Ok(())
  } else {
    Err("risk 只能是 read / write / system".to_string())
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomSkillDef {
  pub id: String,
  pub name: String,
  pub description: String,
  pub icon: String,
  pub risk: String,
  pub prompt: String,
}

/// 序列化为落盘格式（frontmatter + 提示词正文）
fn serialize_skill(def: &CustomSkillDef) -> String {
  format!(
    "---\nname: {}\ndescription: {}\nicon: {}\nrisk: {}\n---\n{}",
    def.name, def.description, def.icon, def.risk, def.prompt
  )
}

/// 解析单个技能文件内容。无 frontmatter 或缺 name/description 视为格式错误。
fn parse_skill_content(id: &str, content: &str) -> Result<CustomSkillDef, String> {
  let trimmed = content.trim_start_matches('\u{feff}');
  let rest = match trimmed.strip_prefix("---\n") {
    Some(r) => r,
    None => return Err(format!("技能 {} 缺少 frontmatter", id)),
  };
  let end = rest.find("\n---").ok_or_else(|| format!("技能 {} 的 frontmatter 未闭合", id))?;
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
  let mut risk = "read".to_string();

  for line in header.lines() {
    if line.trim().is_empty() {
      continue;
    }
    let Some((key, value)) = line.split_once(':') else {
      continue; // 容错：跳过无法解析的行
    };
    let (key, value) = (key.trim(), value.trim());
    match key {
      "name" => name = Some(value.to_string()),
      "description" => description = Some(value.to_string()),
      "icon" if !value.is_empty() => icon = value.to_string(),
      "risk" if matches!(value, "read" | "write" | "system") => risk = value.to_string(),
      _ => {}
    }
  }

  Ok(CustomSkillDef {
    id: id.to_string(),
    name: name.ok_or_else(|| format!("技能 {} 缺少 name", id))?,
    description: description.ok_or_else(|| format!("技能 {} 缺少 description", id))?,
    icon,
    risk,
    prompt: body.to_string(),
  })
}

/// 列出 ~/.jedi/skills/ 下所有自定义技能（格式非法的文件会被跳过）
#[tauri::command]
pub fn skills_list_custom() -> Result<Vec<CustomSkillDef>, String> {
  let dir = skills_dir()?;
  let mut defs = Vec::new();
  let entries = match std::fs::read_dir(&dir) {
    Ok(e) => e,
    Err(_) => return Ok(defs), // 目录不可读时按空处理
  };
  for entry in entries.flatten() {
    let path = entry.path();
    if path.extension().and_then(|e| e.to_str()) != Some("md") {
      continue;
    }
    let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
      continue;
    };
    if let Ok(content) = std::fs::read_to_string(&path) {
      if let Ok(def) = parse_skill_content(stem, &content) {
        defs.push(def);
      }
    }
  }
  defs.sort_by(|a, b| a.id.cmp(&b.id));
  Ok(defs)
}

/// 保存（新建或覆盖）一个自定义技能
#[tauri::command]
pub fn skills_save(skill: CustomSkillDef) -> Result<CustomSkillDef, String> {
  let def = CustomSkillDef {
    name: skill.name.trim().to_string(),
    description: skill.description.trim().to_string(),
    icon: if skill.icon.trim().is_empty() { "🧩".to_string() } else { skill.icon.trim().to_string() },
    risk: if skill.risk.is_empty() { "read".to_string() } else { skill.risk },
    ..skill
  };
  validate_id(&def.id)?;
  if def.name.is_empty() {
    return Err("技能名称不能为空".to_string());
  }
  if def.description.is_empty() {
    return Err("技能描述不能为空".to_string());
  }
  validate_risk(&def.risk)?;
  if def.prompt.len() > MAX_PROMPT_BYTES {
    return Err("提示词正文超过 64KB 上限".to_string());
  }
  let path = skills_dir()?.join(format!("{}.md", def.id));
  std::fs::write(&path, serialize_skill(&def)).map_err(|e| format!("写入技能文件失败: {}", e))?;
  Ok(def)
}

/// 删除一个自定义技能；文件不存在时返回 false 而非报错
#[tauri::command]
pub fn skills_delete(id: String) -> Result<bool, String> {
  validate_id(&id)?;
  let path = skills_dir()?.join(format!("{}.md", id));
  match std::fs::remove_file(&path) {
    Ok(_) => Ok(true),
    Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
    Err(e) => Err(format!("删除技能文件失败: {}", e)),
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

  fn def() -> CustomSkillDef {
    CustomSkillDef {
      id: "deploy-frontend".to_string(),
      name: "DEPLOY_FRONTEND".to_string(),
      description: "部署前端项目到测试环境".to_string(),
      icon: "🚀".to_string(),
      risk: "read".to_string(),
      prompt: "请部署项目：{{project}}".to_string(),
    }
  }

  #[test]
  fn parse_roundtrip() {
    let content = serialize_skill(&def());
    let parsed = parse_skill_content("deploy-frontend", &content).unwrap();
    assert_eq!(parsed.name, "DEPLOY_FRONTEND");
    assert_eq!(parsed.description, "部署前端项目到测试环境");
    assert_eq!(parsed.icon, "🚀");
    assert_eq!(parsed.risk, "read");
    assert_eq!(parsed.prompt, "请部署项目：{{project}}");
  }

  #[test]
  fn parse_defaults_icon_and_risk() {
    let content = "---\nname: A\ndescription: d\n---\nbody";
    let parsed = parse_skill_content("a", content).unwrap();
    assert_eq!(parsed.icon, "🧩");
    assert_eq!(parsed.risk, "read");
    assert_eq!(parsed.prompt, "body");
  }

  #[test]
  fn parse_rejects_missing_frontmatter() {
    assert!(parse_skill_content("a", "只有正文没有 frontmatter").is_err());
  }

  #[test]
  fn parse_rejects_unclosed_frontmatter() {
    assert!(parse_skill_content("a", "---\nname: A\n").is_err());
  }

  #[test]
  fn parse_rejects_missing_required_fields() {
    assert!(parse_skill_content("a", "---\nname: A\n---\nbody").is_err());
    assert!(parse_skill_content("a", "---\ndescription: d\n---\nbody").is_err());
  }

  #[test]
  fn parse_invalid_risk_falls_back_to_read() {
    let content = "---\nname: A\ndescription: d\nrisk: yolo\n---\nbody";
    let parsed = parse_skill_content("a", content).unwrap();
    assert_eq!(parsed.risk, "read");
  }

  #[test]
  fn parse_description_with_colon() {
    let content = "---\nname: A\ndescription: 注意: 冒号测试\n---\nbody";
    let parsed = parse_skill_content("a", content).unwrap();
    assert_eq!(parsed.description, "注意: 冒号测试");
  }

  #[test]
  fn validate_id_accepts_normal_ids() {
    assert!(validate_id("deploy-frontend").is_ok());
    assert!(validate_id("my_skill_2").is_ok());
  }

  #[test]
  fn validate_id_rejects_traversal_and_bad_chars() {
    assert!(validate_id("../evil").is_err());
    assert!(validate_id("a/b").is_err());
    assert!(validate_id("a\\b").is_err());
    assert!(validate_id("").is_err());
    assert!(validate_id("UPPER").is_err());
    assert!(validate_id("中文").is_err());
    let long = "a".repeat(65);
    assert!(validate_id(&long).is_err());
  }

  #[test]
  fn validate_risk_only_allows_known_values() {
    assert!(validate_risk("read").is_ok());
    assert!(validate_risk("write").is_ok());
    assert!(validate_risk("system").is_ok());
    assert!(validate_risk("other").is_err());
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
