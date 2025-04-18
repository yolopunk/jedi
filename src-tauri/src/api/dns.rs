use std::collections::HashMap;
use std::error::Error;
use serde::{Deserialize, Serialize};
use reqwest;
use tauri::utils::platform::resource_dir;
use tauri::{AppHandle, Manager};

static HOSTS_PATH: &str = if cfg!(any(target_os = "linux", target_os = "macos")) {
    "/etc/hosts"
} else if cfg!(target_os = "windows") {
    r"C:\Windows\System32\drivers\etc\hosts"
} else {
    panic!("Unsupported OS");
};

#[derive(Serialize, Deserialize, Clone)]
pub struct HostEntry {
    pub ip: String,
    pub domain: String,
    pub disabled: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GroupHosts {
    pub name: String,
    pub hosts: Vec<HashMap<String, String>>,
}

pub fn load_local_config(app: &AppHandle) -> Result<Vec<GroupHosts>, Box<dyn Error>> {
    let mut hosts_path = resource_dir(app.package_info(), &app.env())?;
    hosts_path.push("config");
    hosts_path.push("hosts.json");

    let hosts_content = std::fs::read_to_string(hosts_path)?;
    let groups: Vec<GroupHosts> = serde_json::from_str(&hosts_content)?;
    Ok(groups)
}

pub async fn fetch_remote_config(url: &str) -> Result<Vec<GroupHosts>, Box<dyn Error>> {
    let resp = reqwest::get(url).await?;
    let text = resp.text().await?;
    let groups: Vec<GroupHosts> = serde_json::from_str(&text)?;
    Ok(groups)
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
            fetch_remote_config(&u).await.map_err(|e| e.to_string())
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
        let mut default_hosts = Vec::new();

        // localhost
        let mut localhost_map = HashMap::new();
        localhost_map.insert("localhost".to_string(), "127.0.0.1".to_string());
        default_hosts.push(localhost_map);

        // localhost IPv6
        let mut localhost_ipv6_map = HashMap::new();
        localhost_ipv6_map.insert("localhost".to_string(), "::1".to_string());
        default_hosts.push(localhost_ipv6_map);

        // 常用的测试域名
        let mut example_map = HashMap::new();
        example_map.insert("example.test".to_string(), "127.0.0.1".to_string());
        default_hosts.push(example_map);

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

    let hosts_content = match std::fs::read_to_string(HOSTS_PATH) {
        Ok(content) => content,
        Err(e) => return Err(format!("Failed to read hosts file: {}", e)),
    };

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
        let mut sorted_hosts: Vec<(String, String, bool)> = Vec::new();
        for host_map in &g.hosts {
            // 检查是否禁用
            let is_disabled = host_map.contains_key("__disabled");

            // 提取域名和IP（跳过特殊键）
            for (hostname, ip) in host_map {
                if hostname != "__disabled" {
                    sorted_hosts.push((hostname.clone(), ip.clone(), is_disabled));
                    // 只处理第一个非__disabled键值对，因为每个host_map应该只有一个域名和IP
                    break;
                }
            }
        }

        // 按域名排序
        sorted_hosts.sort_by(|a, b| a.0.cmp(&b.0));

        for (hostname, ip, is_disabled) in sorted_hosts {
            if is_disabled {
                // 如果禁用，添加注释符号
                new_lines.push(format!("# {} {}", ip, hostname));
            } else {
                // 如果启用，正常显示
                new_lines.push(format!("{} {}", ip, hostname));
            }
        }

        // 不需要分组结束标记，下一个分组开始即表示上一个分组结束
    }

    new_lines.push("# === END JEDI HOSTS MANAGER ===".to_string());

    let new_content = new_lines.join("\n") + "\n";
    match std::fs::write(HOSTS_PATH, new_content) {
        Ok(_) => {},
        Err(e) => return Err(format!("Failed to write hosts file: {}", e)),
    };

    Ok("Hosts updated successfully".to_string())
}

#[tauri::command]
pub fn revert_hosts() -> Result<String, String> {
    // 读取hosts文件
    let hosts_content = match std::fs::read_to_string(HOSTS_PATH) {
        Ok(content) => content,
        Err(e) => return Err(format!("Failed to read hosts file: {}", e)),
    };

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

        if trimmed.starts_with("# ===") || trimmed.starts_with("# +") ||
           line.trim().is_empty() || (trimmed.starts_with("#") && trimmed.len() == 1) {
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
            } else if trimmed.starts_with("# ") && !trimmed.starts_with("# ===") && !trimmed.starts_with("# +") {
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

    // 写入新的hosts文件
    let new_content = new_lines.join("\n");
    match std::fs::write(HOSTS_PATH, new_content) {
        Ok(_) => Ok("Hosts entries disabled successfully.".to_string()),
        Err(e) => Err(format!("Failed to write hosts file: {}", e)),
    }
}

#[tauri::command]
pub fn read_system_hosts() -> Result<Vec<GroupHosts>, String> {
    // 读取系统hosts文件
    let hosts_content = match std::fs::read_to_string(HOSTS_PATH) {
        Ok(content) => content,
        Err(e) => return Err(format!("Failed to read hosts file: {}", e)),
    };

    // 初始化一个默认的分组，包含一些常用的hosts条目
    let mut default_hosts = Vec::new();
    let mut localhost_map = HashMap::new();
    localhost_map.insert("localhost".to_string(), "127.0.0.1".to_string());
    default_hosts.push(localhost_map);

    // 如果没有找到Jedi管理的部分，就使用这个默认分组
    let mut result = vec![GroupHosts {
        name: "默认".to_string(),
        hosts: default_hosts,
    }];

    let mut current_group: Option<String> = None;
    let mut current_hosts: Vec<HashMap<String, String>> = Vec::new();
    let mut in_jedi_section = false;
    let mut found_jedi_section = false;

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
            found_jedi_section = true;

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
        if current_group.is_some() && !trimmed.is_empty() &&
           !trimmed.starts_with("# +") && !trimmed.starts_with("# ===") {
            // 检查是否是注释行（已禁用的条目）
            let (line_to_parse, is_disabled) = if trimmed.starts_with("# ") && !trimmed.starts_with("# +") && !trimmed.starts_with("# ===") {
                // 如果是普通注释行，去除注释符号
                (trimmed[2..].to_string(), true)
            } else if trimmed.starts_with('#') && !trimmed.starts_with("#+") && !trimmed.starts_with("#===") {
                // 如果只有#没有空格，也去除
                (trimmed[1..].to_string(), true)
            } else {
                (trimmed.to_string(), false)
            };

            // 解析IP和域名
            let parts: Vec<&str> = line_to_parse.split_whitespace().collect();
            if parts.len() >= 2 {
                let ip = parts[0].to_string();
                let domain = parts[1].to_string();

                // 检查是否已经存在相同的域名，避免重复
                let domain_exists = current_hosts.iter().any(|h| h.contains_key(&domain));
                if !domain_exists {
                    let mut host_map = HashMap::new();
                    host_map.insert(domain, ip);

                    // 将启用/禁用状态存储为布尔值
                    if is_disabled {
                        host_map.insert("__disabled".to_string(), "true".to_string());
                    }

                    current_hosts.push(host_map);
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

    // 如果找到了Jedi管理的部分，则移除默认分组
    if found_jedi_section && result.len() > 1 {
        result.remove(0); // 移除默认分组
    }

    Ok(result)
}
