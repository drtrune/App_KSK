use egui::{Color32, RichText, Ui};
use crate::models::catalog::CatalogRecord;
use crate::models::record::KskRecordPayload;
use crate::ui::widgets::search_combo::SearchCombo;

pub struct TienSuView;

impl TienSuView {
    pub fn show(
        ui: &mut Ui,
        form: &mut KskRecordPayload,
        is_locked: bool,
        catalogs: &[CatalogRecord],
    ) {
        ui.add_enabled_ui(!is_locked, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add_space(4.0);

                    // ===================================================================
                    // 1. TIỀN SỬ GIA ĐÌNH
                    // ===================================================================
                    ui.heading(RichText::new("1. Tiền sử gia đình:").size(14.0).strong().color(Color32::from_rgb(14, 165, 233)));
                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Có ai trong gia đình mắc một trong các bệnh: truyền nhiễm, tim mạch, đái tháo đường, lao, hen phế quản, ung thư, động kinh, rối loạn tâm thần không?").size(12.5));
                        ui.add_space(12.0);
                        ui.radio_value(&mut form.tsgd_mac_benh, 0, "Không");
                        ui.radio_value(&mut form.tsgd_mac_benh, 1, "Có");
                    });

                    // Chỉ hiện khi chọn Có
                    if form.tsgd_mac_benh == 1 {
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Ghi rõ tên bệnh / mã ICD-10 tiền sử gia đình:").strong().size(12.0).color(Color32::from_rgb(180, 83, 9)));
                            SearchCombo::show(
                                ui,
                                "cb_tsgd_icd10",
                                &mut form.tsgd_ma_benh,
                                catalogs,
                                "icd10",
                                400.0,
                                "Chọn hoặc nhập bệnh lý gia đình...",
                            );
                        });
                    }

                    ui.add_space(14.0);
                    ui.separator();
                    ui.add_space(10.0);

                    // ===================================================================
                    // 2. TIỀN SỬ BẢN THÂN (2 CỘT SONG SONG THEO ĐÚNG UI THIẾT KẾ)
                    // ===================================================================
                    ui.heading(RichText::new("2. Tiền sử bản thân:").size(14.0).strong().color(Color32::from_rgb(14, 165, 233)));
                    ui.add_space(6.0);

                    // Danh sách 22 câu hỏi tiền sử
                    // Cột 1: STT 1 -> 11
                    // Cột 2: STT 12 -> 22
                    ui.columns(2, |columns| {
                        // CỘT 1 (1 - 11)
                        let ui1 = &mut columns[0];
                        egui::Grid::new("grid_tsbt_col1")
                            .striped(true)
                            .min_col_width(20.0)
                            .spacing([10.0, 8.0])
                            .show(ui1, |ui| {
                                ui.label(RichText::new("STT").strong().size(11.5).color(Color32::from_rgb(100, 116, 139)));
                                ui.label(RichText::new("Tiền sử bệnh lý").strong().size(11.5).color(Color32::from_rgb(100, 116, 139)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new("Chọn").strong().size(11.5).color(Color32::from_rgb(100, 116, 139)));
                                });
                                ui.end_row();

                                Self::render_tsbt_row(ui, 1, "Có bệnh hay bị thương trong 5 năm qua", &mut form.tsbt_benh_trong_5_nam_qua);
                                Self::render_tsbt_row(ui, 2, "Có bệnh thần kinh hay bị thương ở đầu", &mut form.tsbt_benh_than_kinh);
                                Self::render_tsbt_row(ui, 3, "Bệnh mắt hoặc giảm thị lực (trừ trường hợp đeo kính thuốc)", &mut form.tsbt_benh_mat);
                                Self::render_tsbt_row(ui, 4, "Bệnh ở tai, giảm sức nghe hoặc thăng bằng", &mut form.tsbt_benh_tai);
                                Self::render_tsbt_row(ui, 5, "Bệnh ở tim, hoặc nhồi máu cơ tim, các bệnh tim mạch khác", &mut form.tsbt_benh_tim);
                                Self::render_tsbt_row(ui, 6, "Phẫu thuật can thiệp tim - mạch (thay van, bắc cầu nối, tạo hình mạch, máy tạo nhịp tim, đặt stent, ghép tim)", &mut form.tsbt_phau_thuat_tim);
                                Self::render_tsbt_row(ui, 7, "Tăng huyết áp", &mut form.tsbt_tang_huyet_ap);
                                Self::render_tsbt_row(ui, 8, "Khó thở", &mut form.tsbt_kho_tho);
                                Self::render_tsbt_row(ui, 9, "Bệnh phổi, hen, khí phế thũng, viêm phế quản mạn tính", &mut form.tsbt_benh_phoi);
                                Self::render_tsbt_row(ui, 10, "Bệnh thận, lọc máu", &mut form.tsbt_benh_than);
                                Self::render_tsbt_row(ui, 11, "Nghiện rượu, bia", &mut form.tsbt_nghien_ruou);
                            });

                        // CỘT 2 (12 - 22)
                        let ui2 = &mut columns[1];
                        egui::Grid::new("grid_tsbt_col2")
                            .striped(true)
                            .min_col_width(20.0)
                            .spacing([10.0, 8.0])
                            .show(ui2, |ui| {
                                ui.label(RichText::new("STT").strong().size(11.5).color(Color32::from_rgb(100, 116, 139)));
                                ui.label(RichText::new("Tiền sử bệnh lý").strong().size(11.5).color(Color32::from_rgb(100, 116, 139)));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(RichText::new("Chọn").strong().size(11.5).color(Color32::from_rgb(100, 116, 139)));
                                });
                                ui.end_row();

                                Self::render_tsbt_row(ui, 12, "Đái tháo đường hoặc kiểm soát tăng đường huyết", &mut form.tsbt_dai_thao_duong);
                                Self::render_tsbt_row(ui, 13, "Bệnh tâm thần", &mut form.tsbt_benh_tam_than);
                                Self::render_tsbt_row(ui, 14, "Mất ý thức, rối loạn ý thức", &mut form.tsbt_mat_y_thuc);
                                Self::render_tsbt_row(ui, 15, "Ngất, chóng mặt", &mut form.tsbt_ngat);
                                Self::render_tsbt_row(ui, 16, "Bệnh tiêu hóa", &mut form.tsbt_benh_tieu_hoa);
                                Self::render_tsbt_row(ui, 17, "Rối loạn giấc ngủ, ngừng thở khi ngủ, ngủ rũ ban ngày, ngáy to", &mut form.tsbt_roi_loan_giac_ngu);
                                Self::render_tsbt_row(ui, 18, "Tai biến mạch máu não hoặc liệt", &mut form.tsbt_tai_bien);
                                Self::render_tsbt_row(ui, 19, "Bệnh hoặc tổn thương cột sống", &mut form.tsbt_benh_cot_song);
                                Self::render_tsbt_row(ui, 20, "Sử dụng rượu thường xuyên, liên tục", &mut form.tsbt_ruou_thuong_xuyen);
                                Self::render_tsbt_row(ui, 21, "Sử dụng ma túy và chất gây nghiện", &mut form.tsbt_ma_tuy);
                                Self::render_tsbt_row(ui, 22, "Bệnh khác (ghi rõ)", &mut form.tsbt_benh_khac);
                            });
                    });

                    // Chi tiết bệnh khác nếu chọn Có ở mục 22
                    if form.tsbt_benh_khac == 1 {
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("↳ Ghi cụ thể tên bệnh đang mắc hoặc đã mắc (nếu có):").size(12.0).color(Color32::from_rgb(180, 83, 9)).strong());
                            ui.add(egui::TextEdit::singleline(&mut form.tsbt_ma_benh_khac).hint_text("Nhập tên bệnh khác...").desired_width(500.0));
                        });
                    }

                    ui.add_space(14.0);
                    ui.separator();
                    ui.add_space(10.0);

                    // ===================================================================
                    // 3. CÁC CÂU HỎI KHÁC (NẾU CÓ)
                    // ===================================================================
                    ui.heading(RichText::new("3. Các câu hỏi khác (nếu có):").size(14.0).strong().color(Color32::from_rgb(14, 165, 233)));
                    ui.add_space(8.0);

                    // a. Hiện tại có đang điều trị bệnh gì không?
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("a. Hiện tại có đang điều trị bệnh gì không?").strong().size(12.5));
                        ui.add_space(16.0);
                        ui.radio_value(&mut form.tsbt_dang_dieu_tri_benh, 0, "Không");
                        ui.radio_value(&mut form.tsbt_dang_dieu_tri_benh, 1, "Có");
                    });

                    if form.tsbt_dang_dieu_tri_benh == 1 {
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(RichText::new("Tên bệnh đang điều trị (Mã bệnh ICD-10)").size(11.5).color(Color32::from_rgb(100, 116, 139)));
                                ui.add(egui::TextEdit::singleline(&mut form.tsbt_ma_benh).hint_text("Nhập tên bệnh đang điều trị").desired_width(320.0));
                            });

                            ui.add_space(20.0);

                            ui.vertical(|ui| {
                                ui.label(RichText::new("Thuốc đang sử dụng và liều lượng").size(11.5).color(Color32::from_rgb(100, 116, 139)));
                                ui.add(egui::TextEdit::singleline(&mut form.tsbt_ten_thuoc_lieu_luong).hint_text("Nhập tên thuốc đang dùng").desired_width(320.0));
                            });
                        });
                    }

                    ui.add_space(10.0);

                    // b. Tiền sử thai sản (CHỈ HIỆN KHI GIỚI TÍNH LÀ NỮ: gioi_tinh == 2)
                    if form.gioi_tinh == 2 {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("b. Tiền sử thai sản").strong().size(12.5));
                            ui.add_space(16.0);
                            ui.radio_value(&mut form.tsbt_thai_san, 0, "Không");
                            ui.radio_value(&mut form.tsbt_thai_san, 1, "Có");
                        });

                        if form.tsbt_thai_san == 1 {
                            ui.add_space(6.0);
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(RichText::new("Cụ thể tên bệnh thai sản (Mã bệnh ICD-10)").size(11.5).color(Color32::from_rgb(100, 116, 139)));
                                    ui.add(egui::TextEdit::singleline(&mut form.tsbt_ma_benh_thai_san).hint_text("Nhập tên bệnh sản khoa").desired_width(320.0));
                                });

                                ui.add_space(20.0);

                                ui.vertical(|ui| {
                                    ui.label(RichText::new("Thuốc đang sử dụng điều trị thai sản").size(11.5).color(Color32::from_rgb(100, 116, 139)));
                                    ui.add(egui::TextEdit::singleline(&mut form.tsbt_ten_thuoc_thai_san).hint_text("Nhập thuốc đang sử dụng").desired_width(320.0));
                                });
                            });
                        }
                    }

                    ui.add_space(20.0);
                });
        });
    }

    fn render_tsbt_row(ui: &mut Ui, stt: usize, label: &str, val: &mut i32) {
        ui.label(stt.to_string());
        ui.label(RichText::new(label).size(12.0));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.radio_value(val, 1, "Có");
            ui.radio_value(val, 0, "Không");
        });
        ui.end_row();
    }
}
