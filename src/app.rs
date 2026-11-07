use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use egui::{Color32, RichText};
use rusqlite::Connection;

use crate::auth::permissions::Perm;
use crate::config::AppConfig;
use crate::db;
use crate::domain::{
    export_cls_to_csv, export_contract_full_dossier, export_record_to_mau03_docx,
    export_records_to_csv, export_records_to_xml, export_roster_template_csv,
    export_to_xlsm_file, import_roster_from_csv, open_path_in_os, sanitize_filename,
    store_attachment_file,
};
use crate::models::catalog::CatalogRecord;
use crate::models::company::{AttachmentRecord, CompanyRecord, ContractRecord};
use crate::models::qd1613::{QdClassification, QdDisease, QdSpecialty};
use crate::models::record::{KskRecordPayload, RecordItem};
use crate::models::user::{RoleRecord, UserRecord};
use crate::theme::apply_theme;
use crate::ui::layout::{AppPage, Sidebar};
use crate::ui::pages::admin::*;
use crate::ui::pages::record_form::kham::SpecialtySubTab;
use crate::ui::pages::{
    CompaniesPage, CompanyPageAction, CompanyUiState, FormAction, FormStep, LoginPage,
    RecordFormView, RecordsAction, RecordsPage,
};

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

    // Records & QR state
    pub records: Vec<RecordItem>,
    pub search_query: String,
    pub qr_scan_input: String,
    pub status_filter: String,

    // Companies, Contracts & Attachments
    pub companies: Vec<CompanyRecord>,
    pub contracts: Vec<ContractRecord>,
    pub attachments: Vec<AttachmentRecord>,
    pub company_ui: CompanyUiState,

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
            qr_scan_input: "".into(),
            status_filter: "all".into(),
            companies: Vec::new(),
            contracts: Vec::new(),
            attachments: Vec::new(),
            company_ui: CompanyUiState::default(),
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
        if let Ok(comps) = db::companies::load_companies(&self.conn) {
            self.companies = comps;
        }
        if let Ok(cts) = db::companies::load_contracts(&self.conn) {
            self.contracts = cts;
        }
        if let Ok(atts) = db::companies::load_attachments(&self.conn) {
            self.attachments = atts;
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
        let u = self.login_user.trim().to_string();
        let p = self.login_pass.trim().to_string();

        if let Ok(users) = db::users::load_users(&self.conn) {
            if let Some(user) = users.into_iter().find(|user| user.username == u) {
                if !user.is_active {
                    self.login_error = Some("Tài khoản này đã bị khóa.".into());
                    return;
                }
                let hashed = crate::auth::hash_password(&p);
                if user.password == hashed || user.password == p {
                    self.logged_in = true;
                    self.login_error = None;
                    self.login_pass.clear();

                    if user.password == p {
                        let mut upgraded_user = user.clone();
                        upgraded_user.password = hashed;
                        let _ = db::users::save_user(&self.conn, &upgraded_user);
                    }

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

    pub fn start_new_record(&mut self, contract_id: Option<i64>) {
        let mut new_payload = KskRecordPayload::default();
        new_payload.contract_id = contract_id;
        new_payload.ma_cskcb = self.config.default_cskcb.clone();
        new_payload.ma_gtin_cskcb = self.config.default_gtin_cskcb.clone();
        new_payload.matinh_cu_tru = self.config.default_province_code.clone();
        new_payload.maxa_cu_tru = self.config.default_ward_code.clone();
        new_payload.ngay_vao = chrono::Local::now().format("%d/%m/%Y").to_string();
        new_payload.so_luu_tru = "".into();

        if let Some(cid) = contract_id {
            if let Some(ct) = self.contracts.iter().find(|c| c.id == cid) {
                new_payload.doi_tuong = ct.doi_tuong_mac_dinh.clone();
                new_payload.nguon_chi_tra = ct.nguon_chi_tra_mac_dinh;
                if let Some(comp) = self.companies.iter().find(|co| co.id == ct.company_id) {
                    new_payload.noi_lam_viec_hoc_tap = comp.ten_cong_ty.clone();
                }
            }
        }

        self.active_record_id = None;
        self.active_record_is_locked = false;
        self.active_form = new_payload;
        self.form_step = FormStep::HanhChinh;
        self.kham_subtab = SpecialtySubTab::TheLuc;
        self.expanded_diseases.clear();
        self.current_page = AppPage::RecordEdit;
    }

    pub fn open_record(&mut self, record_id: i64, step: FormStep) {
        if let Some(rec) = self.records.iter().find(|r| r.id == record_id) {
            self.active_record_id = Some(rec.id);
            self.active_record_is_locked = rec.status != 0;
            self.active_form = rec.data.clone();
            self.form_step = step;
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

        self.active_form.sync_quick_cls_to_list();
        let item = RecordItem {
            id: self.active_record_id.unwrap_or(0),
            ho_ten: self.active_form.ho_ten.clone(),
            so_cccd: self.active_form.so_cccd.clone(),
            ngay_vao: self.active_form.ngay_vao.clone(),
            status: if self.active_record_is_locked { 1 } else { 0 },
            contract_id: self.active_form.contract_id,
            ma_y_te: self.active_form.ma_y_te.clone(),
            so_luu_tru: self.active_form.so_luu_tru.clone(),
            data: self.active_form.clone(),
        };

        match db::records::save_record(&self.conn, &item) {
            Ok(new_id) => {
                self.active_record_id = Some(new_id);
                self.toast = Some("💾 Lưu hồ sơ thành công!".into());
                self.reload_all_data();
            }
            Err(e) => {
                self.toast = Some(format!("Lỗi khi lưu hồ sơ: {}", e));
            }
        }
    }

    pub fn assign_archive_number_and_save(&mut self) {
        if self.active_form.so_luu_tru.trim().is_empty() {
            let new_no = format!("{:05}/{}", self.config.next_archive_seq, self.config.archive_suffix);
            self.active_form.so_luu_tru = new_no;
            self.config.next_archive_seq += 1;
            let _ = self.config.save();
        }
        self.save_current_record();
        self.toast = Some(format!(
            "✅ ĐÃ KẾT LUẬN & CẤP SỐ LƯU TRỮ: {} (Ghi số này lên tờ phiếu khám giấy)",
            self.active_form.so_luu_tru
        ));
    }

    pub fn print_mau03_record(&mut self, rec: &RecordItem, pre_exam_only: bool) {
        let dir = PathBuf::from("Xuat_Phieu_Mau03");
        let _ = fs::create_dir_all(&dir);
        let tag = if pre_exam_only { "TruocKham_QR" } else { "DayDu" };
        let fname = sanitize_filename(&format!(
            "Mau03_{}_{}_{}.docx",
            rec.data.ma_y_te,
            rec.ho_ten.to_uppercase(),
            tag
        ));
        let out_path = dir.join(fname);
        match export_record_to_mau03_docx(rec, &self.config, &out_path, pre_exam_only) {
            Ok(()) => {
                self.toast = Some(format!("🖨 Đã xuất phiếu Mẫu 03 kèm QR: {}", out_path.display()));
                open_path_in_os(&out_path.to_string_lossy());
            }
            Err(e) => {
                self.toast = Some(format!("Lỗi xuất phiếu Mẫu 03: {}", e));
            }
        }
    }

    pub fn attach_files_for(&mut self, owner_type: &str, owner_id: i64, doc_type: &str, prefix: &str) {
        if let Some(files) = rfd::FileDialog::new()
            .set_title("Chọn file Scan (PDF / Hình ảnh) để đính kèm")
            .pick_files()
        {
            let mut count = 0;
            for f in files {
                if let Ok(att) = store_attachment_file(&f, owner_type, owner_id, doc_type, prefix) {
                    let _ = db::companies::add_attachment(&self.conn, &att);
                    count += 1;
                }
            }
            self.reload_all_data();
            self.toast = Some(format!("📎 Đã đính kèm {} file scan vào hồ sơ!", count));
        }
    }
}

impl eframe::App for KskApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        apply_theme(ctx, self.dark_mode);

        // 1. LOGIN SCREEN
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

        // 2. MAIN SHELL
        let mut on_logout = false;
        let mut on_new_record = false;

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
            self.start_new_record(None);
        }

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
                        &self.attachments,
                        &mut self.search_query,
                        &mut self.qr_scan_input,
                        &mut self.status_filter,
                        &self.user_perms,
                        self.dark_mode,
                    );
                    match action {
                        RecordsAction::None => {}
                        RecordsAction::New => self.start_new_record(None),
                        RecordsAction::Open(id) => self.open_record(id, FormStep::HanhChinh),
                        RecordsAction::OpenFastEntry(id) => self.open_record(id, FormStep::NhapNhanh1Cua),
                        RecordsAction::PrintMau03(id) => {
                            if let Some(rec) = self.records.iter().find(|r| r.id == id).cloned() {
                                let pre_only = rec.data.so_luu_tru.trim().is_empty();
                                self.print_mau03_record(&rec, pre_only);
                            }
                        }
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
                        RecordsAction::ExportCsv => {
                            let ts = chrono::Local::now().format("%Y%m%d_%H%M%S");
                            if let Ok(csv_main) = export_records_to_csv(&self.records) {
                                let f1 = format!("KSK_Tren18_{}.csv", ts);
                                let mut bom = vec![0xEF, 0xBB, 0xBF];
                                bom.extend_from_slice(csv_main.as_bytes());
                                let _ = fs::write(&f1, bom);
                            }
                            if let Ok(csv_cls) = export_cls_to_csv(&self.records) {
                                let f2 = format!("KSK_DM_CanLamSang_{}.csv", ts);
                                let mut bom = vec![0xEF, 0xBB, 0xBF];
                                bom.extend_from_slice(csv_cls.as_bytes());
                                let _ = fs::write(&f2, bom);
                            }
                            self.toast = Some(format!("📥 Đã xuất 2 file CSV (Trên 18 & DM_CanLamSang) cho {} hồ sơ!", self.records.len()));
                        }
                        RecordsAction::ExportXlsm => {
                            let fname = PathBuf::from(format!(
                                "Import_KSK_Tren_18_Xuat_{}.xlsm",
                                chrono::Local::now().format("%Y%m%d_%H%M%S")
                            ));
                            match export_to_xlsm_file(&self.records, &fname) {
                                Ok(()) => {
                                    self.toast = Some(format!("📊 Đã xuất file chuẩn '{}' (Đủ 2 sheet Trên 18 & DM_CanLamSang)!", fname.display()));
                                    open_path_in_os(&fname.to_string_lossy());
                                }
                                Err(e) => self.toast = Some(format!("Lỗi xuất XLSM: {}", e)),
                            }
                        }
                        RecordsAction::ExportXml => {
                            let xml_str = export_records_to_xml(&self.records);
                            let fname = format!("KSK_QD3176_{}.xml", chrono::Local::now().format("%Y%m%d_%H%M%S"));
                            if fs::write(&fname, xml_str.as_bytes()).is_ok() {
                                self.toast = Some(format!("📄 Đã xuất XML chuẩn QĐ 3176 ra tệp '{}'", fname));
                            }
                        }
                        RecordsAction::Refresh => self.reload_all_data(),
                    }
                }

                AppPage::Companies => {
                    let action = CompaniesPage::show(
                        ui,
                        &self.companies,
                        &self.contracts,
                        &self.records,
                        &self.attachments,
                        &mut self.company_ui,
                        self.dark_mode,
                    );
                    match action {
                        CompanyPageAction::None => {}
                        CompanyPageAction::SaveCompany(c) => {
                            if let Ok(id) = db::companies::save_company(&self.conn, &c) {
                                self.company_ui.selected_company_id = Some(id);
                                self.reload_all_data();
                                self.toast = Some("💾 Đã lưu thông tin Công ty!".into());
                            }
                        }
                        CompanyPageAction::DeleteCompany(id) => {
                            let _ = db::companies::delete_company(&self.conn, id);
                            self.company_ui.selected_company_id = None;
                            self.reload_all_data();
                        }
                        CompanyPageAction::SaveContract(ct) => {
                            if let Ok(id) = db::companies::save_contract(&self.conn, &ct) {
                                self.company_ui.selected_contract_id = Some(id);
                                self.reload_all_data();
                                self.toast = Some("💾 Đã lưu Hợp đồng khám đoàn!".into());
                            }
                        }
                        CompanyPageAction::DeleteContract(id) => {
                            let _ = db::companies::delete_contract(&self.conn, id);
                            self.company_ui.selected_contract_id = None;
                            self.reload_all_data();
                        }
                        CompanyPageAction::AttachContractScan(ct_id) => {
                            self.attach_files_for("contract", ct_id, "Hợp đồng", &format!("HD_{}", ct_id));
                        }
                        CompanyPageAction::OpenAttachment(p) => open_path_in_os(&p),
                        CompanyPageAction::DeleteAttachment(id) => {
                            let _ = db::companies::delete_attachment(&self.conn, id);
                            self.reload_all_data();
                        }
                        CompanyPageAction::DownloadRosterTemplate => {
                            let p = PathBuf::from("Mau_Danh_Sach_Kham_Doan.csv");
                            if export_roster_template_csv(&p).is_ok() {
                                self.toast = Some("📄 Đã tạo file 'Mau_Danh_Sach_Kham_Doan.csv' tại thư mục phần mềm!".into());
                                open_path_in_os(&p.to_string_lossy());
                            }
                        }
                        CompanyPageAction::ImportRosterCsv(ct_id) => {
                            if let Some(csv_file) = rfd::FileDialog::new()
                                .add_filter("CSV UTF-8", &["csv"])
                                .set_title("Chọn file CSV Danh sách Nhân viên Khám đoàn")
                                .pick_file()
                            {
                                if let Some(ct) = self.contracts.iter().find(|c| c.id == ct_id).cloned() {
                                    let comp = self
                                        .companies
                                        .iter()
                                        .find(|co| co.id == ct.company_id)
                                        .cloned()
                                        .unwrap_or_default();
                                    let existing_cnt = self.records.iter().filter(|r| r.contract_id == Some(ct_id)).count();
                                    match import_roster_from_csv(&csv_file, &comp, &ct, &self.config, existing_cnt) {
                                        Ok(new_items) => {
                                            let cnt = new_items.len();
                                            for item in new_items {
                                                let _ = db::records::save_record(&self.conn, &item);
                                            }
                                            self.reload_all_data();
                                            self.toast = Some(format!(
                                                "📥 Đã import {} nhân viên vào Hợp đồng '{}' (Đã sinh Mã tiếp nhận, để trống Số lưu trữ)!",
                                                cnt, ct.so_hop_dong
                                            ));
                                        }
                                        Err(e) => self.toast = Some(format!("Lỗi import CSV: {}", e)),
                                    }
                                }
                            }
                        }
                        CompanyPageAction::NewRecordForContract(ct_id) => self.start_new_record(Some(ct_id)),
                        CompanyPageAction::BatchPrintMau03PreExam(ct_id) => {
                            let ct_recs: Vec<RecordItem> = self
                                .records
                                .iter()
                                .filter(|r| r.contract_id == Some(ct_id))
                                .cloned()
                                .collect();
                            let dir = PathBuf::from(format!("Xuat_Phieu_Mau03/HopDong_{}", ct_id));
                            let _ = fs::create_dir_all(&dir);
                            let mut ok_cnt = 0;
                            for (idx, r) in ct_recs.iter().enumerate() {
                                let fname = sanitize_filename(&format!(
                                    "{:03}_{}_{}_Mau03_QR.docx",
                                    idx + 1,
                                    r.data.ma_y_te,
                                    r.ho_ten.to_uppercase()
                                ));
                                if export_record_to_mau03_docx(r, &self.config, &dir.join(fname), true).is_ok() {
                                    ok_cnt += 1;
                                }
                            }
                            self.toast = Some(format!(
                                "🖨 Đã trộn & xuất {} phiếu khám Mẫu 03 kèm QR Code tại '{}'",
                                ok_cnt,
                                dir.display()
                            ));
                            open_path_in_os(&dir.to_string_lossy());
                        }
                        CompanyPageAction::ExportContractXlsm(ct_id) => {
                            let ct_recs: Vec<RecordItem> = self
                                .records
                                .iter()
                                .filter(|r| r.contract_id == Some(ct_id))
                                .cloned()
                                .collect();
                            let fname = PathBuf::from(format!("Import_KSK_Tren_18_HD{}.xlsm", ct_id));
                            if export_to_xlsm_file(&ct_recs, &fname).is_ok() {
                                self.toast = Some(format!("📊 Đã xuất file '{}' cho đoàn khám!", fname.display()));
                                open_path_in_os(&fname.to_string_lossy());
                            }
                        }
                        CompanyPageAction::ExportFullDossierPackage(ct_id) => {
                            if let Some(ct) = self.contracts.iter().find(|c| c.id == ct_id).cloned() {
                                let comp = self
                                    .companies
                                    .iter()
                                    .find(|co| co.id == ct.company_id)
                                    .cloned()
                                    .unwrap_or_default();
                                let ct_recs: Vec<RecordItem> = self
                                    .records
                                    .iter()
                                    .filter(|r| r.contract_id == Some(ct_id))
                                    .cloned()
                                    .collect();
                                let base = PathBuf::from("Bo_Ho_So_Doan_Luu_Tru");
                                match export_contract_full_dossier(&base, &comp, &ct, &ct_recs, &self.attachments, &self.config) {
                                    Ok(pkg_dir) => {
                                        self.toast = Some(format!("📦 Đã đóng gói Trọn Bộ Hồ Sơ Đoàn tại: {}", pkg_dir.display()));
                                        open_path_in_os(&pkg_dir.to_string_lossy());
                                    }
                                    Err(e) => self.toast = Some(format!("Lỗi đóng gói bộ hồ sơ: {}", e)),
                                }
                            }
                        }
                        CompanyPageAction::OpenRecordFastEntry(rid) => self.open_record(rid, FormStep::NhapNhanh1Cua),
                        CompanyPageAction::PrintSingleMau03(rid) => {
                            if let Some(rec) = self.records.iter().find(|r| r.id == rid).cloned() {
                                let pre_only = rec.data.so_luu_tru.trim().is_empty();
                                self.print_mau03_record(&rec, pre_only);
                            }
                        }
                        CompanyPageAction::AttachRecordScan(rid) => {
                            if let Some(rec) = self.records.iter().find(|r| r.id == rid).cloned() {
                                let tag = if !rec.data.so_luu_tru.trim().is_empty() {
                                    rec.data.so_luu_tru.clone()
                                } else {
                                    rec.data.ma_y_te.clone()
                                };
                                self.attach_files_for("record", rid, "Phiếu khám Mẫu 03", &format!("{}_{}", tag, rec.ho_ten));
                            }
                        }
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
                        &self.attachments,
                        &self.user_perms,
                        self.dark_mode,
                    );
                    match action {
                        FormAction::None => {}
                        FormAction::BackToList => self.current_page = AppPage::Records,
                        FormAction::Save => self.save_current_record(),
                        FormAction::AssignArchiveAndSave => self.assign_archive_number_and_save(),
                        FormAction::PrintMau03PreExam => {
                            let temp_rec = RecordItem {
                                id: self.active_record_id.unwrap_or(0),
                                ho_ten: self.active_form.ho_ten.clone(),
                                so_cccd: self.active_form.so_cccd.clone(),
                                ngay_vao: self.active_form.ngay_vao.clone(),
                                status: 0,
                                contract_id: self.active_form.contract_id,
                                ma_y_te: self.active_form.ma_y_te.clone(),
                                so_luu_tru: self.active_form.so_luu_tru.clone(),
                                data: self.active_form.clone(),
                            };
                            self.print_mau03_record(&temp_rec, true);
                        }
                        FormAction::PrintMau03Full => {
                            let temp_rec = RecordItem {
                                id: self.active_record_id.unwrap_or(0),
                                ho_ten: self.active_form.ho_ten.clone(),
                                so_cccd: self.active_form.so_cccd.clone(),
                                ngay_vao: self.active_form.ngay_vao.clone(),
                                status: 1,
                                contract_id: self.active_form.contract_id,
                                ma_y_te: self.active_form.ma_y_te.clone(),
                                so_luu_tru: self.active_form.so_luu_tru.clone(),
                                data: self.active_form.clone(),
                            };
                            self.print_mau03_record(&temp_rec, false);
                        }
                        FormAction::AttachScanFile => {
                            if let Some(rid) = self.active_record_id {
                                let prefix = if !self.active_form.so_luu_tru.trim().is_empty() {
                                    format!("{}_{}", self.active_form.so_luu_tru, self.active_form.ho_ten)
                                } else {
                                    format!("{}_{}", self.active_form.ma_y_te, self.active_form.ho_ten)
                                };
                                self.attach_files_for("record", rid, "Phiếu khám Mẫu 03", &prefix);
                            }
                        }
                        FormAction::OpenAttachment(p) => open_path_in_os(&p),
                        FormAction::DeleteAttachment(id) => {
                            let _ = db::companies::delete_attachment(&self.conn, id);
                            self.reload_all_data();
                        }
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
                            self.toast = Some("💾 Lưu cài đặt đơn vị & quy tắc cấp số lưu trữ thành công!".into());
                        }
                    }
                }
            }
        });
    }
}
