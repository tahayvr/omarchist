use gpui::{IntoElement, ParentElement, Styled, div, prelude::FluentBuilder as _, px};
use gpui_component::{Icon, h_flex, v_flex};

use super::sparkline::Sparkline;

/// A metric card displaying a value with icon, label, and sparkline
pub struct MetricCard {
    icon: Icon,
    label: String,
    value: String,
    sub_value: Option<String>,
    sparkline_data: Option<Vec<f64>>,
    color: gpui::Hsla,
    alert_color: Option<gpui::Hsla>,
    compact: bool,
    large: bool,
    border_color: gpui::Hsla,
}

impl MetricCard {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            icon: Icon::new(Icon::empty()),
            label: label.into(),
            value: value.into(),
            sub_value: None,
            sparkline_data: None,
            color: gpui::Hsla::default(),
            alert_color: None,
            compact: false,
            large: false,
            border_color: gpui::Hsla::default(),
        }
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = icon;
        self
    }

    pub fn sub_value(mut self, sub_value: impl Into<String>) -> Self {
        self.sub_value = Some(sub_value.into());
        self
    }

    pub fn sparkline(mut self, data: Vec<f64>) -> Self {
        self.sparkline_data = Some(data);
        self
    }

    pub fn color(mut self, color: gpui::Hsla) -> Self {
        self.color = color;
        self
    }

    pub fn alert_color(mut self, color: gpui::Hsla) -> Self {
        self.alert_color = Some(color);
        self
    }

    pub fn compact(mut self) -> Self {
        self.compact = true;
        self
    }

    pub fn large(mut self) -> Self {
        self.large = true;
        self
    }

    pub fn border_color(mut self, color: gpui::Hsla) -> Self {
        self.border_color = color;
        self
    }
}

impl IntoElement for MetricCard {
    type Element = gpui::Div;

    fn into_element(self) -> Self::Element {
        let value_color = self.alert_color.unwrap_or(self.color);
        let padding = if self.compact {
            px(14.0)
        } else if self.large {
            px(28.0)
        } else {
            px(22.0)
        };
        let icon_size = if self.compact {
            px(18.0)
        } else if self.large {
            px(32.0)
        } else {
            px(24.0)
        };
        let value_text_class = if self.large {
            |d: gpui::Div| d.text_3xl()
        } else {
            |d: gpui::Div| d.text_2xl()
        };

        div()
            .border_1()
            .border_color(self.border_color)
            .h_full()
            .child(
                v_flex()
                    .gap_3()
                    .p(padding)
                    .h_full()
                    .justify_between()
                    .child(
                        h_flex()
                            .gap_3()
                            .items_center()
                            .child(self.icon.size(icon_size))
                            .child(div().text_sm().child(self.label)),
                    )
                    .child(
                        v_flex()
                            .gap_2()
                            .child(
                                value_text_class(div())
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(value_color)
                                    .child(self.value),
                            )
                            .when_some(self.sub_value, |this, sub| {
                                this.child(div().text_xs().opacity(0.7).child(sub))
                            }),
                    )
                    .when_some(self.sparkline_data, |this, data| {
                        let sparkline_height = if self.large { px(160.0) } else { px(80.0) };
                        this.child(
                            div().flex_grow().child(
                                Sparkline::new(data)
                                    .color(self.color)
                                    .height(sparkline_height),
                            ),
                        )
                    }),
            )
            .into_element()
    }
}

/// A small metric display for the status bar
pub struct MiniMetric {
    icon: Icon,
    value: String,
    color: gpui::Hsla,
    sparkline_data: Option<Vec<f64>>,
}

impl MiniMetric {
    pub fn new(icon: Icon, value: impl Into<String>) -> Self {
        Self {
            icon,
            value: value.into(),
            color: gpui::Hsla::default(),
            sparkline_data: None,
        }
    }

    pub fn color(mut self, color: gpui::Hsla) -> Self {
        self.color = color;
        self
    }

    pub fn sparkline(mut self, data: Vec<f64>) -> Self {
        self.sparkline_data = Some(data);
        self
    }
}

impl IntoElement for MiniMetric {
    type Element = gpui::Div;

    fn into_element(self) -> Self::Element {
        h_flex()
            .gap_3()
            .items_center()
            .min_w(px(120.0))
            .child(self.icon.size(px(16.0)))
            .when_some(self.sparkline_data, |this, data| {
                this.child(
                    Sparkline::new(data)
                        .color(self.color)
                        .height(px(24.0))
                        .no_fill(),
                )
            })
            .child(div().text_sm().text_color(self.color).child(self.value))
            .into_element()
    }
}

/// Create a grid of metric cards
pub struct MetricGrid {
    children: Vec<MetricCard>,
    columns: usize,
    gap: gpui::Pixels,
}

impl Default for MetricGrid {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricGrid {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            columns: 3,
            gap: px(20.0),
        }
    }

    pub fn columns(mut self, columns: usize) -> Self {
        self.columns = columns;
        self
    }

    pub fn gap(mut self, gap: gpui::Pixels) -> Self {
        self.gap = gap;
        self
    }

    pub fn child(mut self, card: MetricCard) -> Self {
        self.children.push(card);
        self
    }
}

impl IntoElement for MetricGrid {
    type Element = gpui::Div;

    fn into_element(self) -> Self::Element {
        let mut grid = div().flex().flex_wrap().gap(self.gap);

        for card in self.children {
            grid = grid.child(
                div()
                    .flex()
                    .flex_basis(px(0.0))
                    .flex_grow()
                    .min_w(px(240.0))
                    .child(card),
            );
        }

        grid.into_element()
    }
}
