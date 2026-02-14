use gpui::{div, Hsla, IntoElement, ParentElement, Pixels, Styled};
use gpui_component::{chart::AreaChart, h_flex};

/// A mini sparkline chart for displaying trends in small spaces
#[derive(Clone)]
pub struct Sparkline {
    data: Vec<f64>,
    color: Hsla,
    height: Pixels,
    fill: bool,
}

impl Sparkline {
    pub fn new(data: Vec<f64>) -> Self {
        Self {
            data,
            color: gpui::Hsla::default(),
            height: gpui::px(40.0),
            fill: true,
        }
    }

    pub fn color(mut self, color: Hsla) -> Self {
        self.color = color;
        self
    }

    pub fn height(mut self, height: Pixels) -> Self {
        self.height = height;
        self
    }

    pub fn no_fill(mut self) -> Self {
        self.fill = false;
        self
    }
}

impl IntoElement for Sparkline {
    type Element = gpui::Div;

    fn into_element(self) -> Self::Element {
        let data_points = build_chart_points(&self.data, 5);
        let tick_margin = data_points.len() + 1;

        h_flex()
            .h(self.height)
            .flex_1()
            .child(
                AreaChart::new(data_points)
                    .x(|(t, _)| t.clone())
                    .y(|(_, v)| *v)
                    .stroke(self.color)
                    .fill(if self.fill {
                        gpui::linear_gradient(
                            0.0,
                            gpui::linear_color_stop(self.color.opacity(0.3), 1.0),
                            gpui::linear_color_stop(gpui::Hsla::transparent_black(), 0.0),
                        )
                    } else {
                        gpui::linear_gradient(
                            0.0,
                            gpui::linear_color_stop(gpui::Hsla::transparent_black(), 0.0),
                            gpui::linear_color_stop(gpui::Hsla::transparent_black(), 0.0),
                        )
                    })
                    .tick_margin(tick_margin),
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

    // Find max value to normalize the scale
    let max_val = data.iter().copied().fold(0.0_f64, f64::max);
    let scale = if max_val > 0.0 { max_val } else { 1.0 };

    // Normalize to 0.0-1.0 range and add an anchor point at 1.0 to fix Y-axis scale
    let mut points: Vec<(String, f64)> = data
        .iter()
        .enumerate()
        .map(|(i, v)| (format!("{}", i), *v / scale))
        .collect();

    // Add hidden anchor point at max to fix Y-axis scale
    points.push(("".to_string(), 1.0));

    points
}

/// A colored indicator dot showing status
pub struct StatusDot {
    color: Hsla,
    size: Pixels,
}

impl StatusDot {
    pub fn new(color: Hsla) -> Self {
        Self {
            color,
            size: gpui::px(8.0),
        }
    }

    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }
}

impl IntoElement for StatusDot {
    type Element = gpui::Div;

    fn into_element(self) -> Self::Element {
        div()
            .w(self.size)
            .h(self.size)
            .rounded_full()
            .bg(self.color)
            .into_element()
    }
}

/// Get trend indicator (up/down/stable arrow)
pub fn trend_indicator(current: f64, previous: f64) -> &'static str {
    let diff = current - previous;
    let threshold = 0.1;

    if diff > threshold {
        "↑"
    } else if diff < -threshold {
        "↓"
    } else {
        "→"
    }
}

/// Get color based on trend
pub fn trend_color(
    current: f64,
    previous: f64,
    higher_is_better: bool,
    theme: &gpui_component::Theme,
) -> Hsla {
    let diff = current - previous;
    let threshold = 0.1;

    if higher_is_better {
        if diff > threshold {
            theme.green
        } else if diff < -threshold {
            theme.red
        } else {
            theme.muted_foreground
        }
    } else {
        if diff > threshold {
            theme.red
        } else if diff < -threshold {
            theme.green
        } else {
            theme.muted_foreground
        }
    }
}
