use gpui::*;
use gpui_component::{
    ActiveTheme, IconName, IndexPath, Sizable, StyledExt, WindowExt,
    button::{Button, ButtonVariants as _},
    h_flex,
    scroll::ScrollableElement,
    select::{Select, SelectState},
    v_flex,
};

use crate::system::waybar::{LibraryModule, WaybarZone, add_module_to_zone, module_library};
use crate::ui::status_bar_page::waybar_preview::WaybarPreview;

const ZONES: &[&str] = &["Left", "Center", "Right"];

// Per-module row state
struct LibraryRowState {
    module: &'static LibraryModule,
    zone_select: Entity<SelectState<Vec<SharedString>>>,
}

impl LibraryRowState {
    fn new(module: &'static LibraryModule, window: &mut Window, cx: &mut App) -> Self {
        let zone_items: Vec<SharedString> = ZONES.iter().map(|s| SharedString::from(*s)).collect();
        let zone_select =
            cx.new(|cx| SelectState::new(zone_items, Some(IndexPath::new(0)), window, cx));
        Self {
            module,
            zone_select,
        }
    }
}

// ModuleLibraryPanel — stateful inline panel
pub struct ModuleLibraryPanel {
    profile_name: String,
    is_open: bool,
    rows: Vec<LibraryRowState>,
    preview: Entity<WaybarPreview>,
}

impl ModuleLibraryPanel {
    pub fn new(
        profile_name: &str,
        preview: Entity<WaybarPreview>,
        window: &mut Window,
        cx: &mut App,
    ) -> Self {
        let modules: &'static [LibraryModule] = Box::leak(module_library().into_boxed_slice());
        let rows = modules
            .iter()
            .map(|m| LibraryRowState::new(m, window, cx))
            .collect();
        Self {
            profile_name: profile_name.to_string(),
            is_open: false,
            rows,
            preview,
        }
    }

    pub fn switch_profile(&mut self, profile_name: &str) {
        self.profile_name = profile_name.to_string();
    }

    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        self.is_open = !self.is_open;
        cx.notify();
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }
}

/// Build a single module row element.
fn render_row(
    row: &LibraryRowState,
    profile: String,
    preview_entity: Entity<WaybarPreview>,
    theme_border: Hsla,
    theme_fg: Hsla,
    theme_muted: Hsla,
) -> AnyElement {
    let module_key = row.module.key;
    let module_name = row.module.name;
    let module_icon = row.module.icon;
    let default_config = row.module.default_config;
    let description = row.module.description;
    let zone_select_read = row.zone_select.clone();
    let zone_select_render = row.zone_select.clone();

    let add_btn = Button::new(SharedString::from(format!("add-{}", module_key)))
        .label("Add")
        .xsmall()
        .primary()
        .on_click(move |_, window: &mut Window, cx| {
            let zone = {
                let state = zone_select_read.read(cx);
                match state.selected_index(cx) {
                    Some(idx) if idx.row == 1 => WaybarZone::Center,
                    Some(idx) if idx.row == 2 => WaybarZone::Right,
                    _ => WaybarZone::Left,
                }
            };
            match add_module_to_zone(&profile, module_key, &zone, default_config) {
                Ok(()) => {
                    let p = profile.clone();
                    preview_entity.update(cx, |preview, cx| {
                        preview.reload(&p);
                        cx.notify();
                    });
                    window.push_notification(format!("Added {} to bar", module_name), cx);
                }
                Err(e) => {
                    window.push_notification(format!("Error: {}", e), cx);
                }
            }
        });

    h_flex()
        .gap_2()
        .py_1p5()
        .w_full()
        .min_w_0()
        .items_center()
        .border_b_1()
        .border_color(theme_border.opacity(0.3))
        .child(
            div()
                .w(px(20.))
                .flex_shrink_0()
                .text_sm()
                .text_color(theme_muted)
                .child(module_icon),
        )
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .gap_0()
                .child(div().text_sm().text_color(theme_fg).child(module_name))
                .child(div().text_xs().text_color(theme_muted).child(description)),
        )
        .child(
            div()
                .flex_shrink_0()
                .child(Select::new(&zone_select_render).xsmall().w(px(72.))),
        )
        .child(div().flex_shrink_0().child(add_btn))
        .into_any_element()
}

impl Render for ModuleLibraryPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.is_open {
            return div().into_any();
        }

        let theme = cx.theme();
        let two_col = window.viewport_size().width >= px(1000.0);

        // Collect unique categories in insertion order
        let mut categories: Vec<&'static str> = Vec::new();
        for row in &self.rows {
            if !categories.contains(&row.module.category) {
                categories.push(row.module.category);
            }
        }

        let mut sections: Vec<AnyElement> = Vec::new();

        for category in categories {
            let cat_rows: Vec<&LibraryRowState> = self
                .rows
                .iter()
                .filter(|r| r.module.category == category)
                .collect();

            let cat_header = div()
                .text_xs()
                .font_semibold()
                .text_color(theme.muted_foreground)
                .pt_2()
                .pb_1()
                .child(category);

            let rows_el: AnyElement = if two_col {
                // Split into two explicit columns: even indices left, odd right
                let mut left_rows: Vec<AnyElement> = Vec::new();
                let mut right_rows: Vec<AnyElement> = Vec::new();
                for (i, row) in cat_rows.iter().enumerate() {
                    let el = render_row(
                        row,
                        self.profile_name.clone(),
                        self.preview.clone(),
                        theme.border,
                        theme.foreground,
                        theme.muted_foreground,
                    );
                    if i % 2 == 0 {
                        left_rows.push(el);
                    } else {
                        right_rows.push(el);
                    }
                }
                h_flex()
                    .w_full()
                    .gap_0()
                    .items_start()
                    .child(
                        v_flex()
                            .w_1_2()
                            .min_w_0()
                            .pr_2()
                            .gap_0()
                            .children(left_rows),
                    )
                    .child(
                        v_flex()
                            .w_1_2()
                            .min_w_0()
                            .pl_2()
                            .gap_0()
                            .children(right_rows),
                    )
                    .into_any_element()
            } else {
                let all_rows: Vec<AnyElement> = cat_rows
                    .iter()
                    .map(|row| {
                        render_row(
                            row,
                            self.profile_name.clone(),
                            self.preview.clone(),
                            theme.border,
                            theme.foreground,
                            theme.muted_foreground,
                        )
                    })
                    .collect();
                v_flex()
                    .w_full()
                    .gap_0()
                    .children(all_rows)
                    .into_any_element()
            };

            sections.push(
                v_flex()
                    .w_full()
                    .gap_0()
                    .child(cat_header)
                    .child(rows_el)
                    .into_any_element(),
            );
        }

        v_flex()
            .w_full()
            .p_3()
            .gap_1()
            .border_1()
            .border_color(theme.border)
            .rounded_md()
            .child(
                h_flex()
                    .w_full()
                    .items_center()
                    .justify_between()
                    .pb_1()
                    .border_b_1()
                    .border_color(theme.border.opacity(0.5))
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child("Choose a zone and click Add to insert a module into your bar."),
                    )
                    .child(
                        Button::new("module-library-close")
                            .icon(IconName::Close)
                            .ghost()
                            .xsmall()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.is_open = false;
                                cx.notify();
                            })),
                    ),
            )
            .child(
                v_flex()
                    .w_full()
                    .p_4()
                    .max_h(px(400.))
                    .overflow_y_scrollbar()
                    .gap_0()
                    .children(sections),
            )
            .into_any()
    }
}
