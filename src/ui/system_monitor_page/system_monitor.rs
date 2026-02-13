use std::time::Duration;

use gpui::{prelude::FluentBuilder as _, *};
use gpui_component::{
    ActiveTheme, Icon, h_flex,
    progress::Progress,
    tab::{Tab, TabBar},
    v_flex,
};
use smol::Timer;

use super::{
    data_collector::{DataCollector, format_bytes_speed},
    disks_tab::DisksTab,
    network_tab::NetworkTab,
    overview_tab::OverviewTab,
    system_tab::SystemTab,
};

const INTERVAL: Duration = Duration::from_millis(500);

/// Tab indices
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum MonitorTab {
    #[default]
    Overview = 0,
    System = 1,
    Network = 2,
    Disks = 3,
}

impl MonitorTab {
    fn from_index(index: usize) -> Self {
        match index {
            0 => MonitorTab::Overview,
            1 => MonitorTab::System,
            2 => MonitorTab::Network,
            3 => MonitorTab::Disks,
            _ => MonitorTab::Overview,
        }
    }

    fn to_index(&self) -> usize {
        match self {
            MonitorTab::Overview => 0,
            MonitorTab::System => 1,
            MonitorTab::Network => 2,
            MonitorTab::Disks => 3,
        }
    }
}

/// System monitor page with comprehensive metrics
pub struct SystemMonitorPage {
    collector: DataCollector,
    active_tab: MonitorTab,
    overview_tab: OverviewTab,
    system_tab: SystemTab,
    network_tab: NetworkTab,
    disks_tab: DisksTab,
}

impl SystemMonitorPage {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let collector = DataCollector::new();

        let network_table = super::network_tab::create_interface_table(window, cx);
        let network_tab = NetworkTab::new(network_table);

        let disk_table = super::disks_tab::create_disk_table(window, cx);
        let disks_tab = DisksTab::new(disk_table);

        let monitor = Self {
            collector,
            active_tab: MonitorTab::Overview,
            overview_tab: OverviewTab::new(),
            system_tab: SystemTab::new(),
            network_tab,
            disks_tab,
        };

        // Start the update loop
        cx.spawn(async move |this, cx| {
            loop {
                Timer::after(INTERVAL).await;

                let result = this.update(cx, |this, cx| {
                    this.collector.collect();

                    // Update tabs that need it
                    let interfaces = this.collector.get_interfaces();
                    this.network_tab.update_interfaces(interfaces, cx);

                    let disks = this.collector.get_disks();
                    this.disks_tab.update_disks(disks, cx);

                    cx.notify();
                });

                if result.is_err() {
                    break;
                }
            }
        })
        .detach();

        monitor
    }

    fn set_active_tab(&mut self, index: usize, _window: &mut Window, cx: &mut Context<Self>) {
        self.active_tab = MonitorTab::from_index(index);
        cx.notify();
    }

    fn render_status_bar(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let metrics = self.collector.get_current_metrics();

        let cpu_percent = metrics.map(|m| m.cpu as f32).unwrap_or(0.0);
        let memory_percent = metrics.map(|m| m.memory as f32).unwrap_or(0.0);
        let disk_percent = self.collector.get_primary_disk_usage();

        let network_up = metrics.map(|m| m.network_up as u64).unwrap_or(0);
        let network_down = metrics.map(|m| m.network_down as u64).unwrap_or(0);

        // Determine colors based on thresholds
        let cpu_color = if cpu_percent >= self.collector.thresholds.cpu_critical {
            theme.red
        } else if cpu_percent >= self.collector.thresholds.cpu_warning {
            theme.yellow
        } else {
            theme.blue
        };

        let memory_color = if memory_percent >= self.collector.thresholds.memory_critical {
            theme.red
        } else if memory_percent >= self.collector.thresholds.memory_warning {
            theme.yellow
        } else {
            theme.green
        };

        let disk_color = if disk_percent >= self.collector.thresholds.disk_warning {
            theme.red
        } else {
            theme.chart_3
        };

        h_flex()
            .px_5()
            .gap_6()
            .h_12()
            .items_center()
            .justify_between()
            .flex_wrap()
            .border_t_1()
            .border_color(theme.border)
            .bg(theme.tab_bar)
            .text_color(theme.muted_foreground)
            // Left side - metrics with mini sparklines
            .child(
                h_flex()
                    .gap_6()
                    .flex_wrap()
                    // CPU
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                Icon::new(Icon::empty())
                                    .path("icons/cpu.svg")
                                    .size(px(16.0)),
                            )
                            .child(Progress::new().w(px(96.0)).h(px(6.0)).value(cpu_percent))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(cpu_color)
                                    .child(format!("{:.0}%", cpu_percent)),
                            ),
                    )
                    // Memory
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                Icon::new(Icon::empty())
                                    .path("icons/memory-stick.svg")
                                    .size(px(16.0)),
                            )
                            .child(Progress::new().w(px(96.0)).h(px(6.0)).value(memory_percent))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(memory_color)
                                    .child(format!("{:.0}%", memory_percent)),
                            ),
                    )
                    // Disk
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                Icon::new(Icon::empty())
                                    .path("icons/hard-drive.svg")
                                    .size(px(16.0)),
                            )
                            .child(Progress::new().w(px(96.0)).h(px(6.0)).value(disk_percent))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(disk_color)
                                    .child(format!("{:.0}%", disk_percent)),
                            ),
                    )
                    // Network
                    .child(
                        h_flex()
                            .gap_3()
                            .child(
                                h_flex()
                                    .gap_1()
                                    .child(div().text_color(theme.green).child("↓"))
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(theme.green)
                                            .child(format_bytes_speed(network_down)),
                                    ),
                            )
                            .child(
                                h_flex()
                                    .gap_1()
                                    .child(div().text_color(theme.yellow).child("↑"))
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(theme.yellow)
                                            .child(format_bytes_speed(network_up)),
                                    ),
                            ),
                    ),
            )
            // Right side - battery
            .into_element()
    }
}

impl Render for SystemMonitorPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let active_index = self.active_tab.to_index();

        let tab_bar = TabBar::new("monitor-tabs")
            .segmented()
            .selected_index(active_index)
            .on_click(cx.listener(|this, ix: &usize, window, cx| {
                this.set_active_tab(*ix, window, cx);
            }))
            .child(Tab::new().label("Overview"))
            .child(Tab::new().label("System"))
            .child(Tab::new().label("Network"))
            .child(Tab::new().label("Disks"));

        v_flex()
            .size_full()
            .child(tab_bar)
            .child(
                div()
                    .id("tab-content")
                    .flex_1()
                    .overflow_y_scroll()
                    .map(|this| match self.active_tab {
                        MonitorTab::Overview => {
                            this.child(self.overview_tab.render(&self.collector, theme))
                        }
                        MonitorTab::System => {
                            this.child(self.system_tab.render(&self.collector, theme))
                        }
                        MonitorTab::Network => {
                            this.child(self.network_tab.render(&self.collector, theme))
                        }
                        MonitorTab::Disks => {
                            this.child(self.disks_tab.render(&self.collector, theme))
                        }
                    }),
            )
            .child(self.render_status_bar(cx))
            .into_element()
    }
}
