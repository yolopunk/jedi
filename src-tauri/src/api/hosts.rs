use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::PathBuf;
use std::time::Duration;
use tauri::{AppHandle, Manager};

pub const HOSTS_PATH: &str = if cfg!(any(target_os = "linux", target_os = "macos")) {
  "/etc/hosts"
} else if cfg!(target_os = "windows") {
  r"C:\Windows\System32\drivers\etc\hosts"
} else {
  "/etc/hosts"
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HostEntry {
  pub ip: String,
  pub domain: String,
  pub disabled: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GroupHosts {
  pub name: String,
  pub hosts: Vec<HostEntry>,
}

pub fn load_local_config(_app: &AppHandle) -> Result<Vec<GroupHosts>, Box<dyn Error>> {
  // 不再从文件读取，直接返回默认配置
  let default_hosts = vec![
    HostEntry {
      ip: "127.0.0.1".to_string(),
      domain: "localhost".to_string(),
      disabled: false,
    },
    HostEntry {
      ip: "::1".to_string(),
      domain: "localhost".to_string(),
      disabled: false,
    },
    HostEntry {
      ip: "127.0.0.1".to_string(),
      domain: "example.test".to_string(),
      disabled: false,
    },
  ];

  let default_group = GroupHosts {
    name: "开发环境".to_string(),
    hosts: default_hosts,
  };

  Ok(vec![default_group])
}

const HOSTS_REQUEST_TIMEOUT_SECS: u64 = 20;
const HOSTS_USER_AGENT: &str = "Jedi/1.0";

/// 拉取 URL 远程配置并解析为分组数据（URL 需返回 Jedi JSON 分组格式）
async fn fetch_url_groups(url: &str) -> Result<Vec<GroupHosts>, Box<dyn Error>> {
  let client = Client::builder()
    .user_agent(HOSTS_USER_AGENT)
    .timeout(Duration::from_secs(HOSTS_REQUEST_TIMEOUT_SECS))
    .build()?;
  let resp = client.get(url).send().await?;
  let status = resp.status();
  if !status.is_success() {
    return Err(format!("HTTP {}", status).into());
  }
  let text = resp.text().await?;
  let groups: Vec<GroupHosts> = serde_json::from_str(&text)?;
  Ok(groups)
}

/// 获取远程 hosts 分组配置
#[tauri::command]
pub async fn fetch_remote_config(url: String) -> Result<Vec<GroupHosts>, String> {
  fetch_url_groups(&url).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_hosts_with_groups(
  app: AppHandle,
  source: String,
  url: Option<String>,
  groups: Option<Vec<GroupHosts>>,
) -> Result<String, String> {
  let groups_result = if source == "remote" {
    if let Some(u) = url {
      fetch_url_groups(&u).await.map_err(|e| e.to_string())
    } else {
      return Err("Remote source requires URL".to_string());
    }
  } else if source == "current" {
    // 使用前端传递的分组数据
    if let Some(g) = groups {
      Ok(g)
    } else {
      return Err("Current source requires groups".to_string());
    }
  } else if source == "default" {
    // 创建默认的分组和条目
    let default_hosts = vec![
      HostEntry {
        ip: "127.0.0.1".to_string(),
        domain: "localhost".to_string(),
        disabled: false,
      },
      HostEntry {
        ip: "::1".to_string(),
        domain: "localhost".to_string(),
        disabled: false,
      },
      HostEntry {
        ip: "127.0.0.1".to_string(),
        domain: "example.test".to_string(),
        disabled: false,
      },
    ];

    let default_group = GroupHosts {
      name: "开发环境".to_string(),
      hosts: default_hosts,
    };

    Ok(vec![default_group])
  } else {
    load_local_config(&app).map_err(|e| e.to_string())
  };

  let groups = match groups_result {
    Ok(g) => g,
    Err(e) => return Err(e),
  };

  // 读不到现有 hosts 内容时不能继续写：那样会把整份文件重建成只剩 Jedi 段。
  let hosts_content = std::fs::read_to_string(HOSTS_PATH)
    .map_err(|e| format!("读取 hosts 文件失败（可能需要管理员权限）: {}", e))?;

  // 分析原始 hosts 文件，提取非 Jedi 管理部分
  let mut new_lines: Vec<String> = Vec::new();
  let mut in_jedi_section = false;

  for line in hosts_content.lines() {
    let trimmed = line.trim_start();

    // 检查是否进入 Jedi 管理部分
    if trimmed.starts_with("# === JEDI HOSTS MANAGER ===") {
      in_jedi_section = true;
      continue;
    }

    // 检查是否离开 Jedi 管理部分
    if trimmed.starts_with("# === END JEDI HOSTS MANAGER ===") {
      in_jedi_section = false;
      continue;
    }

    // 如果不在 Jedi 管理部分中，添加到非 Jedi 行
    if !in_jedi_section {
      new_lines.push(line.to_string());
    }
  }

  new_lines.push("# === JEDI HOSTS MANAGER ===".to_string());

  for g in &groups {
    // 创建分组标题，表格式格式
    let group_name = g.name.clone();
    new_lines.push(format!("# +{}+", group_name));

    // 按域名排序显示
    let mut sorted_hosts: Vec<HostEntry> = g.hosts.clone();
    sorted_hosts.sort_by(|a, b| a.domain.cmp(&b.domain));

    for host in sorted_hosts {
      if host.disabled {
        // 如果禁用，添加注释符号
        new_lines.push(format!("# {} {}", host.ip, host.domain));
      } else {
        // 如果启用，正常显示
        new_lines.push(format!("{} {}", host.ip, host.domain));
      }
    }

    // 不需要分组结束标记，下一个分组开始即表示上一个分组结束
  }

  new_lines.push("# === END JEDI HOSTS MANAGER ===".to_string());

  let new_content = new_lines.join("\n") + "\n";

  // 写入失败必须回传：静默吞掉会让前端以为已经生效（例如没有管理员权限时）
  std::fs::write(HOSTS_PATH, new_content)
    .map_err(|e| format!("写入 hosts 文件失败（可能需要管理员权限）: {}", e))?;

  Ok("Hosts updated successfully".to_string())
}

#[tauri::command]
pub fn revert_hosts() -> Result<String, String> {
  // 读取hosts文件
  let hosts_content = std::fs::read_to_string(HOSTS_PATH)
    .map_err(|e| format!("读取 hosts 文件失败（可能需要管理员权限）: {}", e))?;

  let mut new_lines = Vec::new();
  let mut in_jedi_section = false;
  let mut jedi_section_lines = Vec::new();

  // 首先收集所有非Jedi部分的行
  for line in hosts_content.lines() {
    let trimmed = line.trim_start();

    if trimmed.starts_with("# === JEDI HOSTS MANAGER ===") {
      in_jedi_section = true;
      jedi_section_lines.push(line.to_string());
      continue;
    }

    if trimmed.starts_with("# === END JEDI HOSTS MANAGER ===") {
      in_jedi_section = false;
      jedi_section_lines.push(line.to_string());
      continue;
    }

    if in_jedi_section {
      // 如果是Jedi部分，收集这些行
      jedi_section_lines.push(line.to_string());
    } else {
      // 如果不是Jedi部分，直接添加到新行中
      new_lines.push(line.to_string());
    }
  }

  // 如果没有找到Jedi部分，直接返回
  if jedi_section_lines.is_empty() {
    return Ok("No Jedi section found in hosts file.".to_string());
  }

  // 处理Jedi部分，确保所有hosts条目都被注释
  let mut in_group = false;
  for line in &jedi_section_lines {
    let trimmed = line.trim_start();

    if trimmed.starts_with("# ===")
      || trimmed.starts_with("# +")
      || line.trim().is_empty()
      || (trimmed.starts_with("#") && trimmed.len() == 1)
    {
      // 保留分组行和分隔线
      new_lines.push(line.to_string());
      if trimmed.starts_with("# +") && trimmed.ends_with("+") {
        in_group = true;
      }
    } else if in_group {
      // 如果是hosts条目，确保它被注释
      if !trimmed.starts_with('#') {
        // 如果还没有注释，添加注释
        new_lines.push(format!("# {}", line));
      } else if trimmed.starts_with("# ")
        && !trimmed.starts_with("# ===")
        && !trimmed.starts_with("# +")
      {
        // 如果是被注释的hosts条目，保持注释
        new_lines.push(line.to_string());
      } else {
        // 其他注释行直接添加
        new_lines.push(line.to_string());
      }
    } else {
      // 其他行直接添加
      new_lines.push(line.to_string());
    }
  }

  // 写入新的hosts文件（失败必须回传，否则界面会谎报「已禁用」）
  let new_content = new_lines.join("\n");

  std::fs::write(HOSTS_PATH, new_content)
    .map_err(|e| format!("写入 hosts 文件失败（可能需要管理员权限）: {}", e))?;

  Ok("Hosts entries disabled successfully.".to_string())
}

#[tauri::command]
pub fn get_hosts_path() -> String {
  HOSTS_PATH.to_string()
}

#[tauri::command]
pub fn read_system_hosts() -> Result<Vec<GroupHosts>, String> {
  // 读取系统hosts文件
  let hosts_content = std::fs::read_to_string(HOSTS_PATH)
    .map_err(|e| format!("读取 hosts 文件失败: {}", e))?;

  // 初始化一个空的结果数组，不再创建默认分组
  let mut result = Vec::new();

  let mut current_group: Option<String> = None;
  let mut current_hosts: Vec<HostEntry> = Vec::new();
  let mut in_jedi_section = false;

  // 解析hosts文件内容
  for line in hosts_content.lines() {
    let trimmed = line.trim_start();

    // 检查是否进入Jedi管理的部分
    if trimmed.starts_with("# === JEDI HOSTS MANAGER ===") {
      // 如果已经在Jedi部分中，先完成当前部分的处理
      if in_jedi_section {
        // 保存最后一个分组的数据
        if let Some(name) = current_group.take() {
          if !current_hosts.is_empty() {
            result.push(GroupHosts {
              name,
              hosts: current_hosts,
            });
            current_hosts = Vec::new();
          }
        }
      }

      // 开始新的Jedi部分
      in_jedi_section = true;

      // 清除之前的结果，只保留默认分组
      if result.len() > 1 {
        result.truncate(1);
      }

      continue;
    }

    // 检查是否离开Jedi管理的部分
    if trimmed.starts_with("# === END JEDI HOSTS MANAGER ===") {
      in_jedi_section = false;

      // 保存最后一个分组的数据
      if let Some(name) = current_group.take() {
        if !current_hosts.is_empty() {
          result.push(GroupHosts {
            name,
            hosts: current_hosts,
          });
          current_hosts = Vec::new();
        }
      }
      continue;
    }

    // 如果不在Jedi管理的部分，跳过
    if !in_jedi_section {
      continue;
    }

    // 解析分组行
    if trimmed.starts_with("# +") && trimmed.ends_with("+") {
      // 保存之前的分组数据（如果有）
      if let Some(name) = current_group.take() {
        if !current_hosts.is_empty() {
          result.push(GroupHosts {
            name,
            hosts: current_hosts,
          });
          current_hosts = Vec::new();
        }
      }

      // 提取新分组
      // 从分组行提取分组名称
      let start_idx = "# +".len();
      let end_idx = trimmed.len() - 1; // 去除右加号
      let name = trimmed[start_idx..end_idx].trim().to_string();

      current_group = Some(name);
      continue;
    }

    // 解析hosts条目 - 处理注释行和非注释行
    if current_group.is_some()
      && !trimmed.is_empty()
      && !trimmed.starts_with("# +")
      && !trimmed.starts_with("# ===")
    {
      // 检查是否是注释行（已禁用的条目）
      let (line_to_parse, is_disabled) = if trimmed.starts_with("# ")
        && !trimmed.starts_with("# +")
        && !trimmed.starts_with("# ===")
      {
        // 如果是普通注释行，去除注释符号
        (trimmed[2..].to_string(), true)
      } else if trimmed.starts_with('#')
        && !trimmed.starts_with("#+")
        && !trimmed.starts_with("#===")
      {
        // 如果只有#没有空格，也去除
        (trimmed[1..].to_string(), true)
      } else {
        (trimmed.to_string(), false)
      };

      // 解析IP和域名
      let parts: Vec<&str> = line_to_parse.split_whitespace().collect();
      if parts.len() >= 2 {
        let ip = parts[0].to_string();

        // 支持一行多个域名 (Aliases)
        for part in parts.iter().skip(1) {
          let domain = part.to_string();
          // 跳过注释部分（如果在行尾有注释）
          if domain.starts_with('#') {
            break;
          }

          // 检查是否已经存在相同的域名，避免重复
          let domain_exists = current_hosts.iter().any(|h| h.domain == domain);
          if !domain_exists {
            current_hosts.push(HostEntry {
              ip: ip.clone(),
              domain,
              disabled: is_disabled,
            });
          }
        }
      }
    }
  }

  // 如果文件结束时还有未保存的分组数据，保存它
  if let Some(name) = current_group {
    if !current_hosts.is_empty() {
      result.push(GroupHosts {
        name,
        hosts: current_hosts,
      });
    }
  }

  if result.is_empty() {
    let default_hosts = vec![
      HostEntry {
        ip: "127.0.0.1".to_string(),
        domain: "localhost".to_string(),
        disabled: false,
      },
      HostEntry {
        ip: "255.255.255.255".to_string(),
        domain: "broadcasthost".to_string(),
        disabled: false,
      },
      HostEntry {
        ip: "::1".to_string(),
        domain: "localhost".to_string(),
        disabled: false,
      },
    ];

    result.push(GroupHosts {
      name: "System Default".to_string(),
      hosts: default_hosts,
    });
  }

  Ok(result)
}

// ==================== 远程数据源 ====================

/// 远程 hosts 数据源：一个返回 Jedi JSON 分组格式的 URL，
/// 与分组一一对应。URL 本身携带全部必要参数（如兜底 IP、命名空间），
/// Jedi 不感知数据源背后的实现细节。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HostsSource {
  /// 分组名
  pub name: String,
  /// 数据源 URL
  pub url: String,
  /// 上次同步时间（Unix 秒）
  #[serde(default)]
  pub last_synced_at: Option<u64>,
}

// ==================== 远程数据源持久化 ====================

fn get_hosts_sources_path(app: &AppHandle) -> Result<PathBuf, String> {
  let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
  if !app_data_dir.exists() {
    std::fs::create_dir_all(&app_data_dir).map_err(|e| e.to_string())?;
  }
  Ok(app_data_dir.join("hosts_sources.json"))
}

fn read_hosts_sources(path: &PathBuf) -> Result<Vec<HostsSource>, String> {
  if !path.exists() {
    return Ok(Vec::new());
  }
  let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
  if content.trim().is_empty() {
    return Ok(Vec::new());
  }
  serde_json::from_str(&content).map_err(|e| format!("hosts_sources.json 解析失败: {}", e))
}

fn write_hosts_sources(path: &PathBuf, sources: &[HostsSource]) -> Result<(), String> {
  let content = serde_json::to_string_pretty(sources).map_err(|e| e.to_string())?;
  std::fs::write(path, content).map_err(|e| e.to_string())
}

/// 读取远程数据源列表
#[tauri::command]
pub fn get_hosts_sources(app: AppHandle) -> Result<Vec<HostsSource>, String> {
  read_hosts_sources(&get_hosts_sources_path(&app)?)
}

/// 保存/更新远程数据源（按分组名 upsert），返回最新列表
#[tauri::command]
pub fn save_hosts_source(app: AppHandle, source: HostsSource) -> Result<Vec<HostsSource>, String> {
  let path = get_hosts_sources_path(&app)?;
  let mut sources = read_hosts_sources(&path)?;
  match sources.iter().position(|s| s.name == source.name) {
    Some(idx) => sources[idx] = source,
    None => sources.push(source),
  }
  write_hosts_sources(&path, &sources)?;
  Ok(sources)
}

/// 删除远程数据源，返回最新列表
#[tauri::command]
pub fn remove_hosts_source(app: AppHandle, name: String) -> Result<Vec<HostsSource>, String> {
  let path = get_hosts_sources_path(&app)?;
  let mut sources = read_hosts_sources(&path)?;
  sources.retain(|s| s.name != name);
  write_hosts_sources(&path, &sources)?;
  Ok(sources)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_hosts_source_serde_camel_case() {
    // 持久化格式与前端字段（camelCase）保持一致
    let json = r#"{"name":"k8s-ingress","url":"http://10.0.0.5:30180/hosts.json?ip=10.0.0.5","lastSyncedAt":1730000000}"#;
    let source: HostsSource = serde_json::from_str(json).unwrap();
    assert_eq!(source.name, "k8s-ingress");
    assert!(source.url.contains("hosts.json"));
    assert_eq!(source.last_synced_at, Some(1730000000));

    let out = serde_json::to_string(&source).unwrap();
    assert!(out.contains("\"lastSyncedAt\":1730000000"));
    // lastSyncedAt 缺省时可解析（向前兼容）
    let legacy: HostsSource = serde_json::from_str(r#"{"name":"g","url":"http://x"}"#).unwrap();
    assert_eq!(legacy.last_synced_at, None);
  }
}
