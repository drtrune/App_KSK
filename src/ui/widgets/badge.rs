use eframe::egui;
use crate::theme::Theme;

pub fn ui_score_badge(ui: &mut egui::Ui, score: i32, is_dyn: bool) {
    let (fg, bg) = Theme::score_colors(score);
    let txt = if is_dyn {
        format!("Loại {} *", score)
    } else {
        format!("Loại {}", score)
    };
    egui::Frame::none()
        .fill(bg)
        .rounding(4.0)
        .inner_margin(egui::Margin::symmetric(6.0, 2.0))
        .show(ui, |ui| {
            ui.colored_label(fg, egui::RichText::new(txt).strong().size(11.5));
        });
}

pub fn score_badge(ui: &mut egui::Ui, score: i32) {
    ui_score_badge(ui, score, false);
}

pub fn ui_status_badge(ui: &mut egui::Ui, locked: bool) {
    let (txt, fg, bg) = if locked {
        ("Đã Khóa", egui::Color32::from_rgb(180, 83, 9), egui::Color32::from_rgb(254, 243, 199))
    } else {
        ("Đang Soạn", egui::Color32::from_rgb(22, 101, 52), egui::Color32::from_rgb(220, 252, 231))
    };
    egui::Frame::none()
        .fill(bg)
        .rounding(4.0)
        .inner_margin(egui::Margin::symmetric(6.0, 2.0))
        .show(ui, |ui| {
            ui.colored_label(fg, egui::RichText::new(txt).strong().size(11.0));
        });
}

pub fn status_badge(ui: &mut egui::Ui, locked: bool) {
    ui_status_badge(ui, locked);
}
