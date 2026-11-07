use egui::{Color32, RichText, Ui};
use crate::models::qd1613::{QdClassification, QdDisease, QdSpecialty};
use crate::ui::widgets::badge::ui_score_badge;

pub struct AdminQd1613Page;

pub enum Qd1613Action {
    None,
    SaveClassification(QdClassification),
    ResetToDefaults,
}

pub struct QdClassificationEditState {
    pub is_open: bool,
    pub id: String,
    pub disease_id: String,
    pub name: String,
    pub score: i32,
    pub ref_rule: String,
}

impl Default for QdClassificationEditState {
    fn default() -> Self {
        Self {
            is_open: false,
            id: "".into(),
            disease_id: "".into(),
            name: "".into(),
            score: 1,
            ref_rule: "".into(),
        }
    }
}

impl AdminQd1613Page {
    pub fn show(
        ui: &mut Ui,
        specialties: &[QdSpecialty],
        diseases: &[QdDisease],
        classifications: &[QdClassification],
        selected_spec: &mut String,
        search_query: &mut String,
        edit_state: &mut QdClassificationEditState,
        dark_mode: bool,
    ) -> Qd1613Action {
        let mut action = Qd1613Action::None;

        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.heading(RichText::new("TIÊU CHUẨN PHÂN LOẠI SỨC KHỎE THEO QĐ 1613/BYT").size(18.0).strong());
                    ui.label(RichText::new("Danh mục phân cấp: Chuyên khoa › Danh mục bệnh tật › Tiêu chuẩn phân loại (Loại 1 đến 5).")
                        .size(12.0)
                        .color(Color32::from_rgb(100, 116, 139)));
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(RichText::new("🔄 Khôi phục mặc định QĐ 1613").size(12.0).color(Color32::from_rgb(239, 68, 68)))
                        .on_hover_text("Khôi phục toàn bộ bảng mã và tiêu chuẩn QĐ 1613 về thiết lập gốc")
                        .clicked()
                    {
                        action = Qd1613Action::ResetToDefaults;
                    }
                });
            });

            ui.add_space(10.0);

            // Specialty selector
            ui.horizontal_wrapped(|ui| {
                for sp in specialties {
                    let is_sel = *selected_spec == sp.id;
                    let text_color = if is_sel {
                        Color32::from_rgb(14, 165, 233)
                    } else if dark_mode {
                        Color32::from_rgb(203, 213, 225)
                    } else {
                        Color32::from_rgb(71, 85, 105)
                    };
                    let btn_bg = if is_sel {
                        if dark_mode { Color32::from_rgb(30, 41, 59) } else { Color32::from_rgb(224, 242, 254) }
                    } else {
                        Color32::TRANSPARENT
                    };

                    let btn = egui::Button::new(RichText::new(&sp.name).size(12.0).color(text_color).strong())
                        .fill(btn_bg)
                        .rounding(4.0);

                    if ui.add(btn).clicked() {
                        *selected_spec = sp.id.clone();
                    }
                }
            });

            ui.add_space(8.0);

            // Search filter
            ui.horizontal(|ui| {
                ui.label(RichText::new("Tìm bệnh / phân loại:").size(12.0));
                ui.add(egui::TextEdit::singleline(search_query).hint_text("Tìm theo tên bệnh tật hoặc tiêu chuẩn...").desired_width(320.0));
                if !search_query.is_empty() && ui.button("✖").clicked() {
                    search_query.clear();
                }
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(6.0);

            let q = search_query.trim().to_lowercase();
            let spec_diseases: Vec<&QdDisease> = diseases.iter().filter(|d| d.spec_id == *selected_spec).collect();

            // Tree / List of diseases & classifications
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if spec_diseases.is_empty() {
                        ui.label(RichText::new("Chưa có danh mục bệnh nào trong chuyên khoa này.").color(Color32::GRAY));
                    } else {
                        for dis in &spec_diseases {
                            let clss: Vec<&QdClassification> = classifications
                                .iter()
                                .filter(|c| c.disease_id == dis.id)
                                .filter(|c| q.is_empty() || dis.name.to_lowercase().contains(&q) || c.name.to_lowercase().contains(&q))
                                .collect();

                            if clss.is_empty() && !q.is_empty() {
                                continue;
                            }

                            ui.push_id(&dis.id, |ui| {
                                egui::Frame::none()
                                    .fill(if dark_mode { Color32::from_rgb(30, 41, 59) } else { Color32::from_rgb(248, 250, 252) })
                                    .stroke(egui::Stroke::new(1.0_f32, if dark_mode { Color32::from_rgb(51, 65, 85) } else { Color32::from_rgb(226, 232, 240) }))
                                    .rounding(6.0)
                                    .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new("📁").size(13.0));
                                            ui.label(RichText::new(&dis.name).strong().size(13.0));
                                            ui.label(RichText::new(format!("({})", dis.id)).monospace().size(11.0).color(Color32::from_rgb(148, 163, 184)));
                                        });

                                        ui.add_space(4.0);

                                        for c in &clss {
                                            ui.push_id(&c.id, |ui| {
                                                ui.horizontal(|ui| {
                                                    ui_score_badge(ui, c.score.unwrap_or(1), c.ref_disease_id.is_some());
                                                    ui.label(RichText::new(&c.name).size(12.0));
                                                    if let Some(ref r) = c.ref_rule {
                                                        ui.label(RichText::new(format!("↳ {}", r)).size(10.5).color(Color32::from_rgb(180, 83, 9)).italics());
                                                    }

                                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                        if ui.button(RichText::new("Sửa").size(11.0).color(Color32::from_rgb(14, 165, 233))).clicked() {
                                                            edit_state.is_open = true;
                                                            edit_state.id = c.id.clone();
                                                            edit_state.disease_id = c.disease_id.clone();
                                                            edit_state.name = c.name.clone();
                                                            edit_state.score = c.score.unwrap_or(1);
                                                            edit_state.ref_rule = c.ref_rule.clone().unwrap_or_default();
                                                        }
                                                    });
                                                });
                                            });
                                            ui.add_space(2.0);
                                        }
                                    });
                                ui.add_space(6.0);
                            });
                        }
                    }
                });
        });

        // Edit Modal
        if edit_state.is_open {
            egui::Window::new("Chỉnh sửa tiêu chuẩn phân loại")
                .collapsible(false)
                .resizable(false)
                .min_width(400.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new(format!("Mã: {}", edit_state.id)).monospace().size(11.5).color(Color32::from_rgb(100, 116, 139)));
                        ui.add_space(8.0);

                        ui.label(RichText::new("Tên tiêu chuẩn lâm sàng (*):").strong().size(12.0));
                        ui.add(egui::TextEdit::multiline(&mut edit_state.name).desired_rows(3).desired_width(380.0));

                        ui.add_space(6.0);
                        ui.label(RichText::new("Phân loại điểm (1 - 5):").strong().size(12.0));
                        ui.horizontal(|ui| {
                            for n in 1..=5 {
                                ui.radio_value(&mut edit_state.score, n, format!("Loại {}", n));
                            }
                            ui_score_badge(ui, edit_state.score, false);
                        });

                        ui.add_space(6.0);
                        ui.label(RichText::new("Quy tắc phụ (nếu có):").size(12.0));
                        ui.add(egui::TextEdit::singleline(&mut edit_state.ref_rule).desired_width(380.0));

                        ui.add_space(14.0);
                        ui.horizontal(|ui| {
                            let btn = egui::Button::new(RichText::new("Lưu tiêu chuẩn").strong().color(Color32::WHITE))
                                .fill(Color32::from_rgb(14, 165, 233));
                            if ui.add(btn).clicked() {
                                if !edit_state.name.trim().is_empty() {
                                    action = Qd1613Action::SaveClassification(QdClassification {
                                        id: edit_state.id.clone(),
                                        disease_id: edit_state.disease_id.clone(),
                                        name: edit_state.name.trim().to_string(),
                                        score: Some(edit_state.score),
                                        ref_rule: if edit_state.ref_rule.trim().is_empty() { None } else { Some(edit_state.ref_rule.trim().to_string()) },
                                        ref_disease_id: None,
                                        modifier: None,
                                    });
                                    edit_state.is_open = false;
                                }
                            }

                            if ui.button("Hủy").clicked() {
                                edit_state.is_open = false;
                            }
                        });
                    });
                });
        }

        action
    }
}
