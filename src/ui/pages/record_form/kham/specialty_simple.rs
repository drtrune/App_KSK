use egui::{Color32, RichText, Ui};
use crate::domain::update_global_health_score;
use crate::models::record::KskRecordPayload;
use crate::ui::widgets::badge::ui_score_badge;

pub struct SpecialtySimpleView;

impl SpecialtySimpleView {
    pub fn show(
        ui: &mut Ui,
        form: &mut KskRecordPayload,
        spec_id: &str,
        spec_name: &str,
        is_locked: bool,
    ) {
        ui.add_enabled_ui(!is_locked, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.heading(RichText::new(format!("Chuyên khoa: {} [Chế độ đơn giản]", spec_name)).size(15.0).strong().color(Color32::from_rgb(14, 165, 233)));
                    });
                    ui.label(RichText::new("Chế độ nhập nhanh dành cho bác sĩ ghi nhận kết quả trực tiếp hoặc chọn mẫu có sẵn.")
                        .size(11.5)
                        .color(Color32::from_rgb(100, 116, 139)));

                    ui.add_space(14.0);

                    let (mut desc, mut score) = Self::get_values(form, spec_id);
                    let mut changed = false;

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Phân loại kết quả (*):").strong().size(13.0));
                        ui.add_space(8.0);
                        for i in 1..=5 {
                            if ui.radio_value(&mut score, i, format!("Loại {}", i)).changed() {
                                changed = true;
                            }
                        }
                        ui_score_badge(ui, score, false);
                    });

                    ui.add_space(14.0);

                    // Quick Template Buttons
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Mẫu nhanh:").size(12.0).color(Color32::from_rgb(100, 116, 139)));
                        if ui.button("Bình thường").clicked() {
                            desc = "Bình thường".into();
                            score = 1;
                            changed = true;
                        }
                        if ui.button("Chưa phát hiện bất thường").clicked() {
                            desc = "Chưa phát hiện bất thường".into();
                            score = 1;
                            changed = true;
                        }
                        if ui.button("Viêm nhẹ").clicked() {
                            desc = "Viêm nhẹ, theo dõi điều trị ngoại trú".into();
                            score = 2;
                            changed = true;
                        }
                    });

                    ui.add_space(10.0);

                    // Nội dung mô tả bệnh lý
                    ui.label(RichText::new("Mô tả bệnh lý / Kết quả khám:").strong().size(12.5));
                    let text_resp = ui.add(
                        egui::TextEdit::multiline(&mut desc)
                            .desired_rows(4)
                            .desired_width(ui.available_width() - 20.0)
                            .hint_text("Nhập mô tả kết quả khám bệnh hoặc chỉ định theo dõi..."),
                    );
                    if text_resp.changed() {
                        changed = true;
                    }

                    if changed {
                        Self::set_values(form, spec_id, desc, score);
                        update_global_health_score(form);
                    }

                    ui.add_space(20.0);
                });
        });
    }

    fn get_values(form: &KskRecordPayload, spec_id: &str) -> (String, i32) {
        match spec_id {
            "tuan_hoan" => (form.noi_khoa_tuan_hoan.clone(), form.noi_khoa_tuan_hoan_pl),
            "ho_hap" => (form.noi_khoa_ho_hap.clone(), form.noi_khoa_ho_hap_pl),
            "tieu_hoa" => (form.noi_khoa_tieu_hoa.clone(), form.noi_khoa_tieu_hoa_pl),
            "than_tiet_nieu" => (form.noi_khoa_than_tn_sd.clone(), form.noi_khoa_than_tn_sd_pl),
            "noi_tiet" => (form.noi_khoa_noi_tiet.clone(), form.noi_khoa_noi_tiet_pl),
            "co_xuong_khop" => (form.noi_khoa_co_xuong_khop.clone(), form.noi_khoa_co_xuong_khop_pl),
            "than_kinh" => (form.noi_khoa_than_kinh.clone(), form.noi_khoa_than_kinh_pl),
            "tam_than" => (form.noi_khoa_tam_than.clone(), form.noi_khoa_tam_than_pl),
            "ngoai_khoa" => (form.ket_qua_kham_ngoai_khoa.clone(), form.kham_ngoai_khoa_pl),
            "san_phu_khoa" => (form.ket_qua_kham_san_phu_khoa.clone(), form.kham_san_phu_khoa_pl),
            "mat" => (form.benh_khac_mat.clone(), form.kham_mat_pl),
            "tai_mui_hong" => (form.benh_khac_tai_mui_hong.clone(), form.kham_tai_mui_hong_pl),
            "rang_ham_mat" => (form.benh_khac_rang_ham_mat.clone(), form.kham_rang_ham_mat_pl),
            "da_lieu" => (form.ket_qua_kham_da_lieu.clone(), form.kham_da_lieu_pl),
            _ => (form.ket_qua_kham_ngoai_khoa.clone(), form.kham_ngoai_khoa_pl),
        }
    }

    fn set_values(form: &mut KskRecordPayload, spec_id: &str, desc: String, score: i32) {
        match spec_id {
            "tuan_hoan" => { form.noi_khoa_tuan_hoan = desc; form.noi_khoa_tuan_hoan_pl = score; }
            "ho_hap" => { form.noi_khoa_ho_hap = desc; form.noi_khoa_ho_hap_pl = score; }
            "tieu_hoa" => { form.noi_khoa_tieu_hoa = desc; form.noi_khoa_tieu_hoa_pl = score; }
            "than_tiet_nieu" => { form.noi_khoa_than_tn_sd = desc; form.noi_khoa_than_tn_sd_pl = score; }
            "noi_tiet" => { form.noi_khoa_noi_tiet = desc; form.noi_khoa_noi_tiet_pl = score; }
            "co_xuong_khop" => { form.noi_khoa_co_xuong_khop = desc; form.noi_khoa_co_xuong_khop_pl = score; }
            "than_kinh" => { form.noi_khoa_than_kinh = desc; form.noi_khoa_than_kinh_pl = score; }
            "tam_than" => { form.noi_khoa_tam_than = desc; form.noi_khoa_tam_than_pl = score; }
            "ngoai_khoa" => { form.ket_qua_kham_ngoai_khoa = desc; form.kham_ngoai_khoa_pl = score; }
            "san_phu_khoa" => { form.ket_qua_kham_san_phu_khoa = desc; form.kham_san_phu_khoa_pl = score; }
            "mat" => { form.benh_khac_mat = desc; form.kham_mat_pl = score; }
            "tai_mui_hong" => { form.benh_khac_tai_mui_hong = desc; form.kham_tai_mui_hong_pl = score; }
            "rang_ham_mat" => { form.benh_khac_rang_ham_mat = desc; form.kham_rang_ham_mat_pl = score; }
            "da_lieu" => { form.ket_qua_kham_da_lieu = desc; form.kham_da_lieu_pl = score; }
            _ => { form.ket_qua_kham_ngoai_khoa = desc; form.kham_ngoai_khoa_pl = score; }
        }
    }
}
