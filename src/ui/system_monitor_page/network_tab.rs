use gpui::{App, AppContext, IntoElement, ParentElement, Styled, Window, div, px};
use gpui_component::{
    ActiveTheme,
    chart::AreaChart,
    group_box::GroupBox,
    h_flex,
    table::{Column, Table, TableDelegate, TableState},
    v_flex,
};

use super::data_collector::{DataCollector, InterfaceInfo, format_bytes, format_bytes_speed};

/// Network tab with traffic visualization and interface details
pub struct NetworkTab {
    interface_table: gpui::Entity<TableState<InterfaceTableDelegate>>,
}

impl NetworkTab {
    pub fn new(table: gpui::Entity<TableState<InterfaceTableDelegate>>) -> Self {
        Self {
            interface_table: table,
        }
    }

    pub fn update_interfaces(&mut self, interfaces: Vec<InterfaceInfo>, cx: &mut App) {
        self.interface_table.update(cx, |table, cx| {
            table.delegate_mut().update_interfaces(interfaces);
            cx.notify();
        });
    }

    pub fn render(
        &self,
        collector: &DataCollector,
        theme: &gpui_component::Theme,
    ) -> impl IntoElement {
        let up_values: Vec<f64> = collector.data.iter().map(|p| p.network_up).collect();
        let down_values: Vec<f64> = collector.data.iter().map(|p| p.network_down).collect();
        let up_data = build_chart_points(&up_values, 5);
        let down_data = build_chart_points(&down_values, 5);

        let current_metrics = collector.get_current_metrics();
        let up_speed = current_metrics.map(|m| m.network_up as u64).unwrap_or(0);
        let down_speed = current_metrics.map(|m| m.network_down as u64).unwrap_or(0);

        // Clone colors to avoid lifetime issues in closures
        let green = theme.green;
        let yellow = theme.yellow;
        let background = theme.background;

        v_flex()
            .p_6()
            .gap_6()
            // Traffic Section
            .child(
                GroupBox::new().title("Network Traffic").child(
                    v_flex()
                        .gap_5()
                        // Current speeds
                        .child(
                            h_flex()
                                .gap_10()
                                .child(
                                    h_flex()
                                        .gap_2()
                                        .items_center()
                                        .child(div().text_color(green).child("↓"))
                                        .child(
                                            div()
                                                .text_lg()
                                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                                .child(format_bytes_speed(down_speed)),
                                        )
                                        .child(div().text_xs().child("Download")),
                                )
                                .child(
                                    h_flex()
                                        .gap_2()
                                        .items_center()
                                        .child(div().text_color(yellow).child("↑"))
                                        .child(
                                            div()
                                                .text_lg()
                                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                                .child(format_bytes_speed(up_speed)),
                                        )
                                        .child(div().text_xs().child("Upload")),
                                ),
                        )
                        // Traffic charts
                        .child(
                            h_flex()
                                .gap_6()
                                .child(
                                    v_flex()
                                        .flex_1()
                                        .gap_3()
                                        .child(div().text_xs().child("Download"))
                                        .child(
                                            div().h(px(180.0)).child(
                                                AreaChart::new(down_data)
                                                    .x(|(t, _)| t.clone())
                                                    .y(|(_, v)| *v)
                                                    .stroke(green)
                                                    .fill(gpui::linear_gradient(
                                                        0.0,
                                                        gpui::linear_color_stop(
                                                            green.opacity(0.3),
                                                            1.0,
                                                        ),
                                                        gpui::linear_color_stop(
                                                            background.opacity(0.1),
                                                            0.0,
                                                        ),
                                                    ))
                                                    .tick_margin(10),
                                            ),
                                        ),
                                )
                                .child(
                                    v_flex()
                                        .flex_1()
                                        .gap_3()
                                        .child(div().text_xs().child("Upload"))
                                        .child(
                                            div().h(px(180.0)).child(
                                                AreaChart::new(up_data)
                                                    .x(|(t, _)| t.clone())
                                                    .y(|(_, v)| *v)
                                                    .stroke(yellow)
                                                    .fill(gpui::linear_gradient(
                                                        0.0,
                                                        gpui::linear_color_stop(
                                                            yellow.opacity(0.3),
                                                            1.0,
                                                        ),
                                                        gpui::linear_color_stop(
                                                            background.opacity(0.1),
                                                            0.0,
                                                        ),
                                                    ))
                                                    .tick_margin(10),
                                            ),
                                        ),
                                ),
                        ),
                ),
            )
            // Interfaces Section
            .child(
                GroupBox::new().title("Network Interfaces").child(
                    v_flex().h(px(260.0)).child(
                        Table::new(&self.interface_table)
                            .bordered(false)
                            .stripe(true),
                    ),
                ),
            )
            .into_element()
    }
}

fn build_chart_points(values: &[f64], min_len: usize) -> Vec<(String, f64)> {
    let mut data: Vec<f64> = if values.is_empty() {
        vec![0.0]
    } else {
        values.to_vec()
    };

    if data.len() < min_len {
        let last = *data.last().unwrap_or(&0.0);
        data.extend(std::iter::repeat(last).take(min_len - data.len()));
    }

    data.iter()
        .enumerate()
        .map(|(i, v)| (format!("{}", i), *v))
        .collect()
}

/// Create the interface table entity
pub fn create_interface_table(
    window: &mut Window,
    cx: &mut App,
) -> gpui::Entity<TableState<InterfaceTableDelegate>> {
    let delegate = InterfaceTableDelegate::new();
    cx.new(|cx| {
        TableState::new(delegate, window, cx)
            .col_selectable(false)
            .col_movable(false)
    })
}

/// Table delegate for network interfaces
pub struct InterfaceTableDelegate {
    interfaces: Vec<InterfaceInfo>,
    columns: Vec<Column>,
}

impl InterfaceTableDelegate {
    fn new() -> Self {
        Self {
            interfaces: Vec::new(),
            columns: vec![
                Column::new("name", "Interface").width(px(120.0)),
                Column::new("ip", "IP Address").width(px(150.0)),
                Column::new("status", "Status").width(px(80.0)),
                Column::new("received", "Received").width(px(100.0)),
                Column::new("transmitted", "Transmitted").width(px(100.0)),
            ],
        }
    }

    fn update_interfaces(&mut self, interfaces: Vec<InterfaceInfo>) {
        self.interfaces = interfaces;
    }
}

impl TableDelegate for InterfaceTableDelegate {
    fn columns_count(&self, _cx: &App) -> usize {
        self.columns.len()
    }

    fn rows_count(&self, _cx: &App) -> usize {
        self.interfaces.len()
    }

    fn column(&self, col_ix: usize, _cx: &App) -> &Column {
        &self.columns[col_ix]
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        _window: &mut Window,
        cx: &mut gpui::Context<TableState<Self>>,
    ) -> impl IntoElement {
        let Some(interface) = self.interfaces.get(row_ix) else {
            return div().into_any_element();
        };

        let theme = cx.theme();

        match col_ix {
            0 => div()
                .text_sm()
                .child(interface.name.clone())
                .into_any_element(),
            1 => div()
                .text_xs()
                .child(interface.ip_addresses.join(", "))
                .into_any_element(),
            2 => div()
                .text_xs()
                .text_color(if interface.is_up {
                    theme.green
                } else {
                    theme.red
                })
                .child(if interface.is_up { "Up" } else { "Down" })
                .into_any_element(),
            3 => div()
                .text_xs()
                .child(format_bytes(interface.total_received))
                .into_any_element(),
            4 => div()
                .text_xs()
                .child(format_bytes(interface.total_transmitted))
                .into_any_element(),
            _ => div().into_any_element(),
        }
    }
}
