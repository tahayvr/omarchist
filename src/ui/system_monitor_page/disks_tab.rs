use gpui::{App, AppContext, IntoElement, ParentElement, Styled, Window, div, px};
use gpui_component::{
    ActiveTheme,
    group_box::GroupBox,
    h_flex,
    progress::Progress,
    table::{Column, Table, TableDelegate, TableState},
    v_flex,
};

use super::data_collector::{DataCollector, DiskInfo, format_bytes};

pub struct DisksTab {
    disk_table: gpui::Entity<TableState<DiskTableDelegate>>,
}

impl DisksTab {
    pub fn new(table: gpui::Entity<TableState<DiskTableDelegate>>) -> Self {
        Self { disk_table: table }
    }

    pub fn update_disks(&mut self, disks: Vec<DiskInfo>, cx: &mut App) {
        self.disk_table.update(cx, |table, cx| {
            table.delegate_mut().update_disks(disks);
            cx.notify();
        });
    }

    pub fn render(
        &self,
        collector: &DataCollector,
        theme: &gpui_component::Theme,
        viewport_width: gpui::Pixels,
    ) -> impl IntoElement {
        let disks = collector.get_disks();

        // Clone colors to avoid lifetime issues
        let red = theme.red;
        let yellow = theme.yellow;
        let chart_3 = theme.chart_3;
        let border = theme.border;

        v_flex()
            .gap_6()
            // Disk Usage Overview
            .child(
                GroupBox::new().title("Disk Usage").child(
                    v_flex()
                        .gap_5()
                        // Individual disk cards
                        .child(
                            h_flex()
                                .gap_4()
                                .flex_wrap()
                                .children(disks.iter().map(|disk| {
                                    let usage_percent = if disk.total > 0 {
                                        (disk.used as f64 / disk.total as f64 * 100.0) as f32
                                    } else {
                                        0.0
                                    };
                                    let color = if usage_percent >= 85.0 {
                                        red
                                    } else if usage_percent >= 70.0 {
                                        yellow
                                    } else {
                                        chart_3
                                    };

                                    v_flex()
                                        .flex_basis(if viewport_width < px(640.0) {
                                            px(0.0)
                                        } else {
                                            px(240.0)
                                        })
                                        .flex_grow()
                                        .min_w(if viewport_width < px(640.0) {
                                            px(0.0)
                                        } else {
                                            px(200.0)
                                        })
                                        .p_4()
                                        .border_1()
                                        .border_color(border)
                                        .gap_3()
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(gpui::FontWeight::MEDIUM)
                                                .child(disk.name.clone()),
                                        )
                                        .child(div().text_xs().child(disk.mount_point.clone()))
                                        .child(
                                            h_flex()
                                                .gap_3()
                                                .items_center()
                                                .child(
                                                    Progress::new()
                                                        .w(px(140.0))
                                                        .h(px(8.0))
                                                        .value(usage_percent),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(color)
                                                        .child(format!("{:.1}%", usage_percent)),
                                                ),
                                        )
                                        .child(
                                            h_flex()
                                                .gap_3()
                                                .child(
                                                    v_flex()
                                                        .gap_1()
                                                        .child(div().text_xs().child("Used"))
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .child(format_bytes(disk.used)),
                                                        ),
                                                )
                                                .child(
                                                    v_flex()
                                                        .gap_1()
                                                        .child(div().text_xs().child("Free"))
                                                        .child(
                                                            div().text_xs().child(format_bytes(
                                                                disk.available,
                                                            )),
                                                        ),
                                                ),
                                        )
                                        .into_any_element()
                                })),
                        ),
                ),
            )
            // Disk Details Table
            .child(
                GroupBox::new().title("Disk Details").child(
                    v_flex()
                        .h(px(240.0))
                        .min_h(px(150.0))
                        .flex_1()
                        .overflow_hidden()
                        .child(Table::new(&self.disk_table).bordered(false).stripe(true)),
                ),
            )
            .into_element()
    }
}

pub fn create_disk_table(
    window: &mut Window,
    cx: &mut App,
) -> gpui::Entity<TableState<DiskTableDelegate>> {
    let delegate = DiskTableDelegate::new();
    cx.new(|cx| {
        TableState::new(delegate, window, cx)
            .col_selectable(false)
            .col_movable(false)
    })
}

pub struct DiskTableDelegate {
    disks: Vec<DiskInfo>,
    columns: Vec<Column>,
}

impl DiskTableDelegate {
    fn new() -> Self {
        Self {
            disks: Vec::new(),
            columns: vec![
                Column::new("name", "Name").width(px(80.0)),
                Column::new("mount", "Mount").width(px(100.0)),
                Column::new("filesystem", "FS").width(px(60.0)),
                Column::new("total", "Total").width(px(70.0)),
                Column::new("used", "Used").width(px(70.0)),
                Column::new("available", "Free").width(px(70.0)),
            ],
        }
    }

    fn update_disks(&mut self, disks: Vec<DiskInfo>) {
        self.disks = disks;
    }
}

impl TableDelegate for DiskTableDelegate {
    fn columns_count(&self, _cx: &App) -> usize {
        self.columns.len()
    }

    fn rows_count(&self, _cx: &App) -> usize {
        self.disks.len()
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
        let Some(disk) = self.disks.get(row_ix) else {
            return div().into_any_element();
        };

        let theme = cx.theme();

        match col_ix {
            0 => div().text_sm().child(disk.name.clone()).into_any_element(),
            1 => div()
                .text_xs()
                .child(disk.mount_point.clone())
                .into_any_element(),
            2 => div()
                .text_xs()
                .child(disk.filesystem.clone())
                .into_any_element(),
            3 => div()
                .text_xs()
                .child(format_bytes(disk.total))
                .into_any_element(),
            4 => div()
                .text_xs()
                .text_color(theme.blue)
                .child(format_bytes(disk.used))
                .into_any_element(),
            5 => div()
                .text_xs()
                .text_color(theme.green)
                .child(format_bytes(disk.available))
                .into_any_element(),
            _ => div().into_any_element(),
        }
    }
}
