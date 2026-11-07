use std::collections::HashSet;
use egui::{Color32, RichText, Ui};
use crate::domain::conclusion::sync_specialty_result;
use crate::domain::scoring::{
    calc_blood_pressure_score, calc_classification_score, calc_hearing_from_distances,
    calc_pulse_score, calc_vision_score,
};
use crate::models::{ManualConclusionRow, QdClassification, QdDisease, QdSpecialty, KskRecordPayload};
use crate::ui::widgets::badge::ui_score_badge;

pub struct SpecialtyCatalogView;

impl SpecialtyCatalogView {
    pub fn show(
        ui: &mut Ui,
        form: &mut KskRecordPayload,
        spec_id: &str,
        spec_name: &str,
        _specialties: &[QdSpecialty],
        diseases: &[QdDisease],
        classifications: &[QdClassification],
        expanded_diseases: &mut HashSet<String>,
        is_locked: bool,
        dark_mode: bool,
    ) {
        ui.add_enabled_ui(!is_locked, |ui| {
            let spec_diseases: Vec<&QdDisease> = diseases.iter().filter(|d| d.spec_id == spec_id).collect();

            // 1. Header & Actions
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.heading(RichText::new(format!("Chuyên khoa: {} [Chuẩn QĐ 1613]", spec_name)).size(15.0).strong().color(Color32::from_rgb(14, 165, 233)));
                    ui.label(RichText::new("Tick chọn bệnh lý (checkbox) để tự động phân loại. Khi không chọn bệnh nào → Mặc định Loại 1 (Bình thường).")
                        .size(11.5)
                        .color(Color32::from_rgb(100, 116, 139)));
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Thu gọn tất cả").clicked() {
                        for d in &spec_diseases {
                            expanded_diseases.remove(&d.id);
                        }
                    }
                    if ui.button("Mở tất cả").clicked() {
                        for d in &spec_diseases {
                            expanded_diseases.insert(d.id.clone());
                        }
                    }
                });
            });

            ui.add_space(8.0);

            // 2. Base scores for dynamic evaluations
            let base_vision = calc_vision_score(&form.khong_kinh_mat_phai, &form.khong_kinh_mat_trai);
            let has_glasses = !form.co_kinh_mat_phai.trim().is_empty() || !form.co_kinh_mat_trai.trim().is_empty();
            let vision_glasses = if has_glasses {
                calc_vision_score(&form.co_kinh_mat_phai, &form.co_kinh_mat_trai)
            } else {
                base_vision
            };
            let (base_hearing, _) = calc_hearing_from_distances(
                &form.tai_phai_noi_thuong,
                &form.tai_trai_noi_thuong,
                &form.tai_phai_noi_tham,
                &form.tai_trai_noi_tham,
            );

            // SPECIAL PANEL: MẮT (CK_MAT) - Đo thị lực không kính & có kính
            if spec_id == "CK_MAT" {
                Self::render_vision_panel(ui, form, base_vision, vision_glasses, has_glasses, dark_mode);
                ui.add_space(8.0);
            }

            // SPECIAL PANEL: TAI MŨI HỌNG (CK_TMH) - Thính lực 5m & nói thầm
            if spec_id == "CK_TMH" {
                Self::render_hearing_panel(ui, form, base_hearing, dark_mode);
                ui.add_space(8.0);
            }

            // SPECIAL AUTO CARDS FOR TIM MẠCH
            let age = crate::domain::dates::calculate_age(&form.ngay_sinh);
            let (bp_score, bp_desc) = calc_blood_pressure_score(&form.huyet_ap, age);
            let (pulse_score, pulse_desc) = calc_pulse_score(&form.mach);

            // 3. DANH SÁCH THẺ BỆNH TẬT (TICKBOX CHECKBOX)
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if spec_diseases.is_empty() {
                        ui.label(RichText::new("Chuyên khoa này chưa có mục bệnh tật nào trong danh mục.").color(Color32::GRAY));
                    } else {
                        for disease in &spec_diseases {
                            let clss: Vec<&QdClassification> = classifications.iter().filter(|c| c.disease_id == disease.id).collect();
                            if clss.is_empty() {
                                continue;
                            }

                            // AUTO: Huyết áp
                            if disease.id == "BT_TM_HUYET_AP" {
                                Self::render_auto_card(ui, &disease.name, bp_score, &format!("Huyết áp {} mmHg ({})", form.huyet_ap, bp_desc), dark_mode);
                                ui.add_space(4.0);
                                continue;
                            }

                            // AUTO: Tần số mạch
                            if disease.id == "BT_TM_MACH" {
                                Self::render_auto_card(ui, &disease.name, pulse_score, &format!("Tần số {} lần/phút ({})", form.mach, pulse_desc), dark_mode);
                                ui.add_space(4.0);
                                continue;
                            }

                            // STANDARD DISEASE CARD WITH CLEAN ACCORDION
                            ui.push_id(&disease.id, |ui| {
                                let is_expanded = expanded_diseases.contains(&disease.id);

                                // Check which classifications under this disease are ticked
                                let ticked: Vec<&&QdClassification> = clss.iter()
                                    .filter(|c| form.selected_items.contains(&c.id))
                                    .collect();

                                let disease_score = if ticked.is_empty() {
                                    1
                                } else {
                                    ticked.iter()
                                        .map(|c| calc_classification_score(c, base_vision, base_hearing))
                                        .max()
                                        .unwrap_or(1)
                                };

                                let subtitle_text = if ticked.is_empty() {
                                    "Bình thường / Chưa phát hiện bệnh".to_string()
                                } else {
                                    ticked.iter().map(|c| c.name.as_str()).collect::<Vec<&str>>().join("; ")
                                };

                                let border_color = if disease_score > 1 {
                                    Color32::from_rgb(248, 113, 113)
                                } else if dark_mode {
                                    Color32::from_rgb(51, 65, 85)
                                } else {
                                    Color32::from_rgb(226, 232, 240)
                                };

                                let card_bg = if dark_mode {
                                    if disease_score > 1 { Color32::from_rgb(35, 30, 45) } else { Color32::from_rgb(24, 30, 42) }
                                } else if disease_score > 1 {
                                    Color32::from_rgb(254, 242, 242)
                                } else {
                                    Color32::from_rgb(248, 250, 252)
                                };

                                egui::Frame::none()
                                    .fill(card_bg)
                                    .stroke(egui::Stroke::new(1.0_f32, border_color))
                                    .rounding(6.0)
                                    .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                                    .show(ui, |ui| {
                                        // Header row: clickable via button or label click
                                        let chevron = if is_expanded { "▼" } else { "▶" };
                                        let header_clicked = ui.horizontal(|ui| {
                                            let mut clicked = false;
                                            if ui.button(RichText::new(chevron).monospace().size(11.0)).clicked() {
                                                clicked = true;
                                            }
                                            ui.vertical(|ui| {
                                                if ui.link(RichText::new(&disease.name).strong().size(12.5)).clicked() {
                                                    clicked = true;
                                                }
                                                ui.label(
                                                    RichText::new(&subtitle_text)
                                                        .size(11.0)
                                                        .color(if disease_score > 1 { Color32::from_rgb(220, 38, 38) } else { Color32::from_rgb(100, 116, 139) }),
                                                );
                                            });

                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                ui_score_badge(ui, disease_score, false);
                                            });

                                            clicked
                                        }).inner;

                                        if header_clicked {
                                            if is_expanded {
                                                expanded_diseases.remove(&disease.id);
                                            } else {
                                                expanded_diseases.insert(disease.id.clone());
                                            }
                                        }

                                        // Expanded classification options: CHECKBOXES
                                        if is_expanded {
                                            ui.separator();
                                            ui.vertical(|ui| {
                                                for c in &clss {
                                                    ui.push_id(&c.id, |ui| {
                                                        let mut is_checked = form.selected_items.contains(&c.id);
                                                        let c_score = calc_classification_score(c, base_vision, base_hearing);

                                                        ui.horizontal(|ui| {
                                                            if ui.checkbox(&mut is_checked, "").changed() {
                                                                if is_checked {
                                                                    form.selected_items.insert(c.id.clone());
                                                                } else {
                                                                    form.selected_items.remove(&c.id);
                                                                }
                                                                sync_specialty_result(form, spec_id, diseases, classifications);
                                                            }

                                                            ui.vertical(|ui| {
                                                                let text_color = if is_checked {
                                                                    Color32::from_rgb(14, 165, 233)
                                                                } else if dark_mode {
                                                                    Color32::from_rgb(226, 232, 240)
                                                                } else {
                                                                    Color32::from_rgb(30, 41, 59)
                                                                };
                                                                ui.label(RichText::new(&c.name).size(12.0).color(text_color));
                                                                if let Some(ref r) = c.ref_rule {
                                                                    ui.label(RichText::new(format!("↳ {}", r)).size(10.5).color(Color32::from_rgb(180, 83, 9)).italics());
                                                                }
                                                            });

                                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                                ui_score_badge(ui, c_score, c.ref_disease_id.is_some());
                                                            });
                                                        });
                                                    });
                                                    ui.add_space(2.0);
                                                }
                                            });
                                        }
                                    });
                                ui.add_space(4.0);
                            });
                        }
                    }

                    ui.add_space(8.0);

                    // 4. BỆNH NGOÀI DANH SÁCH (MANUAL ENTRIES)
                    ui.label(RichText::new("Bệnh ngoài danh mục QĐ 1613:").size(12.5).strong().color(Color32::from_rgb(100, 116, 139)));
                    ui.add_space(4.0);

                    let mut remove_idx = None;
                    let mut manual_changed = false;

                    {
                        let extra = form.specialty_extras.entry(spec_id.to_string()).or_default();

                        for (idx, row) in extra.manual_rows.iter_mut().enumerate() {
                            ui.push_id(idx, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(format!("{}.", idx + 1));
                                    egui::ComboBox::from_id_source(format!("cb_man_{}_{}", spec_id, idx))
                                        .width(80.0)
                                        .selected_text(format!("Loại {}", row.classification))
                                        .show_ui(ui, |ui| {
                                            for n in 1..=5 {
                                                if ui.selectable_value(&mut row.classification, n, format!("Loại {}", n)).clicked() {
                                                    manual_changed = true;
                                                }
                                            }
                                        });

                                    if ui.add(egui::TextEdit::singleline(&mut row.manifestation).hint_text("Nhập tên bệnh ngoài danh mục hoặc mô tả...").desired_width(320.0)).changed() {
                                        manual_changed = true;
                                    }

                                    if ui.button(RichText::new("Xóa").size(11.0).color(Color32::from_rgb(239, 68, 68))).clicked() {
                                        remove_idx = Some(idx);
                                    }
                                });
                            });
                            ui.add_space(2.0);
                        }

                        if let Some(r_idx) = remove_idx {
                            extra.manual_rows.remove(r_idx);
                            manual_changed = true;
                        }
                    }

                    if manual_changed {
                        sync_specialty_result(form, spec_id, diseases, classifications);
                    }

                    if ui.button(RichText::new("➕ Thêm bệnh ngoài danh sách").strong().size(12.0).color(Color32::from_rgb(14, 165, 233))).clicked() {
                        form.specialty_extras.entry(spec_id.to_string()).or_default().manual_rows.push(ManualConclusionRow {
                            classification: 2,
                            classification_id: "Ngoài danh mục".into(),
                            manifestation: "".into(),
                        });
                        sync_specialty_result(form, spec_id, diseases, classifications);
                    }

                    ui.add_space(10.0);

                    // 5. GHI CHÚ TỔNG HỢP CHO CHUYÊN KHOA
                    ui.label(RichText::new("GHI CHÚ CHUYÊN KHOA (HIỂN THỊ KẾT LUẬN):").size(12.0).strong());
                    let note_changed = {
                        let note_ref = &mut form.specialty_extras.entry(spec_id.to_string()).or_default().final_note;
                        ui.add(egui::TextEdit::multiline(note_ref).desired_rows(2).desired_width(ui.available_width() - 20.0)).changed()
                    };
                    if note_changed {
                        sync_specialty_result(form, spec_id, diseases, classifications);
                    }

                    ui.add_space(20.0);
                });
        });
    }

    fn render_auto_card(ui: &mut Ui, title: &str, score: i32, desc: &str, dark_mode: bool) {
        let card_bg = if dark_mode { Color32::from_rgb(30, 41, 59) } else { Color32::from_rgb(240, 249, 255) };
        let stroke_color = if dark_mode { Color32::from_rgb(51, 65, 85) } else { Color32::from_rgb(186, 230, 253) };

        egui::Frame::none()
            .fill(card_bg)
            .stroke(egui::Stroke::new(1.0_f32, stroke_color))
            .rounding(6.0)
            .inner_margin(egui::Margin::symmetric(10.0, 8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("⚡ {} [TỰ ĐỘNG]", title)).strong().size(12.5));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui_score_badge(ui, score, false);
                        ui.label(RichText::new(desc).size(12.0));
                    });
                });
            });
    }

    fn render_vision_panel(
        ui: &mut Ui,
        form: &mut KskRecordPayload,
        base_vision: i32,
        vision_glasses: i32,
        has_glasses: bool,
        dark_mode: bool,
    ) {
        let card_bg = if dark_mode { Color32::from_rgb(30, 41, 59) } else { Color32::from_rgb(239, 246, 255) };
        let stroke_color = if dark_mode { Color32::from_rgb(51, 65, 85) } else { Color32::from_rgb(147, 197, 253) };

        egui::Frame::none()
            .fill(card_bg)
            .stroke(egui::Stroke::new(1.0_f32, stroke_color))
            .rounding(6.0)
            .inner_margin(egui::Margin::symmetric(12.0, 10.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("👁 Đo thị lực & Tự động phân loại").strong().size(13.0).color(Color32::from_rgb(29, 78, 216)));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui_score_badge(ui, base_vision, false);
                        ui.label(RichText::new(format!("Thị lực chuẩn: LOẠI {}", base_vision)).strong().size(12.0));
                        if has_glasses {
                            ui.label(" | Có kính:");
                            ui_score_badge(ui, vision_glasses, false);
                        }
                    });
                });

                ui.add_space(8.0);

                egui::Grid::new("grid_vision_inputs")
                    .spacing([18.0, 8.0])
                    .show(ui, |ui| {
                        // Tay thuận
                        ui.label(RichText::new("Tay thuận:").strong().size(12.0));
                        ui.horizontal(|ui| {
                            ui.radio_value(&mut form.dominant_hand, "right".into(), "Tay Phải");
                            ui.radio_value(&mut form.dominant_hand, "left".into(), "Tay Trái");
                        });
                        ui.end_row();

                        // Không kính (bắt buộc)
                        ui.label(RichText::new("Không kính (*):").strong().size(12.0));
                        ui.horizontal(|ui| {
                            ui.label("MP:");
                            ui.add(egui::TextEdit::singleline(&mut form.khong_kinh_mat_phai).desired_width(45.0).hint_text("10/10"));
                            ui.label("MT:");
                            ui.add(egui::TextEdit::singleline(&mut form.khong_kinh_mat_trai).desired_width(45.0).hint_text("10/10"));
                            ui.separator();
                            if ui.button("10/10").clicked() {
                                form.khong_kinh_mat_phai = "10/10".into();
                                form.khong_kinh_mat_trai = "10/10".into();
                            }
                            if ui.button("9/10").clicked() {
                                form.khong_kinh_mat_phai = "9/10".into();
                                form.khong_kinh_mat_trai = "9/10".into();
                            }
                            if ui.button("8/10").clicked() {
                                form.khong_kinh_mat_phai = "8/10".into();
                                form.khong_kinh_mat_trai = "8/10".into();
                            }
                            if ui.button("0/10").clicked() {
                                form.khong_kinh_mat_phai = "0/10".into();
                                form.khong_kinh_mat_trai = "0/10".into();
                            }
                        });
                        ui.end_row();

                        // Có kính (không bắt buộc)
                        ui.label(RichText::new("Có kính:").size(12.0));
                        ui.horizontal(|ui| {
                            ui.label("MP:");
                            ui.add(egui::TextEdit::singleline(&mut form.co_kinh_mat_phai).desired_width(45.0).hint_text("10/10"));
                            ui.label("MT:");
                            ui.add(egui::TextEdit::singleline(&mut form.co_kinh_mat_trai).desired_width(45.0).hint_text("10/10"));
                            ui.separator();
                            if ui.button("10/10 (Kính)").clicked() {
                                form.co_kinh_mat_phai = "10/10".into();
                                form.co_kinh_mat_trai = "10/10".into();
                            }
                        });
                        ui.end_row();
                    });
            });
    }

    fn render_hearing_panel(
        ui: &mut Ui,
        form: &mut KskRecordPayload,
        base_hearing: i32,
        dark_mode: bool,
    ) {
        let card_bg = if dark_mode { Color32::from_rgb(30, 41, 59) } else { Color32::from_rgb(240, 253, 244) };
        let stroke_color = if dark_mode { Color32::from_rgb(51, 65, 85) } else { Color32::from_rgb(134, 239, 172) };

        egui::Frame::none()
            .fill(card_bg)
            .stroke(egui::Stroke::new(1.0_f32, stroke_color))
            .rounding(6.0)
            .inner_margin(egui::Margin::symmetric(12.0, 10.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("👂 Thính lực & Tự động phân loại").strong().size(13.0).color(Color32::from_rgb(21, 128, 61)));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui_score_badge(ui, base_hearing, false);
                        ui.label(RichText::new(format!("Thính lực chuẩn: LOẠI {}", base_hearing)).strong().size(12.0));
                    });
                });

                ui.add_space(8.0);

                egui::Grid::new("grid_hearing_inputs")
                    .spacing([18.0, 8.0])
                    .show(ui, |ui| {
                        ui.label(RichText::new("Nói thường (m):").strong().size(12.0));
                        ui.horizontal(|ui| {
                            ui.label("Phải:");
                            ui.add(egui::TextEdit::singleline(&mut form.tai_phai_noi_thuong).desired_width(45.0).hint_text("5m"));
                            ui.label("Trái:");
                            ui.add(egui::TextEdit::singleline(&mut form.tai_trai_noi_thuong).desired_width(45.0).hint_text("5m"));
                            ui.separator();
                            if ui.button("5m").clicked() {
                                form.tai_phai_noi_thuong = "5m".into();
                                form.tai_trai_noi_thuong = "5m".into();
                            }
                            if ui.button(">5m").clicked() {
                                form.tai_phai_noi_thuong = ">5m".into();
                                form.tai_trai_noi_thuong = ">5m".into();
                            }
                            if ui.button("4m").clicked() {
                                form.tai_phai_noi_thuong = "4m".into();
                                form.tai_trai_noi_thuong = "4m".into();
                            }
                            if ui.button("Điếc").clicked() {
                                form.tai_phai_noi_thuong = "Điếc".into();
                                form.tai_trai_noi_thuong = "Điếc".into();
                            }
                        });
                        ui.end_row();

                        ui.label(RichText::new("Nói thầm (m):").strong().size(12.0));
                        ui.horizontal(|ui| {
                            ui.label("Phải:");
                            ui.add(egui::TextEdit::singleline(&mut form.tai_phai_noi_tham).desired_width(45.0).hint_text("0.5m"));
                            ui.label("Trái:");
                            ui.add(egui::TextEdit::singleline(&mut form.tai_trai_noi_tham).desired_width(45.0).hint_text("0.5m"));
                            ui.separator();
                            if ui.button("0.5m").clicked() {
                                form.tai_phai_noi_tham = "0.5m".into();
                                form.tai_trai_noi_tham = "0.5m".into();
                            }
                            if ui.button(">0.5m").clicked() {
                                form.tai_phai_noi_tham = ">0.5m".into();
                                form.tai_trai_noi_tham = ">0.5m".into();
                            }
                            if ui.button("Điếc").clicked() {
                                form.tai_phai_noi_tham = "Điếc".into();
                                form.tai_trai_noi_tham = "Điếc".into();
                            }
                        });
                        ui.end_row();
                    });
            });
    }
}
