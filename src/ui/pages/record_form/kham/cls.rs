use egui::{Color32, RichText, Ui};
use crate::models::record::{ClsItem, KskRecordPayload};

pub struct ClsView;

impl ClsView {
    pub fn show(ui: &mut Ui, form: &mut KskRecordPayload, is_locked: bool) {
        ui.add_enabled_ui(!is_locked, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add_space(4.0);
                    ui.heading(RichText::new("KHÁM CẬN LÂM SÀNG").size(15.0).strong().color(Color32::from_rgb(14, 165, 233)));
                    ui.label(RichText::new("Ghi nhận danh sách các chỉ định xét nghiệm, chẩn đoán hình ảnh và thăm dò chức năng.")
                        .size(11.5)
                        .color(Color32::from_rgb(100, 116, 139)));

                    ui.add_space(14.0);

                    // Danh sách xét nghiệm cận lâm sàng
                    ui.label(RichText::new("DANH SÁCH CHỈ ĐỊNH CẬN LÂM SÀNG (CLS):").strong().size(12.5));
                    ui.add_space(6.0);

                    let mut remove_idx = None;
                    for (i, item) in form.cls_list.iter_mut().enumerate() {
                        ui.push_id(i, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(format!("{}.", i + 1));
                                ui.add(egui::TextEdit::singleline(&mut item.ma_dich_vu).hint_text("Mã dịch vụ").desired_width(90.0));
                                ui.add(egui::TextEdit::singleline(&mut item.ma_chi_so).hint_text("Mã / Tên chỉ số").desired_width(180.0));
                                ui.add(egui::TextEdit::singleline(&mut item.gia_tri).hint_text("Kết quả đo").desired_width(100.0));
                                ui.add(egui::TextEdit::singleline(&mut item.don_vi_do).hint_text("Đơn vị").desired_width(70.0));
                                ui.add(egui::TextEdit::singleline(&mut item.mo_ta).hint_text("Mô tả / Đánh giá").desired_width(200.0));
                                ui.add(egui::TextEdit::singleline(&mut item.ket_luan).hint_text("Kết luận").desired_width(120.0));

                                if ui.button(RichText::new("Xóa").size(11.0).color(Color32::from_rgb(239, 68, 68))).clicked() {
                                    remove_idx = Some(i);
                                }
                            });
                        });
                        ui.add_space(3.0);
                    }

                    if let Some(idx) = remove_idx {
                        form.cls_list.remove(idx);
                    }

                    ui.add_space(6.0);
                    if ui.button(RichText::new("➕ Thêm chỉ định CLS mới").strong().size(12.0).color(Color32::from_rgb(14, 165, 233))).clicked() {
                        form.cls_list.push(ClsItem {
                            ma_dich_vu: "".into(),
                            ma_chi_so: "".into(),
                            gia_tri: "".into(),
                            don_vi_do: "".into(),
                            mo_ta: "".into(),
                            ket_luan: "Bình thường".into(),
                        });
                    }

                    ui.add_space(20.0);
                });
        });
    }
}
