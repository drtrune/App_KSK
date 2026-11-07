#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod auth;
pub mod config;
pub mod db;
pub mod domain;
pub mod models;
pub mod theme;
pub mod ui;
pub mod app;

use app::KskApp;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Hệ Thống Khám Sức Khỏe Định Kỳ - QĐ 3176 & QĐ 1613/BYT")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([1024.0, 680.0]),
        ..Default::default()
    };

    eframe::run_native(
        "ksk_app_3176",
        native_options,
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            Box::new(KskApp::new(cc))
        }),
    )
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Prefer system Segoe UI / Arial for crisp Vietnamese font rendering on Windows
    #[cfg(target_os = "windows")]
    {
        if let Ok(font_data) = std::fs::read("C:\\Windows\\Fonts\\segoeui.ttf") {
            fonts.font_data.insert("segoe_ui".to_owned(), egui::FontData::from_owned(font_data));
            fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "segoe_ui".to_owned());
        }
    }

    ctx.set_fonts(fonts);
}
