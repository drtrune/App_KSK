use egui::{Color32, RichText, Ui, Vec2};
use crate::auth::permissions::Perm;
use crate::models::user::UserRecord;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppPage {
    Records,
    Companies,
    RecordEdit,
    AdminCatalogs,
    AdminQd1613,
    AdminUsers,
    AdminRoles,
    AdminSettings,
}

pub struct Sidebar;

impl Sidebar {
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        ui: &mut Ui,
        current_page: &mut AppPage,
        collapsed: &mut bool,
        current_user: &UserRecord,
        dark_mode: &mut bool,
        user_perms: &[Perm],
        on_logout: &mut bool,
        on_new_record: &mut bool,
    ) {
        let bg = if *dark_mode {
            Color32::from_rgb(15, 23, 42)
        } else {
            Color32::from_rgb(248, 250, 252)
        };

        let width = if *collapsed { 64.0 } else { 230.0 };

        egui::Frame::none()
            .fill(bg)
            .inner_margin(egui::Margin::symmetric(10.0, 14.0))
            .show(ui, |ui| {
                ui.set_width(width);
                ui.set_height(ui.available_height());

                // 1. Header: Logo & App Title
                ui.horizontal(|ui| {
                    let logo_color = Color32::from_rgb(14, 165, 233);
                    ui.label(RichText::new("⚕").size(22.0).strong().color(logo_color));
                    if !*collapsed {
                        ui.vertical(|ui| {
                            ui.label(RichText::new("SIÊU APP KSK").size(14.0).strong());
                            ui.label(RichText::new("Đoàn & Lẻ • QĐ 3176").size(10.5).color(Color32::from_rgb(100, 116, 139)));
                        });
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let icon = if *collapsed { "»" } else { "«" };
                        if ui.button(RichText::new(icon).size(13.0).monospace()).on_hover_text("Thu gọn / Mở rộng menu").clicked() {
                            *collapsed = !*collapsed;
                        }
                    });
                });

                ui.add_space(14.0);
                ui.separator();
                ui.add_space(8.0);

                // Quick Action: Tạo hồ sơ mới
                let can_create = user_perms.contains(&Perm::RecordCreate);
                if can_create {
                    let btn_text = if *collapsed { "➕" } else { "➕  Tạo hồ sơ lẻ mới" };
                    let btn = egui::Button::new(RichText::new(btn_text).strong().size(12.5).color(Color32::WHITE))
                        .fill(Color32::from_rgb(14, 165, 233))
                        .rounding(6.0)
                        .min_size(Vec2::new(if *collapsed { 42.0 } else { 200.0 }, 32.0));
                    if ui.add(btn).on_hover_text("Bắt đầu khám sức khỏe cho đối tượng mới").clicked() {
                        *on_new_record = true;
                    }
                    ui.add_space(12.0);
                }

                // 2. Navigation items
                let nav_item = |ui: &mut Ui, page: AppPage, icon: &str, label: &str, cur: &mut AppPage, is_col: bool| {
                    let is_active = *cur == page;
                    let text = if is_col {
                        icon.to_string()
                    } else {
                        format!("{}  {}", icon, label)
                    };

                    let text_color = if is_active {
                        Color32::from_rgb(14, 165, 233)
                    } else if *dark_mode {
                        Color32::from_rgb(203, 213, 225)
                    } else {
                        Color32::from_rgb(51, 65, 85)
                    };

                    let btn_bg = if is_active {
                        if *dark_mode {
                            Color32::from_rgb(30, 41, 59)
                        } else {
                            Color32::from_rgb(224, 242, 254)
                        }
                    } else {
                        Color32::TRANSPARENT
                    };

                    let btn = egui::Button::new(RichText::new(text).size(12.5).color(text_color).strong())
                        .fill(btn_bg)
                        .rounding(6.0)
                        .min_size(Vec2::new(if is_col { 42.0 } else { 200.0 }, 30.0));

                    if ui.add(btn).clicked() {
                        *cur = page;
                    }
                    ui.add_space(2.0);
                };

                // Section: Hồ sơ & Đoàn khám
                if !*collapsed {
                    ui.label(RichText::new("NGHIỆP VỤ KHÁM").size(10.0).color(Color32::from_rgb(148, 163, 184)).strong());
                    ui.add_space(2.0);
                }
                nav_item(ui, AppPage::Records, "📋", "Danh sách & Quét QR", current_page, *collapsed);
                nav_item(ui, AppPage::Companies, "🏢", "Khám Đoàn & Hợp đồng", current_page, *collapsed);

                ui.add_space(10.0);

                // Section: Quản trị & Danh mục
                let can_view_admin = user_perms.contains(&Perm::CatalogManage)
                    || user_perms.contains(&Perm::Qd1613Manage)
                    || user_perms.contains(&Perm::UserManage)
                    || user_perms.contains(&Perm::RoleManage);

                if can_view_admin {
                    if !*collapsed {
                        ui.label(RichText::new("QUẢN TRỊ & HỆ THỐNG").size(10.0).color(Color32::from_rgb(148, 163, 184)).strong());
                        ui.add_space(2.0);
                    }

                    if user_perms.contains(&Perm::CatalogManage) {
                        nav_item(ui, AppPage::AdminCatalogs, "📚", "Danh mục dùng chung", current_page, *collapsed);
                    }
                    if user_perms.contains(&Perm::Qd1613Manage) {
                        nav_item(ui, AppPage::AdminQd1613, "⚕", "Chuẩn QĐ 1613", current_page, *collapsed);
                    }
                    if user_perms.contains(&Perm::UserManage) {
                        nav_item(ui, AppPage::AdminUsers, "👥", "Người dùng & Bác sĩ", current_page, *collapsed);
                    }
                    if user_perms.contains(&Perm::RoleManage) {
                        nav_item(ui, AppPage::AdminRoles, "🛡", "Vai trò & Phân quyền", current_page, *collapsed);
                    }
                    nav_item(ui, AppPage::AdminSettings, "⚙", "Cài đặt đơn vị", current_page, *collapsed);
                }

                // 3. Bottom Section: User Info, Theme Toggle & Logout
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    ui.add_space(6.0);

                    let logout_txt = if *collapsed { "🚪" } else { "🚪  Đăng xuất" };
                    let logout_btn = egui::Button::new(RichText::new(logout_txt).size(12.0).color(Color32::from_rgb(239, 68, 68)))
                        .fill(Color32::TRANSPARENT)
                        .min_size(Vec2::new(if *collapsed { 42.0 } else { 200.0 }, 28.0));
                    if ui.add(logout_btn).on_hover_text("Đăng xuất khỏi tài khoản").clicked() {
                        *on_logout = true;
                    }

                    ui.add_space(4.0);

                    let theme_txt = if *collapsed {
                        if *dark_mode { "☀" } else { "🌙" }
                    } else if *dark_mode {
                        "☀  Giao diện Sáng"
                    } else {
                        "🌙  Giao diện Tối"
                    };
                    let theme_btn = egui::Button::new(RichText::new(theme_txt).size(11.5))
                        .fill(Color32::TRANSPARENT)
                        .min_size(Vec2::new(if *collapsed { 42.0 } else { 200.0 }, 26.0));
                    if ui.add(theme_btn).clicked() {
                        *dark_mode = !*dark_mode;
                    }

                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(6.0);

                    if !*collapsed {
                        ui.horizontal(|ui| {
                            let avatar_bg = Color32::from_rgb(224, 242, 254);
                            let avatar_fg = Color32::from_rgb(3, 105, 161);
                            egui::Frame::none()
                                .fill(avatar_bg)
                                .rounding(14.0)
                                .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                                .show(ui, |ui| {
                                    ui.label(RichText::new("👤").size(13.0).color(avatar_fg));
                                });
                            ui.vertical(|ui| {
                                ui.label(RichText::new(&current_user.full_name).strong().size(12.0));
                                ui.label(RichText::new(format!("@{} ({})", current_user.username, current_user.role)).size(10.0).color(Color32::from_rgb(100, 116, 139)));
                            });
                        });
                    }
                });
            });
    }
}
