use egui::{Color32, RichText, Ui, Vec2};

pub struct LoginPage;

impl LoginPage {
    pub fn show(
        ui: &mut Ui,
        username: &mut String,
        password: &mut String,
        error_msg: &Option<String>,
        dark_mode: bool,
        on_login: &mut bool,
    ) {
        let card_bg = if dark_mode {
            Color32::from_rgb(30, 41, 59)
        } else {
            Color32::WHITE
        };
        let border_color = if dark_mode {
            Color32::from_rgb(51, 65, 85)
        } else {
            Color32::from_rgb(226, 232, 240)
        };

        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() * 0.12);

            // Brand Header
            ui.label(RichText::new("⚕").size(48.0).color(Color32::from_rgb(14, 165, 233)));
            ui.add_space(8.0);
            ui.heading(
                RichText::new("HỆ THỐNG KHÁM SỨC KHỎE ĐỊNH KỲ")
                    .size(22.0)
                    .strong()
                    .color(if dark_mode { Color32::WHITE } else { Color32::from_rgb(15, 23, 42) }),
            );
            ui.label(
                RichText::new("Chuẩn hóa biểu mẫu QĐ 3176/QĐ-BYT & Tiêu chuẩn phân loại QĐ 1613/BYT")
                    .size(13.0)
                    .color(Color32::from_rgb(100, 116, 139)),
            );
            ui.add_space(28.0);

            // Login Card
            egui::Frame::none()
                .fill(card_bg)
                .rounding(10.0)
                .stroke(egui::Stroke::new(1.0_f32, border_color))
                .inner_margin(egui::Margin::symmetric(32.0, 32.0))
                .show(ui, |ui| {
                    ui.set_width(360.0);

                    ui.label(RichText::new("Đăng nhập vào hệ thống").size(15.0).strong());
                    ui.add_space(16.0);

                    // Username
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Tên đăng nhập:").size(12.5));
                    });
                    let user_resp = ui.add(
                        egui::TextEdit::singleline(username)
                            .hint_text("Nhập tên đăng nhập...")
                            .desired_width(360.0)
                            .margin(egui::Margin::symmetric(8.0, 8.0)),
                    );

                    ui.add_space(10.0);

                    // Password
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Mật khẩu:").size(12.5));
                    });
                    let pass_resp = ui.add(
                        egui::TextEdit::singleline(password)
                            .password(true)
                            .hint_text("••••••••")
                            .desired_width(360.0)
                            .margin(egui::Margin::symmetric(8.0, 8.0)),
                    );

                    // Enter key submission
                    if (user_resp.lost_focus() || pass_resp.lost_focus())
                        && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    {
                        *on_login = true;
                    }

                    if let Some(err) = error_msg {
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("⚠").size(13.0).color(Color32::from_rgb(239, 68, 68)));
                            ui.label(RichText::new(err).size(12.0).color(Color32::from_rgb(239, 68, 68)));
                        });
                    }

                    ui.add_space(20.0);

                    // Submit button
                    let submit_btn = egui::Button::new(
                        RichText::new("ĐĂNG NHẬP")
                            .strong()
                            .size(13.0)
                            .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(14, 165, 233))
                    .rounding(6.0)
                    .min_size(Vec2::new(360.0, 38.0));

                    if ui.add(submit_btn).clicked() {
                        *on_login = true;
                    }

                    ui.add_space(14.0);
                    ui.label(
                        RichText::new("Mặc định: tài khoản admin / mật khẩu admin123")
                            .size(11.0)
                            .color(Color32::from_rgb(148, 163, 184)),
                    );
                });
        });
    }
}
