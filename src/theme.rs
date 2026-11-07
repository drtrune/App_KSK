use eframe::egui;

/// Theme tokens and styling for minimalist, professional medical UI
pub struct Theme;

impl Theme {
    // Primary brand & accent colors
    pub const PRIMARY: egui::Color32 = egui::Color32::from_rgb(14, 165, 233);      // Sky 500
    pub const PRIMARY_DARK: egui::Color32 = egui::Color32::from_rgb(2, 132, 199); // Sky 600
    pub const PRIMARY_LIGHT: egui::Color32 = egui::Color32::from_rgb(224, 242, 254); // Sky 100

    // Neutral backgrounds & surfaces
    pub const BG_LIGHT: egui::Color32 = egui::Color32::from_rgb(248, 250, 252);     // Slate 50
    pub const BG_DARK: egui::Color32 = egui::Color32::from_rgb(15, 23, 42);         // Slate 900
    pub const CARD_LIGHT: egui::Color32 = egui::Color32::from_rgb(255, 255, 255);   // White
    pub const CARD_DARK: egui::Color32 = egui::Color32::from_rgb(30, 41, 59);       // Slate 800

    // Borders & dividers
    pub const BORDER_LIGHT: egui::Color32 = egui::Color32::from_rgb(226, 232, 240); // Slate 200
    pub const BORDER_DARK: egui::Color32 = egui::Color32::from_rgb(51, 65, 85);     // Slate 700

    // Text colors
    pub const TEXT_MAIN: egui::Color32 = egui::Color32::from_rgb(15, 23, 42);       // Slate 900
    pub const TEXT_MUTED: egui::Color32 = egui::Color32::from_rgb(100, 116, 139);   // Slate 500
    pub const TEXT_LIGHT_MUTED: egui::Color32 = egui::Color32::from_rgb(148, 163, 184);

    // Status colors
    pub const SUCCESS: egui::Color32 = egui::Color32::from_rgb(16, 185, 129);      // Emerald 500
    pub const SUCCESS_BG: egui::Color32 = egui::Color32::from_rgb(236, 253, 245);   // Emerald 50
    pub const WARNING: egui::Color32 = egui::Color32::from_rgb(245, 158, 11);       // Amber 500
    pub const WARNING_BG: egui::Color32 = egui::Color32::from_rgb(254, 243, 199);   // Amber 50
    pub const DANGER: egui::Color32 = egui::Color32::from_rgb(239, 68, 68);         // Red 500
    pub const DANGER_BG: egui::Color32 = egui::Color32::from_rgb(254, 242, 242);    // Red 50

    /// Returns (fg, bg) color tuple for standard health classification grades (1 to 5)
    pub fn score_colors(score: i32) -> (egui::Color32, egui::Color32) {
        match score {
            1 => (egui::Color32::from_rgb(22, 101, 52), egui::Color32::from_rgb(220, 252, 231)),  // Green / Loại 1
            2 => (egui::Color32::from_rgb(30, 64, 175), egui::Color32::from_rgb(219, 234, 254)), // Blue / Loại 2
            3 => (egui::Color32::from_rgb(146, 64, 14), egui::Color32::from_rgb(254, 243, 199)), // Yellow / Loại 3
            4 => (egui::Color32::from_rgb(154, 52, 18), egui::Color32::from_rgb(255, 237, 213)), // Orange / Loại 4
            _ => (egui::Color32::from_rgb(153, 27, 27), egui::Color32::from_rgb(254, 226, 226)), // Red / Loại 5
        }
    }
}

pub fn apply_theme(ctx: &egui::Context, dark: bool) {
    let mut visuals = if dark {
        egui::Visuals::dark()
    } else {
        let mut light = egui::Visuals::light();
        light.window_fill = Theme::CARD_LIGHT;
        light.panel_fill = Theme::BG_LIGHT;
        light
    };
    visuals.window_rounding = 6.0.into();
    ctx.set_visuals(visuals);
}
