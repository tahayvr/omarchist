use gpui::{IntoElement, ParentElement, Styled, prelude::FluentBuilder as _, px};
use gpui_component::{Icon, v_flex};

use super::{
    data_collector::{DataCollector, format_bytes, format_bytes_speed},
    metric_card::{MetricCard, MetricGrid},
};

/// Overview tab showing key metrics at a glance
pub struct OverviewTab;

impl OverviewTab {
    pub fn new() -> Self {
        Self
    }

    pub fn render(
        &self,
        collector: &DataCollector,
        theme: &gpui_component::Theme,
    ) -> impl IntoElement {
        let metrics = collector.get_current_metrics();
        let cpu_data: Vec<f64> = if collector.data.is_empty() {
            vec![0.0]
        } else {
            collector.data.iter().map(|p| p.cpu).collect()
        };
        let memory_data: Vec<f64> = if collector.data.is_empty() {
            vec![0.0]
        } else {
            collector.data.iter().map(|p| p.memory).collect()
        };
        let network_down_data: Vec<f64> = if collector.data.is_empty() {
            vec![0.0]
        } else {
            collector.data.iter().map(|p| p.network_down).collect()
        };

        let cpu_percent = metrics.map(|m| m.cpu as f32).unwrap_or(0.0);
        let memory_percent = metrics.map(|m| m.memory as f32).unwrap_or(0.0);
        let disk_percent = collector.get_primary_disk_usage();
        let process_count = collector.get_process_count();

        let network_up = metrics.map(|m| m.network_up as u64).unwrap_or(0);
        let network_down = metrics.map(|m| m.network_down as u64).unwrap_or(0);

        // Get alert colors based on thresholds
        let cpu_color = if cpu_percent >= collector.thresholds.cpu_critical {
            theme.red
        } else if cpu_percent >= collector.thresholds.cpu_warning {
            theme.yellow
        } else {
            theme.blue
        };

        let memory_color = if memory_percent >= collector.thresholds.memory_critical {
            theme.red
        } else if memory_percent >= collector.thresholds.memory_warning {
            theme.yellow
        } else {
            theme.green
        };

        let disk_color = if disk_percent >= collector.thresholds.disk_warning {
            theme.red
        } else {
            theme.chart_3
        };

        let battery_info = collector.get_battery_info().first().cloned();

        v_flex()
            .p_6()
            .gap_6()
            .child(
                MetricGrid::new()
                    .columns(3)
                    .gap(px(20.0))
                    // CPU Card
                    .child(
                        MetricCard::new("CPU Usage", format!("{:.1}%", cpu_percent))
                            .icon(Icon::new(Icon::empty()).path("icons/cpu.svg"))
                            .color(theme.blue)
                            .alert_color(cpu_color)
                            .sparkline(cpu_data)
                            .sub_value(format!("{} cores", collector.get_cpu_cores().len())),
                    )
                    // Memory Card
                    .child(
                        MetricCard::new("Memory", format!("{:.1}%", memory_percent))
                            .icon(Icon::new(Icon::empty()).path("icons/memory-stick.svg"))
                            .color(theme.green)
                            .alert_color(memory_color)
                            .sparkline(memory_data)
                            .sub_value({
                                let mem_info = collector.get_memory_info();
                                format!(
                                    "{} / {}",
                                    format_bytes(mem_info.used),
                                    format_bytes(mem_info.total)
                                )
                            }),
                    )
                    // Network Card
                    .child(
                        MetricCard::new("Network", format_bytes_speed(network_down))
                            .icon(Icon::new(Icon::empty()).path("icons/globe.svg"))
                            .color(theme.cyan)
                            .sparkline(network_down_data)
                            .sub_value(format!("↑ {}", format_bytes_speed(network_up))),
                    )
                    // Disk Card
                    .child(
                        MetricCard::new("Disk", format!("{:.1}%", disk_percent))
                            .icon(Icon::new(Icon::empty()).path("icons/hard-drive.svg"))
                            .color(theme.chart_3)
                            .alert_color(disk_color)
                            .sparkline({
                                // Create synthetic disk trend from single value
                                vec![disk_percent as f64; 10]
                            })
                            .sub_value({
                                let disks = collector.get_disks();
                                if let Some(disk) = disks.first() {
                                    format!(
                                        "{} / {}",
                                        format_bytes(disk.used),
                                        format_bytes(disk.total)
                                    )
                                } else {
                                    "No disks".to_string()
                                }
                            }),
                    )
                    // Processes Card
                    .child(
                        MetricCard::new("Processes", format!("{}", process_count))
                            .icon(Icon::new(Icon::empty()).path("icons/layout-dashboard.svg"))
                            .color(theme.chart_2)
                            .sub_value("Active tasks"),
                    )
                    // Battery Card (if available)
                    .when(battery_info.is_some(), |this| {
                        let battery = battery_info.unwrap();
                        let battery_percent = battery.percentage;
                        let battery_color = if battery_percent <= collector.thresholds.battery_low {
                            theme.red
                        } else if battery.state == "Charging" {
                            theme.green
                        } else {
                            theme.yellow
                        };

                        let icon_path = if battery.state == "Charging" {
                            "icons/battery-charging.svg"
                        } else if battery_percent >= 80.0 {
                            "icons/battery-full.svg"
                        } else if battery_percent >= 50.0 {
                            "icons/battery-medium.svg"
                        } else if battery_percent >= 20.0 {
                            "icons/battery-low.svg"
                        } else {
                            "icons/battery-warning.svg"
                        };

                        this.child(
                            MetricCard::new("Battery", format!("{:.0}%", battery_percent))
                                .icon(Icon::new(Icon::empty()).path(icon_path))
                                .color(theme.yellow)
                                .alert_color(battery_color)
                                .sub_value(if let Some(seconds) = battery.time_remaining {
                                    if battery.state == "Discharging" {
                                        format!("{} remaining", format_duration(seconds))
                                    } else {
                                        format!("{} to full", format_duration(seconds))
                                    }
                                } else {
                                    battery.state
                                }),
                        )
                    }),
            )
            .into_element()
    }
}

fn format_duration(seconds: u64) -> String {
    if seconds >= 3600 {
        format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
    } else if seconds >= 60 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else {
        format!("{}s", seconds)
    }
}
