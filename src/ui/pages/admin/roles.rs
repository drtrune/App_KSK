use egui::{Color32, RichText, Ui};
use crate::auth::permissions::Perm;
use crate::models::user::RoleRecord;

pub struct AdminRolesPage;

pub enum RoleAction {
    None,
    SaveRole(RoleRecord),
}

impl AdminRolesPage {
    pub fn show(
        ui: &mut Ui,
        roles: &[RoleRecord],
        selected_role: &mut String,
        dark_mode: bool,
    ) -> RoleAction {
        let mut action = RoleAction::None;

        ui.vertical(|ui| {
            // Header
            ui.heading(RichText::new("PHÂN QUYỀN VAI TRÒ & CHỨC NĂNG").size(18.0).strong());
            ui.label(RichText::new("Cấu hình ma trận phân quyền chi tiết cho từng vai trò bác sĩ và nhân viên.")
                .size(12.0)
                .color(Color32::from_rgb(100, 116, 139)));

            ui.add_space(10.0);

            // Role selection pills
            ui.horizontal_wrapped(|ui| {
                for r in roles {
                    let is_sel = *selected_role == r.code;
                    let text_color = if is_sel {
                        Color32::from_rgb(14, 165, 233)
                    } else if dark_mode {
                        Color32::from_rgb(203, 213, 225)
                    } else {
                        Color32::from_rgb(71, 85, 105)
                    };
                    let btn_bg = if is_sel {
                        if dark_mode { Color32::from_rgb(30, 41, 59) } else { Color32::from_rgb(224, 242, 254) }
                    } else {
                        Color32::TRANSPARENT
                    };

                    let btn = egui::Button::new(RichText::new(&r.name).size(12.0).color(text_color).strong())
                        .fill(btn_bg)
                        .rounding(4.0);

                    if ui.add(btn).clicked() {
                        *selected_role = r.code.clone();
                    }
                }
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Role details & permissions
            let current_role = roles.iter().find(|r| r.code == *selected_role);
            if let Some(r) = current_role {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("Vai trò: {} (@{})", r.name, r.code)).strong().size(14.0));
                    if !r.description.is_empty() {
                        ui.label(RichText::new(format!("— {}", r.description)).size(12.0).color(Color32::from_rgb(100, 116, 139)));
                    }
                });

                ui.add_space(10.0);

                let mut current_perms: Vec<Perm> = r.permissions
                    .iter()
                    .filter_map(|p| p.parse::<Perm>().ok())
                    .collect();

                let mut changed = false;

                // Group 1: Quyền hồ sơ
                ui.label(RichText::new("1. QUYỀN THAO TÁC HỒ SƠ:").strong().size(12.5).color(Color32::from_rgb(14, 165, 233)));
                ui.add_space(4.0);
                let rec_perms = [
                    (Perm::RecordView, "Xem danh sách hồ sơ"),
                    (Perm::RecordCreate, "Tạo hồ sơ mới"),
                    (Perm::RecordDelete, "Xóa hồ sơ khám"),
                    (Perm::RecordConfirm, "Khóa / Xác nhận hồ sơ"),
                    (Perm::RecordUnlock, "Mở khóa hồ sơ"),
                    (Perm::RecordExport, "Xuất CSV 97 cột / In ấn"),
                ];
                for (p, label) in rec_perms {
                    let mut has = current_perms.contains(&p);
                    if ui.checkbox(&mut has, label).changed() {
                        if has {
                            current_perms.push(p);
                        } else {
                            current_perms.retain(|x| x != &p);
                        }
                        changed = true;
                    }
                }

                ui.add_space(12.0);

                // Group 2: Quyền nhập liệu chuyên khoa
                ui.label(RichText::new("2. QUYỀN NHẬP LIỆU CHUYÊN KHOA LÂM SÀNG:").strong().size(12.5).color(Color32::from_rgb(14, 165, 233)));
                ui.add_space(4.0);
                let clin_perms = [
                    (Perm::FormHanhChinh, "Nhập thông tin hành chính"),
                    (Perm::FormTienSu, "Nhập tiền sử bệnh tật"),
                    (Perm::ExamTheLuc, "Khám thể lực"),
                    (Perm::ExamNoi, "Khám nội khoa (Tuần hoàn, Hô hấp, Tiêu hóa, Thận, Tiết niệu...)"),
                    (Perm::ExamNgoai, "Khám ngoại khoa"),
                    (Perm::ExamDaLieu, "Khám da liễu"),
                    (Perm::ExamSan, "Khám sản phụ khoa"),
                    (Perm::ExamMat, "Khám mắt"),
                    (Perm::ExamTmh, "Khám tai mũi họng"),
                    (Perm::ExamRhm, "Khám răng hàm mặt"),
                    (Perm::ExamCls, "Khám & Đọc kết quả cận lâm sàng"),
                    (Perm::FormKetLuan, "Bác sĩ kết luận & Ký xác nhận"),
                ];
                for (p, label) in clin_perms {
                    let mut has = current_perms.contains(&p);
                    if ui.checkbox(&mut has, label).changed() {
                        if has {
                            current_perms.push(p);
                        } else {
                            current_perms.retain(|x| x != &p);
                        }
                        changed = true;
                    }
                }

                ui.add_space(12.0);

                // Group 3: Quyền quản trị hệ thống
                ui.label(RichText::new("3. QUYỀN QUẢN TRỊ HỆ THỐNG:").strong().size(12.5).color(Color32::from_rgb(14, 165, 233)));
                ui.add_space(4.0);
                let adm_perms = [
                    (Perm::CatalogManage, "Quản lý danh mục dùng chung"),
                    (Perm::Qd1613Manage, "Cấu hình chuẩn phân loại QĐ 1613"),
                    (Perm::UserManage, "Quản lý người dùng & Bác sĩ"),
                    (Perm::RoleManage, "Quản lý vai trò & Phân quyền"),
                    (Perm::SettingsManage, "Cài đặt đơn vị"),
                ];
                for (p, label) in adm_perms {
                    let mut has = current_perms.contains(&p);
                    if ui.checkbox(&mut has, label).changed() {
                        if has {
                            current_perms.push(p);
                        } else {
                            current_perms.retain(|x| x != &p);
                        }
                        changed = true;
                    }
                }

                if changed {
                    let mut updated_role = r.clone();
                    updated_role.permissions = current_perms.iter().map(|p| p.as_str().to_string()).collect();
                    action = RoleAction::SaveRole(updated_role);
                }
            }
        });

        action
    }
}
