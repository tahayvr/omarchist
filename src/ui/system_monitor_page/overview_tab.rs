use gpui::{IntoElement, ParentElement, Styled, div, prelude::FluentBuilder as _, px};
use gpui_component::{Icon, v_flex};

use super::{
    data_collector::{DataCollector, format_bytes, format_bytes_speed},
    metric_card::MetricCard,
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
        viewport_width: gpui::Pixels,
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

        // Determine layout mode based on viewport width
        let is_compact = viewport_width < px(768.0);
        let is_medium = viewport_width >= px(768.0) && viewport_width < px(1200.0);

        // Bento Box Layout:
        // Desktop (>=1200px): 2x3 grid with Network as a large hero card on the left
        // Tablet (768-1200px): 2-column grid
        // Mobile (<768px): Single column

        if is_compact {
            // Mobile layout - single column
            let has_battery = battery_info.is_some();
            v_flex()
                .gap_4()
                .child(create_network_card(
                    theme,
                    network_down,
                    network_up,
                    network_down_data.clone(),
                ))
                .child(create_cpu_card(
                    theme,
                    cpu_percent,
                    cpu_color,
                    cpu_data.clone(),
                    collector,
                ))
                .child(create_memory_card(
                    theme,
                    memory_percent,
                    memory_color,
                    memory_data.clone(),
                    collector,
                ))
                .child(create_disk_card(theme, disk_percent, disk_color, collector))
                .child(create_processes_card(theme, process_count))
                .when(has_battery, |this| {
                    let battery = battery_info.unwrap();
                    this.child(create_battery_card(theme, battery, collector))
                })
                .into_element()
        } else if is_medium {
            // Tablet layout - 2 column bento
            v_flex()
                .gap_4()
                // Row 1: Network (spans full width, larger)
                .child(
                    create_network_card(theme, network_down, network_up, network_down_data.clone())
                        .large(),
                )
                // Row 2: CPU | Memory
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .gap_4()
                        .child(div().flex_1().child(create_cpu_card(
                            theme,
                            cpu_percent,
                            cpu_color,
                            cpu_data.clone(),
                            collector,
                        )))
                        .child(div().flex_1().child(create_memory_card(
                            theme,
                            memory_percent,
                            memory_color,
                            memory_data.clone(),
                            collector,
                        ))),
                )
                // Row 3: Disk | Processes (and Battery if present)
                .child({
                    let has_battery = battery_info.is_some();
                    div()
                        .flex()
                        .flex_row()
                        .gap_4()
                        .child(div().flex_1().child(create_disk_card(
                            theme,
                            disk_percent,
                            disk_color,
                            collector,
                        )))
                        .child(
                            div()
                                .flex_1()
                                .child(if let Some(ref battery) = battery_info {
                                    create_battery_card(theme, battery.clone(), collector)
                                } else {
                                    create_processes_card(theme, process_count)
                                }),
                        )
                        .when(has_battery, |this| {
                            this.child(create_processes_card(theme, process_count))
                        })
                })
                .into_element()
        } else {
            // Desktop Bento Layout
            // ┌───────────────────┬───────────┬───────────┐
            // │                   │    CPU    │  Memory   │
            // │    Network        ├───────────┼───────────┤
            // │    (hero)         │ Processes │   Disk    │
            // │                   │           │           │
            // └───────────────────┴───────────┴───────────┘
            // If battery present, it goes in bottom right instead of Disk
            let has_battery = battery_info.is_some();

            div()
                .flex()
                .flex_col()
                .gap_4()
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .gap_4()
                        // Left side: Network card (~65% width) - same height as right side
                        .child(div().w_2_3().child(create_network_card(
                            theme,
                            network_down,
                            network_up,
                            network_down_data.clone(),
                        )))
                        // Right side: 2x2 grid (~35% width)
                        .child(
                            div().w_1_3().child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_4()
                                    // Top row: Disk | Memory
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .gap_4()
                                            .child(div().flex_1().child(create_disk_card(
                                                theme,
                                                disk_percent,
                                                disk_color,
                                                collector,
                                            )))
                                            .child(div().flex_1().child(create_memory_card(
                                                theme,
                                                memory_percent,
                                                memory_color,
                                                memory_data.clone(),
                                                collector,
                                            ))),
                                    )
                                    // Bottom row: Processes | CPU (or Battery if present)
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .gap_4()
                                            .child(
                                                div().flex_1().child(create_processes_card(
                                                    theme,
                                                    process_count,
                                                )),
                                            )
                                            .child(div().flex_1().child(
                                                if let Some(ref battery) = battery_info {
                                                    create_battery_card(
                                                        theme,
                                                        battery.clone(),
                                                        collector,
                                                    )
                                                } else {
                                                    create_cpu_card(
                                                        theme,
                                                        cpu_percent,
                                                        cpu_color,
                                                        cpu_data.clone(),
                                                        collector,
                                                    )
                                                },
                                            )),
                                    ),
                            ),
                        ),
                )
                // If battery is present, show CPU in a second row
                .when(has_battery, |this| {
                    this.child(div().w_full().child(create_cpu_card(
                        theme,
                        cpu_percent,
                        cpu_color,
                        cpu_data.clone(),
                        collector,
                    )))
                })
                .into_element()
        }
    }
}

// Helper functions to create metric cards with consistent styling

fn create_cpu_card(
    theme: &gpui_component::Theme,
    cpu_percent: f32,
    cpu_color: gpui::Hsla,
    cpu_data: Vec<f64>,
    collector: &DataCollector,
) -> MetricCard {
    MetricCard::new("CPU Usage", format!("{:.1}%", cpu_percent))
        .icon(Icon::new(Icon::empty()).path("icons/cpu.svg"))
        .color(theme.blue)
        .alert_color(cpu_color)
        .sparkline(cpu_data)
        .sub_value(format!("{} cores", collector.get_cpu_cores().len()))
        .border_color(theme.border)
}

fn create_memory_card(
    theme: &gpui_component::Theme,
    memory_percent: f32,
    memory_color: gpui::Hsla,
    memory_data: Vec<f64>,
    collector: &DataCollector,
) -> MetricCard {
    let mem_info = collector.get_memory_info();
    MetricCard::new("Memory", format!("{:.1}%", memory_percent))
        .icon(Icon::new(Icon::empty()).path("icons/memory-stick.svg"))
        .color(theme.green)
        .alert_color(memory_color)
        .sparkline(memory_data)
        .sub_value(format!(
            "{} / {}",
            format_bytes(mem_info.used),
            format_bytes(mem_info.total)
        ))
        .border_color(theme.border)
}

fn create_network_card(
    theme: &gpui_component::Theme,
    network_down: u64,
    network_up: u64,
    network_down_data: Vec<f64>,
) -> MetricCard {
    MetricCard::new("Network", format_bytes_speed(network_down))
        .icon(Icon::new(Icon::empty()).path("icons/globe.svg"))
        .color(theme.cyan)
        .sparkline(network_down_data)
        .sub_value(format!("↑ {}", format_bytes_speed(network_up)))
        .border_color(theme.border)
}

fn create_disk_card(
    theme: &gpui_component::Theme,
    disk_percent: f32,
    disk_color: gpui::Hsla,
    collector: &DataCollector,
) -> MetricCard {
    let disks = collector.get_disks();
    let sub_value = if let Some(disk) = disks.first() {
        format!("{} / {}", format_bytes(disk.used), format_bytes(disk.total))
    } else {
        "No disks".to_string()
    };

    MetricCard::new("Disk", format!("{:.1}%", disk_percent))
        .icon(Icon::new(Icon::empty()).path("icons/hard-drive.svg"))
        .color(theme.chart_3)
        .alert_color(disk_color)
        .sparkline(vec![disk_percent as f64; 10])
        .sub_value(sub_value)
        .border_color(theme.border)
}

fn create_processes_card(theme: &gpui_component::Theme, process_count: usize) -> MetricCard {
    MetricCard::new("Processes", format!("{}", process_count))
        .icon(Icon::new(Icon::empty()).path("icons/layout-dashboard.svg"))
        .color(theme.chart_2)
        .sub_value("Active tasks")
        .border_color(theme.border)
}

fn create_battery_card(
    theme: &gpui_component::Theme,
    battery: super::data_collector::BatteryInfo,
    collector: &DataCollector,
) -> MetricCard {
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

    let sub_value = if let Some(seconds) = battery.time_remaining {
        if battery.state == "Discharging" {
            format!("{} remaining", format_duration(seconds))
        } else {
            format!("{} to full", format_duration(seconds))
        }
    } else {
        battery.state
    };

    MetricCard::new("Battery", format!("{:.0}%", battery_percent))
        .icon(Icon::new(Icon::empty()).path(icon_path))
        .color(theme.yellow)
        .alert_color(battery_color)
        .sub_value(sub_value)
        .border_color(theme.border)
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
