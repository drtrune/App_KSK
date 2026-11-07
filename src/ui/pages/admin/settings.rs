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
            ui.heading(RichText::new("CÀI ĐẶT THÔNG TIN ĐƠN VỊ & HỆ THỐNG").size(18.0).strong());
            ui.label(RichText::new("Cấu hình mã cơ sở khám chữa bệnh, mã định danh GLN và các giá trị mặc định của đơn vị.")
                .size(12.0)
                .color(Color32::from_rgb(100, 116, 139)));

            ui.add_space(14.0);

            egui::Grid::new("grid_settings")
                .spacing([24.0, 14.0])
                .min_col_width(180.0)
                .show(ui, |ui| {
                    ui.label(RichText::new("Mã Cơ sở KCB (mặc định) (*):").strong().size(12.5));
                    ui.add(egui::TextEdit::singleline(&mut config.default_cskcb).desired_width(240.0));
                    ui.end_row();

                    ui.label(RichText::new("Mã số địa điểm GLN (mặc định) (*):").strong().size(12.5));
                    ui.add(egui::TextEdit::singleline(&mut config.default_gtin_cskcb).desired_width(240.0));
                    ui.end_row();

                    ui.label(RichText::new("Mã Tỉnh / TP mặc định:").strong().size(12.5));
                    ui.add(egui::TextEdit::singleline(&mut config.default_province_code).desired_width(240.0));
                    ui.end_row();

                    ui.label(RichText::new("Mã Xã / Phường mặc định:").strong().size(12.5));
                    ui.add(egui::TextEdit::singleline(&mut config.default_ward_code).desired_width(240.0));
                    ui.end_row();

                    ui.label(RichText::new("Giao diện tối (Dark Mode):").strong().size(12.5));
                    ui.checkbox(&mut config.dark_mode, "Mặc định sử dụng giao diện tối khi khởi động");
                    ui.end_row();
                });

            ui.add_space(20.0);

            let save_btn = egui::Button::new(RichText::new("💾 Lưu cấu hình đơn vị").strong().size(12.5).color(Color32::WHITE))
                .fill(Color32::from_rgb(14, 165, 233));
            if ui.add(save_btn).clicked() {
                action = SettingsAction::Save(config.clone());
            }

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(12.0);

            ui.label(RichText::new("THÔNG TIN DỮ LIỆU & PHIÊN BẢN:").strong().size(12.5));
            ui.label(RichText::new("• Cơ sở dữ liệu: ksk_qd3176.db (SQLite bundled with WAL mode)").size(11.5).color(Color32::from_rgb(100, 116, 139)));
            ui.label(RichText::new("• Phiên bản ứng dụng: 0.4.0 (Kiến trúc mô-đun hóa, Antigravity Minimalist UI)").size(11.5).color(Color32::from_rgb(100, 116, 139)));
            ui.label(RichText::new("• Chuẩn Quyết định BYT: QĐ 3176/QĐ-BYT (biểu mẫu 97 cột) & QĐ 1613/BYT (phân loại sức khỏe)").size(11.5).color(Color32::from_rgb(100, 116, 139)));
        });

        action
    }
}
