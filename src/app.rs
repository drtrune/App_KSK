use std::collections::HashSet;
use std::fs;
use egui::{Color32, RichText};
use rusqlite::Connection;

use crate::auth::permissions::Perm;
use crate::config::AppConfig;
use crate::db;
use crate::domain::export::export_records_to_csv;
use crate::models::catalog::CatalogRecord;
use crate::models::qd1613::{QdClassification, QdDisease, QdSpecialty};
use crate::models::record::{KskRecordPayload, RecordItem};
use crate::models::user::{RoleRecord, UserRecord};
use crate::theme::apply_theme;
use crate::ui::layout::{AppPage, Sidebar};
use crate::ui::pages::admin::*;
use crate::ui::pages::record_form::kham::SpecialtySubTab;
use crate::ui::pages::record_form::{FormAction, FormStep, RecordFormView};
use crate::ui::pages::{LoginPage, RecordsAction, RecordsPage};

pub struct KskApp {
    pub config: AppConfig,
    pub conn: Connection,

    // Auth state
    pub logged_in: bool,
    pub current_user: Option<UserRecord>,
    pub user_perms: Vec<Perm>,
    pub login_user: String,
    pub login_pass: String,
    pub login_error: Option<String>,

    // Navigation & UI state
    pub current_page: AppPage,
    pub sidebar_collapsed: bool,
    pub dark_mode: bool,

    // Records state
    pub records: Vec<RecordItem>,
    pub search_query: String,
    pub status_filter: String,

    // Form editing state
    pub active_record_id: Option<i64>,
    pub active_record_is_locked: bool,
    pub active_form: KskRecordPayload,
    pub form_step: FormStep,
    pub kham_subtab: SpecialtySubTab,
    pub simple_mode: bool,
    pub expanded_diseases: HashSet<String>,

    // Admin: Catalogs
    pub catalogs: Vec<CatalogRecord>,
    pub selected_catalog_type: String,
    pub catalog_search: String,
    pub catalog_edit_state: CatalogEditState,

    // Admin: QĐ 1613
    pub specialties: Vec<QdSpecialty>,
    pub diseases: Vec<QdDisease>,
    pub classifications: Vec<QdClassification>,
    pub selected_qd_spec: String,
    pub qd_search: String,
    pub qd_edit_state: QdClassificationEditState,

    // Admin: Users & Roles
    pub users: Vec<UserRecord>,
    pub roles: Vec<RoleRecord>,
    pub user_search: String,
    pub user_edit_state: UserEditState,
    pub selected_role: String,

    // Notification toast
    pub toast: Option<String>,
}

impl KskApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = AppConfig::load();
        let conn = Connection::open(&config.db_path).expect("Failed to open database");
        let _ = conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;");
        let _ = db::migrations::run_migrations(&conn);
        let _ = db::seed::seed_roles(&conn);
        let _ = db::seed::seed_admin_user(&conn);
        let _ = db::seed::seed_catalogs(&conn);

        let mut app = Self {
            dark_mode: config.dark_mode,
            config,
            conn,
            logged_in: false,
            current_user: None,
            user_perms: Vec::new(),
            login_user: "".into(),
            login_pass: "".into(),
            login_error: None,
            current_page: AppPage::Records,
            sidebar_collapsed: false,
            records: Vec::new(),
            search_query: "".into(),
            status_filter: "all".into(),
            active_record_id: None,
            active_record_is_locked: false,
            active_form: KskRecordPayload::default(),
            form_step: FormStep::HanhChinh,
            kham_subtab: SpecialtySubTab::TheLuc,
            simple_mode: false,
            expanded_diseases: HashSet::new(),
            catalogs: Vec::new(),
            selected_catalog_type: "priority".into(),
            catalog_search: "".into(),
            catalog_edit_state: CatalogEditState::default(),
            specialties: Vec::new(),
            diseases: Vec::new(),
            classifications: Vec::new(),
            selected_qd_spec: "CK_NOI_TIM_MACH".into(),
            qd_search: "".into(),
            qd_edit_state: QdClassificationEditState::default(),
            users: Vec::new(),
            roles: Vec::new(),
            user_search: "".into(),
            user_edit_state: UserEditState::default(),
            selected_role: "admin".into(),
            toast: None,
        };

        app.reload_all_data();
        app
    }

    pub fn reload_all_data(&mut self) {
        if let Ok(recs) = db::records::load_records(&self.conn) {
            self.records = recs;
        }
        if let Ok(cats) = db::catalogs::load_catalogs(&self.conn) {
            self.catalogs = cats;
        }
        if let Ok((specs, dis, cls)) = db::qd1613::load_qd1613(&self.conn) {
            self.specialties = specs;
            self.diseases = dis;
            self.classifications = cls;
        }
        if let Ok(us) = db::users::load_users(&self.conn) {
            self.users = us;
        }
        if let Ok(rs) = db::users::load_roles(&self.conn) {
            self.roles = rs;
        }
    }

    pub fn handle_login(&mut self) {
        let u = self.login_user.trim();
        let p = self.login_pass.trim();

        if let Ok(users) = db::users::load_users(&self.conn) {
            if let Some(user) = users.into_iter().find(|user| user.username == u) {
                if !user.is_active {
                    self.login_error = Some("Tài khoản này đã bị khóa.".into());
                    return;
                }
                let hashed = crate::auth::hash_password(p);
                if user.password == hashed {
                    self.logged_in = true;
                    self.login_error = None;
                    self.login_pass.clear();

                    // Load role permissions
                    if let Ok(roles) = db::users::load_roles(&self.conn) {
                        if let Some(r) = roles.into_iter().find(|r| r.code == user.role) {
                            self.user_perms = r.permissions.iter().filter_map(|p| p.parse::<Perm>().ok()).collect();
                        }
                    }
                    self.current_user = Some(user);
                    self.current_page = AppPage::Records;
                    return;
                }
            }
        }
        self.login_error = Some("Tên đăng nhập hoặc mật khẩu không chính xác.".into());
    }

    pub fn start_new_record(&mut self) {
        let mut new_payload = KskRecordPayload::default();
        new_payload.ma_cskcb = self.config.default_cskcb.clone();
        new_payload.ma_gtin_cskcb = self.config.default_gtin_cskcb.clone();
        new_payload.matinh_cu_tru = self.config.default_province_code.clone();
        new_payload.maxa_cu_tru = self.config.default_ward_code.clone();
        new_payload.ngay_vao = chrono::Local::now().format("%d/%m/%Y").to_string();

        self.active_record_id = None;
        self.active_record_is_locked = false;
        self.active_form = new_payload;
        self.form_step = FormStep::HanhChinh;
        self.kham_subtab = SpecialtySubTab::TheLuc;
        self.expanded_diseases.clear();
        self.current_page = AppPage::RecordEdit;
    }

    pub fn open_record(&mut self, record_id: i64) {
        if let Some(rec) = self.records.iter().find(|r| r.id == record_id) {
            self.active_record_id = Some(rec.id);
            self.active_record_is_locked = rec.status != 0;
            self.active_form = rec.data.clone();
            self.form_step = FormStep::HanhChinh;
            self.kham_subtab = SpecialtySubTab::TheLuc;
            self.expanded_diseases.clear();
            self.current_page = AppPage::RecordEdit;
        }
    }

    pub fn save_current_record(&mut self) {
        if self.active_record_is_locked {
            self.toast = Some("Hồ sơ đã khóa, không thể chỉnh sửa.".into());
            return;
        }

        let item = RecordItem {
            id: self.active_record_id.unwrap_or(0),
            ho_ten: self.active_form.ho_ten.clone(),
            so_cccd: self.active_form.so_cccd.clone(),
            ngay_vao: self.active_form.ngay_vao.clone(),
            status: if self.active_record_is_locked { 1 } else { 0 },
            data: self.active_form.clone(),
        };

        match db::records::save_record(&self.conn, &item) {
            Ok(new_id) => {
                self.active_record_id = Some(new_id);
                self.toast = Some("Lưu hồ sơ thành công!".into());
                self.reload_all_data();
            }
            Err(e) => {
                self.toast = Some(format!("Lỗi khi lưu hồ sơ: {}", e));
            }
        }
    }

    pub fn export_csv(&mut self) {
        match export_records_to_csv(&self.records) {
            Ok(csv_data) => {
                let filename = format!("xuat_ksk_97cot_{}.csv", chrono::Local::now().format("%Y%m%d_%H%M%S"));
                if fs::write(&filename, csv_data).is_ok() {
                    self.toast = Some(format!("Đã xuất {} hồ sơ ra tệp '{}'", self.records.len(), filename));
                } else {
                    self.toast = Some("Không thể ghi tệp CSV.".into());
                }
            }
            Err(e) => {
                self.toast = Some(format!("Lỗi xuất CSV: {}", e));
            }
        }
    }
}

impl eframe::App for KskApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        apply_theme(ctx, self.dark_mode);

        // 1. LOGIN SCREEN IF NOT AUTHENTICATED
        if !self.logged_in {
            let mut do_login = false;
            egui::CentralPanel::default().show(ctx, |ui| {
                LoginPage::show(
                    ui,
                    &mut self.login_user,
                    &mut self.login_pass,
                    &self.login_error,
                    self.dark_mode,
                    &mut do_login,
                );
            });
            if do_login {
                self.handle_login();
            }
            return;
        }

        // 2. MAIN APPLICATION SHELL
        let mut on_logout = false;
        let mut on_new_record = false;

        // Collapsible Modern Sidebar
        egui::SidePanel::left("app_sidebar")
            .resizable(false)
            .exact_width(if self.sidebar_collapsed { 64.0 } else { 230.0 })
            .show(ctx, |ui| {
                if let Some(ref cur_u) = self.current_user {
                    Sidebar::show(
                        ui,
                        &mut self.current_page,
                        &mut self.sidebar_collapsed,
                        cur_u,
                        &mut self.dark_mode,
                        &self.user_perms,
                        &mut on_logout,
                        &mut on_new_record,
                    );
                }
            });

        if on_logout {
            self.logged_in = false;
            self.current_user = None;
            self.user_perms.clear();
            return;
        }

        if on_new_record {
            self.start_new_record();
        }

        // Central Content Area
        egui::CentralPanel::default().show(ctx, |ui| {
            // Toast Notification
            let mut dismiss_toast = false;
            if let Some(ref msg) = self.toast {
                egui::Frame::none()
                    .fill(Color32::from_rgb(14, 165, 233))
                    .rounding(6.0)
                    .inner_margin(egui::Margin::symmetric(14.0, 8.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("ℹ").color(Color32::WHITE).strong());
                            ui.label(RichText::new(msg).color(Color32::WHITE).strong().size(12.0));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button(RichText::new("✖").color(Color32::WHITE)).clicked() {
                                    dismiss_toast = true;
                                }
                            });
                        });
                    });
                ui.add_space(8.0);
            }
            if dismiss_toast {
                self.toast = None;
            }

            match self.current_page {
                AppPage::Records => {
                    let action = RecordsPage::show(
                        ui,
                        &self.records,
                        &mut self.search_query,
                        &mut self.status_filter,
                        &self.user_perms,
                        self.dark_mode,
                    );
                    match action {
                        RecordsAction::None => {}
                        RecordsAction::New => self.start_new_record(),
                        RecordsAction::Open(id) => self.open_record(id),
                        RecordsAction::Delete(id) => {
                            let _ = db::records::delete_record(&self.conn, id);
                            self.reload_all_data();
                        }
                        RecordsAction::ToggleLock(id, locked) => {
                            if let Some(rec) = self.records.iter_mut().find(|r| r.id == id) {
                                rec.status = if locked { 1 } else { 0 };
                                let _ = db::records::save_record(&self.conn, rec);
                            }
                            self.reload_all_data();
                        }
                        RecordsAction::ExportCsv => self.export_csv(),
                        RecordsAction::Refresh => self.reload_all_data(),
                    }
                }

                AppPage::RecordEdit => {
                    let action = RecordFormView::show(
                        ui,
                        &mut self.active_form,
                        self.active_record_id,
                        &mut self.active_record_is_locked,
                        &mut self.form_step,
                        &mut self.kham_subtab,
                        &mut self.simple_mode,
                        &mut self.expanded_diseases,
                        &self.catalogs,
                        &self.specialties,
                        &self.diseases,
                        &self.classifications,
                        &self.user_perms,
                        self.dark_mode,
                    );
                    match action {
                        FormAction::None => {}
                        FormAction::BackToList => self.current_page = AppPage::Records,
                        FormAction::Save => self.save_current_record(),
                        FormAction::ToggleLock => {
                            if let Some(id) = self.active_record_id {
                                self.active_record_is_locked = !self.active_record_is_locked;
                                if let Some(rec) = self.records.iter_mut().find(|r| r.id == id) {
                                    rec.status = if self.active_record_is_locked { 1 } else { 0 };
                                    let _ = db::records::save_record(&self.conn, rec);
                                }
                                self.reload_all_data();
                            }
                        }
                    }
                }

                AppPage::AdminCatalogs => {
                    let action = AdminCatalogsPage::show(
                        ui,
                        &self.catalogs,
                        &mut self.selected_catalog_type,
                        &mut self.catalog_search,
                        &mut self.catalog_edit_state,
                        self.dark_mode,
                    );
                    match action {
                        CatalogAction::None => {}
                        CatalogAction::Save(c) => {
                            let _ = db::catalogs::save_catalog(&self.conn, &c);
                            if let Ok(cats) = db::catalogs::load_catalogs(&self.conn) {
                                self.catalogs = cats;
                            }
                        }
                        CatalogAction::Delete(id) => {
                            let _ = db::catalogs::delete_catalog(&self.conn, id);
                            if let Ok(cats) = db::catalogs::load_catalogs(&self.conn) {
                                self.catalogs = cats;
                            }
                        }
                        CatalogAction::ToggleActive(id, active) => {
                            if let Some(cat) = self.catalogs.iter_mut().find(|c| c.id == id) {
                                cat.is_active = active;
                                let _ = db::catalogs::save_catalog(&self.conn, cat);
                            }
                            if let Ok(cats) = db::catalogs::load_catalogs(&self.conn) {
                                self.catalogs = cats;
                            }
                        }
                    }
                }

                AppPage::AdminQd1613 => {
                    let action = AdminQd1613Page::show(
                        ui,
                        &self.specialties,
                        &self.diseases,
                        &self.classifications,
                        &mut self.selected_qd_spec,
                        &mut self.qd_search,
                        &mut self.qd_edit_state,
                        self.dark_mode,
                    );
                    match action {
                        Qd1613Action::None => {}
                        Qd1613Action::SaveClassification(c) => {
                            let _ = db::qd1613::save_classification(&self.conn, &c);
                            if let Ok((specs, dis, cls)) = db::qd1613::load_qd1613(&self.conn) {
                                self.specialties = specs;
                                self.diseases = dis;
                                self.classifications = cls;
                            }
                        }
                        Qd1613Action::ResetToDefaults => {
                            self.reload_all_data();
                            self.toast = Some("Đã nạp lại dữ liệu chuẩn QĐ 1613.".into());
                        }
                    }
                }

                AppPage::AdminUsers => {
                    let action = AdminUsersPage::show(
                        ui,
                        &self.users,
                        &self.roles,
                        &mut self.user_search,
                        &mut self.user_edit_state,
                        self.dark_mode,
                    );
                    match action {
                        UserAction::None => {}
                        UserAction::Save(u) => {
                            let _ = db::users::save_user(&self.conn, &u);
                            if let Ok(us) = db::users::load_users(&self.conn) {
                                self.users = us;
                            }
                        }
                        UserAction::Delete(id) => {
                            let _ = db::users::delete_user(&self.conn, id);
                            if let Ok(us) = db::users::load_users(&self.conn) {
                                self.users = us;
                            }
                        }
                        UserAction::ToggleActive(id, active) => {
                            if let Some(u) = self.users.iter_mut().find(|user| user.id == id) {
                                u.is_active = active;
                                let _ = db::users::save_user(&self.conn, u);
                            }
                            if let Ok(us) = db::users::load_users(&self.conn) {
                                self.users = us;
                            }
                        }
                    }
                }

                AppPage::AdminRoles => {
                    let action = AdminRolesPage::show(
                        ui,
                        &self.roles,
                        &mut self.selected_role,
                        self.dark_mode,
                    );
                    match action {
                        RoleAction::None => {}
                        RoleAction::SaveRole(r) => {
                            let _ = db::users::save_role(&self.conn, &r);
                            if let Ok(rs) = db::users::load_roles(&self.conn) {
                                self.roles = rs;
                            }
                            if let Some(ref cur_u) = self.current_user {
                                if cur_u.role == r.code {
                                    self.user_perms = r.permissions.iter().filter_map(|p| p.parse::<Perm>().ok()).collect();
                                }
                            }
                        }
                    }
                }

                AppPage::AdminSettings => {
                    let action = AdminSettingsPage::show(ui, &mut self.config);
                    match action {
                        SettingsAction::None => {}
                        SettingsAction::Save(cfg) => {
                            let _ = cfg.save();
                            self.toast = Some("Lưu cài đặt đơn vị thành công!".into());
                        }
                    }
                }
            }
        });
    }
}
