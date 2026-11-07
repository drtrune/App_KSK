use egui::{Color32, RichText, Ui, Vec2};
use crate::domain::{calc_bmi, update_global_health_score};
use crate::models::{AttachmentRecord, KskRecordPayload};
use crate::ui::widgets::badge::ui_score_badge;

pub enum QuickEntryAction {
    None,
    AssignArchiveAndSave,
    AttachScanFile,
    OpenAttachment(String),
    DeleteAttachment(i64),
}

pub struct QuickEntryView;

impl QuickEntryView {
    pub fn show(
        ui: &mut Ui,
        form: &mut KskRecordPayload,
        record_id: Option<i64>,
        attachments: &[AttachmentRecord],
        is_locked: bool,
        dark_mode: bool,
    ) -> QuickEntryAction {
        let mut action = QuickEntryAction::None;

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // Banner Số lưu trữ & Mã Y Tế
                let banner_bg = if !form.so_luu_tru.trim().is_empty() {
                    Color32::from_rgba_unmultiplied(16, 185, 129, 25)
                } else {
                    Color32::from_rgba_unmultiplied(245, 158, 11, 20)
                };
                let banner_border = if !form.so_luu_tru.trim().is_empty() {
                    Color32::from_rgb(16, 185, 129)
                } else {
                    Color32::from_rgb(245, 158, 11)
                };

                egui::Frame::none()
                    .fill(banner_bg)
                    .stroke(egui::Stroke::new(1.5_f32, banner_border))
                    .rounding(8.0)
                    .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new(format!(
                                        "👤 {}   |   CCCD: {}   |   Mã Y Tế (QR): {}",
                                        if form.ho_ten.is_empty() { "CHƯA NHẬP HỌ TÊN" } else { &form.ho_ten },
                                        if form.so_cccd.is_empty() { "---" } else { &form.so_cccd },
                                        &form.ma_y_te
                                    ))
                                    .strong()
                                    .size(13.5),
                                );
                                if form.so_luu_tru.trim().is_empty() {
                                    ui.label(
                                        RichText::new("⏳ Trạng thái: Chưa cấp Số lưu trữ (Số GKSK). Nhập kết quả từ phiếu giấy và bấm 'Hoàn tất Kết luận & Cấp Số Lưu Trữ' bên dưới.")
                                            .size(12.0)
                                            .color(Color32::from_rgb(217, 119, 6)),
                                    );
                                } else {
                                    ui.label(
                                        RichText::new(format!(
                                            "✅ SỐ LƯU TRỮ HỒ SƠ (GHI LÊN PHIẾU GIẤY):   Số: {}",
                                            form.so_luu_tru
                                        ))
                                        .strong()
                                        .size(16.0)
                                        .color(Color32::from_rgb(5, 150, 105)),
                                    );
                                }
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if !is_locked
                                    && ui
                                        .button(
                                            RichText::new("✨ Mặc định Bình thường (Loại I) toàn bộ")
                                                .strong()
                                                .size(12.0)
                                                .color(Color32::from_rgb(14, 165, 233)),
                                        )
                                        .on_hover_text("Điền nhanh Bình thường (Loại 1) cho tất cả chuyên khoa, chỉ cần sửa lại mục bất thường")
                                        .clicked()
                                {
                                    form.reset_clinical_to_normal();
                                }
                            });
                        });
                    });

                ui.add_space(10.0);

                let card_bg = if dark_mode { Color32::from_rgb(30, 41, 59) } else { Color32::WHITE };
                let border_color = if dark_mode { Color32::from_rgb(51, 65, 85) } else { Color32::from_rgb(226, 232, 240) };

                ui.add_enabled_ui(!is_locked, |ui| {
                    // I. KHÁM THỂ LỰC
                    egui::Frame::none()
                        .fill(card_bg)
                        .stroke(egui::Stroke::new(1.0_f32, border_color))
                        .rounding(8.0)
                        .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new("I. KHÁM THỂ LỰC").strong().size(13.0).color(Color32::from_rgb(14, 165, 233)));
                            ui.add_space(4.0);
                            ui.horizontal_wrapped(|ui| {
                                ui.label("Chiều cao (cm):");
                                if ui.add(egui::TextEdit::singleline(&mut form.chieu_cao).desired_width(60.0)).changed() {
                                    if let Some(b) = calc_bmi(&form.chieu_cao, &form.can_nang) {
                                        form.chi_so_bmi = format!("{:.2}", b);
                                    }
                                }
                                ui.add_space(8.0);
                                ui.label("Cân nặng (kg):");
                                if ui.add(egui::TextEdit::singleline(&mut form.can_nang).desired_width(60.0)).changed() {
                                    if let Some(b) = calc_bmi(&form.chieu_cao, &form.can_nang) {
                                        form.chi_so_bmi = format!("{:.2}", b);
                                    }
                                }
                                ui.add_space(8.0);
                                ui.label("BMI:");
                                ui.add(egui::TextEdit::singleline(&mut form.chi_so_bmi).desired_width(60.0));
                                ui.add_space(8.0);
                                ui.label("Mạch (l/p):");
                                ui.add(egui::TextEdit::singleline(&mut form.mach).desired_width(60.0));
                                ui.add_space(8.0);
                                ui.label("Huyết áp (mmHg):");
                                ui.add(egui::TextEdit::singleline(&mut form.huyet_ap).desired_width(80.0));
                                ui.add_space(12.0);
                                ui.label(RichText::new("Phân loại Thể lực:").strong());
                                for sc in 1..=5 {
                                    ui.radio_value(&mut form.kham_the_luc_pl, sc, format!("L{}", sc));
                                }
                            });
                        });

                    ui.add_space(8.0);

                    // II. KHÁM LÂM SÀNG
                    egui::Frame::none()
                        .fill(card_bg)
                        .stroke(egui::Stroke::new(1.0_f32, border_color))
                        .rounding(8.0)
                        .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new("II. KHÁM LÂM SÀNG (NỘI - NGOẠI - SẢN - DA LIỄU - MẮT - TMH - RHM)").strong().size(13.0).color(Color32::from_rgb(14, 165, 233)));
                            ui.add_space(6.0);

                            egui::Grid::new("quick_clinical_grid")
                                .striped(true)
                                .spacing([12.0, 6.0])
                                .show(ui, |ui| {
                                    ui.label(RichText::new("Chuyên khoa").strong().size(11.5));
                                    ui.label(RichText::new("Kết quả khám / Bệnh tật").strong().size(11.5));
                                    ui.label(RichText::new("Phân loại").strong().size(11.5));
                                    ui.label(RichText::new("Chuyên khoa").strong().size(11.5));
                                    ui.label(RichText::new("Kết quả khám / Bệnh tật").strong().size(11.5));
                                    ui.label(RichText::new("Phân loại").strong().size(11.5));
                                    ui.end_row();

                                    let row_pair = |ui: &mut Ui,
                                                        lbl1: &str, txt1: &mut String, pl1: &mut i32,
                                                        lbl2: &str, txt2: &mut String, pl2: &mut i32| {
                                        ui.label(RichText::new(lbl1).strong().size(12.0));
                                        ui.add(egui::TextEdit::singleline(txt1).desired_width(180.0));
                                        ui.horizontal(|ui| {
                                            for sc in 1..=5 {
                                                ui.selectable_value(pl1, sc, format!("{}", sc));
                                            }
                                        });

                                        ui.label(RichText::new(lbl2).strong().size(12.0));
                                        ui.add(egui::TextEdit::singleline(txt2).desired_width(180.0));
                                        ui.horizontal(|ui| {
                                            for sc in 1..=5 {
                                                ui.selectable_value(pl2, sc, format!("{}", sc));
                                            }
                                        });
                                        ui.end_row();
                                    };

                                    row_pair(
                                        ui,
                                        "1a. Tuần hoàn", &mut form.noi_khoa_tuan_hoan, &mut form.noi_khoa_tuan_hoan_pl,
                                        "1e. Cơ xương khớp", &mut form.noi_khoa_co_xuong_khop, &mut form.noi_khoa_co_xuong_khop_pl,
                                    );
                                    row_pair(
                                        ui,
                                        "1b. Hô hấp", &mut form.noi_khoa_ho_hap, &mut form.noi_khoa_ho_hap_pl,
                                        "1g. Thần kinh", &mut form.noi_khoa_than_kinh, &mut form.noi_khoa_than_kinh_pl,
                                    );
                                    row_pair(
                                        ui,
                                        "1c. Tiêu hóa", &mut form.noi_khoa_tieu_hoa, &mut form.noi_khoa_tieu_hoa_pl,
                                        "1h. Tâm thần", &mut form.noi_khoa_tam_than, &mut form.noi_khoa_tam_than_pl,
                                    );
                                    row_pair(
                                        ui,
                                        "1d. Thận-Tiết niệu", &mut form.noi_khoa_than_tn_sd, &mut form.noi_khoa_than_tn_sd_pl,
                                        "2. Ngoại khoa", &mut form.ket_qua_kham_ngoai_khoa, &mut form.kham_ngoai_khoa_pl,
                                    );
                                    row_pair(
                                        ui,
                                        "1đ. Nội tiết", &mut form.noi_khoa_noi_tiet, &mut form.noi_khoa_noi_tiet_pl,
                                        "3. Da liễu", &mut form.ket_qua_kham_da_lieu, &mut form.kham_da_lieu_pl,
                                    );
                                });

                            ui.add_space(6.0);
                            ui.separator();
                            ui.add_space(6.0);

                            // Sản phụ khoa, Mắt, TMH, RHM
                            ui.horizontal_wrapped(|ui| {
                                ui.label(RichText::new("4. Sản phụ khoa:").strong());
                                ui.add(egui::TextEdit::singleline(&mut form.ket_qua_kham_san_phu_khoa).desired_width(180.0));
                                for sc in 1..=5 {
                                    ui.selectable_value(&mut form.kham_san_phu_khoa_pl, sc, format!("L{}", sc));
                                }
                            });

                            ui.add_space(4.0);
                            ui.horizontal_wrapped(|ui| {
                                ui.label(RichText::new("5. Mắt - Không kính (MP/MT):").strong());
                                ui.add(egui::TextEdit::singleline(&mut form.khong_kinh_mat_phai).desired_width(50.0));
                                ui.label("/");
                                ui.add(egui::TextEdit::singleline(&mut form.khong_kinh_mat_trai).desired_width(50.0));
                                ui.label("Có kính (MP/MT):");
                                ui.add(egui::TextEdit::singleline(&mut form.co_kinh_mat_phai).desired_width(50.0));
                                ui.label("/");
                                ui.add(egui::TextEdit::singleline(&mut form.co_kinh_mat_trai).desired_width(50.0));
                                ui.label("Bệnh mắt:");
                                ui.add(egui::TextEdit::singleline(&mut form.benh_khac_mat).desired_width(150.0));
                                for sc in 1..=5 {
                                    ui.selectable_value(&mut form.kham_mat_pl, sc, format!("L{}", sc));
                                }
                            });

                            ui.add_space(4.0);
                            ui.horizontal_wrapped(|ui| {
                                ui.label(RichText::new("6. TMH - Tai trái (Thường/Thầm):").strong());
                                ui.add(egui::TextEdit::singleline(&mut form.tai_trai_noi_thuong).desired_width(40.0));
                                ui.label("/");
                                ui.add(egui::TextEdit::singleline(&mut form.tai_trai_noi_tham).desired_width(40.0));
                                ui.label("Tai phải (Thường/Thầm):");
                                ui.add(egui::TextEdit::singleline(&mut form.tai_phai_noi_thuong).desired_width(40.0));
                                ui.label("/");
                                ui.add(egui::TextEdit::singleline(&mut form.tai_phai_noi_tham).desired_width(40.0));
                                ui.label("Bệnh TMH:");
                                ui.add(egui::TextEdit::singleline(&mut form.benh_khac_tai_mui_hong).desired_width(150.0));
                                for sc in 1..=5 {
                                    ui.selectable_value(&mut form.kham_tai_mui_hong_pl, sc, format!("L{}", sc));
                                }
                            });

                            ui.add_space(4.0);
                            ui.horizontal_wrapped(|ui| {
                                ui.label(RichText::new("7. RHM - Hàm trên:").strong());
                                ui.add(egui::TextEdit::singleline(&mut form.ham_tren).desired_width(110.0));
                                ui.label("Hàm dưới:");
                                ui.add(egui::TextEdit::singleline(&mut form.ham_duoi).desired_width(110.0));
                                ui.label("Bệnh RHM:");
                                ui.add(egui::TextEdit::singleline(&mut form.benh_khac_rang_ham_mat).desired_width(150.0));
                                for sc in 1..=5 {
                                    ui.selectable_value(&mut form.kham_rang_ham_mat_pl, sc, format!("L{}", sc));
                                }
                            });
                        });

                    ui.add_space(8.0);

                    // III. CẬN LÂM SÀNG (MẪU 03)
                    egui::Frame::none()
                        .fill(card_bg)
                        .stroke(egui::Stroke::new(1.0_f32, border_color))
                        .rounding(8.0)
                        .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new("III. KHÁM CẬN LÂM SÀNG (TỰ ĐỘNG ĐỒNG BỘ SANG DM_CanLamSang)").strong().size(13.0).color(Color32::from_rgb(14, 165, 233)));
                            ui.add_space(4.0);
                            ui.horizontal_wrapped(|ui| {
                                ui.label("Huyết học:");
                                ui.add(egui::TextEdit::singleline(&mut form.cls_huyet_hoc).desired_width(110.0));
                                ui.label("Đường máu:");
                                ui.add(egui::TextEdit::singleline(&mut form.cls_duong_mau).desired_width(60.0));
                                ui.label("Urê:");
                                ui.add(egui::TextEdit::singleline(&mut form.cls_ure).desired_width(60.0));
                                ui.label("Creatinin:");
                                ui.add(egui::TextEdit::singleline(&mut form.cls_creatinin).desired_width(60.0));
                                ui.label("GOT:");
                                ui.add(egui::TextEdit::singleline(&mut form.cls_asat_got).desired_width(50.0));
                                ui.label("GPT:");
                                ui.add(egui::TextEdit::singleline(&mut form.cls_alat_gpt).desired_width(50.0));
                            });
                            ui.add_space(4.0);
                            ui.horizontal_wrapped(|ui| {
                                ui.label("Nước tiểu:");
                                ui.add(egui::TextEdit::singleline(&mut form.cls_nuoc_tieu).desired_width(130.0));
                                ui.label("XQ tim phổi thẳng:");
                                ui.add(egui::TextEdit::singleline(&mut form.cls_xq_tim_phoi).desired_width(180.0));
                                ui.label("CLS khác:");
                                ui.add(egui::TextEdit::singleline(&mut form.cls_khac_mo_ta).desired_width(200.0));
                            });
                        });

                    ui.add_space(8.0);

                    // IV. KẾT LUẬN & CẤP SỐ LƯU TRỮ
                    egui::Frame::none()
                        .fill(card_bg)
                        .stroke(egui::Stroke::new(1.0_f32, border_color))
                        .rounding(8.0)
                        .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("IV. KẾT LUẬN & CẤP SỐ LƯU TRỮ HỒ SƠ").strong().size(13.0).color(Color32::from_rgb(14, 165, 233)));
                                if ui.button("🔄 Tự tính phân loại từ các chuyên khoa").clicked() {
                                    update_global_health_score(form);
                                }
                            });
                            ui.add_space(6.0);
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Phân loại sức khỏe chung:").strong());
                                for i in 1..=5 {
                                    ui.radio_value(&mut form.phan_loai_sk, i, format!("Loại {}", i));
                                }
                                ui_score_badge(ui, form.phan_loai_sk, false);
                            });
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.label("Các bệnh tật (nếu có):");
                                ui.add(egui::TextEdit::singleline(&mut form.cac_benh_tat_neu_co).desired_width(280.0));
                                ui.label("Kết luận:");
                                ui.add(egui::TextEdit::singleline(&mut form.ket_luan_benh).desired_width(220.0));
                            });

                            ui.add_space(10.0);
                            let confirm_btn = egui::Button::new(
                                RichText::new("✅ HOÀN TẤT KẾT LUẬN, LƯU HỒ SƠ & CẤP SỐ LƯU TRỮ (SỐ GKSK)")
                                    .strong()
                                    .size(13.5)
                                    .color(Color32::WHITE),
                            )
                            .fill(Color32::from_rgb(16, 185, 129))
                            .rounding(8.0)
                            .min_size(Vec2::new(420.0, 38.0));

                            if ui.add(confirm_btn).clicked() {
                                action = QuickEntryAction::AssignArchiveAndSave;
                            }
                        });
                });

                ui.add_space(10.0);

                // V. ĐÍNH KÈM FILE SCAN PHIẾU KHÁM GIẤY
                egui::Frame::none()
                    .fill(card_bg)
                    .stroke(egui::Stroke::new(1.0_f32, border_color))
                    .rounding(8.0)
                    .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("📎 BẢN SCAN PHIẾU KHÁM GIẤY & KẾT QUẢ XÉT NGHIỆM ĐÍNH KÈM")
                                    .strong()
                                    .size(13.0)
                                    .color(Color32::from_rgb(14, 165, 233)),
                            );
                            if record_id.is_some() {
                                if ui
                                    .button(RichText::new("➕ Chọn file Scan (PDF/Ảnh) đính kèm vào hồ sơ").strong().color(Color32::from_rgb(14, 165, 233)))
                                    .clicked()
                                {
                                    action = QuickEntryAction::AttachScanFile;
                                }
                            } else {
                                ui.label(RichText::new("(Hãy bấm Lưu hồ sơ trước khi đính kèm file Scan)").size(11.5).color(Color32::from_rgb(245, 158, 11)));
                            }
                        });

                        if let Some(rid) = record_id {
                            let rec_atts: Vec<&AttachmentRecord> = attachments
                                .iter()
                                .filter(|a| a.owner_type == "record" && a.owner_id == rid)
                                .collect();
                            if rec_atts.is_empty() {
                                ui.label(RichText::new("Chưa có bản scan phiếu khám nào được đính kèm.").size(11.5).color(Color32::from_rgb(148, 163, 184)));
                            } else {
                                ui.add_space(4.0);
                                for att in rec_atts {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(format!("📄 {} ({})", att.file_name, att.uploaded_at)).size(12.0).strong());
                                        if ui.button("👁 Mở xem").clicked() {
                                            action = QuickEntryAction::OpenAttachment(att.file_path.clone());
                                        }
                                        if !is_locked && ui.button(RichText::new("Xóa").color(Color32::from_rgb(239, 68, 68))).clicked() {
                                            action = QuickEntryAction::DeleteAttachment(att.id);
                                        }
                                    });
                                }
                            }
                        }
                    });

                ui.add_space(20.0);
            });

        action
    }
}
