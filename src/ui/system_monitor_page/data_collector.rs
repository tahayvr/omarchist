use std::collections::VecDeque;
use std::time::Duration;
use sysinfo::{Disks, Networks, Pid, System};

const INTERVAL: Duration = Duration::from_millis(500);
const MAX_DATA_POINTS: usize = 120;

#[derive(Clone, Debug)]
pub struct MetricPoint {
    pub time: String,
    pub cpu: f64,
    pub memory: f64,
    pub network_up: f64,
    pub network_down: f64,
}

#[derive(Clone, Debug)]
pub struct CoreInfo {
    pub name: String,
    pub usage: f32,
    pub frequency: u64,
}

#[derive(Clone, Debug)]
pub struct MemoryInfo {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub cached: u64,
    pub buffers: u64,
    pub swap_total: u64,
    pub swap_used: u64,
}

#[derive(Clone, Debug)]
pub struct ProcessInfo {
    pub pid: Pid,
    pub name: String,
    pub user: String,
    pub status: String,
    pub cpu_usage: f32,
    pub memory: u64,
    pub command: String,
}

#[derive(Clone, Debug)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub filesystem: String,
}

#[derive(Clone, Debug)]
pub struct InterfaceInfo {
    pub name: String,
    pub ip_addresses: Vec<String>,
    pub mac_address: Option<String>,
    pub is_up: bool,
    pub total_received: u64,
    pub total_transmitted: u64,
}

#[derive(Clone, Debug)]
pub struct ConnectionInfo {
    pub protocol: String,
    pub local_addr: String,
    pub remote_addr: String,
    pub state: String,
}

#[derive(Clone, Debug)]
pub struct BatteryInfo {
    pub model: String,
    pub percentage: f32,
    pub state: String,
    pub time_remaining: Option<u64>,
    pub health: f32,
    pub voltage: f32,
    pub temperature: Option<f32>,
    pub cycle_count: Option<u32>,
}

#[derive(Clone, Debug)]
pub struct AlertThresholds {
    pub cpu_warning: f32,
    pub cpu_critical: f32,
    pub memory_warning: f32,
    pub memory_critical: f32,
    pub disk_warning: f32,
    pub battery_low: f32,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            cpu_warning: 80.0,
            cpu_critical: 95.0,
            memory_warning: 85.0,
            memory_critical: 95.0,
            disk_warning: 85.0,
            battery_low: 20.0,
        }
    }
}

pub struct DataCollector {
    sys: System,
    disks: Disks,
    networks: Networks,
    pub data: VecDeque<MetricPoint>,
    pub time_index: usize,
    pub thresholds: AlertThresholds,
    last_network_up: u64,
    last_network_down: u64,
    pub has_battery: bool,
}

impl Default for DataCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl DataCollector {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        let disks = Disks::new_with_refreshed_list();
        let networks = Networks::new_with_refreshed_list();
        let has_battery = Self::detect_battery();

        let mut collector = Self {
            sys,
            disks,
            networks,
            data: VecDeque::with_capacity(MAX_DATA_POINTS),
            time_index: 0,
            thresholds: AlertThresholds::default(),
            last_network_up: 0,
            last_network_down: 0,
            has_battery,
        };

        collector.collect();
        collector
    }

    fn detect_battery() -> bool {
        battery::Manager::new()
            .and_then(|m| m.batteries())
            .map(|mut b| b.next().is_some())
            .unwrap_or(false)
    }

    pub fn collect(&mut self) {
        // Refresh all system data
        self.sys.refresh_all();
        self.disks.refresh(true);
        self.networks.refresh(true);

        // Calculate metrics
        let cpu_usage = self.sys.global_cpu_usage() as f64;
        let memory_usage = self.calculate_memory_usage();
        let (network_up, network_down) = self.calculate_network_speeds();

        // Create data point
        let point = MetricPoint {
            time: format!("{}s", self.time_index),
            cpu: cpu_usage,
            memory: memory_usage,
            network_up,
            network_down,
        };

        // Add to history
        if self.data.len() >= MAX_DATA_POINTS {
            self.data.pop_front();
        }
        self.data.push_back(point);
        self.time_index += 1;
    }

    fn calculate_memory_usage(&self) -> f64 {
        let total = self.sys.total_memory() as f64;
        let used = self.sys.used_memory() as f64;
        if total > 0.0 {
            (used / total * 100.0).min(100.0)
        } else {
            0.0
        }
    }

    fn calculate_network_speeds(&mut self) -> (f64, f64) {
        let mut total_up: u64 = 0;
        let mut total_down: u64 = 0;

        for (_, network) in self.networks.iter() {
            total_up += network.total_transmitted();
            total_down += network.total_received();
        }

        let up_speed = if self.last_network_up > 0 && total_up >= self.last_network_up {
            ((total_up - self.last_network_up) as f64 / INTERVAL.as_secs_f64()).max(0.0)
        } else {
            0.0
        };

        let down_speed = if self.last_network_down > 0 && total_down >= self.last_network_down {
            ((total_down - self.last_network_down) as f64 / INTERVAL.as_secs_f64()).max(0.0)
        } else {
            0.0
        };

        self.last_network_up = total_up;
        self.last_network_down = total_down;

        (up_speed, down_speed)
    }

    pub fn get_cpu_cores(&self) -> Vec<CoreInfo> {
        self.sys
            .cpus()
            .iter()
            .enumerate()
            .map(|(i, cpu)| CoreInfo {
                name: format!("Core {}", i),
                usage: cpu.cpu_usage(),
                frequency: cpu.frequency(),
            })
            .collect()
    }

    pub fn get_memory_info(&self) -> MemoryInfo {
        let total = self.sys.total_memory();
        let used = self.sys.used_memory();
        let free = total.saturating_sub(used);

        MemoryInfo {
            total,
            used,
            free,
            cached: 0,  // sysinfo doesn't provide this on all platforms
            buffers: 0, // sysinfo doesn't provide this on all platforms
            swap_total: self.sys.total_swap(),
            swap_used: self.sys.used_swap(),
        }
    }

    pub fn get_processes(&self) -> Vec<ProcessInfo> {
        self.sys
            .processes()
            .iter()
            .map(|(pid, process)| ProcessInfo {
                pid: *pid,
                name: process.name().to_string_lossy().to_string(),
                user: process
                    .user_id()
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "Unknown".to_string()),
                status: format!("{:?}", process.status()),
                cpu_usage: process.cpu_usage(),
                memory: process.memory(),
                command: process
                    .cmd()
                    .iter()
                    .map(|s| s.to_string_lossy().to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
                    .chars()
                    .take(100)
                    .collect(),
            })
            .collect()
    }

    pub fn get_disks(&self) -> Vec<DiskInfo> {
        self.disks
            .iter()
            .map(|disk| DiskInfo {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                total: disk.total_space(),
                used: disk.total_space() - disk.available_space(),
                available: disk.available_space(),
                filesystem: disk.file_system().to_string_lossy().to_string(),
            })
            .collect()
    }

    pub fn get_interfaces(&self) -> Vec<InterfaceInfo> {
        self.networks
            .iter()
            .map(|(name, network)| {
                let ip_addresses: Vec<String> = network
                    .ip_networks()
                    .iter()
                    .map(|ip| ip.to_string())
                    .collect();

                InterfaceInfo {
                    name: name.to_string(),
                    ip_addresses,
                    mac_address: None, // sysinfo doesn't expose MAC easily
                    is_up: true,       // Assuming up if we can read it
                    total_received: network.total_received(),
                    total_transmitted: network.total_transmitted(),
                }
            })
            .collect()
    }

    pub fn get_connections(&self) -> Vec<ConnectionInfo> {
        // sysinfo doesn't provide connection list directly
        // This would need platform-specific implementation
        // Returning empty for now as placeholder
        Vec::new()
    }

    pub fn get_battery_info(&self) -> Vec<BatteryInfo> {
        let mut batteries = Vec::new();

        if let Ok(manager) = battery::Manager::new()
            && let Ok(bat_iter) = manager.batteries()
        {
            for battery in bat_iter.flatten() {
                let state = battery.state();
                let state_str = match state {
                    battery::State::Charging => "Charging",
                    battery::State::Discharging => "Discharging",
                    battery::State::Full => "Full",
                    battery::State::Empty => "Empty",
                    _ => "Unknown",
                };

                let time_remaining = if state == battery::State::Discharging {
                    battery.time_to_empty().map(|t| t.value as u64)
                } else if state == battery::State::Charging {
                    battery.time_to_full().map(|t| t.value as u64)
                } else {
                    None
                };

                batteries.push(BatteryInfo {
                    model: battery
                        .model()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "Battery".to_string()),
                    percentage: battery.state_of_charge().value * 100.0,
                    state: state_str.to_string(),
                    time_remaining,
                    health: battery.state_of_health().value * 100.0,
                    voltage: battery.voltage().value,
                    temperature: battery.temperature().map(|t| t.value),
                    cycle_count: battery.cycle_count(),
                });
            }
        }

        batteries
    }

    pub fn get_current_metrics(&self) -> Option<&MetricPoint> {
        self.data.back()
    }

    pub fn get_primary_disk_usage(&self) -> f32 {
        self.disks
            .iter()
            .next()
            .map(|disk| {
                let total = disk.total_space();
                let used = total - disk.available_space();
                if total > 0 {
                    (used as f64 / total as f64 * 100.0) as f32
                } else {
                    0.0
                }
            })
            .unwrap_or(0.0)
    }

    pub fn get_process_count(&self) -> usize {
        self.sys.processes().len()
    }

    pub fn check_alerts(&self) -> Vec<Alert> {
        let mut alerts = Vec::new();

        if let Some(metrics) = self.get_current_metrics() {
            if metrics.cpu >= self.thresholds.cpu_critical as f64 {
                alerts.push(Alert::Critical("CPU usage critical".to_string()));
            } else if metrics.cpu >= self.thresholds.cpu_warning as f64 {
                alerts.push(Alert::Warning("CPU usage high".to_string()));
            }

            if metrics.memory >= self.thresholds.memory_critical as f64 {
                alerts.push(Alert::Critical("Memory usage critical".to_string()));
            } else if metrics.memory >= self.thresholds.memory_warning as f64 {
                alerts.push(Alert::Warning("Memory usage high".to_string()));
            }
        }

        let disk_usage = self.get_primary_disk_usage();
        if disk_usage >= self.thresholds.disk_warning {
            alerts.push(Alert::Warning("Disk usage high".to_string()));
        }

        alerts
    }
}

#[derive(Clone, Debug)]
pub enum Alert {
    Warning(String),
    Critical(String),
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub fn format_bytes_speed(bytes_per_sec: u64) -> String {
    format_bytes(bytes_per_sec) + "/s"
}

pub fn format_duration(seconds: u64) -> String {
    if seconds >= 3600 {
        format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
    } else if seconds >= 60 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else {
        format!("{}s", seconds)
    }
}

pub fn get_metric_color(
    value: f32,
    warning: f32,
    critical: f32,
    theme: &gpui_component::Theme,
) -> gpui::Hsla {
    if value >= critical {
        theme.red
    } else if value >= warning {
        theme.yellow
    } else {
        theme.green
    }
}
