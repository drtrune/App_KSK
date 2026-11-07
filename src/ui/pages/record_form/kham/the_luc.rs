use egui::{Color32, RichText, Ui};
use crate::domain::update_global_health_score;
use crate::models::record::KskRecordPayload;
use crate::ui::widgets::badge::ui_score_badge;

pub struct TheLucView;

impl TheLucView {
    pub fn show(ui: &mut Ui, form: &mut KskRecordPayload, is_locked: bool) {
        ui.add_enabled_ui(!is_locked, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add_space(4.0);
                    ui.heading(RichText::new("KHÁM THỂ LỰC").size(15.0).strong().color(Color32::from_rgb(14, 165, 233)));
                    ui.add_space(10.0);

                    // Grid nhập thông số thể lực
                    egui::Grid::new("grid_the_luc")
                        .spacing([24.0, 14.0])
                        .min_col_width(120.0)
                        .show(ui, |ui| {
                            // Chiều cao
                            ui.label(RichText::new("Chiều cao (cm) (*):").strong().size(12.5));
                            let h_resp = ui.add(egui::TextEdit::singleline(&mut form.chieu_cao).desired_width(120.0));
                            if h_resp.changed() {
                                Self::calc_bmi(form);
                                update_global_health_score(form);
                            }

                            // Cân nặng
                            ui.label(RichText::new("Cân nặng (kg) (*):").strong().size(12.5));
                            let w_resp = ui.add(egui::TextEdit::singleline(&mut form.can_nang).desired_width(120.0));
                            if w_resp.changed() {
                                Self::calc_bmi(form);
                                update_global_health_score(form);
                            }
                            ui.end_row();

                            // BMI (tự động)
                            ui.label(RichText::new("Chỉ số BMI (kg/m²):").strong().size(12.5));
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(&mut form.chi_so_bmi).desired_width(120.0).interactive(false));
                                let bmi_val: f64 = form.chi_so_bmi.parse().unwrap_or(0.0);
                                let (status, color) = if bmi_val < 18.5 {
                                    ("Gầy", Color32::from_rgb(217, 119, 6))
                                } else if bmi_val < 25.0 {
                                    ("Bình thường", Color32::from_rgb(16, 185, 129))
                                } else if bmi_val < 30.0 {
                                    ("Tiền béo phì", Color32::from_rgb(217, 119, 6))
                                } else {
                                    ("Béo phì", Color32::from_rgb(220, 38, 38))
                                };
                                ui.colored_label(color, RichText::new(status).strong().size(12.0));
                            });

                            // Mạch
                            ui.label(RichText::new("Mạch (lần/phút) (*):").strong().size(12.5));
                            let p_resp = ui.add(egui::TextEdit::singleline(&mut form.mach).desired_width(120.0).hint_text("VD: 75"));
                            if p_resp.changed() {
                                update_global_health_score(form);
                            }
                            ui.end_row();

                            // Huyết áp
                            ui.label(RichText::new("Huyết áp (mmHg) (*):").strong().size(12.5));
                            let bp_resp = ui.add(egui::TextEdit::singleline(&mut form.huyet_ap).desired_width(120.0).hint_text("VD: 120/80"));
                            if bp_resp.changed() {
                                update_global_health_score(form);
                            }

                            // Phân loại thể lực (1 - 5)
                            ui.label(RichText::new("Phân loại thể lực (*):").strong().size(12.5));
                            ui.horizontal(|ui| {
                                for i in 1..=5 {
                                    if ui.radio_value(&mut form.kham_the_luc_pl, i, format!("Loại {}", i)).changed() {
                                        update_global_health_score(form);
                                    }
                                }
                                ui_score_badge(ui, form.kham_the_luc_pl, false);
                            });
                            ui.end_row();
                        });

                    ui.add_space(20.0);
                });
        });
    }

    fn calc_bmi(form: &mut KskRecordPayload) {
        let h_cm: f64 = form.chieu_cao.trim().parse().unwrap_or(0.0);
        let w_kg: f64 = form.can_nang.trim().parse().unwrap_or(0.0);
        if h_cm > 50.0 && w_kg > 10.0 {
            let h_m = h_cm / 100.0;
            let bmi = w_kg / (h_m * h_m);
            form.chi_so_bmi = format!("{:.2}", bmi);
        }
    }
}
