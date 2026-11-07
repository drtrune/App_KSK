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
                    ui.horizontal(|ui| {
                        ui.heading(
                            RichText::new("III. KHÁM CẬN LÂM SÀNG (CHUẨN MẪU 03 & SHEET DM_CanLamSang)")
                                .size(15.0)
                                .strong()
                                .color(Color32::from_rgb(14, 165, 233)),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("🔄 Đồng bộ sang bảng DM_CanLamSang").clicked() {
                                form.sync_quick_cls_to_list();
                            }
                        });
                    });
                    ui.label(
                        RichText::new("Nhập nhanh kết quả theo tờ phiếu giấy Mẫu 03; dữ liệu tự động đồng bộ sang bảng DM_CanLamSang khi lưu và xuất Excel/XML.")
                            .size(11.5)
                            .color(Color32::from_rgb(100, 116, 139)),
                    );

                    ui.add_space(10.0);

                    // KHỐI 1: NHẬP NHANH THEO TỜ PHIẾU GIẤY MẪU 03
                    egui::Frame::none()
                        .fill(Color32::from_rgba_unmultiplied(14, 165, 233, 12))
                        .stroke(egui::Stroke::new(1.0_f32, Color32::from_rgb(186, 230, 253)))
                        .rounding(8.0)
                        .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                        .show(ui, |ui| {
                            ui.label(RichText::new("1. KẾT QUẢ CẬN LÂM SÀNG TRÊN PHIẾU MẪU 03:").strong().size(12.5));
                            ui.add_space(6.0);

                            egui::Grid::new("mau03_cls_quick_grid")
                                .num_columns(4)
                                .spacing([14.0, 8.0])
                                .show(ui, |ui| {
                                    ui.label(RichText::new("a) Huyết học (CTM):").strong().size(12.0));
                                    ui.add(egui::TextEdit::singleline(&mut form.cls_huyet_hoc).desired_width(220.0).hint_text("VD: Bình thường"));
                                    ui.label(RichText::new("b) Nước tiểu (TPTNT):").strong().size(12.0));
                                    ui.add(egui::TextEdit::singleline(&mut form.cls_nuoc_tieu).desired_width(220.0).hint_text("VD: Bình thường"));
                                    ui.end_row();

                                    ui.label(RichText::new("- Đường máu (Glucose):").size(12.0));
                                    ui.add(egui::TextEdit::singleline(&mut form.cls_duong_mau).desired_width(140.0).hint_text("mmol/l (VD: 5.2)"));
                                    ui.label(RichText::new("- Urê máu:").size(12.0));
                                    ui.add(egui::TextEdit::singleline(&mut form.cls_ure).desired_width(140.0).hint_text("mmol/l (VD: 4.8)"));
                                    ui.end_row();

                                    ui.label(RichText::new("- Creatinin máu:").size(12.0));
                                    ui.add(egui::TextEdit::singleline(&mut form.cls_creatinin).desired_width(140.0).hint_text("µmol/l (VD: 78)"));
                                    ui.label(RichText::new("- ASAT (GOT) / ALAT (GPT):").size(12.0));
                                    ui.horizontal(|ui| {
                                        ui.add(egui::TextEdit::singleline(&mut form.cls_asat_got).desired_width(80.0).hint_text("GOT (U/L)"));
                                        ui.label("/");
                                        ui.add(egui::TextEdit::singleline(&mut form.cls_alat_gpt).desired_width(80.0).hint_text("GPT (U/L)"));
                                    });
                                    ui.end_row();

                                    ui.label(RichText::new("c) XQ tim phổi thẳng:").strong().size(12.0));
                                    ui.add(egui::TextEdit::singleline(&mut form.cls_xq_tim_phoi).desired_width(220.0).hint_text("VD: Tim phổi bình thường"));
                                    ui.label(RichText::new("d) CLS khác (Siêu âm/ĐT...):").strong().size(12.0));
                                    ui.add(egui::TextEdit::singleline(&mut form.cls_khac_mo_ta).desired_width(220.0).hint_text("Ghi chi tiết nếu có..."));
                                    ui.end_row();
                                });
                        });

                    ui.add_space(14.0);

                    // KHỐI 2: BẢNG CHI TIẾT SHEET DM_CanLamSang (XUẤT EXCEL & XML)
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("2. BẢNG DỊCH VỤ CẬN LÂM SÀNG CHI TIẾT (SHEET DM_CanLamSang):").strong().size(12.5));
                        ui.add_space(8.0);
                        let presets = [
                            ("H01", "Số lượng hồng cầu", "x10^12/lít", "4.6"),
                            ("H02", "Huyết sắc tố (Hb)", "g/lít", "142"),
                            ("S01", "Đường máu (Glucose)", "mmol/l", "5.1"),
                            ("S02", "Urê máu", "mmol/l", "4.5"),
                            ("S03", "Creatinin máu", "µmol/l", "75"),
                            ("S04", "ASAT (GOT)", "U/L", "24"),
                            ("S05", "ALAT (GPT)", "U/L", "26"),
                            ("XQ01", "X-Quang tim phổi thẳng", "", "Bình thường"),
                            ("SA01", "Siêu âm bụng tổng quát", "", "Bình thường"),
                        ];
                        egui::ComboBox::from_id_source("preset_cls_picker")
                            .selected_text("⚡ Thêm nhanh chỉ số chuẩn...")
                            .show_ui(ui, |ui| {
                                for (code, name, unit, def_val) in presets {
                                    if ui.selectable_label(false, format!("{} - {}", code, name)).clicked() {
                                        form.cls_list.push(ClsItem {
                                            ma_dich_vu: code.into(),
                                            ma_chi_so: name.into(),
                                            gia_tri: def_val.into(),
                                            don_vi_do: unit.into(),
                                            mo_ta: "Trong giới hạn bình thường".into(),
                                            ket_luan: "Bình thường".into(),
                                        });
                                    }
                                }
                            });
                    });

                    ui.add_space(6.0);

                    let mut remove_idx = None;
                    egui::Grid::new("cls_list_table_grid")
                        .striped(true)
                        .spacing([8.0, 6.0])
                        .show(ui, |ui| {
                            ui.label(RichText::new("TT").strong().size(11.5));
                            ui.label(RichText::new("DV Cận lâm sàng (MA_DICH_VU)").strong().size(11.5));
                            ui.label(RichText::new("Tên/Mã chỉ số (MA_CHI_SO)").strong().size(11.5));
                            ui.label(RichText::new("Đơn vị (DON_VI_DO)").strong().size(11.5));
                            ui.label(RichText::new("Giá trị (GIA_TRI)").strong().size(11.5));
                            ui.label(RichText::new("Mô tả (MO_TA)").strong().size(11.5));
                            ui.label(RichText::new("Kết luận (KET_LUAN)").strong().size(11.5));
                            ui.label("");
                            ui.end_row();

                            for (i, item) in form.cls_list.iter_mut().enumerate() {
                                ui.push_id(i, |ui| {
                                    ui.label(format!("{}", i + 1));
                                    ui.add(egui::TextEdit::singleline(&mut item.ma_dich_vu).hint_text("Mã DV").desired_width(90.0));
                                    ui.add(egui::TextEdit::singleline(&mut item.ma_chi_so).hint_text("Tên chỉ số").desired_width(170.0));
                                    ui.add(egui::TextEdit::singleline(&mut item.don_vi_do).hint_text("Đơn vị").desired_width(70.0));
                                    ui.add(egui::TextEdit::singleline(&mut item.gia_tri).hint_text("Giá trị").desired_width(90.0));
                                    ui.add(egui::TextEdit::singleline(&mut item.mo_ta).hint_text("Mô tả KQ").desired_width(150.0));
                                    ui.add(egui::TextEdit::singleline(&mut item.ket_luan).hint_text("Kết luận").desired_width(110.0));

                                    if ui.button(RichText::new("✖").size(11.0).color(Color32::from_rgb(239, 68, 68))).clicked() {
                                        remove_idx = Some(i);
                                    }
                                });
                                ui.end_row();
                            }
                        });

                    if let Some(idx) = remove_idx {
                        form.cls_list.remove(idx);
                    }

                    ui.add_space(8.0);
                    if ui.button(RichText::new("➕ Thêm dòng chỉ số CLS").strong().size(12.0).color(Color32::from_rgb(14, 165, 233))).clicked() {
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
