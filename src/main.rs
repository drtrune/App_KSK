fn hash_pass(plain: &str) -> String {
    let mut h = Sha256::new();
    h.update(plain.as_bytes());
    format!("{:x}", h.finalize())
}
#![windows_subsystem = "windows"]

use eframe::egui;
use sha2::{Digest, Sha256};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::fs;

const FONT_VN: &[u8] = include_bytes!("../assets/font_vn.ttf");

#[derive(Serialize, Deserialize, Clone, Debug)]
struct AppConfig {
    db_path: String,
    default_cskcb: String,
    default_gtin_cskcb: String,
    default_province_code: String,
    default_ward_code: String,
}

impl AppConfig {
    fn save(&self) -> Result<(), String> {
        let s = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write("config.toml", s).map_err(|e| e.to_string())
    }

    fn load() -> Self {
        fs::read_to_string("config.toml")
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or(AppConfig {
                db_path: "ksk_qd3176.db".to_string(),
                default_cskcb: "77005".to_string(),
                default_gtin_cskcb: "8934286053806".to_string(),
                default_province_code: "64".to_string(),
                default_ward_code: "23992".to_string(),
            })
    }
}

#[derive(Clone)]
struct UserRecord {
    id: i64,
    username: String,
    role: String,
}

// Cấu trúc danh mục dùng chung
#[derive(Clone, Serialize, Deserialize)]
struct CatalogRecord {
    id: i64,
    cat_type: String, // "province", "ward", "ethnicity", "occupation", "priority", "payment", "kcb_type", "icd10"
    code: String,
    name: String,
    parent_code: String,
}

// 1 Dòng Cận Lâm Sàng (Mục VI)
#[derive(Serialize, Deserialize, Clone, Default)]
struct ClsItem {
    ma_dich_vu: String,
    ma_chi_so: String,
    gia_tri: String,
    don_vi_do: String,
    mo_ta: String,
    ket_luan: String,
}

// CẤU TRÚC ĐẦY ĐỦ 100% CÁC TRƯỜNG DỮ LIỆU THEO QĐ 3176/QĐ-BYT (Trừ các trường chữ ký số)
#[derive(Serialize, Deserialize, Clone)]
struct KskRecordPayload {
    // I. Hành chính
    ho_ten: String,
    gioi_tinh: i32, // 1: Nam, 2: Nữ
    ngay_sinh: String,
    ma_dan_toc: String,
    so_cccd: String,
    ngaycap_cccd: String,
    noicap_cccd: String,
    dia_chi: String,
    matinh_cu_tru: String,
    maxa_cu_tru: String,
    ma_nghe_nghiep: String,
    noi_lam_viec_hoc_tap: String,
    dien_thoai: String,
    nhom_mau: String,
    ly_do_vv: String,

    // II. Thông tin chung lần khám
    ma_lk: String,
    ma_cskcb: String,
    ma_gtin_cskcb: String,
    doi_tuong: String, // Mã 1 -> 16 (có thể chọn nhiều mã nối bằng dấu ;)
    nguon_chi_tra: i32, // 1, 2, 3, 4, 5, 9
    ma_loai_kcb: String, // Theo QĐ 1804 (15: KSK định kỳ)
    ngay_vao: String,

    // III. Tiền sử bệnh tật
    // A. Gia đình
    tsgd_mac_benh: i32, // 0: Không, 1: Có
    tsgd_ma_benh: String, // Mã ICD-10 phân cách bằng ;
    // B. Bản thân (22 chỉ số)
    tsbt_benh_trong_5_nam_qua: i32,
    tsbt_benh_than_kinh: i32,
    tsbt_benh_mat: i32,
    tsbt_benh_tai: i32,
    tsbt_benh_tim: i32,
    tsbt_phau_thuat_tim: i32,
    tsbt_tang_huyet_ap: i32,
    tsbt_kho_tho: i32,
    tsbt_benh_phoi: i32,
    tsbt_benh_than: i32,
    tsbt_nghien_ruou: i32,
    tsbt_dai_thao_duong: i32,
    tsbt_benh_tam_than: i32,
    tsbt_mat_y_thuc: i32,
    tsbt_ngat: i32,
    tsbt_benh_tieu_hoa: i32,
    tsbt_roi_loan_giac_ngu: i32,
    tsbt_tai_bien: i32,
    tsbt_benh_cot_song: i32,
    tsbt_ruou_thuong_xuyen: i32,
    tsbt_ma_tuy: i32,
    tsbt_benh_khac: i32,
    tsbt_ma_benh_khac: String,
    tsbt_dang_dieu_tri_benh: i32,
    tsbt_ma_benh: String,
    tsbt_ten_thuoc_lieu_luong: String,
    tsbt_thai_san: i32,
    tsbt_ma_benh_thai_san: String,
    tsbt_ten_thuoc_thai_san: String,

    // IV. Khám thể lực
    chieu_cao: String,
    can_nang: String,
    chi_so_bmi: String,
    mach: String,
    huyet_ap: String,
    kham_the_luc_pl: i32,

    // V. Khám lâm sàng
    // A. Nội khoa
    noi_khoa_tuan_hoan: String,
    noi_khoa_tuan_hoan_pl: i32,
    noi_khoa_ho_hap: String,
    noi_khoa_ho_hap_pl: i32,
    noi_khoa_tieu_hoa: String,
    noi_khoa_tieu_hoa_pl: i32,
    noi_khoa_than_tn_sd: String,
    noi_khoa_than_tn_sd_pl: i32,
    noi_khoa_noi_tiet: String,
    noi_khoa_noi_tiet_pl: i32,
    noi_khoa_co_xuong_khop: String,
    noi_khoa_co_xuong_khop_pl: i32,
    noi_khoa_than_kinh: String,
    noi_khoa_than_kinh_pl: i32,
    noi_khoa_tam_than: String,
    noi_khoa_tam_than_pl: i32,
    // B. Ngoại khoa
    ket_qua_kham_ngoai_khoa: String,
    kham_ngoai_khoa_pl: i32,
    // C. Da liễu
    ket_qua_kham_da_lieu: String,
    kham_da_lieu_pl: i32,
    // D. Sản phụ khoa
    ket_qua_kham_san_phu_khoa: String,
    kham_san_phu_khoa_pl: i32,
    // E. Mắt
    khong_kinh_mat_phai: String,
    khong_kinh_mat_trai: String,
    co_kinh_mat_phai: String,
    co_kinh_mat_trai: String,
    benh_khac_mat: String,
    kham_mat_pl: i32,
    // F. Tai Mũi Họng
    tai_trai_noi_thuong: String,
    tai_trai_noi_tham: String,
    tai_phai_noi_thuong: String,
    tai_phai_noi_tham: String,
    benh_khac_tai_mui_hong: String,
    kham_tai_mui_hong_pl: i32,
    // G. Răng Hàm Mặt
    ham_tren: String,
    ham_duoi: String,
    benh_khac_rang_ham_mat: String,
    kham_rang_ham_mat_pl: i32,

    // VI. Khám cận lâm sàng (Danh sách nhiều chỉ số)
    cls_list: Vec<ClsItem>,

    // VII. Kết luận
    phan_loai_sk: i32,
    ket_luan_benh: String,
    cac_benh_tat_neu_co: String,
}

impl Default for KskRecordPayload {
    fn default() -> Self {
        let now = chrono::Local::now();
        Self {
            ho_ten: "".into(),
            gioi_tinh: 1, // Nam
            ngay_sinh: "".into(),
            ma_dan_toc: "01".into(),
            so_cccd: "".into(),
            ngaycap_cccd: "".into(),
            noicap_cccd: "Cục Cảnh sát QLHCVTTXH".into(),
            dia_chi: "".into(),
            matinh_cu_tru: "64".into(),
            maxa_cu_tru: "23992".into(),
            ma_nghe_nghiep: "22".into(),
            noi_lam_viec_hoc_tap: "".into(),
            dien_thoai: "".into(),
            nhom_mau: "O".into(),
            ly_do_vv: "Khám sức khỏe định kỳ".into(),
            ma_lk: format!("LK{}", now.format("%Y%m%d%H%M%S")),
            ma_cskcb: "77005".into(),
            ma_gtin_cskcb: "8934286053806".into(),
            doi_tuong: "13".into(), // Người lao động
            nguon_chi_tra: 4, // Người sử dụng LĐ
            ma_loai_kcb: "15".into(),
            ngay_vao: now.format("%d/%m/%Y").to_string(),
            tsgd_mac_benh: 0,
            tsgd_ma_benh: "".into(),
            tsbt_benh_trong_5_nam_qua: 0,
            tsbt_benh_than_kinh: 0,
            tsbt_benh_mat: 0,
            tsbt_benh_tai: 0,
            tsbt_benh_tim: 0,
            tsbt_phau_thuat_tim: 0,
            tsbt_tang_huyet_ap: 0,
            tsbt_kho_tho: 0,
            tsbt_benh_phoi: 0,
            tsbt_benh_than: 0,
            tsbt_nghien_ruou: 0,
            tsbt_dai_thao_duong: 0,
            tsbt_benh_tam_than: 0,
            tsbt_mat_y_thuc: 0,
            tsbt_ngat: 0,
            tsbt_benh_tieu_hoa: 0,
            tsbt_roi_loan_giac_ngu: 0,
            tsbt_tai_bien: 0,
            tsbt_benh_cot_song: 0,
            tsbt_ruou_thuong_xuyen: 0,
            tsbt_ma_tuy: 0,
            tsbt_benh_khac: 0,
            tsbt_ma_benh_khac: "".into(),
            tsbt_dang_dieu_tri_benh: 0,
            tsbt_ma_benh: "".into(),
            tsbt_ten_thuoc_lieu_luong: "".into(),
            tsbt_thai_san: 0,
            tsbt_ma_benh_thai_san: "".into(),
            tsbt_ten_thuoc_thai_san: "".into(),
            chieu_cao: "165.0".into(),
            can_nang: "60.0".into(),
            chi_so_bmi: "22.03".into(),
            mach: "75".into(),
            huyet_ap: "120/80".into(),
            kham_the_luc_pl: 1,
            noi_khoa_tuan_hoan: "Bình thường".into(),
            noi_khoa_tuan_hoan_pl: 1,
            noi_khoa_ho_hap: "Bình thường".into(),
            noi_khoa_ho_hap_pl: 1,
            noi_khoa_tieu_hoa: "Bình thường".into(),
            noi_khoa_tieu_hoa_pl: 1,
            noi_khoa_than_tn_sd: "Bình thường".into(),
            noi_khoa_than_tn_sd_pl: 1,
            noi_khoa_noi_tiet: "Bình thường".into(),
            noi_khoa_noi_tiet_pl: 1,
            noi_khoa_co_xuong_khop: "Bình thường".into(),
            noi_khoa_co_xuong_khop_pl: 1,
            noi_khoa_than_kinh: "Bình thường".into(),
            noi_khoa_than_kinh_pl: 1,
            noi_khoa_tam_than: "Bình thường".into(),
            noi_khoa_tam_than_pl: 1,
            ket_qua_kham_ngoai_khoa: "Bình thường".into(),
            kham_ngoai_khoa_pl: 1,
            ket_qua_kham_da_lieu: "Bình thường".into(),
            kham_da_lieu_pl: 1,
            ket_qua_kham_san_phu_khoa: "Bình thường".into(),
            kham_san_phu_khoa_pl: 1,
            khong_kinh_mat_phai: "10/10".into(),
            khong_kinh_mat_trai: "10/10".into(),
            co_kinh_mat_phai: "".into(),
            co_kinh_mat_trai: "".into(),
            benh_khac_mat: "".into(),
            kham_mat_pl: 1,
            tai_trai_noi_thuong: "5".into(),
            tai_trai_noi_tham: "0.5".into(),
            tai_phai_noi_thuong: "5".into(),
            tai_phai_noi_tham: "0.5".into(),
            benh_khac_tai_mui_hong: "".into(),
            kham_tai_mui_hong_pl: 1,
            ham_tren: "Bình thường".into(),
            ham_duoi: "Bình thường".into(),
            benh_khac_rang_ham_mat: "".into(),
            kham_rang_ham_mat_pl: 1,
            cls_list: vec![
                ClsItem {
                    ma_dich_vu: "XN01".into(),
                    ma_chi_so: "GLU".into(),
                    gia_tri: "5.2".into(),
                    don_vi_do: "mmol/L".into(),
                    mo_ta: "Đường huyết lúc đói".into(),
                    ket_luan: "Bình thường".into(),
                },
                ClsItem {
                    ma_dich_vu: "CDHA01".into(),
                    ma_chi_so: "XQ_TP".into(),
                    gia_tri: "Bình thường".into(),
                    don_vi_do: "".into(),
                    mo_ta: "Xquang tim phổi thẳng".into(),
                    ket_luan: "Không thấy tổn thương khu trú".into(),
                }
            ],
            phan_loai_sk: 1,
            ket_luan_benh: "Z00.0".into(),
            cac_benh_tat_neu_co: "Đủ sức khỏe công tác".into(),
        }
    }
}

struct RecordItem {
    id: i64,
    ho_ten: String,
    so_cccd: String,
    ngay_vao: String,
    status: i32,
    data: KskRecordPayload,
}

struct KskApp {
    conn: Connection,
    logged_in: bool,
    login_user: String,
    login_pass: String,
    current_user: String,
    current_role: String,
    current_id: Option<i64>,
    current_status: i32,
    form: KskRecordPayload,
    records: Vec<RecordItem>,
    status_msg: String,
    active_tab: usize, // 0: Form, 1: Danh sách, 2: Quản lý Danh mục
    dept_tab: usize,   // 0: Hành chính, 1: Thể lực, 2: Nội, 3: Ngoại, 4: Da liễu, 5: Sản, 6: Mắt, 7: TMH, 8: RHM, 9: Cận lâm sàng, 10: Kết luận

    // Danh mục
    catalogs: Vec<CatalogRecord>,
    cur_cat_tab: String,
    import_tsv_text: String,
    cat_code_input: String,
    cat_name_input: String,
    cat_parent_input: String,
}

impl KskApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if !FONT_VN.is_empty() {
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert("vn_font".to_owned(), egui::FontData::from_static(FONT_VN));
            fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "vn_font".to_owned());
            fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap().push("vn_font".to_owned());
            cc.egui_ctx.set_fonts(fonts);
        }

        apply_theme(&cc.egui_ctx, false);
        let cfg = AppConfig::load();
        let conn = Connection::open(&cfg.db_path).expect("Không mở được SQLite!");

        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT UNIQUE NOT NULL,
                password TEXT NOT NULL,
                role TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS catalogs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                cat_type TEXT NOT NULL,
                code TEXT NOT NULL,
                name TEXT NOT NULL,
                parent_code TEXT DEFAULT '',
                UNIQUE(cat_type, code)
            );
            CREATE TABLE IF NOT EXISTS records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ho_ten TEXT,
                so_cccd TEXT,
                ngay_vao TEXT,
                status INTEGER DEFAULT 0,
                json_data TEXT
            );
            -- Tạo sẵn user phân quyền
            INSERT OR IGNORE INTO users (username, password, role) VALUES ('admin', 'admin123', 'admin');
            INSERT OR IGNORE INTO users (username, password, role) VALUES ('client', '123456', 'reception');
            INSERT OR IGNORE INTO users (username, password, role) VALUES ('bs_theluc', '123456', 'bs_theluc');
            INSERT OR IGNORE INTO users (username, password, role) VALUES ('bs_noi', '123456', 'bs_noi');
            INSERT OR IGNORE INTO users (username, password, role) VALUES ('bs_ngoai', '123456', 'bs_ngoai');
            INSERT OR IGNORE INTO users (username, password, role) VALUES ('bs_dalieu', '123456', 'bs_dalieu');
            INSERT OR IGNORE INTO users (username, password, role) VALUES ('bs_san', '123456', 'bs_san');
            INSERT OR IGNORE INTO users (username, password, role) VALUES ('bs_mat', '123456', 'bs_mat');
            INSERT OR IGNORE INTO users (username, password, role) VALUES ('bs_tmh', '123456', 'bs_tmh');
            INSERT OR IGNORE INTO users (username, password, role) VALUES ('bs_rhm', '123456', 'bs_rhm');
            INSERT OR IGNORE INTO users (username, password, role) VALUES ('bs_cls', '123456', 'bs_cls');
            INSERT OR IGNORE INTO users (username, password, role) VALUES ('bs_ketluan', '123456', 'bs_ketluan');
            "#,
        ).unwrap();

        let mut app = Self {
            conn,
            logged_in: false,
            login_user: "client".into(),
            login_pass: "123456".into(),
            current_user: "".into(),
            current_role: "".into(),
            current_id: None,
            current_status: 0,
            form: KskRecordPayload::default(),
            records: vec![],
            status_msg: "Sẵn sàng".into(),
            active_tab: 0,
            dept_tab: 0,
            catalogs: vec![],
            cur_cat_tab: "priority".into(),
            import_tsv_text: "".into(),
            cat_code_input: "".into(),
            cat_name_input: "".into(),
            cat_parent_input: "".into(),
        };

        app.seed_default_catalogs();
        app.reload_catalogs();
        app.reload_records();
        app
    }

    fn seed_default_catalogs(&self) {
        let count: i64 = self.conn.query_row("SELECT COUNT(*) FROM catalogs", [], |r| r.get(0)).unwrap_or(0);
        if count == 0 {
            let seeds = [
                // 1. Đối tượng theo đúng 16 mã QĐ 3176
                ("priority", "1", "1 - Người cao tuổi", ""),
                ("priority", "2", "2 - Người khuyết tật", ""),
                ("priority", "3", "3 - Người thuộc hộ nghèo, cận nghèo", ""),
                ("priority", "4", "4 - Người có công", ""),
                ("priority", "5", "5 - Người mắc bệnh mạn tính", ""),
                ("priority", "6", "6 - Vùng đồng bào DTTS & miền núi", ""),
                ("priority", "7", "7 - Vùng ĐK KT-XH khó khăn, đặc biệt khó khăn", ""),
                ("priority", "8", "8 - Người sống tại xã đảo", ""),
                ("priority", "9", "9 - Người sống tại đặc khu", ""),
                ("priority", "10", "10 - Trẻ em trong cơ sở mầm non", ""),
                ("priority", "11", "11 - Học sinh cơ sở GD phổ thông", ""),
                ("priority", "12", "12 - Sinh viên", ""),
                ("priority", "13", "13 - Người lao động", ""),
                ("priority", "14", "14 - Người lao động không chính thức", ""),
                ("priority", "15", "15 - Người chưa có BHYT", ""),
                ("priority", "16", "16 - Các đối tượng khác", ""),

                // 2. Nguồn chi trả theo QĐ 3176
                ("payment", "1", "1 - Ngân sách Trung ương", ""),
                ("payment", "2", "2 - Ngân sách Địa phương", ""),
                ("payment", "3", "3 - Quỹ Bảo hiểm y tế", ""),
                ("payment", "4", "4 - Người sử dụng lao động", ""),
                ("payment", "5", "5 - Xã hội hóa", ""),
                ("payment", "9", "9 - Khác", ""),

                // 3. Tỉnh / Xã
                ("province", "64", "64 - Tỉnh Gia Lai", ""),
                ("province", "52", "52 - Tỉnh Bình Định", ""),
                ("ward", "23992", "23992 - Phường Quy Nhơn", "64"),
                ("ward", "23995", "23995 - Phường Pleiku", "64"),
                ("ward", "21517", "21517 - Phường Lê Lợi", "52"),

                // 4. Dân tộc & Nghề nghiệp
                ("ethnicity", "01", "01 - Kinh", ""),
                ("ethnicity", "08", "08 - Gia Rai", ""),
                ("ethnicity", "12", "12 - Ba Na", ""),
                ("occupation", "22", "22 - Chuyên môn Y tế", ""),
                ("occupation", "23", "23 - Giảng dạy / Giáo viên", ""),
                ("occupation", "41", "41 - Nhân viên văn phòng", ""),

                // 5. ICD-10
                ("icd10", "Z00.0", "Z00.0 - Khám sức khỏe định kỳ", ""),
                ("icd10", "I10", "I10 - Tăng huyết áp nguyên phát", ""),
                ("icd10", "E11", "E11 - Đái tháo đường Type 2", ""),
            ];
            for s in seeds {
                let _ = self.conn.execute(
                    "INSERT OR IGNORE INTO catalogs (cat_type, code, name, parent_code) VALUES (?1, ?2, ?3, ?4)",
                    params![s.0, s.1, s.2, s.3],
                );
            }
        }
    }

    fn reload_catalogs(&mut self) {
        let mut stmt = self.conn.prepare("SELECT id, cat_type, code, name, parent_code FROM catalogs ORDER BY cat_type, code").unwrap();
        let rows = stmt.query_map([], |r| {
            Ok(CatalogRecord {
                id: r.get(0)?,
                cat_type: r.get(1)?,
                code: r.get(2)?,
                name: r.get(3)?,
                parent_code: r.get(4)?,
            })
        }).unwrap();
        self.catalogs = rows.flatten().collect();
    }

    fn reload_records(&mut self) {
        let mut stmt = self.conn.prepare("SELECT id, ho_ten, so_cccd, ngay_vao, status, json_data FROM records ORDER BY id DESC").unwrap();
        let rows = stmt.query_map([], |r| {
            let json_str: String = r.get(5)?;
            let data: KskRecordPayload = serde_json::from_str(&json_str).unwrap_or_default();
            Ok(RecordItem {
                id: r.get(0)?,
                ho_ten: r.get(1)?,
                so_cccd: r.get(2)?,
                ngay_vao: r.get(3)?,
                status: r.get(4)?,
                data,
            })
        }).unwrap();
        self.records = rows.flatten().collect();
    }

    fn can_edit_dept(&self, dept_name: &str) -> bool {
        if self.current_status == 1 && self.current_role != "admin" {
            return false;
        }
        if self.current_role == "admin" {
            return true;
        }
        match dept_name {
            "hchinh" => self.current_role == "reception" || self.current_role == "client",
            "theluc" => self.current_role == "bs_theluc",
            "noi" => self.current_role == "bs_noi",
            "ngoai" => self.current_role == "bs_ngoai",
            "dalieu" => self.current_role == "bs_dalieu",
            "san" => self.current_role == "bs_san",
            "mat" => self.current_role == "bs_mat",
            "tmh" => self.current_role == "bs_tmh",
            "rhm" => self.current_role == "bs_rhm",
            "cls" => self.current_role == "bs_cls",
            "ketluan" => self.current_role == "bs_ketluan",
            _ => false,
        }
    }

    fn save_record(&mut self, confirm: bool) {
        if self.form.ho_ten.trim().is_empty() || self.form.so_cccd.trim().is_empty() {
            self.status_msg = "LỖI: Vui lòng nhập Họ tên và số CCCD!".into();
            return;
        }

        // Tự tính BMI
        if let (Ok(h), Ok(w)) = (self.form.chieu_cao.parse::<f64>(), self.form.can_nang.parse::<f64>()) {
            if h > 0.0 {
                let h_m = h / 100.0;
                self.form.chi_so_bmi = format!("{:.2}", w / (h_m * h_m));
            }
        }

        let new_status = if confirm { 1 } else { self.current_status };
        let json_str = serde_json::to_string(&self.form).unwrap();

        if let Some(id) = self.current_id {
            if self.current_status == 1 && self.current_role != "admin" {
                self.status_msg = "LỖI: Hồ sơ đã khóa, chỉ Admin mới được chỉnh sửa!".into();
                return;
            }
            self.conn.execute(
                "UPDATE records SET ho_ten = ?1, so_cccd = ?2, ngay_vao = ?3, status = ?4, json_data = ?5 WHERE id = ?6",
                params![self.form.ho_ten, self.form.so_cccd, self.form.ngay_vao, new_status, json_str, id],
            ).unwrap();
            self.current_status = new_status;
            self.status_msg = format!("Đã cập nhật hồ sơ #{}!", id);
        } else {
            self.conn.execute(
                "INSERT INTO records (ho_ten, so_cccd, ngay_vao, status, json_data) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![self.form.ho_ten, self.form.so_cccd, self.form.ngay_vao, new_status, json_str],
            ).unwrap();
            self.current_id = Some(self.conn.last_insert_rowid());
            self.current_status = new_status;
            self.status_msg = "Đã lưu mới hồ sơ thành công!".into();
        }
        self.reload_records();
    }

    // Xuất file CSV chuẩn 100% các cột theo QĐ 3176 (bỏ chữ ký)
    fn export_csv(&mut self) {
        let mut wtr = csv::WriterBuilder::new().from_writer(vec![]);
        let headers = [
            // I. Hành chính
            "HO_TEN", "GIOI_TINH", "NGAY_SINH", "MA_DAN_TOC", "SO_CCCD", "NGAYCAP_CCCD", "NOICAP_CCCD",
            "DIA_CHI", "MATINH_CU_TRU", "MAXA_CU_TRU", "MA_NGHE_NGHIEP", "NOI_LAM_VIEC_HOC_TAP",
            "DIEN_THOAI", "NHOM_MAU", "LY_DO_VV",
            // II. Thông tin chung lần khám
            "MA_LK", "MA_CSKCB", "MA_GTIN_CSKCB", "DOI_TUONG", "NGUON_CHI_TRA", "MA_LOAI_KCB", "NGAY_VAO",
            // III. Tiền sử
            "TSGD_MAC_BENH", "TSGD_MA_BENH", "TSBT_BENH_TRONG_5_NAM_QUA", "TSBT_BENH_THAN_KINH",
            "TSBT_BENH_MAT", "TSBT_BENH_TAI", "TSBT_BENH_TIM", "TSBT_PHAU_THUAT_TIM", "TSBT_TANG_HUYET_AP",
            "TSBT_KHO_THO", "TSBT_BENH_PHOI", "TSBT_BENH_THAN", "TSBT_NGHIEN_RUOU", "TSBT_DAI_THAO_DUONG",
            "TSBT_BENH_TAM_THAN", "TSBT_MAT_Y_THUC", "TSBT_NGAT", "TSBT_BENH_TIEU_HOA", "TSBT_ROI_LOAN_GIAC_NGU",
            "TSBT_TAI_BIEN", "TSBT_BENH_COT_SONG", "TSBT_RUOU_THUONG_XUYEN", "TSBT_MA_TUY", "TSBT_BENH_KHAC",
            "TSBT_MA_BENH_KHAC", "TSBT_DANG_DIEU_TRI_BENH", "TSBT_MA_BENH", "TSBT_TEN_THUOC_LIEU_LUONG",
            "TSBT_THAI_SAN", "TSBT_MA_BENH_THAI_SAN", "TSBT_TEN_THUOC_THAI_SAN",
            // IV. Khám thể lực
            "CHIEU_CAO", "CAN_NANG", "CHI_SO_BMI", "MACH", "HUYET_AP", "KHAM_THE_LUC_PL",
            // V. Khám lâm sàng
            "NOI_KHOA_TUAN_HOAN", "NOI_KHOA_TUAN_HOAN_PL", "NOI_KHOA_HO_HAP", "NOI_KHOA_HO_HAP_PL",
            "NOI_KHOA_TIEU_HOA", "NOI_KHOA_TIEU_HOA_PL", "NOI_KHOA_THAN_TN_SD", "NOI_KHOA_THAN_TN_SD_PL",
            "NOI_KHOA_NOI_TIET", "NOI_KHOA_NOI_TIET_PL", "NOI_KHOA_CO_XUONG_KHOP", "NOI_KHOA_CO_XUONG_KHOP_PL",
            "NOI_KHOA_THAN_KINH", "NOI_KHOA_THAN_KINH_PL", "NOI_KHOA_TAM_THAN", "NOI_KHOA_TAM_THAN_PL",
            "KET_QUA_KHAM_NGOAI_KHOA", "KHAM_NGOAI_KHOA_PL", "KET_QUA_KHAM_DA_LIEU", "KHAM_DA_LIEU_PL",
            "KET_QUA_KHAM_SAN_PHU_KHOA", "KHAM_SAN_PHU_KHOA_PL", "KHONG_KINH_MAT_PHAI", "KHONG_KINH_MAT_TRAI",
            "CO_KINH_MAT_PHAI", "CO_KINH_MAT_TRAI", "BENH_KHAC_MAT", "KHAM_MAT_PL", "TAI_TRAI_NOI_THUONG",
            "TAI_TRAI_NOI_THAM", "TAI_PHAI_NOI_THUONG", "TAI_PHAI_NOI_THAM", "BENH_KHAC_TAI_MUI_HONG",
            "KHAM_TAI_MUI_HONG_PL", "HAM_TREN", "HAM_DUOI", "BENH_KHAC_RANG_HAM_MAT", "KHAM_RANG_HAM_MAT_PL",
            // VI. Kết luận
            "PHAN_LOAI_SK", "KET_LUAN_BENH", "CAC_BENH_TAT_NEU_CO"
        ];
        wtr.write_record(&headers).unwrap();

        for r in &self.records {
            let d = &r.data;
            let row = [
                d.ho_ten.as_str(), &d.gioi_tinh.to_string(), d.ngay_sinh.as_str(), d.ma_dan_toc.as_str(),
                d.so_cccd.as_str(), d.ngaycap_cccd.as_str(), d.noicap_cccd.as_str(), d.dia_chi.as_str(),
                d.matinh_cu_tru.as_str(), d.maxa_cu_tru.as_str(), d.ma_nghe_nghiep.as_str(),
                d.noi_lam_viec_hoc_tap.as_str(), d.dien_thoai.as_str(), d.nhom_mau.as_str(),
                d.ly_do_vv.as_str(), d.ma_lk.as_str(), d.ma_cskcb.as_str(), d.ma_gtin_cskcb.as_str(),
                d.doi_tuong.as_str(), &d.nguon_chi_tra.to_string(), d.ma_loai_kcb.as_str(), d.ngay_vao.as_str(),
                &d.tsgd_mac_benh.to_string(), d.tsgd_ma_benh.as_str(), &d.tsbt_benh_trong_5_nam_qua.to_string(),
                &d.tsbt_benh_than_kinh.to_string(), &d.tsbt_benh_mat.to_string(), &d.tsbt_benh_tai.to_string(),
                &d.tsbt_benh_tim.to_string(), &d.tsbt_phau_thuat_tim.to_string(), &d.tsbt_tang_huyet_ap.to_string(),
                &d.tsbt_kho_tho.to_string(), &d.tsbt_benh_phoi.to_string(), &d.tsbt_benh_than.to_string(),
                &d.tsbt_nghien_ruou.to_string(), &d.tsbt_dai_thao_duong.to_string(), &d.tsbt_benh_tam_than.to_string(),
                &d.tsbt_mat_y_thuc.to_string(), &d.tsbt_ngat.to_string(), &d.tsbt_benh_tieu_hoa.to_string(),
                &d.tsbt_roi_loan_giac_ngu.to_string(), &d.tsbt_tai_bien.to_string(), &d.tsbt_benh_cot_song.to_string(),
                &d.tsbt_ruou_thuong_xuyen.to_string(), &d.tsbt_ma_tuy.to_string(), &d.tsbt_benh_khac.to_string(),
                d.tsbt_ma_benh_khac.as_str(), &d.tsbt_dang_dieu_tri_benh.to_string(), d.tsbt_ma_benh.as_str(),
                d.tsbt_ten_thuoc_lieu_luong.as_str(), &d.tsbt_thai_san.to_string(), d.tsbt_ma_benh_thai_san.as_str(),
                d.tsbt_ten_thuoc_thai_san.as_str(), d.chieu_cao.as_str(), d.can_nang.as_str(),
                d.chi_so_bmi.as_str(), d.mach.as_str(), d.huyet_ap.as_str(), &d.kham_the_luc_pl.to_string(),
                d.noi_khoa_tuan_hoan.as_str(), &d.noi_khoa_tuan_hoan_pl.to_string(), d.noi_khoa_ho_hap.as_str(),
                &d.noi_khoa_ho_hap_pl.to_string(), d.noi_khoa_tieu_hoa.as_str(), &d.noi_khoa_tieu_hoa_pl.to_string(),
                d.noi_khoa_than_tn_sd.as_str(), &d.noi_khoa_than_tn_sd_pl.to_string(), d.noi_khoa_noi_tiet.as_str(),
                &d.noi_khoa_noi_tiet_pl.to_string(), d.noi_khoa_co_xuong_khop.as_str(), &d.noi_khoa_co_xuong_khop_pl.to_string(),
                d.noi_khoa_than_kinh.as_str(), &d.noi_khoa_than_kinh_pl.to_string(), d.noi_khoa_tam_than.as_str(),
                &d.noi_khoa_tam_than_pl.to_string(), d.ket_qua_kham_ngoai_khoa.as_str(), &d.kham_ngoai_khoa_pl.to_string(),
                d.ket_qua_kham_da_lieu.as_str(), &d.kham_da_lieu_pl.to_string(), d.ket_qua_kham_san_phu_khoa.as_str(),
                &d.kham_san_phu_khoa_pl.to_string(), d.khong_kinh_mat_phai.as_str(), d.khong_kinh_mat_trai.as_str(),
                d.co_kinh_mat_phai.as_str(), d.co_kinh_mat_trai.as_str(), d.benh_khac_mat.as_str(),
                &d.kham_mat_pl.to_string(), d.tai_trai_noi_thuong.as_str(), d.tai_trai_noi_tham.as_str(),
                d.tai_phai_noi_thuong.as_str(), d.tai_phai_noi_tham.as_str(), d.benh_khac_tai_mui_hong.as_str(),
                &d.kham_tai_mui_hong_pl.to_string(), d.ham_tren.as_str(), d.ham_duoi.as_str(),
                d.benh_khac_rang_ham_mat.as_str(), &d.kham_rang_ham_mat_pl.to_string(), &d.phan_loai_sk.to_string(),
                d.ket_luan_benh.as_str(), d.cac_benh_tat_neu_co.as_str()
            ];
            wtr.write_record(&row).unwrap();
        }

        let mut out = vec![0xEF, 0xBB, 0xBF];
        out.extend(wtr.into_inner().unwrap());
        fs::write("XUAT_DULIEU_KSK_QD3176.csv", out).unwrap();
        self.status_msg = "Đã xuất file thành công: XUAT_DULIEU_KSK_QD3176.csv".into();
    }

    fn cat_items(cats: &[CatalogRecord], ty: &str, parent: Option<&str>) -> Vec<(String, String)> {
        cats.iter()
            .filter(|c| c.cat_type == ty && parent.map_or(true, |p| c.parent_code == p))
            .map(|c| (c.code.clone(), if ty == "icd10" && !c.name.starts_with(&c.code) { format!("{} - {}", c.code, c.name) } else { c.name.clone() }))
            .collect()
    }

    fn ui_radio_pl_num(ui: &mut egui::Ui, val: &mut i32) {
        ui.horizontal(|ui| {
            ui.radio_value(val, 1, "Loại I");
            ui.radio_value(val, 2, "Loại II");
            ui.radio_value(val, 3, "Loại III");
            ui.radio_value(val, 4, "Loại IV");
            ui.radio_value(val, 5, "Loại V");
        });
    }

    fn ui_radio_yn(ui: &mut egui::Ui, val: &mut i32) {
        ui.horizontal(|ui| {
            ui.radio_value(val, 0, "Không");
            ui.radio_value(val, 1, "Có");
        });
    }

    fn ui_multiline(ui: &mut egui::Ui, label: &str, val: &mut String) {
        ui.vertical(|ui| {
            ui.strong(label);
            ui.add(egui::TextEdit::multiline(val).desired_rows(3).desired_width(f32::INFINITY));
        });
    }
}

impl eframe::App for KskApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.logged_in {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(80.0);
                    ui.heading("HỆ THỐNG NHẬP LIỆU KHÁM SỨC KHỎE - QĐ 3176/QĐ-BYT");
                    ui.label("Phần mềm nhập liệu lâm sàng & quản lý danh mục");
                    ui.add_space(20.0);

                    egui::Frame::group(ui.style()).show(ui, |ui| {
                        ui.set_width(380.0);
                        ui.label("Chọn nhanh tài khoản bác sĩ chuyên khoa:");
                        egui::ComboBox::from_id_source("cb_fast_login")
                            .selected_text(&self.login_user)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.login_user, "admin".into(), "admin (Quản trị viên)");
                                ui.selectable_value(&mut self.login_user, "client".into(), "client (Hành chính / Tiếp đón)");
                                ui.selectable_value(&mut self.login_user, "bs_theluc".into(), "bs_theluc (Bác sĩ Thể lực)");
                                ui.selectable_value(&mut self.login_user, "bs_noi".into(), "bs_noi (Bác sĩ Nội khoa)");
                                ui.selectable_value(&mut self.login_user, "bs_ngoai".into(), "bs_ngoai (Bác sĩ Ngoại khoa)");
                                ui.selectable_value(&mut self.login_user, "bs_dalieu".into(), "bs_dalieu (Bác sĩ Da liễu)");
                                ui.selectable_value(&mut self.login_user, "bs_san".into(), "bs_san (Bác sĩ Sản phụ khoa)");
                                ui.selectable_value(&mut self.login_user, "bs_mat".into(), "bs_mat (Bác sĩ Mắt)");
                                ui.selectable_value(&mut self.login_user, "bs_tmh".into(), "bs_tmh (Bác sĩ Tai Mũi Họng)");
                                ui.selectable_value(&mut self.login_user, "bs_rhm".into(), "bs_rhm (Bác sĩ Răng Hàm Mặt)");
                                ui.selectable_value(&mut self.login_user, "bs_cls".into(), "bs_cls (Bác sĩ Cận lâm sàng)");
                                ui.selectable_value(&mut self.login_user, "bs_ketluan".into(), "bs_ketluan (Bác sĩ Kết luận)");
                            });

                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            ui.label("Tài khoản:");
                            ui.text_edit_singleline(&mut self.login_user);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Mật khẩu:");
                            ui.add(egui::TextEdit::singleline(&mut self.login_pass).password(true));
                        });
                        ui.add_space(10.0);
                        if ui.button("   ĐĂNG NHẬP HỆ THỐNG   ").clicked() {
                            let mut stmt = self.conn.prepare("SELECT role FROM users WHERE username = ?1 AND password = ?2").unwrap();
                            if let Ok(role) = stmt.query_row(params![self.login_user, self.login_pass], |r| r.get::<_, String>(0)) {
                                self.current_user = self.login_user.clone();
                                self.current_role = role.clone();
                                self.logged_in = true;
                                self.dept_tab = match role.as_str() {
                                    "bs_theluc" => 1,
                                    "bs_noi" => 2,
                                    "bs_ngoai" => 3,
                                    "bs_dalieu" => 4,
                                    "bs_san" => 5,
                                    "bs_mat" => 6,
                                    "bs_tmh" => 7,
                                    "bs_rhm" => 8,
                                    "bs_cls" => 9,
                                    "bs_ketluan" => 10,
                                    _ => 0,
                                };
                                self.status_msg = format!("Chào mừng {}", self.current_user);
                            } else {
                                self.status_msg = "Sai tài khoản hoặc mật khẩu!".into();
                            }
                        }
                        if !self.status_msg.is_empty() {
                            ui.colored_label(egui::Color32::RED, &self.status_msg);
                        }
                    });
                });
            });
            return;
        }

        // TOP BAR
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.strong("PHẦN MỀM KSK QĐ 3176");
                ui.separator();
                ui.selectable_value(&mut self.active_tab, 0, "Phiếu Nhập Liệu");
                ui.selectable_value(&mut self.active_tab, 1, format!("Hồ Sơ ({})", self.records.len()));
                ui.selectable_value(&mut self.active_tab, 2, "Quản Lý Danh Mục");
                ui.separator();

                if ui.button(" Xuất CSV (Chuẩn 3176)").clicked() {
                    self.export_csv();
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Đăng xuất").clicked() {
                        self.logged_in = false;
                    }
                    ui.label(format!("BS: {} [{}]", self.current_user, self.current_role));
                });
            });
        });

        // STATUS BAR
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status_msg);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // TAB 2: QUẢN LÝ DANH MỤC RIÊNG BIỆT (IMPORT HÀNG LOẠT EXCEL / TSV)
            if self.active_tab == 2 {
                ui.heading("QUẢN LÝ VÀ CẤU HÌNH DANH MỤC HỆ THỐNG");
                ui.horizontal(|ui| {
                    ui.label("Chọn phân hệ danh mục cần cấu hình:");
                    let cats = [
                        ("priority", "1. Đối tượng (16 mã)"),
                        ("payment", "2. Nguồn chi trả"),
                        ("province", "3. Tỉnh / Thành phố"),
                        ("ward", "4. Phường / Xã"),
                        ("ethnicity", "5. Dân tộc"),
                        ("occupation", "6. Nghề nghiệp"),
                        ("icd10", "7. Mã bệnh ICD-10"),
                    ];
                    for (k, label) in cats {
                        ui.selectable_value(&mut self.cur_cat_tab, k.to_string(), label);
                    }
                });
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Mã:");
                    ui.text_edit_singleline(&mut self.cat_code_input);
                    ui.label("Tên danh mục:");
                    ui.text_edit_singleline(&mut self.cat_name_input);
                    if self.cur_cat_tab == "ward" {
                        ui.label("Mã Tỉnh cha:");
                        ui.text_edit_singleline(&mut self.cat_parent_input);
                    }
                    if ui.button("+ Thêm thủ công").clicked() {
                        if !self.cat_code_input.is_empty() && !self.cat_name_input.is_empty() {
                            let _ = self.conn.execute(
                                "INSERT OR REPLACE INTO catalogs (cat_type, code, name, parent_code) VALUES (?1, ?2, ?3, ?4)",
                                params![self.cur_cat_tab, self.cat_code_input, self.cat_name_input, self.cat_parent_input],
                            );
                            self.cat_code_input.clear();
                            self.cat_name_input.clear();
                            self.reload_catalogs();
                            self.status_msg = "Đã lưu danh mục thành công!".into();
                        }
                    }
                });

                ui.add_space(8.0);
                egui::CollapsingHeader::new("⬇ Import hàng loạt từ Excel (Dán nhiều dòng)")
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.label("Định dạng mỗi dòng copy từ Excel: Mã [Tab] Tên hiển thị [Tab] Mã tỉnh cha (nếu là xã):");
                        ui.add(egui::TextEdit::multiline(&mut self.import_tsv_text).desired_rows(4).desired_width(f32::INFINITY));
                        if ui.button("Bắt đầu Import vào Database").clicked() {
                            for line in self.import_tsv_text.lines() {
                                let parts: Vec<&str> = if line.contains('\t') { line.split('\t').collect() } else { line.split('|').collect() };
                                if parts.len() >= 2 {
                                    let code = parts[0].trim();
                                    let name = parts[1].trim();
                                    let parent = if parts.len() >= 3 { parts[2].trim() } else { "" };
                                    let _ = self.conn.execute(
                                        "INSERT OR REPLACE INTO catalogs (cat_type, code, name, parent_code) VALUES (?1, ?2, ?3, ?4)",
                                        params![self.cur_cat_tab, code, name, parent],
                                    );
                                }
                            }
                            self.import_tsv_text.clear();
                            self.reload_catalogs();
                            self.status_msg = "Đã import thành công dữ liệu vào danh mục!".into();
                        }
                    });

                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("grid_cat_display").striped(true).min_col_width(100.0).show(ui, |ui| {
                        ui.strong("ID"); ui.strong("Mã"); ui.strong("Tên hiển thị"); ui.strong("Thuộc cha"); ui.strong("Thao tác");
                        ui.end_row();

                        let mut del_id = None;
                        for c in self.catalogs.iter().filter(|x| x.cat_type == self.cur_cat_tab) {
                            ui.label(c.id.to_string());
                            ui.label(&c.code);
                            ui.label(&c.name);
                            ui.label(&c.parent_code);
                            if ui.button("Xóa").clicked() {
                                del_id = Some(c.id);
                            }
                            ui.end_row();
                        }
                        if let Some(id) = del_id {
                            let _ = self.conn.execute("DELETE FROM catalogs WHERE id = ?", params![id]);
                            self.reload_catalogs();
                        }
                    });
                });
                return;
            }

            // TAB 1: DANH SÁCH HỒ SƠ
            if self.active_tab == 1 {
                ui.heading("DANH SÁCH HỒ SƠ KHÁM SỨC KHỎE");
                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("grid_records").striped(true).min_col_width(80.0).show(ui, |ui| {
                        ui.strong("ID"); ui.strong("Mã LK"); ui.strong("Họ và tên"); ui.strong("Số CCCD");
                        ui.strong("Ngày khám"); ui.strong("Trạng thái"); ui.strong("Thao tác");
                        ui.end_row();

                        let mut unlock_id = None;
                        let mut load_id = None;

                        for r in &self.records {
                            ui.label(r.id.to_string());
                            ui.label(&r.data.ma_lk);
                            ui.label(&r.ho_ten);
                            ui.label(&r.so_cccd);
                            ui.label(&r.ngay_vao);
                            if r.status == 1 {
                                ui.colored_label(egui::Color32::from_rgb(220, 140, 0), "🔒 Đã ký/khóa");
                            } else {
                                ui.colored_label(egui::Color32::GREEN, "📝 Bản nháp");
                            }

                            ui.horizontal(|ui| {
                                if ui.button("Mở hồ sơ").clicked() {
                                    load_id = Some(r.id);
                                }
                                if r.status == 1 && self.current_role == "admin" && ui.button("Mở khóa").clicked() {
                                    unlock_id = Some(r.id);
                                }
                            });
                            ui.end_row();
                        }

                        if let Some(id) = unlock_id {
                            self.conn.execute("UPDATE records SET status = 0 WHERE id = ?", params![id]).unwrap();
                            self.reload_records();
                            self.status_msg = format!("Đã mở khóa hồ sơ #{}", id);
                        }
                        if let Some(id) = load_id {
                            if let Some(found) = self.records.iter().find(|x| x.id == id) {
                                self.current_id = Some(found.id);
                                self.current_status = found.status;
                                self.form = found.data.clone();
                                self.active_tab = 0;
                            }
                        }
                    });
                });
                return;
            }

            // TAB 0: BIỂU MẪU NHẬP LIỆU (THEO CÁC CHUYÊN KHOA)
            ui.horizontal(|ui| {
                if self.current_status == 1 {
                    ui.colored_label(egui::Color32::from_rgb(220, 140, 0), "🔒 HỒ SƠ ĐÃ XÁC NHẬN KHÓA");
                } else {
                    ui.colored_label(egui::Color32::GREEN, "📝 ĐANG SOẠN THẢO HỒ SƠ");
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("+ Hồ Sơ Mới").clicked() {
                        self.current_id = None;
                        self.current_status = 0;
                        self.form = KskRecordPayload::default();
                    }
                    if self.current_status == 0 || self.current_role == "admin" {
                        if ui.button("Ký / Khóa Hồ Sơ").clicked() {
                            self.save_record(true);
                        }
                        if ui.button("Lưu Nháp").clicked() {
                            self.save_record(false);
                        }
                    } else if self.current_role == "admin" && ui.button("Mở Khóa Hồ Sơ").clicked() {
                        if let Some(id) = self.current_id {
                            self.conn.execute("UPDATE records SET status = 0 WHERE id = ?", params![id]).unwrap();
                            self.current_status = 0;
                            self.reload_records();
                        }
                    }
                });
            });

            ui.separator();

            // MENU PHÂN KHOA
            ui.horizontal_wrapped(|ui| {
                let tabs = [
                    (0, "I. Hành Chính & II. Lần Khám", "hchinh"),
                    (1, "III. Tiền Sử Bệnh Tật", "hchinh"),
                    (2, "IV. Đo Thể Lực", "theluc"),
                    (3, "V.A. Nội Khoa", "noi"),
                    (4, "V.B. Ngoại Khoa", "ngoai"),
                    (5, "V.C. Da Liễu", "dalieu"),
                    (6, "V.D. Sản Phụ Khoa", "san"),
                    (7, "V.E. Khám Mắt", "mat"),
                    (8, "V.F. Tai Mũi Họng", "tmh"),
                    (9, "V.G. Răng Hàm Mặt", "rhm"),
                    (10, "VI. Cận Lâm Sàng", "cls"),
                    (11, "VII. Kết Luận", "ketluan"),
                ];

                for (idx, title, role_key) in tabs {
                    let can_edit = self.can_edit_dept(role_key);
                    let mut btn_text = egui::RichText::new(title);
                    if can_edit {
                        btn_text = btn_text.strong().color(egui::Color32::from_rgb(0, 180, 255));
                    }
                    if ui.selectable_label(self.dept_tab == idx, btn_text).clicked() {
                        self.dept_tab = idx;
                    }
                }
            });

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                match self.dept_tab {
                    // I. HÀNH CHÍNH & II. LẦN KHÁM
                    0 => {
                        let enabled = self.can_edit_dept("hchinh");
                        ui.add_enabled_ui(enabled, |ui| {
                            egui::Grid::new("grid_hc_detail").num_columns(4).spacing([14.0, 8.0]).show(ui, |ui| {
                                ui.label("1. Họ và tên:"); ui.text_edit_singleline(&mut self.form.ho_ten);
                                ui.label("2. Giới tính:");
                                egui::ComboBox::from_id_source("cb_gt").selected_text(if self.form.gioi_tinh == 1 { "1 - Nam" } else { "2 - Nữ" }).show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.form.gioi_tinh, 1, "1 - Nam");
                                    ui.selectable_value(&mut self.form.gioi_tinh, 2, "2 - Nữ");
                                });
                                ui.end_row();

                                ui.label("3. Ngày sinh:"); ui.text_edit_singleline(&mut self.form.ngay_sinh);
                                ui.label("4. Dân tộc:");
                                let eths = Self::cat_items(&self.catalogs, "ethnicity", None);
                                search_combo(ui, "cb_eth", &mut self.form.ma_dan_toc, &eths);
                                ui.end_row();

                                ui.label("5. Số CCCD:"); ui.text_edit_singleline(&mut self.form.so_cccd);
                                ui.label("6. Ngày cấp CCCD:"); ui.text_edit_singleline(&mut self.form.ngaycap_cccd);
                                ui.end_row();

                                ui.label("7. Nơi cấp CCCD:"); ui.text_edit_singleline(&mut self.form.noicap_cccd);
                                ui.label("8. Tỉnh / TP:");
                                let provs = Self::cat_items(&self.catalogs, "province", None);
                                let prov_changed = search_combo(ui, "cb_prov", &mut self.form.matinh_cu_tru, &provs);
                                ui.end_row();

                                if prov_changed {
                                    if let Some(first_w) = self.catalogs.iter().find(|c| c.cat_type == "ward" && c.parent_code == self.form.matinh_cu_tru) {
                                        self.form.maxa_cu_tru = first_w.code.clone();
                                    }
                                }

                                ui.label("9. Phường / Xã:");
                                let wards = Self::cat_items(&self.catalogs, "ward", Some(&self.form.matinh_cu_tru));
                                search_combo(ui, "cb_ward", &mut self.form.maxa_cu_tru, &wards);

                                ui.label("10. Nghề nghiệp:");
                                let occs = Self::cat_items(&self.catalogs, "occupation", None);
                                search_combo(ui, "cb_occ", &mut self.form.ma_nghe_nghiep, &occs);
                                ui.end_row();

                                ui.label("11. Điện thoại:"); ui.text_edit_singleline(&mut self.form.dien_thoai);
                                ui.label("12. Nhóm máu:"); ui.text_edit_singleline(&mut self.form.nhom_mau);
                                ui.end_row();

                                ui.label("13. Mã Lượt khám (MA_LK):"); ui.text_edit_singleline(&mut self.form.ma_lk);
                                ui.label("14. Ngày khám (NGAY_VAO):"); ui.text_edit_singleline(&mut self.form.ngay_vao);
                                ui.end_row();

                                ui.label("15. Mã CSKCB:"); ui.text_edit_singleline(&mut self.form.ma_cskcb);
                                ui.label("16. Mã GLN CSKCB (13 số):"); ui.text_edit_singleline(&mut self.form.ma_gtin_cskcb);
                                ui.end_row();

                                ui.label("17. Đối tượng (QĐ 3176):");
                                let pris = Self::cat_items(&self.catalogs, "priority", None);
                                search_combo(ui, "cb_pri", &mut self.form.doi_tuong, &pris);

                                ui.label("18. Nguồn chi trả:");
                                let pays = Self::cat_items(&self.catalogs, "payment", None);
                                let mut pay_s = self.form.nguon_chi_tra.to_string();
                                if search_combo(ui, "cb_pay", &mut pay_s, &pays) {
                                    if let Ok(n) = pay_s.parse::<i32>() { self.form.nguon_chi_tra = n; }
                                }
                                ui.end_row();
                            });

                            ui.add_space(8.0);
                            Self::ui_multiline(ui, "Nơi ở hiện tại (DIA_CHI chi tiết):", &mut self.form.dia_chi);
                            Self::ui_multiline(ui, "Nơi làm việc, học tập hiện tại:", &mut self.form.noi_lam_viec_hoc_tap);
                            Self::ui_multiline(ui, "Lý do khám sức khỏe (LY_DO_VV):", &mut self.form.ly_do_vv);
                        });
                    }

                    // III. TIỀN SỬ BỆNH TẬT
                    1 => {
                        let enabled = self.can_edit_dept("hchinh");
                        ui.add_enabled_ui(enabled, |ui| {
                            ui.strong("A. Tiền sử gia đình mắc bệnh truyền nhiễm, tim mạch, ĐTĐ, hen, ung thư, tâm thần:");
                            Self::ui_radio_yn(ui, &mut self.form.tsgd_mac_benh);
                            ui.label("Mã bệnh ICD-10 tiền sử gia đình (chọn nhiều):");
                            let icds = Self::cat_items(&self.catalogs, "icd10", None);
                            multi_icd(ui, "icd_tsgd", &mut self.form.tsgd_ma_benh, &icds);

                            ui.separator();
                            ui.strong("B. Tiền sử bản thân (22 chỉ số):");
                            egui::Grid::new("grid_tsbt").num_columns(4).spacing([14.0, 6.0]).show(ui, |ui| {
                                ui.label("1. Bệnh hay bị thương trong 5 năm qua:"); Self::ui_radio_yn(ui, &mut self.form.tsbt_benh_trong_5_nam_qua);
                                ui.label("2. Bệnh thần kinh hay bị thương ở đầu:"); Self::ui_radio_yn(ui, &mut self.form.tsbt_benh_than_kinh);
                                ui.end_row();

                                ui.label("3. Bệnh mắt hoặc giảm thị lực:"); Self::ui_radio_yn(ui, &mut self.form.tsbt_benh_mat);
                                ui.label("4. Bệnh ở tai, giảm sức nghe, thăng bằng:"); Self::ui_radio_yn(ui, &mut self.form.tsbt_benh_tai);
                                ui.end_row();

                                ui.label("5. Bệnh ở tim, nhồi máu cơ tim:"); Self::ui_radio_yn(ui, &mut self.form.tsbt_benh_tim);
                                ui.label("6. Phẫu thuật can thiệp tim mạch:"); Self::ui_radio_yn(ui, &mut self.form.tsbt_phau_thuat_tim);
                                ui.end_row();

                                ui.label("7. Tăng huyết áp:"); Self::ui_radio_yn(ui, &mut self.form.tsbt_tang_huyet_ap);
                                ui.label("8. Khó thở:"); Self::ui_radio_yn(ui, &mut self.form.tsbt_kho_tho);
                                ui.end_row();

                                ui.label("9. Bệnh phổi, hen, viêm phế quản mạn:"); Self::ui_radio_yn(ui, &mut self.form.tsbt_benh_phoi);
                                ui.label("10. Bệnh thận, lọc máu:"); Self::ui_radio_yn(ui, &mut self.form.tsbt_benh_than);
                                ui.end_row();

                                ui.label("11. Nghiện rượu, bia:"); Self::ui_radio_yn(ui, &mut self.form.tsbt_nghien_ruou);
                                ui.label("12. Đái tháo đường:"); Self::ui_radio_yn(ui, &mut self.form.tsbt_dai_thao_duong);
                                ui.end_row();

                                ui.label("13. Bệnh tâm thần:"); Self::ui_radio_yn(ui, &mut self.form.tsbt_benh_tam_than);
                                ui.label("14. Mất ý thức, rối loạn ý thức:"); Self::ui_radio_yn(ui, &mut self.form.tsbt_mat_y_thuc);
                                ui.end_row();

                                ui.label("15. Ngất, chóng mặt:"); Self::ui_radio_yn(ui, &mut self.form.tsbt_ngat);
                                ui.label("16. Bệnh tiêu hóa:"); Self::ui_radio_yn(ui, &mut self.form.tsbt_benh_tieu_hoa);
                                ui.end_row();

                                ui.label("17. Rối loạn giấc ngủ, ngáy to:"); Self::ui_radio_yn(ui, &mut self.form.tsbt_roi_loan_giac_ngu);
                                ui.label("18. Tai biến mạch máu não / liệt:"); Self::ui_radio_yn(ui, &mut self.form.tsbt_tai_bien);
                                ui.end_row();

                                ui.label("19. Bệnh hoặc tổn thương cột sống:"); Self::ui_radio_yn(ui, &mut self.form.tsbt_benh_cot_song);
                                ui.label("20. Sử dụng rượu thường xuyên:"); Self::ui_radio_yn(ui, &mut self.form.tsbt_ruou_thuong_xuyen);
                                ui.end_row();

                                ui.label("21. Sử dụng ma túy, chất gây nghiện:"); Self::ui_radio_yn(ui, &mut self.form.tsbt_ma_tuy);
                                ui.label("22. Mắc bệnh khác:"); Self::ui_radio_yn(ui, &mut self.form.tsbt_benh_khac);
                                ui.end_row();
                            });

                            ui.add_space(8.0);
                            ui.strong("Mã bệnh khác (ICD-10, chọn nhiều):");
                            let icds_bt = Self::cat_items(&self.catalogs, "icd10", None);
                            multi_icd(ui, "icd_tsbt", &mut self.form.tsbt_ma_benh_khac, &icds_bt);
                            Self::ui_multiline(ui, "Thuốc đang sử dụng và liều lượng:", &mut self.form.tsbt_ten_thuoc_lieu_luong);
                            Self::ui_multiline(ui, "Tiền sử thai sản & thuốc điều trị thai sản:", &mut self.form.tsbt_ten_thuoc_thai_san);
                        });
                    }

                    // IV. KHÁM THỂ LỰC
                    2 => {
                        let enabled = self.can_edit_dept("theluc");
                        ui.add_enabled_ui(enabled, |ui| {
                            egui::Grid::new("grid_tl").num_columns(4).spacing([14.0, 8.0]).show(ui, |ui| {
                                ui.label("Chiều cao (cm):");
                                if ui.text_edit_singleline(&mut self.form.chieu_cao).changed() {
                                    if let (Ok(h), Ok(w)) = (self.form.chieu_cao.parse::<f64>(), self.form.can_nang.parse::<f64>()) {
                                        if h > 0.0 { self.form.chi_so_bmi = format!("{:.2}", w / ((h/100.0)*(h/100.0))); }
                                    }
                                }
                                ui.label("Cân nặng (kg):");
                                if ui.text_edit_singleline(&mut self.form.can_nang).changed() {
                                    if let (Ok(h), Ok(w)) = (self.form.chieu_cao.parse::<f64>(), self.form.can_nang.parse::<f64>()) {
                                        if h > 0.0 { self.form.chi_so_bmi = format!("{:.2}", w / ((h/100.0)*(h/100.0))); }
                                    }
                                }
                                ui.end_row();

                                ui.label("Chỉ số BMI (Tự tính):"); ui.text_edit_singleline(&mut self.form.chi_so_bmi);
                                ui.label("Mạch (lần/phút):"); ui.text_edit_singleline(&mut self.form.mach);
                                ui.end_row();

                                ui.label("Huyết áp (mmHg):"); ui.text_edit_singleline(&mut self.form.huyet_ap);
                                ui.label("Phân loại thể lực (1-5):");
                                Self::ui_radio_pl_num(ui, &mut self.form.kham_the_luc_pl);
                                ui.end_row();
                            });
                        });
                    }

                    // V.A. NỘI KHOA
                    3 => {
                        let enabled = self.can_edit_dept("noi");
                        ui.add_enabled_ui(enabled, |ui| {
                            egui::Grid::new("grid_noi").num_columns(3).spacing([14.0, 8.0]).show(ui, |ui| {
                                ui.strong("Chuyên khoa"); ui.strong("Kết quả khám lâm sàng chi tiết"); ui.strong("Phân loại");
                                ui.end_row();

                                ui.label("Tuần hoàn:");
                                ui.add(egui::TextEdit::multiline(&mut self.form.noi_khoa_tuan_hoan).desired_rows(2).desired_width(450.0));
                                Self::ui_radio_pl_num(ui, &mut self.form.noi_khoa_tuan_hoan_pl);
                                ui.end_row();

                                ui.label("Hô hấp:");
                                ui.add(egui::TextEdit::multiline(&mut self.form.noi_khoa_ho_hap).desired_rows(2).desired_width(450.0));
                                Self::ui_radio_pl_num(ui, &mut self.form.noi_khoa_ho_hap_pl);
                                ui.end_row();

                                ui.label("Tiêu hóa:");
                                ui.add(egui::TextEdit::multiline(&mut self.form.noi_khoa_tieu_hoa).desired_rows(2).desired_width(450.0));
                                Self::ui_radio_pl_num(ui, &mut self.form.noi_khoa_tieu_hoa_pl);
                                ui.end_row();

                                ui.label("Thận - Tiết niệu - Sinh dục:");
                                ui.add(egui::TextEdit::multiline(&mut self.form.noi_khoa_than_tn_sd).desired_rows(2).desired_width(450.0));
                                Self::ui_radio_pl_num(ui, &mut self.form.noi_khoa_than_tn_sd_pl);
                                ui.end_row();

                                ui.label("Nội tiết:");
                                ui.add(egui::TextEdit::multiline(&mut self.form.noi_khoa_noi_tiet).desired_rows(2).desired_width(450.0));
                                Self::ui_radio_pl_num(ui, &mut self.form.noi_khoa_noi_tiet_pl);
                                ui.end_row();

                                ui.label("Cơ - Xương - Khớp:");
                                ui.add(egui::TextEdit::multiline(&mut self.form.noi_khoa_co_xuong_khop).desired_rows(2).desired_width(450.0));
                                Self::ui_radio_pl_num(ui, &mut self.form.noi_khoa_co_xuong_khop_pl);
                                ui.end_row();

                                ui.label("Thần kinh:");
                                ui.add(egui::TextEdit::multiline(&mut self.form.noi_khoa_than_kinh).desired_rows(2).desired_width(450.0));
                                Self::ui_radio_pl_num(ui, &mut self.form.noi_khoa_than_kinh_pl);
                                ui.end_row();

                                ui.label("Tâm thần:");
                                ui.add(egui::TextEdit::multiline(&mut self.form.noi_khoa_tam_than).desired_rows(2).desired_width(450.0));
                                Self::ui_radio_pl_num(ui, &mut self.form.noi_khoa_tam_than_pl);
                                ui.end_row();
                            });
                        });
                    }

                    // V.B. NGOẠI KHOA
                    4 => {
                        let enabled = self.can_edit_dept("ngoai");
                        ui.add_enabled_ui(enabled, |ui| {
                            Self::ui_multiline(ui, "Kết quả khám Ngoại khoa (Mô tả chi tiết tổn thương):", &mut self.form.ket_qua_kham_ngoai_khoa);
                            ui.horizontal(|ui| {
                                ui.label("Phân loại Ngoại khoa:");
                                Self::ui_radio_pl_num(ui, &mut self.form.kham_ngoai_khoa_pl);
                            });
                        });
                    }

                    // V.C. DA LIỄU
                    5 => {
                        let enabled = self.can_edit_dept("dalieu");
                        ui.add_enabled_ui(enabled, |ui| {
                            Self::ui_multiline(ui, "Kết quả khám Da liễu:", &mut self.form.ket_qua_kham_da_lieu);
                            ui.horizontal(|ui| {
                                ui.label("Phân loại Da liễu:");
                                Self::ui_radio_pl_num(ui, &mut self.form.kham_da_lieu_pl);
                            });
                        });
                    }

                    // V.D. SẢN PHỤ KHOA
                    6 => {
                        let enabled = self.can_edit_dept("san");
                        ui.add_enabled_ui(enabled, |ui| {
                            Self::ui_multiline(ui, "Kết quả khám Sản phụ khoa (theo Phụ lục XXV TT 32/2023):", &mut self.form.ket_qua_kham_san_phu_khoa);
                            ui.horizontal(|ui| {
                                ui.label("Phân loại Sản phụ khoa:");
                                Self::ui_radio_pl_num(ui, &mut self.form.kham_san_phu_khoa_pl);
                            });
                        });
                    }

                    // V.E. MẮT
                    7 => {
                        let enabled = self.can_edit_dept("mat");
                        ui.add_enabled_ui(enabled, |ui| {
                            egui::Grid::new("grid_mat").num_columns(4).spacing([14.0, 8.0]).show(ui, |ui| {
                                ui.label("Không kính - Phải:"); ui.text_edit_singleline(&mut self.form.khong_kinh_mat_phai);
                                ui.label("Không kính - Trái:"); ui.text_edit_singleline(&mut self.form.khong_kinh_mat_trai);
                                ui.end_row();

                                ui.label("Có kính - Phải:"); ui.text_edit_singleline(&mut self.form.co_kinh_mat_phai);
                                ui.label("Có kính - Trái:"); ui.text_edit_singleline(&mut self.form.co_kinh_mat_trai);
                                ui.end_row();
                            });
                            Self::ui_multiline(ui, "Các bệnh về mắt khác:", &mut self.form.benh_khac_mat);
                            ui.horizontal(|ui| {
                                ui.label("Phân loại Mắt:");
                                Self::ui_radio_pl_num(ui, &mut self.form.kham_mat_pl);
                            });
                        });
                    }

                    // V.F. TAI MŨI HỌNG
                    8 => {
                        let enabled = self.can_edit_dept("tmh");
                        ui.add_enabled_ui(enabled, |ui| {
                            egui::Grid::new("grid_tmh").num_columns(4).spacing([14.0, 8.0]).show(ui, |ui| {
                                ui.label("Nói thường - Phải (m):"); ui.text_edit_singleline(&mut self.form.tai_phai_noi_thuong);
                                ui.label("Nói thường - Trái (m):"); ui.text_edit_singleline(&mut self.form.tai_trai_noi_thuong);
                                ui.end_row();

                                ui.label("Nói thầm - Phải (m):"); ui.text_edit_singleline(&mut self.form.tai_phai_noi_tham);
                                ui.label("Nói thầm - Trái (m):"); ui.text_edit_singleline(&mut self.form.tai_trai_noi_tham);
                                ui.end_row();
                            });
                            Self::ui_multiline(ui, "Các bệnh Tai Mũi Họng khác:", &mut self.form.benh_khac_tai_mui_hong);
                            ui.horizontal(|ui| {
                                ui.label("Phân loại TMH:");
                                Self::ui_radio_pl_num(ui, &mut self.form.kham_tai_mui_hong_pl);
                            });
                        });
                    }

                    // V.G. RĂNG HÀM MẶT
                    9 => {
                        let enabled = self.can_edit_dept("rhm");
                        ui.add_enabled_ui(enabled, |ui| {
                            egui::Grid::new("grid_rhm").num_columns(2).spacing([14.0, 8.0]).show(ui, |ui| {
                                ui.label("Hàm trên:"); ui.text_edit_singleline(&mut self.form.ham_tren);
                                ui.end_row();
                                ui.label("Hàm dưới:"); ui.text_edit_singleline(&mut self.form.ham_duoi);
                                ui.end_row();
                            });
                            Self::ui_multiline(ui, "Các bệnh Răng Hàm Mặt khác:", &mut self.form.benh_khac_rang_ham_mat);
                            ui.horizontal(|ui| {
                                ui.label("Phân loại RHM:");
                                Self::ui_radio_pl_num(ui, &mut self.form.kham_rang_ham_mat_pl);
                            });
                        });
                    }

                    // VI. CẬN LÂM SÀNG (LƯỚI ĐỘNG CÁC CHỈ SỐ)
                    10 => {
                        let enabled = self.can_edit_dept("cls");
                        ui.add_enabled_ui(enabled, |ui| {
                            ui.horizontal(|ui| {
                                ui.strong("DANH SÁCH CHỈ SỐ CẬN LÂM SÀNG");
                                if ui.button("+ Thêm Dòng Chỉ Số").clicked() {
                                    self.form.cls_list.push(ClsItem::default());
                                }
                            });
                            ui.separator();

                            let mut del_idx = None;
                            egui::Grid::new("grid_cls_table").striped(true).min_col_width(80.0).show(ui, |ui| {
                                ui.strong("STT"); ui.strong("Mã Dịch Vụ"); ui.strong("Mã Chỉ Số"); ui.strong("Giá Trị");
                                ui.strong("Đơn Vị"); ui.strong("Mô Tả"); ui.strong("Kết Luận"); ui.strong("Xóa");
                                ui.end_row();

                                for (i, item) in self.form.cls_list.iter_mut().enumerate() {
                                    ui.label((i + 1).to_string());
                                    ui.text_edit_singleline(&mut item.ma_dich_vu);
                                    ui.text_edit_singleline(&mut item.ma_chi_so);
                                    ui.text_edit_singleline(&mut item.gia_tri);
                                    ui.text_edit_singleline(&mut item.don_vi_do);
                                    ui.text_edit_singleline(&mut item.mo_ta);
                                    ui.text_edit_singleline(&mut item.ket_luan);
                                    if ui.button("Xóa").clicked() {
                                        del_idx = Some(i);
                                    }
                                    ui.end_row();
                                }
                            });
                            if let Some(idx) = del_idx {
                                self.form.cls_list.remove(idx);
                            }
                        });
                    }

                    // VII. KẾT LUẬN & ICD-10
                    11 => {
                        let enabled = self.can_edit_dept("ketluan");
                        ui.add_enabled_ui(enabled, |ui| {
                            ui.horizontal(|ui| {
                                ui.strong("Phân loại sức khỏe chung:");
                                Self::ui_radio_pl_num(ui, &mut self.form.phan_loai_sk);
                            });
                            ui.add_space(8.0);

                            ui.strong("Mã bệnh ICD-10 kết luận (chọn nhiều, tìm theo mã/tên):");
                            let icds = Self::cat_items(&self.catalogs, "icd10", None);
                            multi_icd(ui, "icd_kl", &mut self.form.ket_luan_benh, &icds);

                            ui.add_space(8.0);
                            Self::ui_multiline(ui, "Tình trạng sức khỏe; mắc các bệnh, tật nếu có (CAC_BENH_TAT_NEU_CO):", &mut self.form.cac_benh_tat_neu_co);
                        });
                    }

                    _ => {}
                }

                ui.add_space(40.0);
            });
        });
    }
}

fn main() {
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("CRASH ERROR:\n{}", info);
        let _ = fs::write("crash_log.txt", &msg);
    }));

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1220.0, 820.0])
            .with_min_inner_size([1000.0, 700.0])
            .with_title("Hệ Thống Nhập Liệu KSK - Chuẩn Quyết định 3176/QĐ-BYT"),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        "KskApp3176",
        native_options,
        Box::new(|cc| Box::new(KskApp::new(cc))),
    ) {
        let _ = fs::write("crash_log.txt", format!("Lỗi: {:?}", e));
    }
}

// ===================== WIDGET & THEME DÙNG CHUNG =====================

fn fold_vn(s: &str) -> String {
    s.chars().map(|c| {
        let l = c.to_lowercase().next().unwrap_or(c);
        match l {
            'à'|'á'|'ạ'|'ả'|'ã'|'â'|'ầ'|'ấ'|'ậ'|'ẩ'|'ẫ'|'ă'|'ằ'|'ắ'|'ặ'|'ẳ'|'ẵ' => 'a',
            'è'|'é'|'ẹ'|'ẻ'|'ẽ'|'ê'|'ề'|'ế'|'ệ'|'ể'|'ễ' => 'e',
            'ì'|'í'|'ị'|'ỉ'|'ĩ' => 'i',
            'ò'|'ó'|'ọ'|'ỏ'|'õ'|'ô'|'ồ'|'ố'|'ộ'|'ổ'|'ỗ'|'ơ'|'ờ'|'ớ'|'ợ'|'ở'|'ỡ' => 'o',
            'ù'|'ú'|'ụ'|'ủ'|'ũ'|'ư'|'ừ'|'ứ'|'ự'|'ử'|'ữ' => 'u',
            'ỳ'|'ý'|'ỵ'|'ỷ'|'ỹ' => 'y',
            'đ' => 'd',
            x => x,
        }
    }).collect()
}

fn apply_theme(ctx: &egui::Context, dark: bool) {
    let mut v = if dark { egui::Visuals::dark() } else { egui::Visuals::light() };
    let teal = egui::Color32::from_rgb(14, 116, 144);
    v.selection.bg_fill = teal;
    v.hyperlink_color = teal;
    let r = egui::Rounding::same(6.0);
    v.widgets.noninteractive.rounding = r;
    v.widgets.inactive.rounding = r;
    v.widgets.hovered.rounding = r;
    v.widgets.active.rounding = r;
    v.widgets.open.rounding = r;
    v.window_rounding = egui::Rounding::same(10.0);
    if !dark {
        v.panel_fill = egui::Color32::from_rgb(241, 245, 249);
        v.widgets.inactive.bg_fill = egui::Color32::from_rgb(226, 232, 240);
        v.widgets.inactive.weak_bg_fill = egui::Color32::from_rgb(226, 232, 240);
    }
    ctx.set_visuals(v);
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);
    style.spacing.interact_size.y = 26.0;
    ctx.set_style(style);
}

/// Dropdown có ô tìm kiếm (không phân biệt dấu). Trả về true nếu giá trị đổi.
fn search_combo(ui: &mut egui::Ui, id: &str, value: &mut String, items: &[(String, String)]) -> bool {
    let cur = items.iter().find(|(c, _)| c == value).map(|(_, n)| n.clone()).unwrap_or_else(|| "Chọn...".to_string());
    let mut changed = false;
    let fid = egui::Id::new(("sc_filter", id));
    egui::ComboBox::from_id_source(id).width(260.0).selected_text(cur).show_ui(ui, |ui| {
        let mut filter: String = ui.data_mut(|d| d.get_temp(fid).unwrap_or_default());
        let r = ui.add(egui::TextEdit::singleline(&mut filter).hint_text("🔍 Tìm kiếm...").desired_width(240.0));
        if !r.has_focus() && filter.is_empty() { r.request_focus(); }
        let f = fold_vn(&filter);
        egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
            let mut shown = 0;
            for (code, name) in items {
                if f.is_empty() || fold_vn(name).contains(&f) || fold_vn(code).contains(&f) {
                    if ui.selectable_label(code == value, name).clicked() {
                        *value = code.clone();
                        changed = true;
                    }
                    shown += 1;
                    if shown >= 200 { ui.weak("... gõ thêm để thu hẹp"); break; }
                }
            }
            if shown == 0 { ui.weak("Không tìm thấy"); }
        });
        ui.data_mut(|d| d.insert_temp(fid, filter));
    });
    changed
}

/// Chọn nhiều ICD-10 (lưu dạng "A;B;C"), hiển thị chip, có nút xóa.
fn multi_icd(ui: &mut egui::Ui, id: &str, value: &mut String, icds: &[(String, String)]) {
    let mut sel: Vec<String> = value.split(';').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
    let mut remove: Option<usize> = None;
    ui.vertical(|ui| {
        ui.horizontal_wrapped(|ui| {
            for (i, code) in sel.iter().enumerate() {
                let name = icds.iter().find(|(c, _)| c == code).map(|(_, n)| n.as_str()).unwrap_or(code.as_str());
                let label = egui::RichText::new(format!("{}  ✕", name)).color(egui::Color32::WHITE);
                if ui.add(egui::Button::new(label).fill(egui::Color32::from_rgb(14, 116, 144)).rounding(12.0)).on_hover_text("Bấm để xóa").clicked() {
                    remove = Some(i);
                }
            }
            if sel.is_empty() { ui.weak("(chưa chọn mã bệnh)"); }
        });
        let fid = egui::Id::new(("icd_filter", id));
        egui::ComboBox::from_id_source(id).width(320.0).selected_text("＋ Thêm mã ICD-10").show_ui(ui, |ui| {
            let mut filter: String = ui.data_mut(|d| d.get_temp(fid).unwrap_or_default());
            ui.add(egui::TextEdit::singleline(&mut filter).hint_text("🔍 Gõ mã hoặc tên bệnh...").desired_width(300.0));
            let f = fold_vn(&filter);
            egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
                let mut shown = 0;
                for (code, name) in icds {
                    if sel.contains(code) { continue; }
                    if f.is_empty() || fold_vn(name).contains(&f) || fold_vn(code).contains(&f) {
                        if ui.selectable_label(false, name).clicked() { sel.push(code.clone()); }
                        shown += 1;
                        if shown >= 100 { ui.weak("... gõ thêm để thu hẹp"); break; }
                    }
                }
                if shown == 0 { ui.weak("Không tìm thấy"); }
            });
            ui.data_mut(|d| d.insert_temp(fid, filter));
        });
    });
    if let Some(i) = remove { sel.remove(i); }
    *value = sel.join(";");
}

