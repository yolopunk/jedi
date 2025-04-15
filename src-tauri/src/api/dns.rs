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
    println!("Updating hosts with groups, source: {}", source);
    let groups_result = if source == "remote" {
        if let Some(u) = url {
            println!("Fetching remote config from URL: {}", u);
            fetch_remote_config(&u).await.map_err(|e| e.to_string())
        } else {
            return Err("Remote source requires URL".to_string());
        }
    } else if source == "current" {
        // 使用前端传递的分组数据
        if let Some(g) = groups {
            println!("Using current groups from frontend, count: {}", g.len());
            Ok(g)
        } else {
            return Err("Current source requires groups".to_string());
        }
    } else if source == "default" {
        // 创建默认的分组和条目
        println!("Creating default hosts configuration");
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
        println!("Loading local config");
        load_local_config(&app).map_err(|e| e.to_string())
    };

    let groups = match groups_result {
        Ok(g) => {
            println!("Got {} groups", g.len());
            for group in &g {
                println!("  - Group: '{}' with {} hosts", group.name, group.hosts.len());
            }
            g
        },
        Err(e) => {
            println!("Error getting groups: {}", e);
            return Err(e);
        },
    };

    println!("Reading hosts file from: {}", HOSTS_PATH);
    let hosts_content = match std::fs::read_to_string(HOSTS_PATH) {
        Ok(content) => {
            println!("Successfully read hosts file, content length: {}", content.len());
            content
        },
        Err(e) => {
            println!("Failed to read hosts file: {}", e);
            return Err(format!("Failed to read hosts file: {}", e));
        },
    };

    let mut new_lines: Vec<String> = Vec::new();
    let mut skip = false;
    for line in hosts_content.lines() {
        if line.trim_start().starts_with("# Added by Jedi") {
            skip = true;
            continue;
        }
        if line.trim_start().starts_with("# End of section") {
            skip = false;
            continue;
        }
        if !skip {
            new_lines.push(line.to_string());
        }
    }

    println!("Adding Jedi section with {} groups", groups.len());
    new_lines.push("# ====================== JEDI HOSTS MANAGER ======================".to_string());
    new_lines.push("#".to_string());

    for g in &groups {
        // 创建分组标题，居中显示
        let group_name = g.name.clone();
        let header = format!("# +--------------------- {} ---------------------+", group_name);
        new_lines.push(header);
        new_lines.push("#".to_string());

        // 按域名排序显示
        let mut sorted_hosts: Vec<(String, String, bool)> = Vec::new();
        for host_map in &g.hosts {
            // 检查是否禁用
            let is_disabled = host_map.contains_key("__disabled");

            // 提取域名和IP（跳过特殊键）
            for (hostname, ip) in host_map {
                if hostname != "__disabled" {
                    sorted_hosts.push((hostname.clone(), ip.clone(), is_disabled));
                }
            }
        }

        // 按域名排序
        sorted_hosts.sort_by(|a, b| a.0.cmp(&b.0));

        for (hostname, ip, is_disabled) in sorted_hosts {
            if is_disabled {
                // 如果禁用，添加注释符号
                new_lines.push(format!("# {} {}", ip, hostname));
                println!("Writing disabled host entry: # {} {}", ip, hostname);
            } else {
                // 如果启用，正常显示
                new_lines.push(format!("{} {}", ip, hostname));
                println!("Writing enabled host entry: {} {}", ip, hostname);
            }
        }

        new_lines.push("#".to_string());
        let footer = format!("# +---------------------- END {} ----------------------+", g.name);
        new_lines.push(footer);
        new_lines.push("#".to_string());
    }

    new_lines.push("# ====================== END JEDI HOSTS MANAGER ======================".to_string());

    let new_content = new_lines.join("\n");
    println!("Writing new hosts file, content length: {}", new_content.len());
    match std::fs::write(HOSTS_PATH, new_content) {
        Ok(_) => println!("Successfully wrote hosts file"),
        Err(e) => {
            println!("Failed to write hosts file: {}", e);
            return Err(format!("Failed to write hosts file: {}", e));
        },
    };

    Ok("Hosts updated successfully".to_string())
}

#[tauri::command]
pub fn revert_hosts() -> Result<String, String> {
    // 读取hosts文件
    println!("Reading hosts file for disabling: {}", HOSTS_PATH);
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

        if trimmed.starts_with("# Added by Jedi") || trimmed.starts_with("# ====================== JEDI HOSTS MANAGER") {
            in_jedi_section = true;
            jedi_section_lines.push(line.to_string());
            continue;
        }

        if trimmed.starts_with("# End of section") || trimmed.starts_with("# ====================== END JEDI HOSTS MANAGER") {
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

        if trimmed.starts_with("# Added by Jedi") ||
           trimmed.starts_with("# End of section") ||
           trimmed.starts_with("# ----") || trimmed.starts_with("# ====") || line.trim().is_empty() ||
           trimmed.contains("# +---------------------") || trimmed.contains("# +---------------------- END ") ||
           trimmed.starts_with("# ====================== JEDI HOSTS MANAGER") || trimmed.starts_with("# ====================== END JEDI HOSTS MANAGER") ||
           trimmed.starts_with("#") && trimmed.len() == 1 {
            // 保留分组行和分隔线
            new_lines.push(line.to_string());
            if trimmed.contains("# +---------------------") {
                in_group = true;
            } else if trimmed.contains("# +---------------------- END ") {
                in_group = false;
            }
        } else if in_group {
            // 如果是hosts条目，确保它被注释
            if !trimmed.starts_with('#') {
                // 如果还没有注释，添加注释
                new_lines.push(format!("# {}", line));
                println!("Disabling host entry: {}", line);
            } else if trimmed.starts_with("# ") && trimmed.len() > 2 &&
                      !trimmed.contains("# +---------------------") && !trimmed.contains("# +---------------------- END ") &&
                      !trimmed.starts_with("# ====================== JEDI HOSTS MANAGER") && !trimmed.starts_with("# ====================== END JEDI HOSTS MANAGER") {
                // 如果是被注释的hosts条目，保持注释
                new_lines.push(line.to_string());
                println!("Already disabled host entry: {}", line);
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
    println!("Reading hosts file from: {}", HOSTS_PATH);
    let hosts_content = match std::fs::read_to_string(HOSTS_PATH) {
        Ok(content) => {
            println!("Successfully read hosts file, content length: {}", content.len());
            content
        },
        Err(e) => {
            println!("Failed to read hosts file: {}", e);
            return Err(format!("Failed to read hosts file: {}", e))
        },
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
    for (line_num, line) in hosts_content.lines().enumerate() {
        let trimmed = line.trim_start();

        // 检查是否进入Jedi管理的部分
        if trimmed.starts_with("# Added by Jedi") || trimmed.starts_with("# ====================== JEDI HOSTS MANAGER") {
            println!("Found Jedi section at line {}", line_num + 1);
            in_jedi_section = true;
            found_jedi_section = true;
            continue;
        }

        // 检查是否离开Jedi管理的部分
        if trimmed.starts_with("# End of section") || trimmed.starts_with("# ====================== END JEDI HOSTS MANAGER") {
            println!("End of Jedi section at line {}", line_num + 1);
            in_jedi_section = false;
            // 保存最后一个分组的数据
            if let Some(name) = current_group.take() {
                if !current_hosts.is_empty() {
                    println!("Adding group '{}' with {} hosts", name, current_hosts.len());
                    result.push(GroupHosts {
                        name,
                        hosts: current_hosts,
                    });
                    current_hosts = Vec::new();
                } else {
                    println!("Group '{}' has no hosts, skipping", name);
                }
            }
            continue;
        }

        // 如果不在Jedi管理的部分，跳过
        if !in_jedi_section {
            continue;
        }

        // 解析分组行
        if trimmed.contains("# +---------------------") {
            // 保存之前的分组数据（如果有）
            if let Some(name) = current_group.take() {
                if !current_hosts.is_empty() {
                    println!("Adding group '{}' with {} hosts", name, current_hosts.len());
                    result.push(GroupHosts {
                        name,
                        hosts: current_hosts,
                    });
                    current_hosts = Vec::new();
                } else {
                    println!("Group '{}' has no hosts, skipping", name);
                }
            }

            // 提取新分组
            let name = if trimmed.contains("# +---------------------") {
                // 新格式，从标题行提取分组名称
                if let Some(start) = trimmed.find("---------------------") {
                    let start_idx = start + 21; // 安全地添加偏移量
                    if let Some(end) = trimmed.rfind("---------------------") {
                        trimmed[start_idx..end].trim().to_string()
                    } else {
                        "Unknown".to_string()
                    }
                } else {
                    "Unknown".to_string()
                }
            } else {
                "Unknown".to_string()
            };
            println!("Found group: '{}' at line {}", name, line_num + 1);
            current_group = Some(name);
            continue;
        }

        // 解析结束分组行
        if trimmed.contains("# +---------------------- END ") {
            println!("Found end of group at line {}", line_num + 1);
            continue;
        }

        // 解析hosts条目 - 处理注释行和非注释行
        if current_group.is_some() && !trimmed.is_empty() &&
           !trimmed.contains("# +---------------------") && !trimmed.contains("# +---------------------- END ") &&
           !trimmed.starts_with("# ----") && !trimmed.starts_with("# ====") {
            let group_name = current_group.as_ref().unwrap();
            let mut is_disabled = false;
            let mut line_to_parse = trimmed.to_string();

            // 检查是否是注释行（已禁用的条目）
            if trimmed.starts_with("# ") &&
               !trimmed.starts_with("# Added by") && !trimmed.starts_with("# End of") &&
               !trimmed.starts_with("# ----") && !trimmed.starts_with("# ====") {
                // 去除注释符号和空格
                line_to_parse = trimmed[2..].to_string();
                is_disabled = true;
                println!("Found disabled host entry: {}", line_to_parse);
            } else if trimmed.starts_with('#') &&
                      !trimmed.starts_with("#Added by") && !trimmed.starts_with("#End of") &&
                      !trimmed.starts_with("#----") && !trimmed.starts_with("#====") {
                // 如果只有#没有空格，也去除
                line_to_parse = trimmed[1..].to_string();
                is_disabled = true;
                println!("Found disabled host entry: {}", line_to_parse);
            }

            // 解析IP和域名
            let parts: Vec<&str> = line_to_parse.split_whitespace().collect();
            if parts.len() >= 2 {
                let ip = parts[0].to_string();
                let domain = parts[1].to_string();

                // 检查是否已经存在相同的域名，避免重复
                let domain_exists = current_hosts.iter().any(|h| h.contains_key(&domain));
                if !domain_exists {
                    println!("Found host entry: {} -> {} for group '{}', disabled: {}", domain, ip, group_name, is_disabled);
                    let mut host_map = HashMap::new();
                    host_map.insert(domain, ip);

                    // 将启用/禁用状态存储为布尔值
                    if is_disabled {
                        host_map.insert("__disabled".to_string(), "true".to_string());
                    }

                    current_hosts.push(host_map);
                } else {
                    println!("Skipping duplicate domain: {} for group '{}'", domain, group_name);
                }
            }
        }
    }

    // 如果文件结束时还有未保存的分组数据，保存它
    if let Some(name) = current_group {
        if !current_hosts.is_empty() {
            println!("Adding final group '{}' with {} hosts", name, current_hosts.len());
            result.push(GroupHosts {
                name,
                hosts: current_hosts,
            });
        } else {
            println!("Final group '{}' has no hosts, skipping", name);
        }
    }

    // 如果找到了Jedi管理的部分，则移除默认分组
    if found_jedi_section && result.len() > 1 {
        println!("Found Jedi section, removing default group");
        result.remove(0); // 移除默认分组
    } else {
        println!("No Jedi section found or no valid groups, using default group");
    }

    println!("Returning {} groups", result.len());
    for group in &result {
        println!("  - Group: '{}' with {} hosts", group.name, group.hosts.len());
    }

    Ok(result)
}