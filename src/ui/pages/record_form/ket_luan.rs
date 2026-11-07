use egui::{Color32, RichText, Ui};
use crate::auth::permissions::Perm;
use crate::domain::update_global_health_score;
use crate::models::record::KskRecordPayload;
use crate::ui::widgets::badge::ui_score_badge;

pub struct KetLuanView;

impl KetLuanView {
    pub fn show(
        ui: &mut Ui,
        form: &mut KskRecordPayload,
        user_perms: &[Perm],
        is_locked: bool,
        dark_mode: bool,
    ) {
        let can_conclude = user_perms.contains(&Perm::FormKetLuan);

        ui.add_enabled_ui(!is_locked && can_conclude, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add_space(4.0);
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

                    ui.add_space(14.0);

                    // 2. PHÂN LOẠI SỨC KHỎE CHUNG
                    ui.label(RichText::new("2. PHÂN LOẠI SỨC KHỎE TOÀN DIỆN:").strong().size(12.5));
                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Phân loại sức khỏe (*):").strong().size(13.0));
                        for i in 1..=5 {
                            ui.radio_value(&mut form.phan_loai_sk, i, format!("Loại {}", i));
                        }
                        ui_score_badge(ui, form.phan_loai_sk, false);
                    });

                    ui.add_space(10.0);

                    // 3. CÁC BỆNH TẬT NẾU CÓ
                    ui.label(RichText::new("3. Các bệnh tật (nếu có):").strong().size(12.5));
                    ui.add(
                        egui::TextEdit::multiline(&mut form.cac_benh_tat_neu_co)
                            .desired_rows(3)
                            .desired_width(ui.available_width() - 20.0)
                            .hint_text("Liệt kê các bệnh tật phát hiện trong quá trình khám..."),
                    );

                    ui.add_space(10.0);

                    // 4. KẾT LUẬN SỨC KHỎE
                    ui.label(RichText::new("4. Kết luận sức khỏe:").strong().size(12.5));
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

                    ui.add_space(20.0);
                });
        });
    }
}
