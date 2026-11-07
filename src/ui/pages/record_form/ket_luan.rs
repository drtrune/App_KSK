use egui::{Color32, RichText, Ui, Vec2};
use crate::auth::permissions::Perm;
use crate::domain::update_global_health_score;
use crate::models::{AttachmentRecord, KskRecordPayload};
use crate::ui::pages::record_form::quick_entry::QuickEntryAction;
use crate::ui::widgets::badge::ui_score_badge;

pub struct KetLuanView;

impl KetLuanView {
    pub fn show(
        ui: &mut Ui,
        form: &mut KskRecordPayload,
        record_id: Option<i64>,
        attachments: &[AttachmentRecord],
        user_perms: &[Perm],
        is_locked: bool,
        dark_mode: bool,
    ) -> QuickEntryAction {
        let mut action = QuickEntryAction::None;
        let can_conclude = user_perms.contains(&Perm::FormKetLuan);

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add_space(4.0);

                // Banner Số lưu trữ (Số GKSK)
                let has_archive = !form.so_luu_tru.trim().is_empty();
                let banner_bg = if has_archive {
                    Color32::from_rgba_unmultiplied(16, 185, 129, 25)
                } else {
                    Color32::from_rgba_unmultiplied(245, 158, 11, 20)
                };
                let banner_border = if has_archive {
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
                            if has_archive {
                                ui.label(
                                    RichText::new(format!(
                                        "🏷 SỐ LƯU TRỮ HỒ SƠ (GHI LÊN TỜ PHIẾU GIẤY):   Số: {}",
                                        form.so_luu_tru
                                    ))
                                    .strong()
                                    .size(16.0)
                                    .color(Color32::from_rgb(5, 150, 105)),
                                );
                            } else {
                                ui.label(
                                    RichText::new("⏳ Hồ sơ chưa được cấp Số lưu trữ (Số GKSK). Bấm 'Hoàn tất Kết luận & Cấp Số Lưu Trữ' bên dưới để sinh số lưu trữ.")
                                        .strong()
                                        .size(12.5)
                                        .color(Color32::from_rgb(217, 119, 6)),
                                );
                            }
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(RichText::new(format!("Mã Y Tế (QR): {}", form.ma_y_te)).monospace().strong());
                            });
                        });
                    });

                ui.add_space(8.0);

                ui.add_enabled_ui(!is_locked && can_conclude, |ui| {
                    ui.horizontal(|ui| {
                        ui.heading(RichText::new("IV. KẾT LUẬN KHÁM SỨC KHỎE").size(15.0).strong().color(Color32::from_rgb(14, 165, 233)));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("🔄 Tự động tính lại phân loại").clicked() {
                                update_global_health_score(form);
                            }
                        });
                    });

                    ui.add_space(8.0);

                    // 1. BẢNG TỔNG HỢP KẾT QUẢ CÁC CHUYÊN KHOA
                    ui.label(RichText::new("1. BẢNG TỔNG HỢP CÁC CHUYÊN KHOA:").strong().size(12.5));
                    ui.add_space(4.0);

                    let card_bg = if dark_mode { Color32::from_rgb(30, 41, 59) } else { Color32::WHITE };
                    let border_color = if dark_mode { Color32::from_rgb(51, 65, 85) } else { Color32::from_rgb(226, 232, 240) };

                    egui::Frame::none()
                        .fill(card_bg)
                        .stroke(egui::Stroke::new(1.0_f32, border_color))
                        .rounding(6.0)
                        .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                        .show(ui, |ui| {
                            egui::Grid::new("grid_summary_specialties")
                                .striped(true)
                                .spacing([16.0, 6.0])
                                .min_col_width(80.0)
                                .show(ui, |ui| {
                                    ui.label(RichText::new("STT").strong().size(11.5).color(Color32::from_rgb(100, 116, 139)));
                                    ui.label(RichText::new("Nội dung khám").strong().size(11.5).color(Color32::from_rgb(100, 116, 139)));
                                    ui.label(RichText::new("Kết quả / Bệnh tật phát hiện").strong().size(11.5).color(Color32::from_rgb(100, 116, 139)));
                                    ui.label(RichText::new("Phân loại").strong().size(11.5).color(Color32::from_rgb(100, 116, 139)));
                                    ui.end_row();

                                    let specs = [
                                        ("1", "Thể lực", format!("BMI: {}, Mạch: {}, HA: {}", form.chi_so_bmi, form.mach, form.huyet_ap), form.kham_the_luc_pl),
                                        ("2", "Nội khoa - Tuần hoàn", form.noi_khoa_tuan_hoan.clone(), form.noi_khoa_tuan_hoan_pl),
                                        ("3", "Nội khoa - Hô hấp", form.noi_khoa_ho_hap.clone(), form.noi_khoa_ho_hap_pl),
                                        ("4", "Nội khoa - Tiêu hóa", form.noi_khoa_tieu_hoa.clone(), form.noi_khoa_tieu_hoa_pl),
                                        ("5", "Nội khoa - Thận Tiết niệu", form.noi_khoa_than_tn_sd.clone(), form.noi_khoa_than_tn_sd_pl),
                                        ("6", "Nội khoa - Nội tiết", form.noi_khoa_noi_tiet.clone(), form.noi_khoa_noi_tiet_pl),
                                        ("7", "Nội khoa - Cơ xương khớp", form.noi_khoa_co_xuong_khop.clone(), form.noi_khoa_co_xuong_khop_pl),
                                        ("8", "Nội khoa - Thần kinh", form.noi_khoa_than_kinh.clone(), form.noi_khoa_than_kinh_pl),
                                        ("9", "Nội khoa - Tâm thần", form.noi_khoa_tam_than.clone(), form.noi_khoa_tam_than_pl),
                                        ("10", "Ngoại khoa", form.ket_qua_kham_ngoai_khoa.clone(), form.kham_ngoai_khoa_pl),
                                        ("11", "Mắt", form.benh_khac_mat.clone(), form.kham_mat_pl),
                                        ("12", "Tai Mũi Họng", form.benh_khac_tai_mui_hong.clone(), form.kham_tai_mui_hong_pl),
                                        ("13", "Răng Hàm Mặt", form.benh_khac_rang_ham_mat.clone(), form.kham_rang_ham_mat_pl),
                                        ("14", "Da liễu", form.ket_qua_kham_da_lieu.clone(), form.kham_da_lieu_pl),
                                        ("15", "Sản phụ khoa", form.ket_qua_kham_san_phu_khoa.clone(), form.kham_san_phu_khoa_pl),
                                    ];

                                    for (stt, name, res, sc) in specs {
                                        ui.label(stt);
                                        ui.label(RichText::new(name).strong().size(12.0));
                                        ui.label(RichText::new(if res.trim().is_empty() { "Bình thường".to_string() } else { res }).size(12.0));
                                        ui_score_badge(ui, sc, false);
                                        ui.end_row();
                                    }
                                });
                        });

                    ui.add_space(12.0);

                    // 2. PHÂN LOẠI SỨC KHỎE CHUNG
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("2. Phân loại sức khỏe (*):").strong().size(13.0));
                        for i in 1..=5 {
                            ui.radio_value(&mut form.phan_loai_sk, i, format!("Loại {}", i));
                        }
                        ui_score_badge(ui, form.phan_loai_sk, false);
                    });

                    ui.add_space(8.0);

                    // 3. CÁC BỆNH TẬT NẾU CÓ
                    ui.label(RichText::new("3. Tình trạng sức khỏe; mắc các bệnh, tật (nếu có):").strong().size(12.5));
                    ui.add(
                        egui::TextEdit::multiline(&mut form.cac_benh_tat_neu_co)
                            .desired_rows(2)
                            .desired_width(ui.available_width() - 20.0)
                            .hint_text("Liệt kê các bệnh tật phát hiện trong quá trình khám..."),
                    );

                    ui.add_space(8.0);

                    // 4. KẾT LUẬN SỨC KHỎE
                    ui.label(RichText::new("4. Kết luận bệnh / sức khỏe:").strong().size(12.5));
                    ui.horizontal(|ui| {
                        if ui.button("Đủ sức khỏe làm việc").clicked() {
                            form.ket_luan_benh = "Đủ sức khỏe làm việc".into();
                        }
                        if ui.button("Đủ sức khỏe học tập").clicked() {
                            form.ket_luan_benh = "Đủ sức khỏe học tập".into();
                        }
                        if ui.button("Không đủ sức khỏe").clicked() {
                            form.ket_luan_benh = "Không đủ sức khỏe làm việc".into();
                        }
                    });

                    ui.add_space(4.0);
                    ui.add(
                        egui::TextEdit::singleline(&mut form.ket_luan_benh)
                            .desired_width(ui.available_width() - 20.0)
                            .hint_text("VD: Đủ sức khỏe làm việc"),
                    );

                    ui.add_space(12.0);

                    // NÚT HOÀN TẤT KẾT LUẬN & CẤP SỐ LƯU TRỮ
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

                ui.add_space(14.0);
                ui.separator();
                ui.add_space(8.0);

                // 5. ĐÍNH KÈM FILE SCAN PHIẾU KHÁM GIẤY
                ui.horizontal(|ui| {
                    ui.label(RichText::new("5. BẢN SCAN PHIẾU KHÁM GIẤY & TÀI LIỆU ĐÍNH KÈM:").strong().size(13.0));
                    if record_id.is_some() {
                        if ui
                            .button(RichText::new("📎 Đính kèm file Scan (PDF/Ảnh)").strong().color(Color32::from_rgb(14, 165, 233)))
                            .clicked()
                        {
                            action = QuickEntryAction::AttachScanFile;
                        }
                    } else {
                        ui.label(RichText::new("(Hãy Lưu hồ sơ trước khi đính kèm file Scan)").size(11.5).color(Color32::from_rgb(245, 158, 11)));
                    }
                });

                if let Some(rid) = record_id {
                    let rec_atts: Vec<&AttachmentRecord> = attachments
                        .iter()
                        .filter(|a| a.owner_type == "record" && a.owner_id == rid)
                        .collect();
                    if rec_atts.is_empty() {
                        ui.label(RichText::new("Chưa có file Scan phiếu khám nào.").size(11.5).color(Color32::from_rgb(148, 163, 184)));
                    } else {
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

                ui.add_space(20.0);
            });

        action
    }
}
