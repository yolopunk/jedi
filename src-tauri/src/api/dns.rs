use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::Read;
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
pub struct TagHosts {
    pub tag: String,
    pub hosts: Vec<HashMap<String, String>>,
}

pub fn load_local_config(app: &AppHandle) -> Result<Vec<TagHosts>, Box<dyn Error>> {
    let mut hosts_path = resource_dir(app.package_info(), &app.env())?;
    hosts_path.push("config");
    hosts_path.push("hosts.json");

    let hosts_content = std::fs::read_to_string(hosts_path)?;
    let tags: Vec<TagHosts> = serde_json::from_str(&hosts_content)?;
    Ok(tags)
}

pub async fn fetch_remote_config(url: &str) -> Result<Vec<TagHosts>, Box<dyn Error>> {
    let resp = reqwest::get(url).await?;
    let text = resp.text().await?;
    let tags: Vec<TagHosts> = serde_json::from_str(&text)?;
    Ok(tags)
}

#[tauri::command]
pub async fn update_hosts_with_tag(
    app: AppHandle,
    source: String,
    url: Option<String>,
) -> Result<String, String> {
    let tags_result = if source == "remote" {
        if let Some(u) = url {
            fetch_remote_config(&u).await.map_err(|e| e.to_string())
        } else {
            return Err("Remote source requires URL".to_string());
        }
    } else {
        load_local_config(&app).map_err(|e| e.to_string())
    };

    let tags = match tags_result {
        Ok(t) => t,
        Err(e) => return Err(e),
    };

    let hosts_content = std::fs::read_to_string(HOSTS_PATH).map_err(|e| e.to_string())?;

    let mut new_lines: Vec<String> = Vec::new();
    let mut skip = false;
    for line in hosts_content.lines() {
        if line.trim_start().starts_with("# Added By Jedi") {
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

    new_lines.push("# Added By Jedi".to_string());
    for t in &tags {
        new_lines.push(format!("# tag: {}", t.tag));
        for host_map in &t.hosts {
            for (hostname, ip) in host_map {
                new_lines.push(format!("{} {}", ip, hostname));
            }
        }
        new_lines.push("# endtag".to_string());
    }
    new_lines.push("# End of section".to_string());

    let new_content = new_lines.join("\n");
    std::fs::write(HOSTS_PATH, new_content).map_err(|e| e.to_string())?;

    Ok("Hosts updated successfully".to_string())
}

#[tauri::command]
pub fn revert_hosts() -> Result<String, ()> {
    let mut hosts_file = File::open(HOSTS_PATH).unwrap();
    let mut contents = String::new();
    hosts_file.read_to_string(&mut contents).unwrap();

    let mut new_lines = Vec::new();
    let mut skip = false;
    for line in contents.lines() {
        if line.trim_start().starts_with("# Added By Jedi") {
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

    let new_contents = new_lines.join("\n");
    fs::write(HOSTS_PATH, new_contents).unwrap();
    Ok("Hosts file reverted successfully.".to_string())
}