use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::Read;
use tauri::utils::platform::resource_dir;
use tauri::{AppHandle, Manager};

// Path to hosts file
static HOSTS_PATH: &str = if cfg!(any(target_os = "linux", target_os = "macos")) {
  "/etc/hosts"
} else if cfg!(target_os = "windows") {
  r"C:\Windows\System32\drivers\etc\hosts"
} else {
  panic!("Unsupported OS");
};

#[tauri::command]
pub async fn update_hosts(app: AppHandle) -> Result<String, ()> {
  let mut hosts_file = File::open(HOSTS_PATH).unwrap();
  let mut hosts_content = String::new();
  hosts_file.read_to_string(&mut hosts_content).unwrap();

  hosts_content.push_str("\n# dtc.io start\n");
  for (hostname, ip) in read_hosts_file(app).unwrap() {
    hosts_content.push_str(&format!("{} {}\n", ip, hostname));
  }
  hosts_content.push_str("# dtc.io end\n");
  fs::write(HOSTS_PATH, hosts_content).unwrap();
  Ok("Hosts file updated successfully!".to_string())
}

#[tauri::command]
pub fn revert_hosts() -> Result<String, ()> {
  let mut hosts_file = File::open(HOSTS_PATH).unwrap();
  let mut contents = String::new();
  hosts_file.read_to_string(&mut contents).unwrap();

  let lines = contents.split('\n');
  let mut new_lines = vec![];
  let (mut start, mut end, mut skip) = (false, false, false);

  for line in lines {
    if line.contains("dtc.io start") {
      start = true;
      skip = true;
    } else if start && !end {
      skip = true;
    } else if start && !end && line.contains("dtc.io end") {
      end = true;
      skip = true;
    }

    if skip {
      continue;
    }

    new_lines.push(line);
  }

  let new_contents = new_lines.join("\n");

  fs::write(HOSTS_PATH, new_contents).unwrap();
  Ok("Hosts file reverted successfully!".to_string())
}

fn read_hosts_file(app: AppHandle) -> Result<HashMap<String, String>, Box<dyn Error>> {
  let mut hosts_path = resource_dir(app.package_info(), &app.env())?;
  hosts_path.push("config");
  hosts_path.push("hosts.json");

  let hosts_content = fs::read_to_string(hosts_path)?;
  let hosts: HashMap<String, String> = serde_json::from_str(&hosts_content)?;
  Ok(hosts)
}
