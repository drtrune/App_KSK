use egui::{Color32, RichText, Ui};
use crate::models::catalog::CatalogRecord;
use crate::models::record::KskRecordPayload;
use crate::ui::widgets::date_picker::DatePicker;
use crate::ui::widgets::search_combo::SearchCombo;

pub struct HanhChinhView;

impl HanhChinhView {
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
                    ui.heading(RichText::new("I. THÔNG TIN HÀNH CHÍNH").size(15.0).strong().color(Color32::from_rgb(14, 165, 233)));
                    ui.add_space(8.0);

                    // 1. Mã định danh & Ngày khám
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("1. Mã CSKCB:").strong().size(12.5));
                        ui.add(egui::TextEdit::singleline(&mut form.ma_cskcb).desired_width(90.0).interactive(false));

                        ui.add_space(14.0);
                        ui.label(RichText::new("Mã GLN:").strong().size(12.5));
                        ui.add(egui::TextEdit::singleline(&mut form.ma_gtin_cskcb).desired_width(120.0).interactive(false));

                        ui.add_space(14.0);
                        ui.label(RichText::new("Mã LK (*):").strong().size(12.5));
                        ui.add(egui::TextEdit::singleline(&mut form.ma_lk).desired_width(140.0));

                        ui.add_space(14.0);
                        ui.label(RichText::new("Ngày khám (*):").strong().size(12.5));
                        DatePicker::show(ui, "dp_ngay_vao", &mut form.ngay_vao);
                    });

                    ui.add_space(14.0);

                    // 2. Thông tin cá nhân
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("2. Họ và tên (*):").strong().size(12.5));
                        ui.add(egui::TextEdit::singleline(&mut form.ho_ten).desired_width(220.0).hint_text("Họ và tên..."));

                        ui.add_space(14.0);
                        ui.label(RichText::new("Giới tính (*):").strong().size(12.5));
                        ui.radio_value(&mut form.gioi_tinh, 1, "Nam");
                        ui.radio_value(&mut form.gioi_tinh, 2, "Nữ");

                        ui.add_space(14.0);
                        ui.label(RichText::new("Ngày sinh (*):").strong().size(12.5));
                        DatePicker::show(ui, "dp_ngay_sinh", &mut form.ngay_sinh);

                        ui.add_space(14.0);
                        ui.label(RichText::new("Dân tộc:").size(12.5));
                        SearchCombo::show(
                            ui,
                            "cb_dan_toc",
                            &mut form.ma_dan_toc,
                            catalogs,
                            "ethnicity",
                            130.0,
                            "Chọn dân tộc...",
                        );

                        ui.add_space(14.0);
                        ui.label(RichText::new("Nhóm máu:").size(12.5));
                        SearchCombo::show(
                            ui,
                            "cb_nhom_mau",
                            &mut form.nhom_mau,
                            catalogs,
                            "blood_type",
                            90.0,
                            "Nhóm máu...",
                        );
                    });

                    ui.add_space(14.0);

                    // 3. Giấy tờ định danh
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("3. Số CCCD/Mã định danh (*):").strong().size(12.5));
                        ui.add(egui::TextEdit::singleline(&mut form.so_cccd).desired_width(180.0).hint_text("Số CCCD 12 số"));

                        ui.add_space(14.0);
                        ui.label(RichText::new("Ngày cấp:").size(12.5));
                        DatePicker::show(ui, "dp_ngaycap_cccd", &mut form.ngaycap_cccd);

                        ui.add_space(14.0);
                        ui.label(RichText::new("Nơi cấp:").size(12.5));
                        SearchCombo::show(
                            ui,
                            "cb_noicap_cccd",
                            &mut form.noicap_cccd,
                            catalogs,
                            "id_issuer",
                            220.0,
                            "Chọn hoặc nhập nơi cấp...",
                        );
                    });

                    ui.add_space(14.0);

                    // 4. Nơi ở hiện tại (combo liền kề, đánh số rõ ràng)
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("4. Nơi ở hiện tại:").strong().size(12.5));

                        ui.label(RichText::new("Tỉnh/Thành phố (*):").size(12.0));
                        SearchCombo::show(
                            ui,
                            "cb_matinh",
                            &mut form.matinh_cu_tru,
                            catalogs,
                            "province",
                            160.0,
                            "Chọn Tỉnh/TP...",
                        );

                        ui.add_space(8.0);
                        ui.label(RichText::new("Xã/Phường (*):").size(12.0));
                        let prov_code = if form.matinh_cu_tru.trim().is_empty() { None } else { Some(form.matinh_cu_tru.as_str()) };
                        SearchCombo::show_filtered(
                            ui,
                            "cb_maxa",
                            &mut form.maxa_cu_tru,
                            catalogs,
                            "ward",
                            prov_code,
                            160.0,
                            "Chọn Xã/Phường...",
                        );

                        ui.add_space(8.0);
                        ui.label(RichText::new("Số nhà/thôn/xóm:").size(12.0));
                        ui.add(egui::TextEdit::singleline(&mut form.dia_chi).desired_width(260.0).hint_text("Số nhà, phố, thôn..."));
                    });

                    ui.add_space(14.0);

                    // 5. Nghề nghiệp, Nơi công tác, Liên hệ
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("5. Nghề nghiệp:").strong().size(12.5));
                        SearchCombo::show(
                            ui,
                            "cb_nghe_nghiep",
                            &mut form.ma_nghe_nghiep,
                            catalogs,
                            "occupation",
                            200.0,
                            "Chọn nghề nghiệp...",
                        );

                        ui.add_space(14.0);
                        ui.label(RichText::new("Nơi làm việc / học tập:").size(12.5));
                        ui.add(egui::TextEdit::singleline(&mut form.noi_lam_viec_hoc_tap).desired_width(240.0));

                        ui.add_space(14.0);
                        ui.label(RichText::new("Điện thoại:").size(12.5));
                        ui.add(egui::TextEdit::singleline(&mut form.dien_thoai).desired_width(120.0).hint_text("09xxxxxxxx"));
                    });

                    ui.add_space(14.0);

                    // 6. Thông tin khám & Chi trả
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("6. Đối tượng khám (*):").strong().size(12.5));
                        SearchCombo::show(
                            ui,
                            "cb_doi_tuong",
                            &mut form.doi_tuong,
                            catalogs,
                            "priority",
                            150.0,
                            "Chọn đối tượng...",
                        );

                        ui.add_space(14.0);
                        ui.label(RichText::new("Nguồn chi trả (*):").size(12.5));
                        egui::ComboBox::from_id_source("cb_nguon_chi_tra")
                            .width(160.0)
                            .selected_text(match form.nguon_chi_tra {
                                1 => "1. Ngân sách nhà nước",
                                2 => "2. Bảo hiểm y tế",
                                3 => "3. Thu trực tiếp từ người bệnh",
                                4 => "4. Người sử dụng lao động",
                                5 => "5. Nguồn khác",
                                _ => "9. Chưa xác định",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut form.nguon_chi_tra, 1, "1. Ngân sách nhà nước");
                                ui.selectable_value(&mut form.nguon_chi_tra, 2, "2. Bảo hiểm y tế");
                                ui.selectable_value(&mut form.nguon_chi_tra, 3, "3. Thu trực tiếp");
                                ui.selectable_value(&mut form.nguon_chi_tra, 4, "4. Người sử dụng lao động");
                                ui.selectable_value(&mut form.nguon_chi_tra, 5, "5. Nguồn khác");
                                ui.selectable_value(&mut form.nguon_chi_tra, 9, "9. Chưa xác định");
                            });

                        ui.add_space(14.0);
                        ui.label(RichText::new("Lý do khám:").size(12.5));
                        SearchCombo::show(
                            ui,
                            "cb_ly_do",
                            &mut form.ly_do_vv,
                            catalogs,
                            "exam_reason",
                            160.0,
                            "Chọn lý do khám...",
                        );

                        ui.add_space(14.0);
                        ui.label(RichText::new("Loại hình KCB:").size(12.5));
                        SearchCombo::show(
                            ui,
                            "cb_ma_loai_kcb",
                            &mut form.ma_loai_kcb,
                            catalogs,
                            "kcb_type",
                            140.0,
                            "Loại hình...",
                        );
                    });

                    ui.add_space(20.0);
                });
        });
    }
}
