use gpui::{div, px, IntoElement, ParentElement, Styled};
use gpui_component::{chart::AreaChart, group_box::GroupBox, h_flex, progress::Progress, v_flex};

use super::data_collector::{format_bytes, DataCollector};

/// System tab with detailed CPU and Memory information
pub struct SystemTab;

impl Default for SystemTab {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemTab {
    pub fn new() -> Self {
        Self
    }

    pub fn render(
        &self,
        collector: &DataCollector,
        theme: &gpui_component::Theme,
        viewport_width: gpui::Pixels,
    ) -> impl IntoElement {
        let cpu_data: Vec<f64> = collector.data.iter().map(|p| p.cpu).collect();
        let memory_data: Vec<f64> = collector.data.iter().map(|p| p.memory).collect();
        let cpu_cores = collector.get_cpu_cores();
        let memory_info = collector.get_memory_info();

        // Clone colors to avoid lifetime issues
        let cyan = theme.cyan;
        let red = theme.red;
        let yellow = theme.yellow;
        let green = theme.green;
        let blue = theme.blue;
        let background = theme.background;

        // Responsive chart height
        let chart_height = if viewport_width < px(640.0) {
            px(140.0)
        } else {
            px(180.0)
        };

        v_flex()
            .gap_6()
            // CPU Section
            .child(
                GroupBox::new().title("CPU").child(
                    v_flex()
                        .gap_5()
                        // Per-core usage bars
                        .child(h_flex().gap_2().flex_wrap().children(cpu_cores.iter().map(
                            |core| {
                                let usage = core.usage;
                                let color = if usage >= 80.0 {
                                    red
                                } else if usage >= 50.0 {
                                    yellow
                                } else {
                                    green
                                };

                                v_flex()
                                    .w(px(140.0))
                                    .gap_2()
                                    .child(div().text_xs().child(core.name.clone()))
                                    .child(
                                        h_flex()
                                            .gap_2()
                                            .items_center()
                                            .child(
                                                Progress::new().w(px(96.0)).h(px(8.0)).value(usage),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(color)
                                                    .child(format!("{:.1}%", usage)),
                                            ),
                                    )
                                    .child(div().text_xs().child(format!("{} MHz", core.frequency)))
                                    .into_any_element()
                            },
                        )))
                        // CPU History Chart
                        .child(
                            v_flex().gap_3().child(
                                div().h(chart_height).child(
                                    AreaChart::new(build_chart_points(&cpu_data, 5))
                                        .x(|(t, _)| t.clone())
                                        .y(|(_, v)| *v)
                                        .step_after()
                                        .stroke(cyan)
                                        .fill(gpui::linear_gradient(
                                            0.0,
                                            gpui::linear_color_stop(cyan.opacity(0.3), 1.0),
                                            gpui::linear_color_stop(background.opacity(0.1), 0.0),
                                        ))
                                        .tick_margin(10),
                                ),
                            ),
                        ),
                ),
            )
            // Memory Section
            .child(
                GroupBox::new().title("Memory").child(
                    v_flex()
                        .gap_3()
                        // Memory History Chart
                        .child(
                            v_flex().gap_3().child(
                                div().h(chart_height).child(
                                    AreaChart::new(build_chart_points(&memory_data, 5))
                                        .x(|(t, _)| t.clone())
                                        .y(|(_, v)| *v)
                                        .step_after()
                                        .stroke(blue)
                                        .fill(gpui::linear_gradient(
                                            0.0,
                                            gpui::linear_color_stop(blue.opacity(0.3), 1.0),
                                            gpui::linear_color_stop(background.opacity(0.1), 0.0),
                                        ))
                                        .tick_margin(10),
                                ),
                            ),
                        )
                        // Memory stats - flex_wrap for responsiveness
                        .child(
                            h_flex()
                                .gap_6()
                                .flex_wrap()
                                .mt_2()
                                .child(
                                    v_flex()
                                        .gap_1()
                                        .child(div().text_xs().child("Total"))
                                        .child(
                                            div().text_sm().child(format_bytes(memory_info.total)),
                                        ),
                                )
                                .child(
                                    v_flex().gap_1().child(div().text_xs().child("Used")).child(
                                        div().text_sm().child(format_bytes(memory_info.used)),
                                    ),
                                )
                                .child(
                                    v_flex().gap_1().child(div().text_xs().child("Swap")).child(
                                        div().text_sm().child(format!(
                                            "{} / {}",
                                            format_bytes(memory_info.swap_used),
                                            format_bytes(memory_info.swap_total)
                                        )),
                                    ),
                                ),
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
        data.extend(std::iter::repeat_n(last, min_len - data.len()));
    }

    let mut points: Vec<(String, f64)> = data
        .iter()
        .enumerate()
        .map(|(i, v)| (format!("{}", i), *v / 100.0))
        .collect();

    // Add hidden anchor point at 100% to fix Y-axis scale
    points.push(("".to_string(), 1.0));

    points
}
