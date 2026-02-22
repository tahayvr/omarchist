use gpui::*;
use gpui_component::{
    ActiveTheme, IconName, Sizable,
    button::{Button, ButtonVariants as _},
    h_flex, v_flex,
};

use crate::ui::status_bar_page::bar_settings::BarSettingsPanel;
use crate::ui::status_bar_page::module_editor::{ModuleEditorPanel, take_pending_module_edit};
use crate::ui::status_bar_page::module_library::open_module_library;
use crate::ui::status_bar_page::waybar_preview::WaybarPreview;

pub struct DesignArea {
    profile_name: String,
    preview: Entity<WaybarPreview>,
    bar_settings: Entity<BarSettingsPanel>,
    module_editor: Entity<ModuleEditorPanel>,
}

impl DesignArea {
    pub fn new(profile_name: &str, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let name = profile_name.to_string();
        let preview = cx.new(|_| WaybarPreview::new(&name));
        let bar_settings = {
            let n = name.clone();
            cx.new(|cx| BarSettingsPanel::new(&n, window, cx))
        };
        let module_editor = {
            let n = name.clone();
            cx.new(|cx| ModuleEditorPanel::new(&n, window, cx))
        };
        Self {
            profile_name: name,
            preview,
            bar_settings,
            module_editor,
        }
    }

    pub fn switch_profile(
        &mut self,
        profile_name: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.profile_name = profile_name.to_string();
        self.preview.update(cx, |preview, _| {
            preview.reload(profile_name);
        });
        self.bar_settings.update(cx, |panel, cx| {
            panel.reload(profile_name, window, cx);
        });
        self.module_editor.update(cx, |editor, _| {
            editor.switch_profile(profile_name);
        });
    }
}

impl Render for DesignArea {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Handle any pending "Edit" request triggered by a context menu click.
        // Must be done before borrowing `theme` to avoid borrow conflicts.
        if let Some((profile_name, module_key)) = take_pending_module_edit() {
            // Sync profile if it somehow drifted (shouldn't happen normally).
            if profile_name != self.profile_name {
                self.profile_name = profile_name.clone();
            }
            self.module_editor.update(cx, |editor, cx| {
                editor.open(&module_key, window, cx);
            });
        }

        let theme = cx.theme();

        let profile = self.profile_name.clone();
        let preview_entity = self.preview.clone();

        let add_module_btn = Button::new("add-module-btn")
            .icon(IconName::Plus)
            .label("Add Module")
            .small()
            .ghost()
            .on_click(move |_, window: &mut Window, cx| {
                open_module_library(profile.clone(), preview_entity.clone(), window, cx);
            });

        v_flex()
            .w_full()
            .flex_1()
            .p_4()
            .gap_4()
            .border_1()
            .border_color(theme.border)
            .rounded_md()
            .child(h_flex().w_full().justify_end().child(add_module_btn))
            .child(self.preview.clone())
            .child(self.module_editor.clone())
            .child(self.bar_settings.clone())
    }
}
