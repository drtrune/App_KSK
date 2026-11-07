use egui::{Color32, RichText, Ui, Vec2};
use crate::auth::permissions::Perm;
use crate::models::{AttachmentRecord, RecordItem};
use crate::ui::widgets::badge::{ui_score_badge, ui_status_badge};

pub struct RecordsPage;

pub enum RecordsAction {
    None,
    New,
    Open(i64),
    OpenFastEntry(i64),
    PrintMau03(i64),
    Delete(i64),
    ToggleLock(i64, bool),
    ExportCsv,
    ExportXlsm,
    ExportXml,
    Refresh,
}

impl RecordsPage {
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        ui: &mut Ui,
        records: &[RecordItem],
        attachments: &[AttachmentRecord],
        search_query: &mut String,
        qr_scan_input: &mut String,
        status_filter: &mut String, // "all", "draft", "locked"
        user_perms: &[Perm],
        dark_mode: bool,
    ) -> RecordsAction {
        let mut action = RecordsAction::None;

        ui.vertical(|ui| {
            // Header bar
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.heading(RichText::new("DANH SÁCH HỒ SƠ KHÁM SỨC KHỎE (ĐOÀN & LẺ)").size(18.0).strong());
                    ui.label(
                        RichText::new(format!("Tổng số hồ sơ trong hệ thống: {}", records.len()))
                            .size(12.0)
                            .color(Color32::from_rgb(100, 116, 139)),
                    );
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if user_perms.contains(&Perm::RecordExport) {
                        if ui
                            .button(RichText::new("📄 Xuất XML 3176").size(12.0))
                            .on_hover_text("Xuất tệp XML chuẩn QĐ 3176/QĐ-BYT (Bao gồm DSACH_CLS)")
                            .clicked()
                        {
                            action = RecordsAction::ExportXml;
                        }

                        if ui
                            .button(RichText::new("📥 Xuất CSV").size(12.0))
                            .on_hover_text("Xuất CSV cho cả 2 bảng Trên 18 và DM_CanLamSang")
                            .clicked()
                        {
                            action = RecordsAction::ExportCsv;
                        }

                        let xlsm_btn = egui::Button::new(
                            RichText::new("📊 Xuất Import_KSK_Tren_18.xlsm")
                                .strong()
                                .size(12.0)
                                .color(Color32::WHITE),
                        )
                        .fill(Color32::from_rgb(16, 185, 129))
                        .rounding(6.0)
                        .min_size(Vec2::new(190.0, 30.0));
                        if ui
                            .add(xlsm_btn)
                            .on_hover_text("Xuất trực tiếp ra file Excel chuẩn Import_KSK_Tren_18.xlsm (Đủ 2 sheet Trên 18 & DM_CanLamSang)")
                            .clicked()
                        {
                            action = RecordsAction::ExportXlsm;
                        }
                    }

                    if user_perms.contains(&Perm::RecordCreate) {
                        let new_btn = egui::Button::new(
                            RichText::new("➕ Tạo hồ sơ lẻ")
                                .strong()
                                .size(12.0)
                                .color(Color32::WHITE),
                        )
                        .fill(Color32::from_rgb(14, 165, 233))
                        .rounding(6.0)
                        .min_size(Vec2::new(120.0, 30.0));
                        if ui.add(new_btn).clicked() {
                            action = RecordsAction::New;
                        }
                    }

                    if ui.button(RichText::new("🔄").size(13.0)).on_hover_text("Tải lại danh sách").clicked() {
                        action = RecordsAction::Refresh;
                    }
                });
            });

            ui.add_space(10.0);

            // THANH BẮN MÃ VẠCH TIẾP NHẬN / MÃ LK (NHẬP LIỆU NHANH 1 CỬA)
            egui::Frame::none()
                .fill(if dark_mode {
                    Color32::from_rgb(30, 41, 59)
                } else {
                    Color32::from_rgb(240, 249, 255)
                })
                .stroke(egui::Stroke::new(1.0_f32, Color32::from_rgb(14, 165, 233)))
                .rounding(8.0)
                .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("📷 BẮN MÃ VẠCH TIẾP NHẬN (BARCODE - F2):").strong().size(12.5).color(Color32::from_rgb(3, 105, 161)));
                        let resp = ui.add(
                            egui::TextEdit::singleline(qr_scan_input)
                                .hint_text("Đặt con trỏ tại đây và bắn mã vạch trên tờ phiếu khám Mẫu 03 (hoặc nhập Mã tiếp nhận / CCCD)...")
                                .desired_width(450.0),
                        );
                        let trigger_qr = (resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                            || ui.button(RichText::new("⚡ Mở Nhập 1 Cửa").strong().color(Color32::from_rgb(14, 165, 233))).clicked();

                        if trigger_qr && !qr_scan_input.trim().is_empty() {
                            let raw = qr_scan_input.trim();
                            // Hỗ trợ mã tiếp nhận (LK...), Mã Y Tế, CCCD hoặc format KSK|...
                            let parts: Vec<&str> = raw.split('|').collect();
                            let code_key = if parts.len() >= 2 && parts[0].eq_ignore_ascii_case("KSK") {
                                parts[1].trim().to_lowercase()
                            } else {
                                raw.to_lowercase()
                            };

                            if let Some(found) = records.iter().find(|r| {
                                r.data.ma_lk.to_lowercase() == code_key
                                    || r.ma_y_te.to_lowercase() == code_key
                                    || r.data.ma_y_te.to_lowercase() == code_key
                                    || (!r.so_cccd.is_empty() && r.so_cccd.to_lowercase() == code_key)
                            }) {
                                action = RecordsAction::OpenFastEntry(found.id);
                                qr_scan_input.clear();
                            }
                        }
                    });
                });

            ui.add_space(8.0);

            // Filter bar
            ui.horizontal(|ui| {
                ui.label(RichText::new("Tìm kiếm:").size(12.5));
                ui.add(
                    egui::TextEdit::singleline(search_query)
                        .hint_text("Tìm theo Mã tiếp nhận, Số lưu trữ, Họ tên, CCCD, Công ty...")
                        .desired_width(320.0),
                );

                if !search_query.is_empty() && ui.button("✖").clicked() {
                    search_query.clear();
                }

                ui.add_space(16.0);
                ui.label(RichText::new("Trạng thái:").size(12.5));
                ui.selectable_value(status_filter, "all".into(), "Tất cả");
                ui.selectable_value(status_filter, "draft".into(), "Chưa kết luận / Chưa cấp số");
                ui.selectable_value(status_filter, "locked".into(), "Đã kết luận & Cấp số");
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(6.0);

            // Filter records
            let q = search_query.trim().to_lowercase();
            let filtered: Vec<&RecordItem> = records
                .iter()
                .filter(|r| {
                    let matches_q = q.is_empty()
                        || r.ho_ten.to_lowercase().contains(&q)
                        || r.so_cccd.to_lowercase().contains(&q)
                        || r.ngay_vao.to_lowercase().contains(&q)
                        || r.data.ma_lk.to_lowercase().contains(&q)
                        || r.data.ma_y_te.to_lowercase().contains(&q)
                        || r.data.so_luu_tru.to_lowercase().contains(&q)
                        || r.data.noi_lam_viec_hoc_tap.to_lowercase().contains(&q);

                    let matches_status = match status_filter.as_str() {
                        "draft" => r.status == 0 && r.data.so_luu_tru.trim().is_empty(),
                        "locked" => r.status != 0 || !r.data.so_luu_tru.trim().is_empty(),
                        _ => true,
                    };

                    matches_q && matches_status
                })
                .collect();

            if filtered.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    ui.label(RichText::new("Không tìm thấy hồ sơ nào phù hợp.").size(13.0).color(Color32::from_rgb(148, 163, 184)));
                });
            } else {
                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        egui::Grid::new("records_grid")
                            .striped(true)
                            .min_col_width(55.0)
                            .spacing([12.0, 8.0])
                            .show(ui, |ui| {
                                ui.label(RichText::new("STT").strong().size(11.5));
                                ui.label(RichText::new("Số Lưu Trữ (GKSK)").strong().size(11.5));
                                ui.label(RichText::new("Mã Tiếp Nhận").strong().size(11.5));
                                ui.label(RichText::new("Họ và Tên").strong().size(11.5));
                                ui.label(RichText::new("GT").strong().size(11.5));
                                ui.label(RichText::new("Ngày sinh").strong().size(11.5));
                                ui.label(RichText::new("Số CCCD").strong().size(11.5));
                                ui.label(RichText::new("Đơn vị / Công ty").strong().size(11.5));
                                ui.label(RichText::new("Loại SK").strong().size(11.5));
                                ui.label(RichText::new("Scan").strong().size(11.5));
                                ui.label(RichText::new("Trạng thái").strong().size(11.5));
                                ui.label(RichText::new("Thao tác").strong().size(11.5));
                                ui.end_row();

                                for (i, r) in filtered.iter().enumerate() {
                                    ui.label((i + 1).to_string());
                                    if r.data.so_luu_tru.trim().is_empty() {
                                        ui.label(RichText::new("Chưa cấp số").size(11.0).color(Color32::from_rgb(148, 163, 184)));
                                    } else {
                                        ui.label(RichText::new(&r.data.so_luu_tru).monospace().strong().size(12.0).color(Color32::from_rgb(5, 150, 105)));
                                    }
                                    let ma_tn = if !r.data.ma_lk.is_empty() { &r.data.ma_lk } else { &r.data.ma_y_te };
                                    ui.label(RichText::new(ma_tn).monospace().size(11.5));
                                    ui.label(RichText::new(&r.ho_ten).strong().size(12.0));
                                    ui.label(if r.data.gioi_tinh == 2 { "Nữ" } else { "Nam" });
                                    ui.label(&r.data.ngay_sinh);
                                    ui.label(&r.so_cccd);
                                    ui.label(if r.data.noi_lam_viec_hoc_tap.is_empty() { "Khám lẻ" } else { &r.data.noi_lam_viec_hoc_tap });

                                    ui_score_badge(ui, r.data.phan_loai_sk, false);

                                    let scan_count = attachments.iter().filter(|a| a.owner_type == "record" && a.owner_id == r.id).count();
                                    if scan_count > 0 {
                                        ui.label(RichText::new(format!("📎 {}", scan_count)).color(Color32::from_rgb(16, 185, 129)).strong());
                                    } else {
                                        ui.label(RichText::new("-").color(Color32::from_rgb(148, 163, 184)));
                                    }

                                    ui_status_badge(ui, r.status != 0);

                                    ui.horizontal(|ui| {
                                        if ui.button(RichText::new("⚡ Nhập 1 cửa").size(11.0).strong().color(Color32::from_rgb(14, 165, 233))).clicked() {
                                            action = RecordsAction::OpenFastEntry(r.id);
                                        }
                                        if ui.button(RichText::new("Chi tiết").size(11.0)).clicked() {
                                            action = RecordsAction::Open(r.id);
                                        }
                                        if ui.button(RichText::new("🖨 Mẫu 03").size(11.0)).on_hover_text("Xuất phiếu Word Mẫu 03 kèm QR").clicked() {
                                            action = RecordsAction::PrintMau03(r.id);
                                        }

                                        if user_perms.contains(&Perm::RecordDelete) && r.status == 0 {
                                            if ui.button(RichText::new("Xóa").size(11.0).color(Color32::from_rgb(239, 68, 68))).clicked() {
                                                action = RecordsAction::Delete(r.id);
                                            }
                                        }
                                    });

                                    ui.end_row();
                                }
                            });
                    });
            }
        });

        action
    }
}
