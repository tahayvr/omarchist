use gpui::*;
use gpui_component::{
    button::{Button, ButtonVariants as _},
    h_flex,
    select::{Select, SelectState},
    v_flex, ActiveTheme, IndexPath, Sizable, StyledExt, WindowExt,
};

use crate::system::waybar::{add_module_to_zone, module_library, LibraryModule, WaybarZone};
use crate::ui::status_bar_page::waybar_preview::WaybarPreview;

const ZONES: &[&str] = &["Left", "Center", "Right"];

/// One row in the library sheet: the module info + a zone selector + Add button.
/// These entities live outside the sheet closure so they persist across re-renders.
pub struct LibraryRow {
    pub module: &'static LibraryModule,
    pub zone_select: Entity<SelectState<Vec<SharedString>>>,
}

impl LibraryRow {
    fn new(module: &'static LibraryModule, window: &mut Window, cx: &mut App) -> Self {
        let zone_items: Vec<SharedString> = ZONES.iter().map(|s| SharedString::from(*s)).collect();
        // Default to "Left" (index 0)
        let zone_select =
            cx.new(|cx| SelectState::new(zone_items, Some(IndexPath::new(0)), window, cx));

        Self {
            module,
            zone_select,
        }
    }
}

/// Opens the module library sheet.
///
/// `profile_name` — the currently active profile.
/// `preview`      — the `WaybarPreview` entity to reload after a module is added.
pub fn open_module_library(
    profile_name: String,
    preview: Entity<WaybarPreview>,
    window: &mut Window,
    cx: &mut App,
) {
    // Build all rows before opening — they live outside the Fn closure.
    let modules: &'static [LibraryModule] = module_library_static();
    let rows: Vec<LibraryRow> = modules
        .iter()
        .map(|m| LibraryRow::new(m, window, cx))
        .collect();
    // Wrap in Entity so rows are accessible from inside the Fn closure
    let rows_entity: Entity<Vec<LibraryRow>> = cx.new(|_| rows);

    window.open_sheet(cx, move |sheet, _, _| {
        sheet
            .title(div().text_sm().font_semibold().child("Add Module"))
            .size(px(480.))
            .child(LibrarySheetContent {
                profile_name: profile_name.clone(),
                preview: preview.clone(),
                rows: rows_entity.clone(),
            })
    });
}

// ---------------------------------------------------------------------------
// Static module list — leaking is fine; it's a program-lifetime constant.
// ---------------------------------------------------------------------------
fn module_library_static() -> &'static [LibraryModule] {
    Box::leak(module_library().into_boxed_slice())
}

// ---------------------------------------------------------------------------
// The stateless render-once component that populates the sheet.
// ---------------------------------------------------------------------------
#[derive(IntoElement)]
struct LibrarySheetContent {
    profile_name: String,
    preview: Entity<WaybarPreview>,
    rows: Entity<Vec<LibraryRow>>,
}

impl RenderOnce for LibrarySheetContent {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        // Collect unique categories in insertion order
        let rows = self.rows.read(cx);
        let mut categories: Vec<&'static str> = Vec::new();
        for row in rows.iter() {
            if !categories.contains(&row.module.category) {
                categories.push(row.module.category);
            }
        }

        let mut sections: Vec<AnyElement> = Vec::new();

        for category in categories {
            let header = div()
                .text_xs()
                .font_semibold()
                .text_color(theme.muted_foreground)
                .pt_2()
                .pb_1()
                .child(category);

            let mut category_rows: Vec<AnyElement> = Vec::new();
            for row in rows.iter().filter(|r| r.module.category == category) {
                let profile = self.profile_name.clone();
                let preview_entity = self.preview.clone();
                let module_key = row.module.key;
                let default_config = row.module.default_config;
                let zone_select_entity = row.zone_select.clone();

                let add_btn = Button::new(SharedString::from(format!("add-{}", module_key)))
                    .label("Add")
                    .small()
                    .primary()
                    .on_click(move |_, window: &mut Window, cx| {
                        // Read the selected zone index at click-time
                        let zone = {
                            let state = zone_select_entity.read(cx);
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
                                window.push_notification(
                                    format!("Added \"{}\" to waybar", module_key),
                                    cx,
                                );
                            }
                            Err(e) => {
                                eprintln!("Failed to add module: {}", e);
                                window.push_notification(format!("Error: {}", e), cx);
                            }
                        }
                    });

                let zone_select = Select::new(&row.zone_select).small().w(px(80.));

                let row_el = h_flex()
                    .gap_3()
                    .py_2()
                    .w_full()
                    .items_center()
                    .border_b_1()
                    .border_color(theme.border.opacity(0.4))
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap_0p5()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme.foreground)
                                    .child(row.module.name),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.muted_foreground)
                                    .child(row.module.description),
                            ),
                    )
                    .child(div().flex_shrink_0().child(zone_select))
                    .child(div().flex_shrink_0().child(add_btn));

                category_rows.push(row_el.into_any());
            }

            sections.push(
                v_flex()
                    .gap_0()
                    .child(header)
                    .children(category_rows)
                    .into_any(),
            );
        }

        v_flex()
            .w_full()
            .gap_2()
            .child(
                div()
                    .text_xs()
                    .text_color(theme.muted_foreground)
                    .pb_2()
                    .child("Select a zone and click Add to insert a module into your bar."),
            )
            .children(sections)
    }
}
