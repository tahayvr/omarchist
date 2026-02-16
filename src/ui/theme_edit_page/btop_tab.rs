//! Btop tab for theme editing
//!
//! Provides UI for editing Btop activity monitor colors using ColorPicker components.
//! Organized into sections with dividers for better visual separation.

use crate::system::themes::theme_management::{save_theme_data, update_btop_theme};
use crate::types::themes::{BtopConfig, EditingTheme};
use crate::ui::theme_edit_page::shared::{
    color_picker_with_clipboard, form_section, tab_container,
};
use gpui::*;
use gpui_component::{
    Colorize,
    color_picker::{ColorPickerEvent, ColorPickerState},
    divider::Divider,
    h_flex, v_flex,
};

/// Btop tab content for editing btop theme colors
pub struct BtopTab {
    theme_name: String,
    theme_data: EditingTheme,
    // Main colors
    main_bg_picker: Entity<ColorPickerState>,
    main_fg_picker: Entity<ColorPickerState>,
    title_picker: Entity<ColorPickerState>,
    hi_fg_picker: Entity<ColorPickerState>,
    // Selection colors
    selected_bg_picker: Entity<ColorPickerState>,
    selected_fg_picker: Entity<ColorPickerState>,
    // Status colors
    inactive_fg_picker: Entity<ColorPickerState>,
    proc_misc_picker: Entity<ColorPickerState>,
    // Box colors
    cpu_box_picker: Entity<ColorPickerState>,
    mem_box_picker: Entity<ColorPickerState>,
    net_box_picker: Entity<ColorPickerState>,
    proc_box_picker: Entity<ColorPickerState>,
    div_line_picker: Entity<ColorPickerState>,
    // Temperature gradient
    temp_start_picker: Entity<ColorPickerState>,
    temp_mid_picker: Entity<ColorPickerState>,
    temp_end_picker: Entity<ColorPickerState>,
    // CPU gradient
    cpu_start_picker: Entity<ColorPickerState>,
    cpu_mid_picker: Entity<ColorPickerState>,
    cpu_end_picker: Entity<ColorPickerState>,
    // Free meter gradient
    free_start_picker: Entity<ColorPickerState>,
    free_mid_picker: Entity<ColorPickerState>,
    free_end_picker: Entity<ColorPickerState>,
    // Cached meter gradient
    cached_start_picker: Entity<ColorPickerState>,
    cached_mid_picker: Entity<ColorPickerState>,
    cached_end_picker: Entity<ColorPickerState>,
    // Available meter gradient
    available_start_picker: Entity<ColorPickerState>,
    available_mid_picker: Entity<ColorPickerState>,
    available_end_picker: Entity<ColorPickerState>,
    // Used meter gradient
    used_start_picker: Entity<ColorPickerState>,
    used_mid_picker: Entity<ColorPickerState>,
    used_end_picker: Entity<ColorPickerState>,
    // Download gradient
    download_start_picker: Entity<ColorPickerState>,
    download_mid_picker: Entity<ColorPickerState>,
    download_end_picker: Entity<ColorPickerState>,
    // Upload gradient
    upload_start_picker: Entity<ColorPickerState>,
    upload_mid_picker: Entity<ColorPickerState>,
    upload_end_picker: Entity<ColorPickerState>,
    is_saving: bool,
    error_message: Option<String>,
}

impl BtopTab {
    /// Create a new BtopTab instance
    pub fn new(
        theme_name: String,
        theme_data: EditingTheme,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        // Get current btop config or use defaults
        let btop_config = theme_data.apps.btop.as_ref().cloned().unwrap_or_default();

        // Create color picker states with current values
        let main_bg = Self::hex_to_hsla(&btop_config.main_bg).unwrap_or(gpui::rgb(0x0F0F19).into());
        let main_fg = Self::hex_to_hsla(&btop_config.main_fg).unwrap_or(gpui::rgb(0xEDEDFE).into());
        let title = Self::hex_to_hsla(&btop_config.title).unwrap_or(gpui::rgb(0x6e6e92).into());
        let hi_fg = Self::hex_to_hsla(&btop_config.hi_fg).unwrap_or(gpui::rgb(0x33A1FF).into());
        let selected_bg =
            Self::hex_to_hsla(&btop_config.selected_bg).unwrap_or(gpui::rgb(0xf59e0b).into());
        let selected_fg =
            Self::hex_to_hsla(&btop_config.selected_fg).unwrap_or(gpui::rgb(0xEDEDFE).into());
        let inactive_fg =
            Self::hex_to_hsla(&btop_config.inactive_fg).unwrap_or(gpui::rgb(0x333333).into());
        let proc_misc =
            Self::hex_to_hsla(&btop_config.proc_misc).unwrap_or(gpui::rgb(0x8a8a8d).into());
        let cpu_box = Self::hex_to_hsla(&btop_config.cpu_box).unwrap_or(gpui::rgb(0x6e6e92).into());
        let mem_box = Self::hex_to_hsla(&btop_config.mem_box).unwrap_or(gpui::rgb(0x6e6e92).into());
        let net_box = Self::hex_to_hsla(&btop_config.net_box).unwrap_or(gpui::rgb(0x6e6e92).into());
        let proc_box =
            Self::hex_to_hsla(&btop_config.proc_box).unwrap_or(gpui::rgb(0x6e6e92).into());
        let div_line =
            Self::hex_to_hsla(&btop_config.div_line).unwrap_or(gpui::rgb(0x6e6e92).into());
        let temp_start =
            Self::hex_to_hsla(&btop_config.temp_start).unwrap_or(gpui::rgb(0x00F59B).into());
        let temp_mid =
            Self::hex_to_hsla(&btop_config.temp_mid).unwrap_or(gpui::rgb(0xFF66F6).into());
        let temp_end =
            Self::hex_to_hsla(&btop_config.temp_end).unwrap_or(gpui::rgb(0xFF3366).into());
        let cpu_start =
            Self::hex_to_hsla(&btop_config.cpu_start).unwrap_or(gpui::rgb(0x00F59B).into());
        let cpu_mid = Self::hex_to_hsla(&btop_config.cpu_mid).unwrap_or(gpui::rgb(0xFF66F6).into());
        let cpu_end = Self::hex_to_hsla(&btop_config.cpu_end).unwrap_or(gpui::rgb(0xFF3366).into());
        let free_start =
            Self::hex_to_hsla(&btop_config.free_start).unwrap_or(gpui::rgb(0x00F59B).into());
        let free_mid =
            Self::hex_to_hsla(&btop_config.free_mid).unwrap_or(gpui::rgb(0xFF66F6).into());
        let free_end =
            Self::hex_to_hsla(&btop_config.free_end).unwrap_or(gpui::rgb(0xFF3366).into());
        let cached_start =
            Self::hex_to_hsla(&btop_config.cached_start).unwrap_or(gpui::rgb(0x00F59B).into());
        let cached_mid =
            Self::hex_to_hsla(&btop_config.cached_mid).unwrap_or(gpui::rgb(0xFF66F6).into());
        let cached_end =
            Self::hex_to_hsla(&btop_config.cached_end).unwrap_or(gpui::rgb(0xFF3366).into());
        let available_start =
            Self::hex_to_hsla(&btop_config.available_start).unwrap_or(gpui::rgb(0x00F59B).into());
        let available_mid =
            Self::hex_to_hsla(&btop_config.available_mid).unwrap_or(gpui::rgb(0xFF66F6).into());
        let available_end =
            Self::hex_to_hsla(&btop_config.available_end).unwrap_or(gpui::rgb(0xFF3366).into());
        let used_start =
            Self::hex_to_hsla(&btop_config.used_start).unwrap_or(gpui::rgb(0x00F59B).into());
        let used_mid =
            Self::hex_to_hsla(&btop_config.used_mid).unwrap_or(gpui::rgb(0xFF66F6).into());
        let used_end =
            Self::hex_to_hsla(&btop_config.used_end).unwrap_or(gpui::rgb(0xFF3366).into());
        let download_start =
            Self::hex_to_hsla(&btop_config.download_start).unwrap_or(gpui::rgb(0x00F59B).into());
        let download_mid =
            Self::hex_to_hsla(&btop_config.download_mid).unwrap_or(gpui::rgb(0xFF66F6).into());
        let download_end =
            Self::hex_to_hsla(&btop_config.download_end).unwrap_or(gpui::rgb(0xFF3366).into());
        let upload_start =
            Self::hex_to_hsla(&btop_config.upload_start).unwrap_or(gpui::rgb(0x00F59B).into());
        let upload_mid =
            Self::hex_to_hsla(&btop_config.upload_mid).unwrap_or(gpui::rgb(0xFF66F6).into());
        let upload_end =
            Self::hex_to_hsla(&btop_config.upload_end).unwrap_or(gpui::rgb(0xFF3366).into());

        let main_bg_picker = cx.new(|cx| ColorPickerState::new(window, cx).default_value(main_bg));
        let main_fg_picker = cx.new(|cx| ColorPickerState::new(window, cx).default_value(main_fg));
        let title_picker = cx.new(|cx| ColorPickerState::new(window, cx).default_value(title));
        let hi_fg_picker = cx.new(|cx| ColorPickerState::new(window, cx).default_value(hi_fg));
        let selected_bg_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(selected_bg));
        let selected_fg_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(selected_fg));
        let inactive_fg_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(inactive_fg));
        let proc_misc_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(proc_misc));
        let cpu_box_picker = cx.new(|cx| ColorPickerState::new(window, cx).default_value(cpu_box));
        let mem_box_picker = cx.new(|cx| ColorPickerState::new(window, cx).default_value(mem_box));
        let net_box_picker = cx.new(|cx| ColorPickerState::new(window, cx).default_value(net_box));
        let proc_box_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(proc_box));
        let div_line_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(div_line));
        let temp_start_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(temp_start));
        let temp_mid_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(temp_mid));
        let temp_end_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(temp_end));
        let cpu_start_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(cpu_start));
        let cpu_mid_picker = cx.new(|cx| ColorPickerState::new(window, cx).default_value(cpu_mid));
        let cpu_end_picker = cx.new(|cx| ColorPickerState::new(window, cx).default_value(cpu_end));
        let free_start_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(free_start));
        let free_mid_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(free_mid));
        let free_end_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(free_end));
        let cached_start_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(cached_start));
        let cached_mid_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(cached_mid));
        let cached_end_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(cached_end));
        let available_start_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(available_start));
        let available_mid_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(available_mid));
        let available_end_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(available_end));
        let used_start_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(used_start));
        let used_mid_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(used_mid));
        let used_end_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(used_end));
        let download_start_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(download_start));
        let download_mid_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(download_mid));
        let download_end_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(download_end));
        let upload_start_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(upload_start));
        let upload_mid_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(upload_mid));
        let upload_end_picker =
            cx.new(|cx| ColorPickerState::new(window, cx).default_value(upload_end));

        let mut tab = Self {
            theme_name,
            theme_data,
            main_bg_picker,
            main_fg_picker,
            title_picker,
            hi_fg_picker,
            selected_bg_picker,
            selected_fg_picker,
            inactive_fg_picker,
            proc_misc_picker,
            cpu_box_picker,
            mem_box_picker,
            net_box_picker,
            proc_box_picker,
            div_line_picker,
            temp_start_picker,
            temp_mid_picker,
            temp_end_picker,
            cpu_start_picker,
            cpu_mid_picker,
            cpu_end_picker,
            free_start_picker,
            free_mid_picker,
            free_end_picker,
            cached_start_picker,
            cached_mid_picker,
            cached_end_picker,
            available_start_picker,
            available_mid_picker,
            available_end_picker,
            used_start_picker,
            used_mid_picker,
            used_end_picker,
            download_start_picker,
            download_mid_picker,
            download_end_picker,
            upload_start_picker,
            upload_mid_picker,
            upload_end_picker,
            is_saving: false,
            error_message: None,
        };

        // Subscribe to all color picker changes
        tab.subscribe_to_pickers(window, cx);

        tab
    }

    /// Subscribe to all color picker change events
    fn subscribe_to_pickers(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Main colors
        cx.subscribe_in(
            &self.main_bg_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_btop_config(|config| config.main_bg = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &self.main_fg_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_btop_config(|config| config.main_fg = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &self.title_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_btop_config(|config| config.title = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &self.hi_fg_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_btop_config(|config| config.hi_fg = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        // Selection colors
        cx.subscribe_in(
            &self.selected_bg_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_btop_config(|config| config.selected_bg = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &self.selected_fg_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_btop_config(|config| config.selected_fg = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        // Status colors
        cx.subscribe_in(
            &self.inactive_fg_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_btop_config(|config| config.inactive_fg = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &self.proc_misc_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_btop_config(|config| config.proc_misc = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        // Box colors
        cx.subscribe_in(
            &self.cpu_box_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_btop_config(|config| config.cpu_box = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &self.mem_box_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_btop_config(|config| config.mem_box = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &self.net_box_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_btop_config(|config| config.net_box = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &self.proc_box_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_btop_config(|config| config.proc_box = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &self.div_line_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_btop_config(|config| config.div_line = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        // Temperature gradient
        cx.subscribe_in(
            &self.temp_start_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_btop_config(|config| config.temp_start = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &self.temp_mid_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_btop_config(|config| config.temp_mid = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &self.temp_end_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_btop_config(|config| config.temp_end = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        // CPU gradient
        cx.subscribe_in(
            &self.cpu_start_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_btop_config(|config| config.cpu_start = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &self.cpu_mid_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_btop_config(|config| config.cpu_mid = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &self.cpu_end_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_btop_config(|config| config.cpu_end = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        // Free gradient
        cx.subscribe_in(
            &self.free_start_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_btop_config(|config| config.free_start = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &self.free_mid_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_btop_config(|config| config.free_mid = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &self.free_end_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_btop_config(|config| config.free_end = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        // Cached gradient
        cx.subscribe_in(
            &self.cached_start_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_btop_config(|config| config.cached_start = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &self.cached_mid_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_btop_config(|config| config.cached_mid = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &self.cached_end_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_btop_config(|config| config.cached_end = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        // Available gradient
        cx.subscribe_in(
            &self.available_start_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_btop_config(|config| config.available_start = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &self.available_mid_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_btop_config(|config| config.available_mid = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &self.available_end_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_btop_config(|config| config.available_end = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        // Used gradient
        cx.subscribe_in(
            &self.used_start_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_btop_config(|config| config.used_start = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &self.used_mid_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_btop_config(|config| config.used_mid = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &self.used_end_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_btop_config(|config| config.used_end = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        // Download gradient
        cx.subscribe_in(
            &self.download_start_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_btop_config(|config| config.download_start = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &self.download_mid_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_btop_config(|config| config.download_mid = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &self.download_end_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_btop_config(|config| config.download_end = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        // Upload gradient
        cx.subscribe_in(
            &self.upload_start_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_btop_config(|config| config.upload_start = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &self.upload_mid_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_btop_config(|config| config.upload_mid = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &self.upload_end_picker,
            window,
            |this, _picker, event: &ColorPickerEvent, window, cx| {
                if let ColorPickerEvent::Change(Some(color)) = event {
                    let hex = color.to_hex();
                    this.update_btop_config(|config| config.upload_end = hex);
                    this.save(window, cx);
                }
            },
        )
        .detach();
    }

    /// Convert hex color string (#RRGGBB) to Hsla
    fn hex_to_hsla(hex: &str) -> Option<Hsla> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }

        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

        Some(gpui::rgb(u32::from_be_bytes([0, r, g, b])).into())
    }

    /// Update the btop config within theme_data
    fn update_btop_config<F>(&mut self, updater: F)
    where
        F: FnOnce(&mut BtopConfig),
    {
        let mut config = self.theme_data.apps.btop.clone().unwrap_or_default();
        updater(&mut config);
        self.theme_data.apps.btop = Some(config);
    }

    /// Get the current theme data
    pub fn theme_data(&self) -> &EditingTheme {
        &self.theme_data
    }

    /// Save the theme data and update btop.theme
    fn save(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.is_saving {
            return;
        }

        // Validate theme name
        if self.theme_name.is_empty() {
            self.error_message = Some("Theme name cannot be empty".to_string());
            cx.notify();
            return;
        }

        self.is_saving = true;
        self.error_message = None;
        cx.notify();

        // Save theme data
        match save_theme_data(&self.theme_name, &self.theme_data) {
            Ok(()) => {
                // Also update the btop.theme file
                if let Some(ref btop_config) = self.theme_data.apps.btop
                    && let Err(e) = update_btop_theme(&self.theme_name, btop_config)
                {
                    self.error_message = Some(format!("Failed to update btop.theme: {}", e));
                }
                self.is_saving = false;
            }
            Err(e) => {
                self.is_saving = false;
                self.error_message = Some(e);
            }
        }

        cx.notify();
    }
}

impl Render for BtopTab {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        tab_container().child(
            v_flex()
                .gap_6()
                // Main Colors Section
                .child(
                    form_section()
                        .gap_4()
                        .child(
                            div()
                                .text_lg()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child("Main Colors"),
                        )
                        .child(
                            h_flex()
                                .gap_6()
                                .flex_wrap()
                                .child(color_picker_with_clipboard(
                                    "btop-main-bg",
                                    "Background",
                                    &self.main_bg_picker,
                                ))
                                .child(color_picker_with_clipboard(
                                    "btop-main-fg",
                                    "Foreground",
                                    &self.main_fg_picker,
                                ))
                                .child(color_picker_with_clipboard(
                                    "btop-title",
                                    "Title",
                                    &self.title_picker,
                                ))
                                .child(color_picker_with_clipboard(
                                    "btop-hi-fg",
                                    "Highlight",
                                    &self.hi_fg_picker,
                                )),
                        ),
                )
                .child(Divider::horizontal())
                // Selection Colors Section
                .child(
                    form_section()
                        .gap_4()
                        .child(
                            div()
                                .text_lg()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child("Selection Colors"),
                        )
                        .child(
                            h_flex()
                                .gap_6()
                                .flex_wrap()
                                .child(color_picker_with_clipboard(
                                    "btop-selected-bg",
                                    "Selected Background",
                                    &self.selected_bg_picker,
                                ))
                                .child(color_picker_with_clipboard(
                                    "btop-selected-fg",
                                    "Selected Foreground",
                                    &self.selected_fg_picker,
                                )),
                        ),
                )
                .child(Divider::horizontal())
                // Status Colors Section
                .child(
                    form_section()
                        .gap_4()
                        .child(
                            div()
                                .text_lg()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child("Status Colors"),
                        )
                        .child(
                            h_flex()
                                .gap_6()
                                .flex_wrap()
                                .child(color_picker_with_clipboard(
                                    "btop-inactive",
                                    "Inactive",
                                    &self.inactive_fg_picker,
                                ))
                                .child(color_picker_with_clipboard(
                                    "btop-proc-misc",
                                    "Proc Misc",
                                    &self.proc_misc_picker,
                                )),
                        ),
                )
                .child(Divider::horizontal())
                // Box Colors Section
                .child(
                    form_section()
                        .gap_4()
                        .child(
                            div()
                                .text_lg()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child("Box Outline Colors"),
                        )
                        .child(
                            h_flex()
                                .gap_6()
                                .flex_wrap()
                                .child(color_picker_with_clipboard(
                                    "btop-cpu-box",
                                    "CPU Box",
                                    &self.cpu_box_picker,
                                ))
                                .child(color_picker_with_clipboard(
                                    "btop-mem-box",
                                    "Memory Box",
                                    &self.mem_box_picker,
                                ))
                                .child(color_picker_with_clipboard(
                                    "btop-net-box",
                                    "Net Box",
                                    &self.net_box_picker,
                                ))
                                .child(color_picker_with_clipboard(
                                    "btop-proc-box",
                                    "Proc Box",
                                    &self.proc_box_picker,
                                ))
                                .child(color_picker_with_clipboard(
                                    "btop-div-line",
                                    "Divider Line",
                                    &self.div_line_picker,
                                )),
                        ),
                )
                .child(Divider::horizontal())
                // Temperature Gradient Section
                .child(
                    form_section()
                        .gap_4()
                        .child(
                            div()
                                .text_lg()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child("Temperature Graph"),
                        )
                        .child(
                            h_flex()
                                .gap_6()
                                .flex_wrap()
                                .child(color_picker_with_clipboard(
                                    "btop-temp-start",
                                    "Start",
                                    &self.temp_start_picker,
                                ))
                                .child(color_picker_with_clipboard(
                                    "btop-temp-mid",
                                    "Mid",
                                    &self.temp_mid_picker,
                                ))
                                .child(color_picker_with_clipboard(
                                    "btop-temp-end",
                                    "End",
                                    &self.temp_end_picker,
                                )),
                        ),
                )
                .child(Divider::horizontal())
                // CPU Gradient Section
                .child(
                    form_section()
                        .gap_4()
                        .child(
                            div()
                                .text_lg()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child("CPU Graph"),
                        )
                        .child(
                            h_flex()
                                .gap_6()
                                .flex_wrap()
                                .child(color_picker_with_clipboard(
                                    "btop-cpu-start",
                                    "Start",
                                    &self.cpu_start_picker,
                                ))
                                .child(color_picker_with_clipboard(
                                    "btop-cpu-mid",
                                    "Mid",
                                    &self.cpu_mid_picker,
                                ))
                                .child(color_picker_with_clipboard(
                                    "btop-cpu-end",
                                    "End",
                                    &self.cpu_end_picker,
                                )),
                        ),
                )
                .child(Divider::horizontal())
                // Free Meter Gradient Section
                .child(
                    form_section()
                        .gap_4()
                        .child(
                            div()
                                .text_lg()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child("Free Meter"),
                        )
                        .child(
                            h_flex()
                                .gap_6()
                                .flex_wrap()
                                .child(color_picker_with_clipboard(
                                    "btop-free-start",
                                    "Start",
                                    &self.free_start_picker,
                                ))
                                .child(color_picker_with_clipboard(
                                    "btop-free-mid",
                                    "Mid",
                                    &self.free_mid_picker,
                                ))
                                .child(color_picker_with_clipboard(
                                    "btop-free-end",
                                    "End",
                                    &self.free_end_picker,
                                )),
                        ),
                )
                .child(Divider::horizontal())
                // Cached Meter Gradient Section
                .child(
                    form_section()
                        .gap_4()
                        .child(
                            div()
                                .text_lg()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child("Cached Meter"),
                        )
                        .child(
                            h_flex()
                                .gap_6()
                                .flex_wrap()
                                .child(color_picker_with_clipboard(
                                    "btop-cached-start",
                                    "Start",
                                    &self.cached_start_picker,
                                ))
                                .child(color_picker_with_clipboard(
                                    "btop-cached-mid",
                                    "Mid",
                                    &self.cached_mid_picker,
                                ))
                                .child(color_picker_with_clipboard(
                                    "btop-cached-end",
                                    "End",
                                    &self.cached_end_picker,
                                )),
                        ),
                )
                .child(Divider::horizontal())
                // Available Meter Gradient Section
                .child(
                    form_section()
                        .gap_4()
                        .child(
                            div()
                                .text_lg()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child("Available Meter"),
                        )
                        .child(
                            h_flex()
                                .gap_6()
                                .flex_wrap()
                                .child(color_picker_with_clipboard(
                                    "btop-available-start",
                                    "Start",
                                    &self.available_start_picker,
                                ))
                                .child(color_picker_with_clipboard(
                                    "btop-available-mid",
                                    "Mid",
                                    &self.available_mid_picker,
                                ))
                                .child(color_picker_with_clipboard(
                                    "btop-available-end",
                                    "End",
                                    &self.available_end_picker,
                                )),
                        ),
                )
                .child(Divider::horizontal())
                // Used Meter Gradient Section
                .child(
                    form_section()
                        .gap_4()
                        .child(
                            div()
                                .text_lg()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child("Used Meter"),
                        )
                        .child(
                            h_flex()
                                .gap_6()
                                .flex_wrap()
                                .child(color_picker_with_clipboard(
                                    "btop-used-start",
                                    "Start",
                                    &self.used_start_picker,
                                ))
                                .child(color_picker_with_clipboard(
                                    "btop-used-mid",
                                    "Mid",
                                    &self.used_mid_picker,
                                ))
                                .child(color_picker_with_clipboard(
                                    "btop-used-end",
                                    "End",
                                    &self.used_end_picker,
                                )),
                        ),
                )
                .child(Divider::horizontal())
                // Download Gradient Section
                .child(
                    form_section()
                        .gap_4()
                        .child(
                            div()
                                .text_lg()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child("Download Graph"),
                        )
                        .child(
                            h_flex()
                                .gap_6()
                                .flex_wrap()
                                .child(color_picker_with_clipboard(
                                    "btop-download-start",
                                    "Start",
                                    &self.download_start_picker,
                                ))
                                .child(color_picker_with_clipboard(
                                    "btop-download-mid",
                                    "Mid",
                                    &self.download_mid_picker,
                                ))
                                .child(color_picker_with_clipboard(
                                    "btop-download-end",
                                    "End",
                                    &self.download_end_picker,
                                )),
                        ),
                )
                .child(Divider::horizontal())
                // Upload Gradient Section
                .child(
                    form_section()
                        .gap_4()
                        .child(
                            div()
                                .text_lg()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child("Upload Graph"),
                        )
                        .child(
                            h_flex()
                                .gap_6()
                                .flex_wrap()
                                .child(color_picker_with_clipboard(
                                    "btop-upload-start",
                                    "Start",
                                    &self.upload_start_picker,
                                ))
                                .child(color_picker_with_clipboard(
                                    "btop-upload-mid",
                                    "Mid",
                                    &self.upload_mid_picker,
                                ))
                                .child(color_picker_with_clipboard(
                                    "btop-upload-end",
                                    "End",
                                    &self.upload_end_picker,
                                )),
                        ),
                ),
        )
    }
}
