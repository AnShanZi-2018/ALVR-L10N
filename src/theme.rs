//! ALVR-like visual styling: colours, spacing, rounded corners, light/dark.

use eframe::egui;

pub const ACCENT: egui::Color32 = egui::Color32::from_rgb(0x3d, 0x6f, 0xd8);
const BG_DARK: egui::Color32 = egui::Color32::from_rgb(0x1b, 0x1b, 0x1d);
const PANEL_DARK: egui::Color32 = egui::Color32::from_rgb(0x23, 0x23, 0x26);
const WIDGET_DARK: egui::Color32 = egui::Color32::from_rgb(0x2e, 0x2e, 0x32);
const WIDGET_HOVER: egui::Color32 = egui::Color32::from_rgb(0x3a, 0x3a, 0x40);
const TEXT_DARK: egui::Color32 = egui::Color32::from_rgb(0xe4, 0xe4, 0xe8);

const BG_LIGHT: egui::Color32 = egui::Color32::from_rgb(0xf4, 0xf4, 0xf6);
const PANEL_LIGHT: egui::Color32 = egui::Color32::from_rgb(0xff, 0xff, 0xff);
const WIDGET_LIGHT: egui::Color32 = egui::Color32::from_rgb(0xe8, 0xe8, 0xec);
const WIDGET_HOVER_LIGHT: egui::Color32 = egui::Color32::from_rgb(0xda, 0xda, 0xe0);

/// Panel colour the current theme should produce (used to detect overrides).
pub fn expected_panel(dark: bool) -> egui::Color32 {
    if dark { BG_DARK } else { BG_LIGHT }
}

pub fn apply(ctx: &egui::Context, dark: bool) {
    ctx.set_theme(if dark {
        egui::ThemePreference::Dark
    } else {
        egui::ThemePreference::Light
    });
    let mut visuals = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };

    let (bg, panel, widget, hover, text) = if dark {
        (BG_DARK, PANEL_DARK, WIDGET_DARK, WIDGET_HOVER, TEXT_DARK)
    } else {
        (BG_LIGHT, PANEL_LIGHT, WIDGET_LIGHT, WIDGET_HOVER_LIGHT, egui::Color32::from_rgb(0x1c, 0x1c, 0x1e))
    };

    visuals.panel_fill = bg;
    visuals.window_fill = panel;
    visuals.faint_bg_color = panel;
    visuals.extreme_bg_color = if dark {
        egui::Color32::from_rgb(0x15, 0x15, 0x17)
    } else {
        egui::Color32::from_rgb(0xee, 0xee, 0xf1)
    };
    visuals.override_text_color = Some(text);
    visuals.selection.bg_fill = ACCENT;
    visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
    visuals.hyperlink_color = ACCENT;

    let radius = egui::CornerRadius::same(6);
    for widget in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        widget.corner_radius = radius;
    }

    visuals.widgets.noninteractive.bg_fill = panel;
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, text);
    visuals.widgets.inactive.bg_fill = widget;
    visuals.widgets.inactive.weak_bg_fill = widget;
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, text);
    visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
    visuals.widgets.hovered.bg_fill = hover;
    visuals.widgets.hovered.weak_bg_fill = hover;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, ACCENT);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, text);
    visuals.widgets.active.bg_fill = ACCENT;
    visuals.widgets.active.weak_bg_fill = ACCENT;
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
    visuals.widgets.open.bg_fill = hover;
    visuals.widgets.open.weak_bg_fill = hover;

    if let Some(radius) = Some(8u8) {
        visuals.window_corner_radius = egui::CornerRadius::same(radius);
        visuals.menu_corner_radius = egui::CornerRadius::same(radius);
    }

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(14.0, 8.0);
    style.spacing.interact_size.y = 30.0;
    ctx.set_style(style);
}

/// Native window decoration: rounded corners + dark title bar (Windows 11+).
#[cfg(windows)]
pub fn apply_window_decoration(title: &str, dark: bool) {
    use std::os::windows::ffi::OsStrExt;

    #[link(name = "user32")]
    extern "system" {
        fn FindWindowW(class: *const u16, window: *const u16) -> isize;
    }
    #[link(name = "dwmapi")]
    extern "system" {
        fn DwmSetWindowAttribute(hwnd: isize, attribute: u32, value: *const i32, size: u32) -> i32;
    }

    const DWMWA_USE_IMMERSIVE_DARK_MODE: u32 = 20;
    const DWMWA_WINDOW_CORNER_PREFERENCE: u32 = 33;

    let wide: Vec<u16> = std::ffi::OsStr::new(title)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let hwnd = unsafe { FindWindowW(std::ptr::null(), wide.as_ptr()) };
    if hwnd == 0 {
        return;
    }

    let dark_flag: i32 = if dark { 1 } else { 0 };
    unsafe {
        DwmSetWindowAttribute(hwnd, DWMWA_USE_IMMERSIVE_DARK_MODE, &dark_flag, 4);
    }
    let round: i32 = 2; // DWMWCP_ROUND
    unsafe {
        DwmSetWindowAttribute(hwnd, DWMWA_WINDOW_CORNER_PREFERENCE, &round, 4);
    }
}

#[cfg(not(windows))]
pub fn apply_window_decoration(_title: &str, _dark: bool) {}
