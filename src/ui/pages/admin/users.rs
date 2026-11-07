use egui::{Color32, RichText, Ui, Vec2};
use crate::models::user::{RoleRecord, UserRecord};

pub struct AdminUsersPage;

pub enum UserAction {
    None,
    Save(UserRecord),
    Delete(i64),
    ToggleActive(i64, bool),
}

pub struct UserEditState {
    pub is_open: bool,
    pub id: Option<i64>,
    pub username: String,
    pub full_name: String,
    pub role: String,
    pub existing_password: String,
    pub new_password: String,
    pub is_active: bool,
}

impl Default for UserEditState {
    fn default() -> Self {
        Self {
            is_open: false,
            id: None,
            username: "".into(),
            full_name: "".into(),
            role: "bs_theluc".into(),
            existing_password: "".into(),
            new_password: "".into(),
            is_active: true,
        }
    }
}

impl AdminUsersPage {
    pub fn show(
        ui: &mut Ui,
        users: &[UserRecord],
        roles: &[RoleRecord],
        search_query: &mut String,
        edit_state: &mut UserEditState,
        _dark_mode: bool,
    ) -> UserAction {
        let mut action = UserAction::None;

        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.heading(RichText::new("QUẢN LÝ NGƯỜI DÙNG & TÀI KHOẢN").size(18.0).strong());
                    ui.label(RichText::new("Quản lý danh sách bác sĩ, nhân viên tiếp đón, gán vai trò và phân quyền sử dụng.")
                        .size(12.0)
                        .color(Color32::from_rgb(100, 116, 139)));
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let add_btn = egui::Button::new(RichText::new("➕  Thêm tài khoản").strong().size(12.0).color(Color32::WHITE))
                        .fill(Color32::from_rgb(14, 165, 233))
                        .rounding(6.0)
                        .min_size(Vec2::new(130.0, 30.0));

                    if ui.add(add_btn).clicked() {
                        edit_state.is_open = true;
                        edit_state.id = None;
                        edit_state.username = "".into();
                        edit_state.full_name = "".into();
                        edit_state.role = roles.first().map(|r| r.code.clone()).unwrap_or_else(|| "bs_noi".into());
                        edit_state.existing_password = "".into();
                        edit_state.new_password = "".into();
                        edit_state.is_active = true;
                    }
                });
            });

            ui.add_space(10.0);

            // Search Filter
            ui.horizontal(|ui| {
                ui.label(RichText::new("Tìm kiếm:").size(12.0));
                ui.add(egui::TextEdit::singleline(search_query).hint_text("Tìm tên tài khoản, họ tên...").desired_width(260.0));
                if !search_query.is_empty() && ui.button("✖").clicked() {
                    search_query.clear();
                }
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(6.0);

            let q = search_query.trim().to_lowercase();
            let filtered: Vec<&UserRecord> = users
                .iter()
                .filter(|u| q.is_empty() || u.username.to_lowercase().contains(&q) || u.full_name.to_lowercase().contains(&q))
                .collect();

            // Table of users
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    egui::Grid::new("grid_users_table")
                        .striped(true)
                        .spacing([16.0, 8.0])
                        .min_col_width(70.0)
                        .show(ui, |ui| {
                            ui.label(RichText::new("STT").strong().size(11.5).color(Color32::from_rgb(100, 116, 139)));
                            ui.label(RichText::new("Tên đăng nhập").strong().size(11.5).color(Color32::from_rgb(100, 116, 139)));
                            ui.label(RichText::new("Họ và Tên").strong().size(11.5).color(Color32::from_rgb(100, 116, 139)));
                            ui.label(RichText::new("Vai trò").strong().size(11.5).color(Color32::from_rgb(100, 116, 139)));
                            ui.label(RichText::new("Trạng thái").strong().size(11.5).color(Color32::from_rgb(100, 116, 139)));
                            ui.label(RichText::new("Thao tác").strong().size(11.5).color(Color32::from_rgb(100, 116, 139)));
                            ui.end_row();

                            for (i, u) in filtered.iter().enumerate() {
                                ui.label((i + 1).to_string());
                                ui.label(RichText::new(&u.username).strong().monospace().size(12.0));
                                ui.label(RichText::new(&u.full_name).size(12.0));

                                let role_name = roles.iter().find(|r| r.code == u.role).map(|r| r.name.as_str()).unwrap_or(&u.role);
                                ui.label(role_name);

                                let (st_txt, st_col) = if u.is_active {
                                    ("Hoạt động", Color32::from_rgb(16, 185, 129))
                                } else {
                                    ("Bị khóa", Color32::from_rgb(239, 68, 68))
                                };
                                ui.colored_label(st_col, st_txt);

                                ui.horizontal(|ui| {
                                    if ui.button(RichText::new("Sửa").size(11.0).color(Color32::from_rgb(14, 165, 233))).clicked() {
                                        edit_state.is_open = true;
                                        edit_state.id = Some(u.id);
                                        edit_state.username = u.username.clone();
                                        edit_state.full_name = u.full_name.clone();
                                        edit_state.role = u.role.clone();
                                        edit_state.existing_password = u.password.clone();
                                        edit_state.new_password = "".into();
                                        edit_state.is_active = u.is_active;
                                    }

                                    let toggle_txt = if u.is_active { "Khóa" } else { "Kích hoạt" };
                                    if ui.button(RichText::new(toggle_txt).size(11.0)).clicked() {
                                        action = UserAction::ToggleActive(u.id, !u.is_active);
                                    }

                                    if u.username != "admin" && ui.button(RichText::new("Xóa").size(11.0).color(Color32::from_rgb(239, 68, 68))).clicked() {
                                        action = UserAction::Delete(u.id);
                                    }
                                });

                                ui.end_row();
                            }
                        });
                });
        });

        // Edit Modal
        if edit_state.is_open {
            egui::Window::new(if edit_state.id.is_some() { "Chỉnh sửa người dùng" } else { "Thêm người dùng mới" })
                .collapsible(false)
                .resizable(false)
                .min_width(360.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    ui.vertical(|ui| {
                        let is_new = edit_state.id.is_none();

                        ui.label(RichText::new("Tên đăng nhập (*):").strong().size(12.0));
                        ui.add(egui::TextEdit::singleline(&mut edit_state.username).desired_width(340.0).interactive(is_new));

                        ui.add_space(6.0);
                        ui.label(RichText::new("Họ và Tên (*):").strong().size(12.0));
                        ui.add(egui::TextEdit::singleline(&mut edit_state.full_name).desired_width(340.0));

                        ui.add_space(6.0);
                        ui.label(RichText::new("Vai trò (*):").strong().size(12.0));
                        egui::ComboBox::from_id_source("cb_user_edit_role")
                            .width(340.0)
                            .selected_text(roles.iter().find(|r| r.code == edit_state.role).map(|r| r.name.as_str()).unwrap_or(&edit_state.role))
                            .show_ui(ui, |ui| {
                                for r in roles {
                                    ui.selectable_value(&mut edit_state.role, r.code.clone(), &r.name);
                                }
                            });

                        ui.add_space(6.0);
                        let pass_label = if is_new { "Mật khẩu khởi tạo (*):" } else { "Đổi mật khẩu mới (để trống nếu giữ nguyên):" };
                        ui.label(RichText::new(pass_label).strong().size(12.0));
                        ui.add(egui::TextEdit::singleline(&mut edit_state.new_password).password(true).desired_width(340.0));

                        ui.add_space(6.0);
                        ui.checkbox(&mut edit_state.is_active, "Đang hoạt động");

                        ui.add_space(14.0);
                        ui.horizontal(|ui| {
                            let btn = egui::Button::new(RichText::new("Lưu tài khoản").strong().color(Color32::WHITE))
                                .fill(Color32::from_rgb(14, 165, 233));
                            if ui.add(btn).clicked() {
                                if !edit_state.username.trim().is_empty() && !edit_state.full_name.trim().is_empty() {
                                    let pass = if !edit_state.new_password.trim().is_empty() {
                                        crate::auth::hash_password(edit_state.new_password.trim())
                                    } else {
                                        edit_state.existing_password.clone()
                                    };

                                    action = UserAction::Save(UserRecord {
                                        id: edit_state.id.unwrap_or(0),
                                        username: edit_state.username.trim().to_string(),
                                        password: pass,
                                        full_name: edit_state.full_name.trim().to_string(),
                                        role: edit_state.role.clone(),
                                        is_active: edit_state.is_active,
                                    });
                                    edit_state.is_open = false;
                                }
                            }

                            if ui.button("Hủy").clicked() {
                                edit_state.is_open = false;
                            }
                        });
                    });
                });
        }

        action
    }
}
