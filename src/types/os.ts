/**
 * 系统信息类型定义
 * 对应后端 OsInfo 结构
 */

// 内存信息
export interface MemoryInfo {
  total: number;
  used: number;
  available: number;
  free: number;
}

// CPU信息
export interface CpuInfo {
  name: string;
  brand: string;
  vendor: string;
  frequency: number;
  usage: number;
}

// GPU信息
export interface GpuInfo {
  name?: string;
  memory_total?: number;
  memory_used?: number;
  memory_free?: number;
  temperature?: string;
}

// 磁盘信息
export interface DiskInfo {
  type_: string;
  name: string;
  file_system: string;
  mount_point: string;
  total_space: number;
  available_space: number;
  is_removable: boolean;
}

// 网络信息
export interface NetworkInfo {
  mac_addr: string;
  received: number;
  transmitted: number;
}

// 系统指标
export interface Metrics {
  cpu_usage: number;
  gpu_temp: string;
  gpu_memory_total: number;
  gpu_memory_used: number;
  memory_total: number;
  memory_used: number;
  disk_total_space: number;
  disk_used_space: number;
  network_received: number;
  network_transmitted: number;
}

// 系统信息
export interface OsInfo {
  name: string;
  kernel_version?: string;
  os_version?: string;
  long_os_version?: string;
  host_name?: string;
  arch?: string;
  boot_time: number;
  load_average: [number, number, number];
  memory: MemoryInfo;
  cpus: CpuInfo[];
  gpus: GpuInfo[];
  disks: DiskInfo[];
  networks: Record<string, NetworkInfo>;
  metrics: Metrics;
}
