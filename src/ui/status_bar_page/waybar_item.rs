use gpui::*;
use gpui_component::{
    ActiveTheme,
    menu::{ContextMenuExt, PopupMenuItem},
    tooltip::Tooltip,
};

use crate::system::waybar::{WaybarModule, WaybarZone};
use crate::ui::status_bar_page::module_editor::request_module_edit;
use crate::ui::status_bar_page::waybar_preview::WaybarPreview;

/// Drag payload — carries the source zone and index of the dragged module
#[derive(Clone, Debug)]
pub struct DragWaybarModule {
    pub zone: WaybarZone,
    pub index: usize,
    pub label: String,
}

/// Ghost view rendered while the module is being dragged
struct DragGhost {
    icon: String,
    label: String,
}

impl Render for DragGhost {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let content = if self.icon.is_empty() {
            self.label.clone()
        } else {
            self.icon.clone()
        };
        div()
            .px_2()
            .py_1()
            .rounded_md()
            .bg(theme.accent)
            .text_color(theme.accent_foreground)
            .text_sm()
            .shadow_md()
            .child(content)
    }
}

/// Render a single draggable waybar module chip with a right-click context menu.
pub fn render_module_chip(
    module: &WaybarModule,
    zone: WaybarZone,
    index: usize,
    profile_name: &str,
    preview: Entity<WaybarPreview>,
    cx: &mut App,
) -> AnyElement {
    let theme = cx.theme();
    let label = module.label.clone();
    let icon = module.icon.clone();
    let drag_payload = DragWaybarModule {
        zone: zone.clone(),
        index,
        label: label.clone(),
    };

    let label_for_menu = label.clone();
    let label_for_tooltip = label.clone();
    let icon_for_ghost = icon.clone();

    // Show icon if available, fall back to label text
    let display = if icon.is_empty() {
        label.clone()
    } else {
        icon.clone()
    };

    let zone_for_remove = zone.clone();
    let preview_for_remove = preview.clone();

    let profile_for_edit = profile_name.to_string();
    let module_key_for_edit = module.key.clone();

    div()
        .id(SharedString::from(format!(
            "wbmod-{}-{}",
            index,
            label.replace(' ', "-").to_lowercase()
        )))
        .px_2()
        .py_1()
        .rounded_md()
        .cursor_grab()
        .bg(theme.secondary)
        .text_color(theme.secondary_foreground)
        .border_1()
        .border_color(theme.border)
        .hover(|s| s.bg(theme.secondary_hover))
        .child(div().text_lg().line_height(relative(1.0)).child(display))
        .tooltip(move |window, cx| Tooltip::new(label_for_tooltip.clone()).build(window, cx))
        .on_drag(drag_payload, move |payload, _offset, _window, cx| {
            cx.stop_propagation();
            cx.new(|_| DragGhost {
                icon: icon_for_ghost.clone(),
                label: payload.label.clone(),
            })
        })
        .context_menu(move |menu, _, _| {
            let p = profile_for_edit.clone();
            let mk = module_key_for_edit.clone();
            menu.item(
                // TODO: Module editing is disabled for now — re-enable when the feature is ready
                PopupMenuItem::new(format!("Edit \"{}\"", label_for_menu))
                    .disabled(true)
                    .on_click(move |_event, _window, _cx| {
                        request_module_edit(p.clone(), mk.clone());
                    }),
            )
            .separator()
            .item({
                let preview = preview_for_remove.clone();
                let zone = zone_for_remove.clone();
                PopupMenuItem::new("Remove from bar").on_click(move |_event, _window, cx| {
                    preview.update(cx, |this, cx| {
                        this.remove_module(&zone, index);
                        cx.notify();
                    });
                })
            })
        })
        .into_any()
}
