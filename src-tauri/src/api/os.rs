use crate::utils::helper::ToLinuxLoadTuple;
#[cfg(target_os = "windows")]
use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
#[cfg(target_os = "windows")]
use nvml_wrapper::Nvml;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Mutex;
use sysinfo::{Disks, NetworkData, Networks, System};
use tauri::State;

pub struct SystemState {
  pub system: Mutex<System>,
  pub networks: Mutex<Networks>,
}

#[derive(Serialize, Debug)]
pub struct OsInfo {
  pub name: String,
  kernel_version: Option<String>,
  pub os_version: Option<String>,
  pub long_os_version: Option<String>,
  pub host_name: Option<String>,
  arch: Option<String>,
  boot_time: u64,
  load_average: (f64, f64, f64),
  memory: MemoryInfo,
  cpus: Vec<CpuInfo>,
  gpus: Vec<GpuInfo>,
  disks: Vec<DiskInfo>,
  networks: HashMap<String, NetworkInfo>,
  metrics: Metrics,
}

#[derive(Serialize, Debug)]
struct MemoryInfo {
  total: u64,
  used: u64,
  available: u64,
  free: u64,
}

#[derive(Serialize, Debug)]
struct CpuInfo {
  name: String,
  brand: String,
  vendor: String,
  frequency: u64,
  usage: f32,
}

#[derive(Serialize, Debug)]
struct GpuInfo {
  name: Option<String>,
  memory_total: Option<u64>,
  memory_used: Option<u64>,
  memory_free: Option<u64>,
  temperature: Option<String>,
}

#[derive(Serialize, Debug)]
struct DiskInfo {
  type_: String,
  name: String,
  file_system: String,
  mount_point: String,
  total_space: u64,
  available_space: u64,
  is_removable: bool,
}

#[derive(Serialize, Debug)]
struct NetworkInfo {
  mac_addr: String,
  received: u64,
  transmitted: u64,
}

#[derive(Serialize, Debug)]
struct Metrics {
  cpu_usage: f32,
  gpu_temp: String,
  gpu_memory_total: u64,
  gpu_memory_used: u64,
  memory_total: u64,
  memory_used: u64,
  disk_total_space: u64,
  disk_used_space: u64,
  network_received: u64,
  network_transmitted: u64,
}

#[tauri::command]
pub async fn get_os_info(state: State<'_, SystemState>) -> Result<OsInfo, String> {
  let mut sys = state.system.lock().map_err(|e| e.to_string())?;
  sys.refresh_all();

  let mut metrics = Metrics {
    cpu_usage: 0.0,
    gpu_temp: "0 °C".to_string(),
    gpu_memory_total: 0,
    gpu_memory_used: 0,
    memory_total: 0,
    memory_used: 0,
    disk_total_space: 0,
    disk_used_space: 0,
    network_received: 0,
    network_transmitted: 0,
  };

  let mut cpu_infos = Vec::new();
  if let Some(cpu) = sys.cpus().first() {
    let cpu_info = CpuInfo {
      name: cpu.name().to_string(),
      brand: cpu.brand().to_string(),
      vendor: cpu.vendor_id().to_string(),
      frequency: cpu.frequency(),
      usage: cpu.cpu_usage(),
    };

    cpu_infos.push(cpu_info);
  }
  metrics.cpu_usage = sys.global_cpu_usage();

  let mut disk_infos = Vec::new();
  let disks = Disks::new_with_refreshed_list();
  for disk in &disks {
    let mount_point = disk.mount_point().to_string_lossy().to_string();

    let disk_info = DiskInfo {
      type_: disk.kind().to_string(),
      name: disk.name().to_string_lossy().to_string(),
      file_system: disk.file_system().to_string_lossy().to_string(),
      mount_point: mount_point.clone(),
      total_space: disk.total_space(),
      available_space: disk.available_space(),
      is_removable: disk.is_removable(),
    };

    disk_infos.push(disk_info);
  }

  let mut network_infos: HashMap<String, NetworkInfo> = HashMap::new();
  // 使用全局 Networks 实例
  let mut networks = state.networks.lock().map_err(|e| e.to_string())?;

  // 刷新网络数据
  // sysinfo 的 refresh() 会计算自上次刷新以来的流量增量
  // 前端应通过定时轮询(如每秒一次)来获取准确的传输速率
  networks.refresh(true);

  for (interface_name, data) in networks.iter() {
    let interface_name: &String = interface_name;
    let data: &NetworkData = data;
    network_infos.insert(
      interface_name.to_string(),
      NetworkInfo {
        mac_addr: data.mac_address().to_string(),
        received: data.received(),
        transmitted: data.transmitted(),
      },
    );
    metrics.network_received += data.received();
    metrics.network_transmitted += data.transmitted();
  }

  #[allow(unused_mut)]
  let mut gpu_infos = Vec::new();
  #[cfg(target_os = "windows")]
  {
    // 尝试初始化 NVML，如果失败则忽略
    if let Ok(nvml) = Nvml::init() {
      if let Ok(device_count) = nvml.device_count() {
        for i in 0..device_count {
          if let Ok(device) = nvml.device_by_index(i) {
            // 重新编写 NVML 逻辑以更健壮
            let name = device.name().unwrap_or_else(|_| "Unknown GPU".to_string());
            let temp = device.temperature(TemperatureSensor::Gpu).unwrap_or(0);
            let temperature = format!("{} °C", temp);

            let (mem_total, mem_used, mem_free) = match device.memory_info() {
              Ok(m) => (m.total, m.used, m.free),
              Err(_) => (0, 0, 0),
            };

            let gpu_info = GpuInfo {
              name: Some(name),
              memory_total: Some(mem_total),
              memory_used: Some(mem_used),
              memory_free: Some(mem_free),
              temperature: Some(temperature.clone()),
            };

            gpu_infos.push(gpu_info);
            // 简单覆盖 metrics，如果有多个 GPU 可能需要聚合，这里保持原逻辑
            metrics.gpu_temp = temperature;
            metrics.gpu_memory_total = mem_total;
            metrics.gpu_memory_used = mem_used;
          }
        }
      }
    }
  }
  #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
  {
    gpu_infos.push(GpuInfo {
      name: None,
      memory_total: None,
      memory_used: None,
      memory_free: None,
      temperature: None,
    });
  }

  metrics.memory_total = sys.total_memory();
  metrics.memory_used = sys.used_memory();

  Ok(OsInfo {
    name: System::name().unwrap_or_else(|| "Unknown".to_string()),
    kernel_version: System::kernel_version(),
    os_version: System::os_version(),
    long_os_version: System::long_os_version(),
    host_name: System::host_name(),
    arch: Some(System::cpu_arch()),
    boot_time: System::boot_time(),
    load_average: System::load_average().to_tuple(),
    memory: MemoryInfo {
      total: sys.total_memory(),
      used: sys.used_memory(),
      available: sys.available_memory(),
      free: sys.free_memory(),
    },
    cpus: cpu_infos,
    gpus: gpu_infos,
    disks: disk_infos,
    networks: network_infos,
    metrics,
  })
}
