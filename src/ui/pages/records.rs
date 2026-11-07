use egui::{Color32, RichText, Ui, Vec2};
use crate::auth::permissions::Perm;
use crate::models::RecordItem;
use crate::ui::widgets::badge::{ui_score_badge, ui_status_badge};

pub struct RecordsPage;

pub enum RecordsAction {
    None,
    New,
    Open(i64),
    Delete(i64),
    ToggleLock(i64, bool),
    ExportCsv,
    Refresh,
}

impl RecordsPage {
    pub fn show(
        ui: &mut Ui,
        records: &[RecordItem],
        search_query: &mut String,
        status_filter: &mut String, // "all", "draft", "locked"
        user_perms: &[Perm],
        dark_mode: bool,
    ) -> RecordsAction {
        let mut action = RecordsAction::None;

        ui.vertical(|ui| {
            // Header bar
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.heading(RichText::new("DANH SÁCH HỒ SƠ KHÁM SỨC KHỎE").size(18.0).strong());
                    ui.label(RichText::new(format!("Tổng số hồ sơ trong hệ thống: {}", records.len()))
                        .size(12.0)
                        .color(Color32::from_rgb(100, 116, 139)));
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if user_perms.contains(&Perm::RecordExport) {
                        let exp_btn = egui::Button::new(RichText::new("📥  Xuất CSV (97 cột)").size(12.0))
                            .fill(if dark_mode { Color32::from_rgb(30, 41, 59) } else { Color32::from_rgb(241, 245, 249) })
                            .rounding(6.0)
                            .min_size(Vec2::new(140.0, 32.0));
                        if ui.add(exp_btn).on_hover_text("Xuất toàn bộ dữ liệu ra tệp CSV 97 cột chuẩn Bộ Y Tế").clicked() {
                            action = RecordsAction::ExportCsv;
                        }
                    }

                    if user_perms.contains(&Perm::RecordCreate) {
                        let new_btn = egui::Button::new(RichText::new("➕  Tạo hồ sơ mới").strong().size(12.5).color(Color32::WHITE))
                            .fill(Color32::from_rgb(14, 165, 233))
                            .rounding(6.0)
                            .min_size(Vec2::new(130.0, 32.0));
                        if ui.add(new_btn).clicked() {
                            action = RecordsAction::New;
                        }
                    }

                    if ui.button(RichText::new("🔄").size(13.0)).on_hover_text("Tải lại danh sách").clicked() {
                        action = RecordsAction::Refresh;
                    }
                });
            });

            ui.add_space(12.0);

            // Filter bar
            ui.horizontal(|ui| {
                ui.label(RichText::new("Tìm kiếm:").size(12.5));
                ui.add(egui::TextEdit::singleline(search_query)
                    .hint_text("Tìm theo Mã LK, Họ tên, CCCD, Ngày khám...")
                    .desired_width(320.0));

                if !search_query.is_empty() && ui.button("✖").clicked() {
                    search_query.clear();
                }

                ui.add_space(16.0);
                ui.label(RichText::new("Trạng thái:").size(12.5));
                ui.selectable_value(status_filter, "all".into(), "Tất cả");
                ui.selectable_value(status_filter, "draft".into(), "Đang soạn");
                ui.selectable_value(status_filter, "locked".into(), "Đã khóa");
            });

            ui.add_space(10.0);
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
                        || r.data.ma_lk.to_lowercase().contains(&q);

                    let matches_status = match status_filter.as_str() {
                        "draft" => r.status == 0,
                        "locked" => r.status != 0,
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
                // Table view
                egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    egui::Grid::new("records_grid")
                        .striped(true)
                        .min_col_width(70.0)
                        .spacing([14.0, 10.0])
                        .show(ui, |ui| {
                            // Table Header
                            ui.label(RichText::new("STT").strong().size(12.0));
                            ui.label(RichText::new("Mã LK").strong().size(12.0));
                            ui.label(RichText::new("Họ và Tên").strong().size(12.0));
                            ui.label(RichText::new("Giới tính").strong().size(12.0));
                            ui.label(RichText::new("Ngày sinh").strong().size(12.0));
                            ui.label(RichText::new("Số CCCD").strong().size(12.0));
                            ui.label(RichText::new("Ngày khám").strong().size(12.0));
                            ui.label(RichText::new("Kết luận SK").strong().size(12.0));
                            ui.label(RichText::new("Trạng thái").strong().size(12.0));
                            ui.label(RichText::new("Thao tác").strong().size(12.0));
                            ui.end_row();

                            for (i, r) in filtered.iter().enumerate() {
                                ui.label((i + 1).to_string());
                                ui.label(RichText::new(&r.data.ma_lk).monospace().size(12.0).strong());
                                ui.label(RichText::new(&r.ho_ten).strong().size(12.5));
                                ui.label(if r.data.gioi_tinh == 2 { "Nữ" } else { "Nam" });
                                ui.label(&r.data.ngay_sinh);
                                ui.label(&r.so_cccd);
                                ui.label(&r.ngay_vao);

                                // Health classification badge
                                ui_score_badge(ui, r.data.phan_loai_sk, false);

                                // Status badge
                                ui_status_badge(ui, r.status != 0);

                                // Action buttons
                                ui.horizontal(|ui| {
                                    if ui.button(RichText::new("Mở khám").size(11.5).color(Color32::from_rgb(14, 165, 233))).clicked() {
                                        action = RecordsAction::Open(r.id);
                                    }

                                    if user_perms.contains(&Perm::RecordConfirm) || user_perms.contains(&Perm::RecordUnlock) {
                                        let lock_label = if r.status != 0 { "Mở khóa" } else { "Khóa" };
                                        if ui.button(RichText::new(lock_label).size(11.0)).clicked() {
                                            action = RecordsAction::ToggleLock(r.id, r.status == 0);
                                        }
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
