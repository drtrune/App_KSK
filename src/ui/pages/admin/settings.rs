use egui::{Color32, RichText, Ui};
use crate::config::AppConfig;

pub struct AdminSettingsPage;

pub enum SettingsAction {
    None,
    Save(AppConfig),
}

impl AdminSettingsPage {
    pub fn show(ui: &mut Ui, config: &mut AppConfig) -> SettingsAction {
        let mut action = SettingsAction::None;

        ui.vertical(|ui| {
            ui.heading(RichText::new("CÀI ĐẶT THÔNG TIN ĐƠN VỊ & QUY TẮC CẤP SỐ LƯU TRỮ").size(18.0).strong());
            ui.label(
                RichText::new("Cấu hình mã cơ sở khám chữa bệnh (tự động điền khi xuất XML/Excel), hậu tố Số lưu trữ GKSK và các giá trị mặc định.")
                    .size(12.0)
                    .color(Color32::from_rgb(100, 116, 139)),
            );

            ui.add_space(14.0);

            egui::Grid::new("grid_settings")
                .spacing([24.0, 12.0])
                .min_col_width(210.0)
                .show(ui, |ui| {
                    ui.label(RichText::new("Tên Bệnh viện / Cơ sở KCB:").strong().size(12.5));
                    ui.add(egui::TextEdit::singleline(&mut config.hospital_name).desired_width(280.0));
                    ui.end_row();

                    ui.label(RichText::new("Mã Cơ sở KCB (MA_CSKCB cố định) (*):").strong().size(12.5));
                    ui.add(egui::TextEdit::singleline(&mut config.default_cskcb).desired_width(280.0));
                    ui.end_row();

                    ui.label(RichText::new("Mã GLN (MA_GTIN_CSKCB cố định) (*):").strong().size(12.5));
                    ui.add(egui::TextEdit::singleline(&mut config.default_gtin_cskcb).desired_width(280.0));
                    ui.end_row();

                    ui.label(RichText::new("Hậu tố Số lưu trữ GKSK (*):").strong().size(12.5));
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut config.archive_suffix).desired_width(160.0));
                        ui.label(
                            RichText::new(format!(
                                "→ Mẫu sinh tự động: {:05}/{}",
                                config.next_archive_seq, config.archive_suffix
                            ))
                            .strong()
                            .color(Color32::from_rgb(5, 150, 105)),
                        );
                    });
                    ui.end_row();

                    ui.label(RichText::new("Số thứ tự lưu trữ tiếp theo:").strong().size(12.5));
                    let mut seq_str = config.next_archive_seq.to_string();
                    if ui.add(egui::TextEdit::singleline(&mut seq_str).desired_width(120.0)).changed() {
                        if let Ok(v) = seq_str.parse::<i64>() {
                            config.next_archive_seq = v.max(1);
                        }
                    }
                    ui.end_row();

                    ui.label(RichText::new("Mã Tỉnh / TP mặc định:").strong().size(12.5));
                    ui.add(egui::TextEdit::singleline(&mut config.default_province_code).desired_width(280.0));
                    ui.end_row();

                    ui.label(RichText::new("Mã Xã / Phường mặc định:").strong().size(12.5));
                    ui.add(egui::TextEdit::singleline(&mut config.default_ward_code).desired_width(280.0));
                    ui.end_row();

                    ui.label(RichText::new("Giao diện tối (Dark Mode):").strong().size(12.5));
                    ui.checkbox(&mut config.dark_mode, "Mặc định sử dụng giao diện tối khi khởi động");
                    ui.end_row();
                });

            ui.add_space(18.0);

            let save_btn = egui::Button::new(RichText::new("💾 Lưu cấu hình đơn vị").strong().size(12.5).color(Color32::WHITE))
                .fill(Color32::from_rgb(14, 165, 233));
            if ui.add(save_btn).clicked() {
                action = SettingsAction::Save(config.clone());
            }

            ui.add_space(18.0);
            ui.separator();
            ui.add_space(10.0);

            ui.label(RichText::new("THÔNG TIN DỮ LIỆU & PHIÊN BẢN:").strong().size(12.5));
            ui.label(RichText::new("• Cơ sở dữ liệu: ksk_qd3176.db (SQLite bundled with WAL mode)").size(11.5).color(Color32::from_rgb(100, 116, 139)));
            ui.label(RichText::new("• Phiên bản: 0.5.0 - Siêu ứng dụng Khám sức khỏe Đoàn & Lẻ (Mẫu 03 + QR Code + Import_KSK_Tren_18.xlsm)").size(11.5).color(Color32::from_rgb(100, 116, 139)));
        });

        action
    }
}
