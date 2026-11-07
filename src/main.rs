#![windows_subsystem = "windows"]

use eframe::egui;
use chrono::Datelike;
use sha2::{Digest, Sha256};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;

fn hash_pass(plain: &str) -> String {
    let mut h = Sha256::new();
    h.update(plain.as_bytes());
    let res = h.finalize();
    res.iter().map(|b| format!("{:02x}", b)).collect()
}

const FONT_VN: &[u8] = include_bytes!("../assets/font_vn.ttf");
const QD1613_JSON: &str = include_str!("../qd1613_complete.json");

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

// Cấu trúc danh mục dùng chung QĐ 3176
#[derive(Clone, Serialize, Deserialize)]
struct CatalogRecord {
    id: i64,
    cat_type: String, // "province", "ward", "ethnicity", "occupation", "priority", "payment", "kcb_type", "icd10"
    code: String,
    name: String,
    parent_code: String,
}

// Cấu trúc QĐ 1613/BYT
#[derive(Clone, Serialize, Deserialize, Debug)]
struct QdSpecialty {
    id: String,
    name: String,
    parent_id: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct QdDisease {
    id: String,
    name: String,
    spec_id: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct QdClassification {
    id: String,
    disease_id: String,
    name: String,
    score: Option<i32>,
    ref_rule: Option<String>,
    ref_disease_id: Option<String>,
    modifier: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Qd1613CatalogData {
    specialties: Vec<QdSpecialty>,
    diseases: Vec<QdDisease>,
    classifications: Vec<QdClassification>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct ManualConclusionRow {
    classification: i32,
    classification_id: String,
    manifestation: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct SpecialtyConclusionExtra {
    manual_rows: Vec<ManualConclusionRow>,
    auto_overrides: HashMap<String, String>, // disease_id -> custom manifestation text
    final_note: String,
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

    // Lựa chọn động theo QĐ 1613/BYT
    #[serde(default)]
    selected_items: HashSet<String>, // Mã các chi tiết phân loại được tick chọn (Checkbox)
    #[serde(default)]
    selected_classifications: HashMap<String, String>, // Tương thích ngược
    #[serde(default)]
    specialty_extras: HashMap<String, SpecialtyConclusionExtra>, // spec_id -> manual rows, overrides, note
    #[serde(default)]
    dominant_hand: String, // "right" | "left"
    #[serde(default)]
    voice_hearing: i32, // 1..5
    #[serde(default)]
    audiometer_hearing: i32, // 1..5
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
            ma_loai_kcb: "15".into(), // KSK định kỳ
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
            chieu_cao: "168".into(),
            can_nang: "62".into(),
            chi_so_bmi: "21.97".into(),
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
            khong_kinh_mat_phai: "10".into(),
            khong_kinh_mat_trai: "10".into(),
            co_kinh_mat_phai: "".into(),
            co_kinh_mat_trai: "".into(),
            benh_khac_mat: "Bình thường".into(),
            kham_mat_pl: 1,
            tai_trai_noi_thuong: "5".into(),
            tai_trai_noi_tham: "5".into(),
            tai_phai_noi_thuong: "5".into(),
            tai_phai_noi_tham: "5".into(),
            benh_khac_tai_mui_hong: "Bình thường".into(),
            kham_tai_mui_hong_pl: 1,
            ham_tren: "Bình thường".into(),
            ham_duoi: "Bình thường".into(),
            benh_khac_rang_ham_mat: "Bình thường".into(),
            kham_rang_ham_mat_pl: 1,
            cls_list: vec![],
            phan_loai_sk: 1,
            ket_luan_benh: "Đủ sức khỏe làm việc".into(),
            cac_benh_tat_neu_co: "".into(),
            selected_items: HashSet::new(),
            selected_classifications: HashMap::new(),
            specialty_extras: HashMap::new(),
            dominant_hand: "right".into(),
            voice_hearing: 1,
            audiometer_hearing: 1,
        }
    }
}

#[derive(Clone)]
struct RecordItem {
    id: i64,
    ho_ten: String,
    so_cccd: String,
    ngay_vao: String,
    status: i32, // 0: Đang soạn, 1: Đã xác nhận / Khóa
    data: KskRecordPayload,
}

fn score_color_style(score: i32) -> (egui::Color32, egui::Color32) {
    match score {
        1 => (egui::Color32::from_rgb(22, 101, 52), egui::Color32::from_rgb(220, 252, 231)),   // Green
        2 => (egui::Color32::from_rgb(30, 64, 175), egui::Color32::from_rgb(219, 234, 254)),  // Blue
        3 => (egui::Color32::from_rgb(146, 64, 14), egui::Color32::from_rgb(254, 243, 199)),  // Yellow/Amber
        4 => (egui::Color32::from_rgb(154, 52, 18), egui::Color32::from_rgb(255, 237, 213)),  // Orange
        _ => (egui::Color32::from_rgb(153, 27, 27), egui::Color32::from_rgb(254, 226, 226)),  // Red
    }
}

fn ui_score_badge(ui: &mut egui::Ui, score: i32, is_dyn: bool) {
    let (fg, bg) = score_color_style(score);
    let txt = if is_dyn {
        format!("Loại {} *", score)
    } else {
        format!("Loại {}", score)
    };
    egui::Frame::none()
        .fill(bg)
        .rounding(4.0)
        .inner_margin(egui::Margin::symmetric(6.0, 2.0))
        .show(ui, |ui| {
            ui.colored_label(fg, egui::RichText::new(txt).strong().size(11.5));
        });
}

// TÍNH TUỔI TỪ NGÀY SINH (DD/MM/YYYY)
fn calculate_age(ngay_sinh: &str) -> u32 {
    if let Some((_, _, y)) = parse_dmy(ngay_sinh) {
        let now = chrono::Local::now();
        let cur_year = now.year() as u32;
        if cur_year >= y {
            return cur_year - y;
        }
    }
    30 // Mặc định 30 nếu chưa nhập ngày sinh
}

// TÁCH VÀ PHÂN TÍCH HUYẾT ÁP (SYSTOLIC / DIASTOLIC)
fn parse_blood_pressure(s: &str) -> (f64, f64) {
    let cleaned = s.replace(' ', "");
    let parts: Vec<&str> = cleaned.split('/').collect();
    if parts.len() == 2 {
        let sys = parts[0].parse::<f64>().unwrap_or(120.0);
        let dia = parts[1].parse::<f64>().unwrap_or(80.0);
        return (sys, dia);
    }
    (120.0, 80.0)
}

// TỰ ĐỘNG PHÂN LOẠI HUYẾT ÁP THEO ĐỘ TUỔI (QĐ 1613/BYT)
fn calc_blood_pressure_score(bp_str: &str, age: u32) -> (i32, String) {
    let (sys, dia) = parse_blood_pressure(bp_str);
    if age < 30 {
        if sys >= 110.0 && sys <= 125.0 && dia >= 65.0 && dia <= 85.0 {
            (1, format!("< 30 tuổi: {}/{} mmHg (Bình thường)", sys as i32, dia as i32))
        } else if sys <= 135.0 && dia < 90.0 && dia >= 60.0 {
            (2, format!("< 30 tuổi: {}/{} mmHg (Hơi cao/thấp)", sys as i32, dia as i32))
        } else if sys <= 140.0 && dia <= 95.0 {
            (3, format!("< 30 tuổi: {}/{} mmHg (Tăng HA giới hạn)", sys as i32, dia as i32))
        } else if sys < 100.0 || dia < 60.0 {
            (4, format!("< 30 tuổi: {}/{} mmHg (Hạ huyết áp)", sys as i32, dia as i32))
        } else {
            (4, format!("< 30 tuổi: {}/{} mmHg (Tăng huyết áp)", sys as i32, dia as i32))
        }
    } else {
        if sys < 140.0 && dia < 90.0 {
            (1, format!(">= 30 tuổi: {}/{} mmHg (Bình thường)", sys as i32, dia as i32))
        } else if sys <= 145.0 && dia <= 95.0 {
            (3, format!(">= 30 tuổi: {}/{} mmHg (Tăng HA giới hạn)", sys as i32, dia as i32))
        } else {
            (4, format!(">= 30 tuổi: {}/{} mmHg (Tăng huyết áp)", sys as i32, dia as i32))
        }
    }
}

// TỰ ĐỘNG PHÂN LOẠI TẦN SỐ TIM / MẠCH (QĐ 1613/BYT)
fn calc_pulse_score(pulse_str: &str) -> (i32, String) {
    let p: f64 = pulse_str.trim().parse().unwrap_or(75.0);
    if p >= 60.0 && p <= 75.0 {
        (1, format!("Tần số {} lần/phút (Bình thường)", p as i32))
    } else if p > 75.0 && p <= 85.0 {
        (2, format!("Tần số {} lần/phút (Hơi nhanh)", p as i32))
    } else if (p > 85.0 && p <= 95.0) || (p >= 55.0 && p < 60.0) {
        (3, format!("Tần số {} lần/phút (Nhanh/Chậm nhẹ)", p as i32))
    } else {
        (4, format!("Tần số {} lần/phút (Rất nhanh/Rất chậm)", p as i32))
    }
}

// TỰ ĐỘNG PHÂN LOẠI THỊ LỰC (QĐ 1613/BYT)
fn calc_vision_score(right_s: &str, left_s: &str) -> i32 {
    let r: f64 = right_s.trim().parse::<f64>().unwrap_or(10.0).min(10.0_f64).max(0.0_f64);
    let l: f64 = left_s.trim().parse::<f64>().unwrap_or(10.0).min(10.0_f64).max(0.0_f64);
    let max_v = r.max(l);
    let sum_v = r + l;

    if max_v >= 9.99 && sum_v >= 18.99 {
        1
    } else if max_v >= 8.99 && sum_v >= 15.99 {
        2
    } else if max_v >= 6.99 && sum_v >= 13.99 {
        3
    } else if max_v >= 5.99 && sum_v >= 10.99 {
        4
    } else {
        5
    }
}

// TỰ ĐỘNG PHÂN LOẠI THÍNH LỰC TỪ KHOẢNG CÁCH NÓI THƯỜNG (5M) VÀ NÓI THẦM (QĐ 1613/BYT)
fn calc_hearing_from_distances(
    r_thuong: &str, l_thuong: &str,
    r_tham: &str, l_tham: &str,
) -> (i32, String) {
    let rt = r_tham.trim().parse::<f64>().unwrap_or(5.0);
    let lt = l_tham.trim().parse::<f64>().unwrap_or(5.0);
    let rth = r_thuong.trim().parse::<f64>().unwrap_or(5.0);
    let lth = l_thuong.trim().parse::<f64>().unwrap_or(5.0);

    let min_tham = rt.min(lt);
    let max_tham = rt.max(lt);
    let min_thuong = rth.min(lth);

    if min_tham >= 4.9 && min_thuong >= 4.9 {
        (1, "Nói thầm 5m cả hai tai (Bình thường)".into())
    } else if (max_tham >= 3.9 && min_tham >= 1.9) || min_thuong >= 4.0 {
        (2, "Nói thầm 1 bên 4m, bên kia 2m".into())
    } else if (max_tham >= 2.9 && min_tham >= 0.9) || min_thuong >= 3.0 {
        (3, "Nói thầm 1 bên 3m, bên kia 1m".into())
    } else if (max_tham >= 1.9 && min_tham >= 0.4) || min_thuong >= 2.0 {
        (4, "Nói thầm 1 bên 2m, bên kia 0.5m".into())
    } else {
        (5, "Nói thầm 1 bên 1m, bên kia < 0.1m".into())
    }
}

fn calc_classification_score(
    c: &QdClassification,
    base_vision: i32,
    base_hearing: i32,
) -> i32 {
    if let Some(s) = c.score {
        return s;
    }
    if let Some(ref ref_did) = c.ref_disease_id {
        if ref_did == "BT_MAT_THI_LUC" {
            let mut sc = base_vision;
            if let Some(ref m) = c.modifier {
                if m.contains("+1") {
                    sc = (sc + 1).min(5);
                } else if m.contains("+2") {
                    sc = (sc + 2).min(5);
                }
            }
            return sc;
        } else if ref_did == "BT_TMH_TAI_TRONG" {
            let mut sc = base_hearing;
            if let Some(ref m) = c.modifier {
                if m.contains("+1") {
                    sc = (sc + 1).min(5);
                }
            }
            return sc;
        }
    }
    1
}

// BỘ DATEPICKER & VALIDATION CHO NGÀY THÁNG (DD/MM/YYYY)
fn is_valid_date(d: u32, m: u32, y: u32) -> bool {
    if !(1..=12).contains(&m) || !(1900..=2100).contains(&y) || d == 0 {
        return false;
    }
    let max_days = match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0) { 29 } else { 28 }
        }
        _ => return false,
    };
    d <= max_days
}

fn parse_dmy(s: &str) -> Option<(u32, u32, u32)> {
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() == 3 {
        if let (Ok(d), Ok(m), Ok(y)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>(), parts[2].parse::<u32>()) {
            if is_valid_date(d, m, y) {
                return Some((d, m, y));
            }
        }
    }
    None
}

fn date_input_picker(ui: &mut egui::Ui, id_source: &str, date_str: &mut String) {
    let mut digits_only = String::new();
    for ch in date_str.chars() {
        if ch.is_ascii_digit() {
            digits_only.push(ch);
            if digits_only.len() == 8 { break; }
        }
    }

    let mut is_error = false;
    if !date_str.is_empty() {
        if let Some((d, m, y)) = parse_dmy(date_str) {
            if !is_valid_date(d, m, y) { is_error = true; }
        } else if date_str.len() >= 10 {
            is_error = true;
        }
    }

    let cal_id = ui.make_persistent_id(format!("{}_cal_state", id_source));
    let mut view_date: (u32, u32) = ui.data_mut(|d| {
        *d.get_temp_mut_or_insert_with(cal_id, || {
            let now = chrono::Local::now();
            (now.month(), now.year() as u32)
        })
    });

    ui.horizontal(|ui| {
        let mut text_edit = egui::TextEdit::singleline(date_str).desired_width(115.0);
        if is_error {
            text_edit = text_edit.text_color(egui::Color32::from_rgb(220, 38, 38));
        }

        let resp = ui.add(text_edit);
        if resp.changed() {
            let mut formatted = String::new();
            let mut d_count = 0;
            for ch in date_str.chars() {
                if ch.is_ascii_digit() && d_count < 8 {
                    formatted.push(ch);
                    d_count += 1;
                    if (d_count == 2 || d_count == 4) && d_count < digits_only.len() {
                        formatted.push('/');
                    }
                }
            }
            if !formatted.is_empty() {
                *date_str = formatted;
            }
        }

        let popup_id = ui.make_persistent_id(format!("{}_popup", id_source));
        let btn_resp = ui.small_button("📅");
        if btn_resp.clicked() {
            ui.memory_mut(|m| m.toggle_popup(popup_id));
            if let Some((_, m, y)) = parse_dmy(date_str) {
                view_date = (m, y);
                ui.data_mut(|d| d.insert_temp(cal_id, view_date));
            }
        }

        egui::popup_below_widget(ui, popup_id, &btn_resp, |ui| {
            ui.set_min_width(240.0);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    if ui.button("◀").clicked() {
                        if view_date.0 == 1 { view_date.0 = 12; view_date.1 -= 1; } else { view_date.0 -= 1; }
                        ui.data_mut(|d| d.insert_temp(cal_id, view_date));
                    }
                    ui.label(egui::RichText::new(format!("Tháng {:02} / {}", view_date.0, view_date.1)).strong());
                    if ui.button("▶").clicked() {
                        if view_date.0 == 12 { view_date.0 = 1; view_date.1 += 1; } else { view_date.0 += 1; }
                        ui.data_mut(|d| d.insert_temp(cal_id, view_date));
                    }
                    if ui.small_button("Hôm nay").clicked() {
                        let now = chrono::Local::now();
                        *date_str = now.format("%d/%m/%Y").to_string();
                        ui.memory_mut(|m| m.close_popup());
                    }
                });

                ui.separator();

                egui::Grid::new(format!("{}_cal_grid", id_source))
                    .num_columns(7)
                    .spacing([4.0, 4.0])
                    .show(ui, |ui| {
                        let days_header = ["T2", "T3", "T4", "T5", "T6", "T7", "CN"];
                        for h in days_header {
                            ui.label(egui::RichText::new(h).weak().size(11.0));
                        }
                        ui.end_row();

                        let first_of_month = chrono::NaiveDate::from_ymd_opt(view_date.1 as i32, view_date.0, 1)
                            .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
                        let weekday_idx = first_of_month.weekday().num_days_from_monday();

                        for _ in 0..weekday_idx {
                            ui.label("");
                        }

                        let days_in_month = match view_date.0 {
                            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
                            4 | 6 | 9 | 11 => 30,
                            2 => {
                                let y = view_date.1;
                                if (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0) { 29 } else { 28 }
                            }
                            _ => 30,
                        };

                        let parsed = parse_dmy(date_str);

                        for d in 1..=days_in_month {
                            let is_selected = parsed.map_or(false, |(sd, sm, sy)| sd == d && sm == view_date.0 && sy == view_date.1);
                            let mut btn = egui::Button::new(d.to_string());
                            if is_selected {
                                btn = btn.fill(egui::Color32::from_rgb(14, 165, 233));
                            }
                            if ui.add(btn).clicked() {
                                *date_str = format!("{:02}/{:02}/{}", d, view_date.0, view_date.1);
                                ui.memory_mut(|m| m.close_popup());
                            }

                            if (weekday_idx + d) % 7 == 0 {
                                ui.end_row();
                            }
                        }
                    });
            });
        });

        if is_error {
            ui.colored_label(egui::Color32::from_rgb(220, 38, 38), "⚠ Sai định dạng dd/mm/yyyy");
        }
    });
}

// COMBOBOX TÌM KIẾM THEO TÊN HOẶC MÃ
fn search_combo(ui: &mut egui::Ui, id_source: &str, selected_code: &mut String, items: &[(String, String)]) -> bool {
    let mut changed = false;
    let selected_label = items.iter().find(|(c, _)| c == selected_code)
        .map(|(_, n)| n.clone())
        .unwrap_or_else(|| {
            if selected_code.is_empty() { "— Chọn —".to_string() } else { selected_code.clone() }
        });

    let search_id = ui.make_persistent_id(format!("{}_search", id_source));
    let mut search_kw = ui.data_mut(|d| d.get_temp::<String>(search_id).unwrap_or_default());

    egui::ComboBox::from_id_source(id_source)
        .width(260.0)
        .selected_text(selected_label)
        .show_ui(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("🔍");
                if ui.add(egui::TextEdit::singleline(&mut search_kw).hint_text("Gõ mã hoặc tên...")).changed() {
                    ui.data_mut(|d| d.insert_temp(search_id, search_kw.clone()));
                }
            });
            ui.separator();

            let kw_lower = search_kw.to_lowercase();
            egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                for (code, name) in items {
                    if kw_lower.is_empty() || code.to_lowercase().contains(&kw_lower) || name.to_lowercase().contains(&kw_lower) {
                        let is_sel = selected_code == code;
                        if ui.selectable_label(is_sel, format!("{}: {}", code, name)).clicked() {
                            *selected_code = code.clone();
                            changed = true;
                        }
                    }
                }
            });
        });
    changed
}

// BỘ CHỌN NHIỀU MÃ ICD-10 CÁCH NHAU BẰNG DẤU CHẤM PHẨY
fn multi_icd(ui: &mut egui::Ui, id_source: &str, target_val: &mut String, items: &[(String, String)]) {
    ui.horizontal(|ui| {
        ui.add(egui::TextEdit::singleline(target_val).desired_width(220.0).hint_text("Mã ICD-10 (nối bằng ;)"));
        let mut temp_code = String::new();
        if search_combo(ui, id_source, &mut temp_code, items) && !temp_code.is_empty() {
            let mut list: Vec<String> = target_val.split(';')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if !list.contains(&temp_code) {
                list.push(temp_code);
                *target_val = list.join("; ");
            }
        }
        if !target_val.is_empty() && ui.small_button("Xóa").clicked() {
            target_val.clear();
        }
    });
}

#[allow(dead_code)]
struct KskApp {
    conn: Connection,
    config: AppConfig,
    dark_mode: bool,
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
    active_tab: usize, // 0: Form, 1: Danh sách, 2: Quản lý Danh mục, 3: Quản lý Users, 4: Cài đặt hệ thống
    dept_tab: usize,   // 0: Hành chính, 1: Tiền sử, 2: Khám bệnh, 3: Kết luận
    kham_sub_tab: usize, // 0: Thể lực, 1: Nội, 2: Ngoại, 3: Da liễu, 4: Sản, 5: Mắt, 6: TMH, 7: RHM, 8: Cận lâm sàng

    // Quản lý Users
    user_list: Vec<UserRecord>,
    new_username: String,
    new_password: String,
    new_user_role: String,
    change_old_pass: String,
    change_new_pass: String,

    // Cài đặt Settings
    setting_cskcb: String,
    setting_gtin: String,
    setting_prov: String,
    setting_ward: String,

    // Danh mục
    catalogs: Vec<CatalogRecord>,
    cur_cat_tab: String,
    import_tsv_text: String,
    cat_code_input: String,
    cat_name_input: String,
    cat_parent_input: String,

    // Bộ quy tắc QĐ 1613/BYT
    qd_specialties: Vec<QdSpecialty>,
    qd_diseases: Vec<QdDisease>,
    qd_classifications: Vec<QdClassification>,
    expanded_diseases: HashSet<String>,
    noi_khoa_sub_tab: usize, // 0..7
    qd_rule_filter_spec: String,
    qd_rule_search: String,
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
            CREATE TABLE IF NOT EXISTS qd1613_specialties (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                parent_id TEXT
            );
            CREATE TABLE IF NOT EXISTS qd1613_diseases (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                spec_id TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS qd1613_classifications (
                id TEXT PRIMARY KEY,
                disease_id TEXT NOT NULL,
                name TEXT NOT NULL,
                score INTEGER,
                ref_rule TEXT,
                ref_disease_id TEXT,
                modifier TEXT
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
            setting_cskcb: cfg.default_cskcb.clone(),
            setting_gtin: cfg.default_gtin_cskcb.clone(),
            setting_prov: cfg.default_province_code.clone(),
            setting_ward: cfg.default_ward_code.clone(),
            config: cfg,
            dark_mode: false,
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
            kham_sub_tab: 0,
            user_list: vec![],
            new_username: "".into(),
            new_password: "".into(),
            new_user_role: "reception".into(),
            change_old_pass: "".into(),
            change_new_pass: "".into(),
            catalogs: vec![],
            cur_cat_tab: "priority".into(),
            import_tsv_text: "".into(),
            cat_code_input: "".into(),
            cat_name_input: "".into(),
            cat_parent_input: "".into(),
            qd_specialties: vec![],
            qd_diseases: vec![],
            qd_classifications: vec![],
            expanded_diseases: HashSet::new(),
            noi_khoa_sub_tab: 0,
            qd_rule_filter_spec: "all".into(),
            qd_rule_search: "".into(),
        };

        app.seed_default_catalogs();
        app.seed_qd1613();
        app.reload_qd1613();
        app.reload_catalogs();
        app.reload_records();
        app.reload_users();
        app
    }

    fn seed_qd1613(&self) {
        let count: i64 = self.conn.query_row("SELECT COUNT(*) FROM qd1613_specialties", [], |r| r.get(0)).unwrap_or(0);
        if count == 0 {
            self.reset_qd1613_to_default();
        }
    }

    fn reset_qd1613_to_default(&self) {
        if let Ok(data) = serde_json::from_str::<Qd1613CatalogData>(QD1613_JSON) {
            let _ = self.conn.execute("DELETE FROM qd1613_classifications", []);
            let _ = self.conn.execute("DELETE FROM qd1613_diseases", []);
            let _ = self.conn.execute("DELETE FROM qd1613_specialties", []);

            let mut stmt_s = self.conn.prepare("INSERT INTO qd1613_specialties (id, name, parent_id) VALUES (?, ?, ?)").unwrap();
            for s in &data.specialties {
                let _ = stmt_s.execute(params![s.id, s.name, s.parent_id]);
            }
            let mut stmt_d = self.conn.prepare("INSERT INTO qd1613_diseases (id, name, spec_id) VALUES (?, ?, ?)").unwrap();
            for d in &data.diseases {
                let _ = stmt_d.execute(params![d.id, d.name, d.spec_id]);
            }
            let mut stmt_c = self.conn.prepare("INSERT INTO qd1613_classifications (id, disease_id, name, score, ref_rule, ref_disease_id, modifier) VALUES (?, ?, ?, ?, ?, ?, ?)").unwrap();
            for c in &data.classifications {
                let _ = stmt_c.execute(params![c.id, c.disease_id, c.name, c.score, c.ref_rule, c.ref_disease_id, c.modifier]);
            }
        }
    }

    fn reload_qd1613(&mut self) {
        if let Ok(mut stmt) = self.conn.prepare("SELECT id, name, parent_id FROM qd1613_specialties") {
            self.qd_specialties = stmt.query_map([], |r| {
                Ok(QdSpecialty {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    parent_id: r.get(2)?,
                })
            }).unwrap().filter_map(Result::ok).collect();
        }
        if let Ok(mut stmt) = self.conn.prepare("SELECT id, name, spec_id FROM qd1613_diseases") {
            self.qd_diseases = stmt.query_map([], |r| {
                Ok(QdDisease {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    spec_id: r.get(2)?,
                })
            }).unwrap().filter_map(Result::ok).collect();
        }
        if let Ok(mut stmt) = self.conn.prepare("SELECT id, disease_id, name, score, ref_rule, ref_disease_id, modifier FROM qd1613_classifications") {
            self.qd_classifications = stmt.query_map([], |r| {
                Ok(QdClassification {
                    id: r.get(0)?,
                    disease_id: r.get(1)?,
                    name: r.get(2)?,
                    score: r.get(3)?,
                    ref_rule: r.get(4)?,
                    ref_disease_id: r.get(5)?,
                    modifier: r.get(6)?,
                })
            }).unwrap().filter_map(Result::ok).collect();
        }
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

                // 3. Tỉnh mẫu
                ("province", "64", "64 - Tỉnh Gia Lai", ""),
                ("province", "79", "79 - TP. Hồ Chí Minh", ""),
                ("province", "01", "01 - TP. Hà Nội", ""),

                // 4. Xã mẫu
                ("ward", "23992", "23992 - Phường Hoa Lư", "64"),
                ("ward", "23995", "23995 - Phường Tây Sơn", "64"),

                // 5. Dân tộc mẫu
                ("ethnicity", "01", "01 - Kinh", ""),
                ("ethnicity", "02", "02 - Tày", ""),
                ("ethnicity", "03", "03 - Thái", ""),
                ("ethnicity", "10", "10 - Gia Rai", ""),
                ("ethnicity", "11", "11 - Ba Na", ""),

                // 6. Nghề nghiệp mẫu
                ("occupation", "22", "22 - Kỹ sư, Chuyên viên", ""),
                ("occupation", "52", "52 - Nhân viên bán hàng", ""),
                ("occupation", "71", "71 - Công nhân xây dựng", ""),
                ("occupation", "99", "99 - Khác", ""),

                // 7. ICD-10 cơ bản cho tiền sử
                ("icd10", "I10", "I10 - Bệnh tăng huyết áp vô căn", ""),
                ("icd10", "E11", "E11 - Đái tháo đường không phụ thuộc insulin", ""),
                ("icd10", "J45", "J45 - Hen phế quản", ""),
                ("icd10", "A15", "A15 - Lao đường hô hấp", ""),
                ("icd10", "G40", "G40 - Động kinh", ""),
            ];

            let mut stmt = self.conn.prepare(
                "INSERT OR IGNORE INTO catalogs (cat_type, code, name, parent_code) VALUES (?1, ?2, ?3, ?4)"
            ).unwrap();
            for s in seeds {
                stmt.execute(params![s.0, s.1, s.2, s.3]).unwrap();
            }
        }
    }

    fn reload_catalogs(&mut self) {
        let mut stmt = self.conn.prepare("SELECT id, cat_type, code, name, parent_code FROM catalogs ORDER BY code ASC").unwrap();
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

    fn reload_users(&mut self) {
        let mut stmt = self.conn.prepare("SELECT id, username, role FROM users ORDER BY id ASC").unwrap();
        let rows = stmt.query_map([], |r| {
            Ok(UserRecord {
                id: r.get(0)?,
                username: r.get(1)?,
                role: r.get(2)?,
            })
        }).unwrap();
        self.user_list = rows.flatten().collect();
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

        self.update_global_health_score();

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
                &d.noi_khoa_noi_tiet_pl.to_string(), d.noi_khoa_co_xuong_khop.as_str(),
                &d.noi_khoa_co_xuong_khop_pl.to_string(), d.noi_khoa_than_kinh.as_str(),
                &d.noi_khoa_than_kinh_pl.to_string(), d.noi_khoa_tam_than.as_str(),
                &d.noi_khoa_tam_than_pl.to_string(), d.ket_qua_kham_ngoai_khoa.as_str(),
                &d.kham_ngoai_khoa_pl.to_string(), d.ket_qua_kham_da_lieu.as_str(), &d.kham_da_lieu_pl.to_string(),
                d.ket_qua_kham_san_phu_khoa.as_str(), &d.kham_san_phu_khoa_pl.to_string(),
                d.khong_kinh_mat_phai.as_str(), d.khong_kinh_mat_trai.as_str(), d.co_kinh_mat_phai.as_str(),
                d.co_kinh_mat_trai.as_str(), d.benh_khac_mat.as_str(), &d.kham_mat_pl.to_string(),
                d.tai_trai_noi_thuong.as_str(), d.tai_trai_noi_tham.as_str(), d.tai_phai_noi_thuong.as_str(),
                d.tai_phai_noi_tham.as_str(), d.benh_khac_tai_mui_hong.as_str(),
                &d.kham_tai_mui_hong_pl.to_string(), d.ham_tren.as_str(), d.ham_duoi.as_str(),
                d.benh_khac_rang_ham_mat.as_str(), &d.kham_rang_ham_mat_pl.to_string(),
                &d.phan_loai_sk.to_string(), d.ket_luan_benh.as_str(), d.cac_benh_tat_neu_co.as_str(),
            ];
            wtr.write_record(&row).unwrap();
        }

        let csv_data = wtr.into_inner().unwrap();
        let fname = format!("xuat_ksk_qd3176_{}.csv", chrono::Local::now().format("%Y%m%d_%H%M%S"));
        if let Ok(_) = fs::write(&fname, &csv_data) {
            self.status_msg = format!("Đã xuất {} hồ sơ chuẩn QĐ 3176 sang file: {}!", self.records.len(), fname);
        } else {
            self.status_msg = "LỖI: Không ghi được file CSV!".into();
        }
    }

    fn new_record_with_defaults(&mut self) {
        let mut p = KskRecordPayload::default();
        p.ma_cskcb = self.config.default_cskcb.clone();
        p.ma_gtin_cskcb = self.config.default_gtin_cskcb.clone();
        p.matinh_cu_tru = self.config.default_province_code.clone();
        p.maxa_cu_tru = self.config.default_ward_code.clone();
        self.form = p;
        self.current_id = None;
        self.current_status = 0;
        self.status_msg = "Tạo mới hồ sơ thành công!".into();
    }

    fn cat_items<'a>(catalogs: &'a [CatalogRecord], cat_type: &str, parent: Option<&str>) -> Vec<(String, String)> {
        catalogs.iter()
            .filter(|c| c.cat_type == cat_type && parent.map_or(true, |p| c.parent_code == p || c.parent_code.is_empty()))
            .map(|c| (c.code.clone(), c.name.clone()))
            .collect()
    }

    fn ui_radio_yn(ui: &mut egui::Ui, val: &mut i32) {
        ui.horizontal(|ui| {
            ui.radio_value(val, 0, "Không");
            ui.radio_value(val, 1, "Có");
        });
    }

    #[allow(dead_code)]
    fn ui_radio_pl_num(ui: &mut egui::Ui, val: &mut i32) {
        ui.horizontal(|ui| {
            for i in 1..=5 {
                ui.radio_value(val, i, format!("Loại {}", i));
            }
        });
    }

    // ===================================================================
    // BỘ ĐỒNG BỘ KẾT QUẢ VÀ TỰ TÍNH PHÂN LOẠI THEO CHUẨN QĐ 1613/BYT
    // ===================================================================
    fn sync_specialty_result(&mut self, spec_id: &str) {
        let base_vision = calc_vision_score(&self.form.khong_kinh_mat_phai, &self.form.khong_kinh_mat_trai);
        let has_glasses = !self.form.co_kinh_mat_phai.trim().is_empty() || !self.form.co_kinh_mat_trai.trim().is_empty();
        let eff_vision = if has_glasses {
            calc_vision_score(&self.form.co_kinh_mat_phai, &self.form.co_kinh_mat_trai)
        } else {
            base_vision
        };
        let (base_hearing, hearing_desc) = calc_hearing_from_distances(
            &self.form.tai_phai_noi_thuong, &self.form.tai_trai_noi_thuong,
            &self.form.tai_phai_noi_tham, &self.form.tai_trai_noi_tham,
        );
        let age = calculate_age(&self.form.ngay_sinh);
        let (bp_score, bp_desc) = calc_blood_pressure_score(&self.form.huyet_ap, age);
        let (pulse_score, pulse_desc) = calc_pulse_score(&self.form.mach);

        let diseases: Vec<QdDisease> = self.qd_diseases.iter().filter(|d| d.spec_id == spec_id).cloned().collect();
        let mut abnormal_items: Vec<(String, i32)> = Vec::new();
        let mut max_sc = 1;

        let extra = self.form.specialty_extras.entry(spec_id.to_string()).or_default();
        let auto_overrides = extra.auto_overrides.clone();
        let manual_rows = extra.manual_rows.clone();
        let final_note = extra.final_note.clone();

        // 1. Kiểm tra các bệnh tự động riêng cho từng chuyên khoa
        if spec_id == "CK_MAT" {
            if eff_vision > max_sc { max_sc = eff_vision; }
            if eff_vision > 1 || has_glasses {
                let override_text = auto_overrides.get("BT_MAT_THI_LUC").cloned().unwrap_or_else(|| {
                    if has_glasses {
                        format!("Không kính: MP {}/10, MT {}/10 | Có kính: MP {}/10, MT {}/10", self.form.khong_kinh_mat_phai, self.form.khong_kinh_mat_trai, self.form.co_kinh_mat_phai, self.form.co_kinh_mat_trai)
                    } else {
                        format!("Thị lực không kính MP {}/10, MT {}/10", self.form.khong_kinh_mat_phai, self.form.khong_kinh_mat_trai)
                    }
                });
                abnormal_items.push((format!("Thị lực: {}", override_text), eff_vision));
            }
        } else if spec_id == "CK_TMH" {
            if base_hearing > max_sc { max_sc = base_hearing; }
            if base_hearing > 1 {
                let override_text = auto_overrides.get("BT_TMH_TAI_TRONG").cloned().unwrap_or(hearing_desc);
                abnormal_items.push((format!("Sức nghe: {}", override_text), base_hearing));
            }
        } else if spec_id == "CK_NOI_TIM_MACH" {
            if bp_score > max_sc { max_sc = bp_score; }
            if bp_score > 1 {
                let override_text = auto_overrides.get("BT_TM_HUYET_AP").cloned().unwrap_or(bp_desc);
                abnormal_items.push((format!("Huyết áp: {}", override_text), bp_score));
            }
            if pulse_score > max_sc { max_sc = pulse_score; }
            if pulse_score > 1 {
                let override_text = auto_overrides.get("BT_TM_MACH").cloned().unwrap_or(pulse_desc);
                abnormal_items.push((format!("Tần số tim: {}", override_text), pulse_score));
            }
        }

        // 2. Kiểm tra các bệnh được chọn bằng TICKBOX (Checkbox)
        for d in &diseases {
            // Bỏ qua các mã bệnh đã tính tự động ở trên
            if d.id == "BT_MAT_THI_LUC" || d.id == "BT_TMH_TAI_TRONG" || d.id == "BT_TM_HUYET_AP" || d.id == "BT_TM_MACH" {
                continue;
            }

            let clss: Vec<QdClassification> = self.qd_classifications.iter().filter(|c| c.disease_id == d.id).cloned().collect();
            if clss.is_empty() { continue; }

            for c in &clss {
                if self.form.selected_items.contains(&c.id) {
                    let sc = calc_classification_score(c, base_vision, base_hearing);
                    if sc > max_sc { max_sc = sc; }
                    if sc > 1 {
                        let text = if let Some(custom) = auto_overrides.get(&c.id) {
                            if !custom.trim().is_empty() { custom.clone() } else { c.name.clone() }
                        } else {
                            c.name.clone()
                        };
                        abnormal_items.push((format!("{}: {}", d.name, text), sc));
                    }
                }
            }
        }

        // 3. Dòng nhập tay
        for mr in &manual_rows {
            if mr.classification > max_sc {
                max_sc = mr.classification;
            }
            if !mr.manifestation.trim().is_empty() {
                abnormal_items.push((mr.manifestation.clone(), mr.classification));
            }
        }

        let summary = if max_sc == 1 && abnormal_items.is_empty() {
            "Bình thường".to_string()
        } else {
            let mut parts: Vec<String> = abnormal_items.into_iter().map(|(txt, _)| txt).collect();
            if !final_note.trim().is_empty() {
                parts.push(format!("Ghi chú: {}", final_note.trim()));
            }
            parts.join("; ")
        };

        match spec_id {
            "CK_NOI_TIM_MACH" => {
                self.form.noi_khoa_tuan_hoan = summary;
                self.form.noi_khoa_tuan_hoan_pl = max_sc;
            }
            "CK_NOI_HO_HAP" => {
                self.form.noi_khoa_ho_hap = summary;
                self.form.noi_khoa_ho_hap_pl = max_sc;
            }
            "CK_NOI_TIEU_HOA" => {
                self.form.noi_khoa_tieu_hoa = summary;
                self.form.noi_khoa_tieu_hoa_pl = max_sc;
            }
            "CK_NOI_TIET_NIEU_SD" => {
                self.form.noi_khoa_than_tn_sd = summary;
                self.form.noi_khoa_than_tn_sd_pl = max_sc;
            }
            "CK_NOI_THAN_KINH" => {
                self.form.noi_khoa_than_kinh = summary;
                self.form.noi_khoa_than_kinh_pl = max_sc;
            }
            "CK_NOI_TAM_THAN" => {
                self.form.noi_khoa_tam_than = summary;
                self.form.noi_khoa_tam_than_pl = max_sc;
            }
            "CK_NOI_NOITIET" => {
                self.form.noi_khoa_noi_tiet = summary;
                self.form.noi_khoa_noi_tiet_pl = max_sc;
            }
            "CK_NOI_CO_XUONG_KHOP" => {
                self.form.noi_khoa_co_xuong_khop = summary;
                self.form.noi_khoa_co_xuong_khop_pl = max_sc;
            }
            "CK_NGOAI" => {
                self.form.ket_qua_kham_ngoai_khoa = summary;
                self.form.kham_ngoai_khoa_pl = max_sc;
            }
            "CK_DA_LIEU" => {
                self.form.ket_qua_kham_da_lieu = summary;
                self.form.kham_da_lieu_pl = max_sc;
            }
            "CK_PHU_SAN" => {
                self.form.ket_qua_kham_san_phu_khoa = summary;
                self.form.kham_san_phu_khoa_pl = max_sc;
            }
            "CK_MAT" => {
                self.form.benh_khac_mat = summary;
                self.form.kham_mat_pl = max_sc;
            }
            "CK_TMH" => {
                self.form.benh_khac_tai_mui_hong = summary;
                self.form.kham_tai_mui_hong_pl = max_sc;
            }
            "CK_RHM" => {
                self.form.benh_khac_rang_ham_mat = summary;
                self.form.kham_rang_ham_mat_pl = max_sc;
            }
            _ => {}
        }

        self.update_global_health_score();
    }

    fn update_global_health_score(&mut self) {
        let scores = [
            self.form.kham_the_luc_pl,
            self.form.noi_khoa_tuan_hoan_pl,
            self.form.noi_khoa_ho_hap_pl,
            self.form.noi_khoa_tieu_hoa_pl,
            self.form.noi_khoa_than_tn_sd_pl,
            self.form.noi_khoa_noi_tiet_pl,
            self.form.noi_khoa_co_xuong_khop_pl,
            self.form.noi_khoa_than_kinh_pl,
            self.form.noi_khoa_tam_than_pl,
            self.form.kham_ngoai_khoa_pl,
            self.form.kham_da_lieu_pl,
            self.form.kham_san_phu_khoa_pl,
            self.form.kham_mat_pl,
            self.form.kham_tai_mui_hong_pl,
            self.form.kham_rang_ham_mat_pl,
        ];
        let max_val = scores.into_iter().max().unwrap_or(1).clamp(1, 5);
        self.form.phan_loai_sk = max_val;
    }

    // ===================================================================
    // UI: RENDER FULL CONTENT CHO MỖI CHUYÊN KHOA THEO QĐ 1613/BYT
    // ===================================================================
    fn render_qd1613_specialty(&mut self, ui: &mut egui::Ui, spec_id: &str, parent_title: Option<&str>) {
        let spec_name = self.qd_specialties.iter().find(|s| s.id == spec_id).map(|s| s.name.clone()).unwrap_or_else(|| spec_id.to_string());
        let diseases: Vec<QdDisease> = self.qd_diseases.iter().filter(|d| d.spec_id == spec_id).cloned().collect();

        // 1. TIÊU ĐỀ CHUYÊN KHOA & THAO TÁC MỞ/THU GỌN
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                let title = if let Some(p) = parent_title {
                    format!("Chuyên khoa: {} › {}", p, spec_name)
                } else {
                    format!("Chuyên khoa: {}", spec_name)
                };
                ui.heading(egui::RichText::new(title).strong().size(17.0));
                ui.colored_label(egui::Color32::from_rgb(100, 116, 139), "Tick chọn bệnh lý (checkbox) để tự động phân loại. Khi không chọn bệnh nào → Mặc định Loại 1 (Bình thường).");
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Thu gọn tất cả").clicked() {
                    for d in &diseases {
                        self.expanded_diseases.remove(&d.id);
                    }
                }
                if ui.button("Mở tất cả").clicked() {
                    for d in &diseases {
                        self.expanded_diseases.insert(d.id.clone());
                    }
                }
            });
        });

        ui.add_space(8.0);

        // 2. BẢNG ĐO THỊ LỰC NẾU LÀ MẮT (CK_MAT): CẢ KHÔNG KÍNH & CÓ KÍNH
        let base_vision = calc_vision_score(&self.form.khong_kinh_mat_phai, &self.form.khong_kinh_mat_trai);
        let has_glasses = !self.form.co_kinh_mat_phai.trim().is_empty() || !self.form.co_kinh_mat_trai.trim().is_empty();
        let vision_glasses = if has_glasses {
            calc_vision_score(&self.form.co_kinh_mat_phai, &self.form.co_kinh_mat_trai)
        } else {
            base_vision
        };

        if spec_id == "CK_MAT" {
            egui::Frame::group(ui.style())
                .fill(if self.dark_mode { egui::Color32::from_rgb(30, 41, 59) } else { egui::Color32::from_rgb(239, 246, 255) })
                .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(147, 197, 253)))
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.strong(egui::RichText::new("👁 Đo thị lực & Tự động phân loại").size(14.0).color(egui::Color32::from_rgb(29, 78, 216)));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui_score_badge(ui, base_vision, false);
                            ui.strong(format!("Thị lực chuẩn: LOẠI {}", base_vision));
                            if has_glasses {
                                ui.label(" | Có kính:");
                                ui_score_badge(ui, vision_glasses, false);
                            }
                        });
                    });

                    ui.add_space(6.0);
                    egui::Grid::new("grid_vision_auto").num_columns(3).spacing([20.0, 8.0]).show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("Tay thuận:").strong().size(12.0));
                            ui.horizontal(|ui| {
                                if ui.radio_value(&mut self.form.dominant_hand, "right".into(), "Tay Phải").changed() {
                                    self.sync_specialty_result(spec_id);
                                }
                                if ui.radio_value(&mut self.form.dominant_hand, "left".into(), "Tay Trái").changed() {
                                    self.sync_specialty_result(spec_id);
                                }
                            });
                        });

                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("Thị lực KHÔNG KÍNH * (bắt buộc):").strong().size(12.0));
                            ui.horizontal(|ui| {
                                ui.label("MP:");
                                if ui.add(egui::TextEdit::singleline(&mut self.form.khong_kinh_mat_phai).desired_width(55.0)).changed() {
                                    self.sync_specialty_result(spec_id);
                                }
                                ui.label("/10   MT:");
                                if ui.add(egui::TextEdit::singleline(&mut self.form.khong_kinh_mat_trai).desired_width(55.0)).changed() {
                                    self.sync_specialty_result(spec_id);
                                }
                                ui.label("/10");
                            });
                        });

                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("Thị lực CÓ KÍNH (không bắt buộc):").strong().size(12.0));
                            ui.horizontal(|ui| {
                                ui.label("MP:");
                                if ui.add(egui::TextEdit::singleline(&mut self.form.co_kinh_mat_phai).desired_width(55.0).hint_text("VD: 10")).changed() {
                                    self.sync_specialty_result(spec_id);
                                }
                                ui.label("/10   MT:");
                                if ui.add(egui::TextEdit::singleline(&mut self.form.co_kinh_mat_trai).desired_width(55.0).hint_text("VD: 10")).changed() {
                                    self.sync_specialty_result(spec_id);
                                }
                                ui.label("/10");
                            });
                        });
                    });
                    ui.add_space(4.0);
                    ui.colored_label(egui::Color32::from_rgb(30, 64, 175), "*(Nhập > 10, hệ thống tự quy về trần 10/10)*");
                });
            ui.add_space(8.0);
        }

        // 3. BẢNG ĐO SỨC NGHE NẾU LÀ TAI MŨI HỌNG (CK_TMH): NÓI THƯỜNG 5M & NÓI THẦM
        let (base_hearing, hearing_desc) = calc_hearing_from_distances(
            &self.form.tai_phai_noi_thuong, &self.form.tai_trai_noi_thuong,
            &self.form.tai_phai_noi_tham, &self.form.tai_trai_noi_tham,
        );

        if spec_id == "CK_TMH" {
            egui::Frame::group(ui.style())
                .fill(if self.dark_mode { egui::Color32::from_rgb(30, 41, 59) } else { egui::Color32::from_rgb(254, 243, 199) })
                .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(252, 211, 77)))
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.strong(egui::RichText::new("👂 Đo sức nghe & Tự động phân loại").size(14.0).color(egui::Color32::from_rgb(180, 83, 9)));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui_score_badge(ui, base_hearing, false);
                            ui.strong(format!("Sức nghe chuẩn: LOẠI {}", base_hearing));
                        });
                    });

                    ui.add_space(6.0);
                    egui::Grid::new("grid_hearing_auto").num_columns(2).spacing([28.0, 8.0]).show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("Tiếng nói thường (khoảng cách chuẩn 5m):").strong().size(12.0));
                            ui.horizontal(|ui| {
                                ui.label("Tai Phải:");
                                if ui.add(egui::TextEdit::singleline(&mut self.form.tai_phai_noi_thuong).desired_width(60.0)).changed() {
                                    self.sync_specialty_result(spec_id);
                                }
                                ui.label("m   Tai Trái:");
                                if ui.add(egui::TextEdit::singleline(&mut self.form.tai_trai_noi_thuong).desired_width(60.0)).changed() {
                                    self.sync_specialty_result(spec_id);
                                }
                                ui.label("m");
                            });
                        });

                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("Tiếng nói thầm:").strong().size(12.0));
                            ui.horizontal(|ui| {
                                ui.label("Tai Phải:");
                                if ui.add(egui::TextEdit::singleline(&mut self.form.tai_phai_noi_tham).desired_width(60.0)).changed() {
                                    self.sync_specialty_result(spec_id);
                                }
                                ui.label("m   Tai Trái:");
                                if ui.add(egui::TextEdit::singleline(&mut self.form.tai_trai_noi_tham).desired_width(60.0)).changed() {
                                    self.sync_specialty_result(spec_id);
                                }
                                ui.label("m");
                            });
                        });
                    });
                    ui.add_space(4.0);
                    ui.colored_label(egui::Color32::from_rgb(180, 83, 9), format!("*Đánh giá thính lực: {}*", hearing_desc));
                });
            ui.add_space(8.0);
        }

        // 4. BẢNG HIỂN THỊ TIM MẠCH NẾU LÀ NỘI KHOA TIM MẠCH (CK_NOI_TIM_MACH): TỰ TÍNH THEO ĐỘ TUỔI VÀ DỮ LIỆU THỂ LỰC
        let age = calculate_age(&self.form.ngay_sinh);
        let (bp_score, bp_desc) = calc_blood_pressure_score(&self.form.huyet_ap, age);
        let (pulse_score, pulse_desc) = calc_pulse_score(&self.form.mach);

        if spec_id == "CK_NOI_TIM_MACH" {
            egui::Frame::group(ui.style())
                .fill(if self.dark_mode { egui::Color32::from_rgb(30, 41, 59) } else { egui::Color32::from_rgb(255, 241, 242) })
                .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(254, 205, 211)))
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.strong(egui::RichText::new("❤️ Phân loại Tim Mạch tự động (theo Độ tuổi & Chỉ số Thể lực)").size(14.0).color(egui::Color32::from_rgb(190, 18, 60)));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label("Mạch:");
                            ui_score_badge(ui, pulse_score, false);
                            ui.label(" | Huyết áp:");
                            ui_score_badge(ui, bp_score, false);
                        });
                    });

                    ui.add_space(6.0);
                    egui::Grid::new("grid_tim_mach_auto").num_columns(3).spacing([24.0, 8.0]).show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("Độ tuổi bệnh nhân:").strong().size(12.0));
                            ui.colored_label(egui::Color32::from_rgb(190, 18, 60), egui::RichText::new(format!("{} tuổi", age)).strong().size(13.0));
                            ui.weak(format!("(tính từ ngày sinh: {})", if self.form.ngay_sinh.is_empty() { "mặc định" } else { &self.form.ngay_sinh }));
                        });

                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("Huyết áp (lấy từ Thể lực):").strong().size(12.0));
                            ui.horizontal(|ui| {
                                let bp_display = if self.form.huyet_ap.trim().is_empty() { "120/80" } else { &self.form.huyet_ap };
                                ui.strong(egui::RichText::new(format!("{} mmHg", bp_display)).size(13.5).color(egui::Color32::from_rgb(159, 18, 57)));
                                ui_score_badge(ui, bp_score, false);
                            });
                            ui.colored_label(egui::Color32::from_rgb(159, 18, 57), egui::RichText::new(&bp_desc).size(11.0));
                        });

                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("Tần số tim / Mạch (lấy từ Thể lực):").strong().size(12.0));
                            ui.horizontal(|ui| {
                                let mach_display = if self.form.mach.trim().is_empty() { "75" } else { &self.form.mach };
                                ui.strong(egui::RichText::new(format!("{} lần/phút", mach_display)).size(13.5).color(egui::Color32::from_rgb(159, 18, 57)));
                                ui_score_badge(ui, pulse_score, false);
                            });
                            ui.colored_label(egui::Color32::from_rgb(159, 18, 57), egui::RichText::new(&pulse_desc).size(11.0));
                        });
                    });
                    ui.add_space(4.0);
                    ui.colored_label(egui::Color32::from_rgb(100, 116, 139), egui::RichText::new("*(Chỉ số Huyết áp và Mạch được tự động đồng bộ từ mục Khám Thể Lực)*").italics().size(11.0));
                });
            ui.add_space(8.0);
        }

        // 5. DANH SÁCH CÁC THẺ BỆNH TẬT CỦA CHUYÊN KHOA
        if diseases.is_empty() {
            ui.colored_label(egui::Color32::GRAY, "Chuyên khoa này chưa có danh mục bệnh tật nào.");
        } else {
            for disease in &diseases {
                let clss: Vec<QdClassification> = self.qd_classifications.iter().filter(|c| c.disease_id == disease.id).cloned().collect();
                if clss.is_empty() { continue; }

                // Thẻ bệnh tự động: Thị lực
                if disease.id == "BT_MAT_THI_LUC" {
                    egui::Frame::group(ui.style())
                        .fill(if self.dark_mode { egui::Color32::from_rgb(30, 41, 59) } else { egui::Color32::from_rgb(240, 249, 255) })
                        .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(186, 230, 253)))
                        .inner_margin(10.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.strong(format!("⚡ {} [TỰ ĐỘNG]", disease.name));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let eff_vision = if has_glasses { vision_glasses } else { base_vision };
                                    ui_score_badge(ui, eff_vision, false);
                                    let mut info = format!("Không kính: MP {}/10 — MT {}/10", self.form.khong_kinh_mat_phai, self.form.khong_kinh_mat_trai);
                                    if has_glasses {
                                        info.push_str(&format!(" | Có kính: MP {}/10 — MT {}/10", self.form.co_kinh_mat_phai, self.form.co_kinh_mat_trai));
                                    }
                                    ui.label(info);
                                });
                            });
                        });
                    ui.add_space(4.0);
                    continue;
                }

                // Thẻ bệnh tự động: Thính lực
                if disease.id == "BT_TMH_TAI_TRONG" {
                    egui::Frame::group(ui.style())
                        .fill(if self.dark_mode { egui::Color32::from_rgb(30, 41, 59) } else { egui::Color32::from_rgb(255, 251, 235) })
                        .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(253, 230, 138)))
                        .inner_margin(10.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.strong(format!("⚡ {} [TỰ ĐỘNG]", disease.name));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui_score_badge(ui, base_hearing, false);
                                    ui.label(format!("Nói thường: P {}m — T {}m | Nói thầm: P {}m — T {}m", self.form.tai_phai_noi_thuong, self.form.tai_trai_noi_thuong, self.form.tai_phai_noi_tham, self.form.tai_trai_noi_tham));
                                });
                            });
                        });
                    ui.add_space(4.0);
                    continue;
                }

                // Thẻ bệnh tự động: Huyết áp
                if disease.id == "BT_TM_HUYET_AP" {
                    egui::Frame::group(ui.style())
                        .fill(if self.dark_mode { egui::Color32::from_rgb(30, 41, 59) } else { egui::Color32::from_rgb(255, 241, 242) })
                        .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(254, 205, 211)))
                        .inner_margin(10.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.strong(format!("⚡ {} [TỰ ĐỘNG]", disease.name));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui_score_badge(ui, bp_score, false);
                                    ui.label(format!("Huyết áp {} mmHg ({})", self.form.huyet_ap, bp_desc));
                                });
                            });
                        });
                    ui.add_space(4.0);
                    continue;
                }

                // Thẻ bệnh tự động: Tần số mạch / tim
                if disease.id == "BT_TM_MACH" {
                    egui::Frame::group(ui.style())
                        .fill(if self.dark_mode { egui::Color32::from_rgb(30, 41, 59) } else { egui::Color32::from_rgb(255, 241, 242) })
                        .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(254, 205, 211)))
                        .inner_margin(10.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.strong(format!("⚡ {} [TỰ ĐỘNG]", disease.name));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui_score_badge(ui, pulse_score, false);
                                    ui.label(format!("Tần số {} lần/phút ({})", self.form.mach, pulse_desc));
                                });
                            });
                        });
                    ui.add_space(4.0);
                    continue;
                }

                // Thẻ bệnh tiêu chuẩn: CHỌN BẰNG TICKBOX (CHECKBOX)
                ui.push_id(&disease.id, |ui| {
                    let is_expanded = self.expanded_diseases.contains(&disease.id);

                    // Lấy tất cả các tiêu chuẩn đang được tick chọn cho bệnh này
                    let ticked_clss: Vec<&QdClassification> = clss.iter()
                        .filter(|c| self.form.selected_items.contains(&c.id))
                        .collect();

                    let disease_score = if ticked_clss.is_empty() {
                        1
                    } else {
                        ticked_clss.iter()
                            .map(|c| calc_classification_score(c, base_vision, base_hearing))
                            .max()
                            .unwrap_or(1)
                    };

                    let subtitle_text = if ticked_clss.is_empty() {
                        "Bình thường / Chưa phát hiện bệnh".to_string()
                    } else {
                        ticked_clss.iter().map(|c| c.name.as_str()).collect::<Vec<&str>>().join("; ")
                    };

                    egui::Frame::group(ui.style())
                        .fill(if self.dark_mode {
                            if disease_score > 1 { egui::Color32::from_rgb(35, 30, 45) } else { egui::Color32::from_rgb(24, 30, 42) }
                        } else {
                            if disease_score > 1 { egui::Color32::from_rgb(254, 242, 242) } else { egui::Color32::from_rgb(248, 250, 252) }
                        })
                        .stroke(egui::Stroke::new(1.0_f32, if disease_score > 1 { egui::Color32::from_rgb(248, 113, 113) } else { egui::Color32::from_rgb(226, 232, 240) }))
                        .inner_margin(10.0)
                        .show(ui, |ui| {
                            // Header có thể click để bung ra / thu gọn
                            let header_id = ui.id().with("card_hdr");
                            let header_resp = ui.horizontal(|ui| {
                                let chevron = if is_expanded { "▼" } else { "▶" };
                                ui.label(egui::RichText::new(chevron).monospace().size(12.0));
                                ui.vertical(|ui| {
                                    ui.strong(egui::RichText::new(&disease.name).size(13.5));
                                    ui.colored_label(
                                        if disease_score > 1 { egui::Color32::from_rgb(220, 38, 38) } else { egui::Color32::from_rgb(100, 116, 139) },
                                        egui::RichText::new(&subtitle_text).size(11.5)
                                    );
                                });
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui_score_badge(ui, disease_score, false);
                                });
                            }).response;

                            let click_area = ui.interact(header_resp.rect, header_id, egui::Sense::click());
                            if click_area.clicked() {
                                if is_expanded {
                                    self.expanded_diseases.remove(&disease.id);
                                } else {
                                    self.expanded_diseases.insert(disease.id.clone());
                                }
                            }

                            // Nội dung bung ra: các tiêu chuẩn phân loại DẠNG TICKBOX (CHECKBOX)
                            if is_expanded {
                                ui.separator();
                                ui.vertical(|ui| {
                                    for c in &clss {
                                        ui.push_id(&c.id, |ui| {
                                            let mut is_checked = self.form.selected_items.contains(&c.id);
                                            let c_score = calc_classification_score(c, base_vision, base_hearing);
                                            let is_dyn = c.ref_disease_id.is_some();

                                            let row_resp = ui.horizontal(|ui| {
                                                let chk_btn = ui.checkbox(&mut is_checked, "");
                                                ui.vertical(|ui| {
                                                    ui.label(if is_checked {
                                                        egui::RichText::new(&c.name).strong().color(egui::Color32::from_rgb(14, 165, 233))
                                                    } else {
                                                        egui::RichText::new(&c.name)
                                                    });
                                                    if let Some(ref r) = c.ref_rule {
                                                        ui.colored_label(egui::Color32::from_rgb(180, 83, 9), egui::RichText::new(format!("↳ {}", r)).italics().size(11.0));
                                                    }
                                                });

                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                    ui_score_badge(ui, c_score, is_dyn);
                                                });

                                                chk_btn
                                            });

                                            let row_click = ui.interact(row_resp.response.rect, ui.id().with("opt_click"), egui::Sense::click());
                                            if row_resp.inner.changed() || row_click.clicked() {
                                                if row_click.clicked() && !row_resp.inner.changed() {
                                                    is_checked = !is_checked;
                                                }
                                                if is_checked {
                                                    self.form.selected_items.insert(c.id.clone());
                                                } else {
                                                    self.form.selected_items.remove(&c.id);
                                                }
                                                self.sync_specialty_result(spec_id);
                                            }
                                        });
                                        ui.add_space(2.0);
                                    }
                                });
                            }
                        });
                    ui.add_space(4.0);
                });
            }
        }

        ui.add_space(8.0);
        // NÚT BỔ SUNG: THÊM BỆNH LÝ NGOÀI DANH MỤC QĐ 1613
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Bệnh ngoài danh mục QĐ 1613:").size(12.5).color(egui::Color32::from_rgb(100, 116, 139)));
            if ui.button(egui::RichText::new("➕ Nhập bệnh ngoài danh sách").strong().color(egui::Color32::from_rgb(14, 165, 233))).clicked() {
                self.form.specialty_extras.entry(spec_id.to_string()).or_default().manual_rows.push(ManualConclusionRow {
                    classification: 2,
                    classification_id: "Ngoài danh mục".into(),
                    manifestation: "".into(),
                });
                self.sync_specialty_result(spec_id);
            }
        });
        ui.add_space(8.0);

        // 6. KHỐI KẾT LUẬN CHUYÊN KHOA (BÌNH THƯỜNG / CHI TIẾT BỆNH LÝ)
        let extra = self.form.specialty_extras.entry(spec_id.to_string()).or_default().clone();
        let mut abnormal_rows: Vec<(String, String, i32, String)> = Vec::new(); // (item_id, item_name, score, detail_name)
        let mut max_spec_score = 1;

        if spec_id == "CK_MAT" && base_vision > 1 {
            if base_vision > max_spec_score { max_spec_score = base_vision; }
            let desc = format!("Không kính MP {}/10, MT {}/10", self.form.khong_kinh_mat_phai, self.form.khong_kinh_mat_trai);
            abnormal_rows.push(("BT_MAT_THI_LUC".into(), "Thị lực".into(), base_vision, desc));
        }
        if spec_id == "CK_TMH" && base_hearing > 1 {
            if base_hearing > max_spec_score { max_spec_score = base_hearing; }
            abnormal_rows.push(("BT_TMH_TAI_TRONG".into(), "Tai trong (Sức nghe)".into(), base_hearing, hearing_desc));
        }
        if spec_id == "CK_NOI_TIM_MACH" {
            if bp_score > 1 {
                if bp_score > max_spec_score { max_spec_score = bp_score; }
                abnormal_rows.push(("BT_TM_HUYET_AP".into(), "Huyết áp".into(), bp_score, bp_desc));
            }
            if pulse_score > 1 {
                if pulse_score > max_spec_score { max_spec_score = pulse_score; }
                abnormal_rows.push(("BT_TM_MACH".into(), "Tần số tim".into(), pulse_score, pulse_desc));
            }
        }

        for d in &diseases {
            if d.id == "BT_MAT_THI_LUC" || d.id == "BT_TMH_TAI_TRONG" || d.id == "BT_TM_HUYET_AP" || d.id == "BT_TM_MACH" {
                continue;
            }
            let clss: Vec<QdClassification> = self.qd_classifications.iter().filter(|c| c.disease_id == d.id).cloned().collect();
            for c in &clss {
                if self.form.selected_items.contains(&c.id) {
                    let sc = calc_classification_score(c, base_vision, base_hearing);
                    if sc > max_spec_score { max_spec_score = sc; }
                    if sc > 1 {
                        abnormal_rows.push((c.id.clone(), d.name.clone(), sc, c.name.clone()));
                    }
                }
            }
        }

        for mr in &extra.manual_rows {
            if mr.classification > max_spec_score {
                max_spec_score = mr.classification;
            }
        }

        // Trường hợp tất cả Loại 1 và không có dòng nhập tay: HIỂN THỊ HỘP XANH BÌNH THƯỜNG
        if max_spec_score == 1 && abnormal_rows.is_empty() && extra.manual_rows.is_empty() {
            egui::Frame::group(ui.style())
                .fill(if self.dark_mode { egui::Color32::from_rgb(20, 45, 35) } else { egui::Color32::from_rgb(236, 253, 245) })
                .stroke(egui::Stroke::new(2.0_f32, egui::Color32::from_rgb(110, 231, 183)))
                .inner_margin(16.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.strong(egui::RichText::new(format!("🛡 KẾT LUẬN — {}", spec_name)).size(14.0).color(egui::Color32::from_rgb(6, 95, 70)));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.colored_label(egui::Color32::from_rgb(6, 95, 70), egui::RichText::new("BÌNH THƯỜNG (LOẠI 1)").strong().size(13.0));
                        });
                    });
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("✔").size(22.0).color(egui::Color32::from_rgb(16, 185, 129)));
                        ui.vertical(|ui| {
                            ui.strong(egui::RichText::new("Không phát hiện bệnh lý").size(14.0).color(egui::Color32::from_rgb(6, 95, 70)));
                            ui.colored_label(egui::Color32::from_rgb(4, 120, 87), "Tất cả chỉ số khám đều ở mức Loại 1 — bình thường.");
                        });
                    });
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.colored_label(egui::Color32::from_rgb(100, 116, 139), egui::RichText::new("* Kết luận được hệ thống tự động xác định. Nếu phát hiện bất thường ngoài danh mục:").italics().size(11.0));
                        if ui.button(egui::RichText::new("➕ Nhập bệnh ngoài danh sách").strong().color(egui::Color32::from_rgb(180, 83, 9))).clicked() {
                            self.form.specialty_extras.entry(spec_id.to_string()).or_default().manual_rows.push(ManualConclusionRow {
                                classification: 2,
                                classification_id: "Ngoài danh mục".into(),
                                manifestation: "".into(),
                            });
                            self.sync_specialty_result(spec_id);
                        }
                    });
                });
        } else {
            // Trường hợp có bệnh lý > Loại 1: HIỂN THỊ BẢNG KẾT LUẬN & TEXT THỂ HIỆN CHO PHÉP SỬA
            egui::Frame::group(ui.style())
                .fill(if self.dark_mode { egui::Color32::from_rgb(30, 41, 59) } else { egui::Color32::from_rgb(240, 253, 250) })
                .stroke(egui::Stroke::new(2.0_f32, egui::Color32::from_rgb(94, 234, 212)))
                .inner_margin(16.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.strong(egui::RichText::new(format!("🛡 KẾT LUẬN — {}", spec_name)).size(14.0).color(egui::Color32::from_rgb(19, 78, 74)));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui_score_badge(ui, max_spec_score, false);
                            ui.strong(format!("Phân loại tổng: Loại {}", max_spec_score));
                        });
                    });

                    ui.separator();

                    ui.strong("CÁC BỆNH GHI NHẬN (THEO DANH MỤC & NGOÀI DANH SÁCH):");
                    ui.add_space(4.0);

                    let mut changed_anything = false;

                    egui::Grid::new(format!("grid_concl_{}", spec_id)).striped(true).min_col_width(70.0).spacing([12.0, 8.0]).show(ui, |ui| {
                        ui.strong("STT");
                        ui.strong("Phân loại");
                        ui.strong("Nguồn / Tiêu chuẩn");
                        ui.strong("Tên bệnh / Thể hiện (chỉnh sửa được)");
                        ui.strong("Hành động");
                        ui.end_row();

                        let mut stt = 1;

                        // Các hàng tự động từ bệnh tật
                        for (item_id, d_name, sc, c_name) in abnormal_rows {
                            ui.push_id(&item_id, |ui| {
                                ui.label(stt.to_string());
                                ui_score_badge(ui, sc, false);
                                ui.vertical(|ui| {
                                    ui.strong(&c_name);
                                    ui.colored_label(egui::Color32::GRAY, egui::RichText::new(&d_name).size(11.0));
                                });

                                let cur_override = self.form.specialty_extras.entry(spec_id.to_string()).or_default()
                                    .auto_overrides.entry(item_id.clone()).or_insert_with(|| c_name.clone());

                                if ui.add(egui::TextEdit::singleline(cur_override).desired_width(320.0)).changed() {
                                    changed_anything = true;
                                }

                                ui.colored_label(egui::Color32::from_rgb(16, 185, 129), egui::RichText::new("tự động").italics().size(11.0));
                            });
                            ui.end_row();
                            stt += 1;
                        }

                        // Các hàng nhập tay (Bệnh ngoài danh sách)
                        let mut remove_idx = None;
                        let extra_ref = self.form.specialty_extras.entry(spec_id.to_string()).or_default();
                        for (idx, row) in extra_ref.manual_rows.iter_mut().enumerate() {
                            ui.push_id(idx, |ui| {
                                ui.label(stt.to_string());
                                egui::ComboBox::from_id_source(format!("cb_pl_m_{}_{}", spec_id, idx))
                                    .width(80.0)
                                    .selected_text(format!("Loại {}", row.classification))
                                    .show_ui(ui, |ui| {
                                        for n in 1..=5 {
                                            if ui.selectable_value(&mut row.classification, n, format!("Loại {}", n)).clicked() {
                                                changed_anything = true;
                                            }
                                        }
                                    });

                                ui.colored_label(egui::Color32::from_rgb(180, 83, 9), egui::RichText::new("Ngoài danh mục").strong().size(11.5));
                                if ui.add(egui::TextEdit::singleline(&mut row.manifestation).hint_text("Nhập tên bệnh ngoài danh mục hoặc mô tả...").desired_width(320.0)).changed() {
                                    changed_anything = true;
                                }
                                if ui.button(egui::RichText::new("Xóa").color(egui::Color32::from_rgb(220, 38, 38))).clicked() {
                                    remove_idx = Some(idx);
                                }
                            });
                            ui.end_row();
                            stt += 1;
                        }

                        if let Some(r_idx) = remove_idx {
                            self.form.specialty_extras.entry(spec_id.to_string()).or_default().manual_rows.remove(r_idx);
                            changed_anything = true;
                        }
                    });

                    ui.add_space(8.0);
                    if ui.button(egui::RichText::new("➕ Thêm bệnh ngoài danh sách").strong().color(egui::Color32::from_rgb(14, 165, 233))).clicked() {
                        self.form.specialty_extras.entry(spec_id.to_string()).or_default().manual_rows.push(ManualConclusionRow {
                            classification: 2,
                            classification_id: "Ngoài danh mục".into(),
                            manifestation: "".into(),
                        });
                        changed_anything = true;
                    }

                    ui.add_space(8.0);
                    ui.strong("GHI CHÚ TỔNG HỢP (PHẦN HIỂN THỊ CUỐI CÙNG):");
                    let final_note_ref = &mut self.form.specialty_extras.entry(spec_id.to_string()).or_default().final_note;
                    if ui.add(egui::TextEdit::multiline(final_note_ref).desired_rows(2).desired_width(ui.available_width())).changed() {
                        changed_anything = true;
                    }

                    if changed_anything {
                        self.sync_specialty_result(spec_id);
                    }
                });
        }
    }
}

fn apply_theme(ctx: &egui::Context, dark: bool) {
    let mut visuals = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    visuals.window_rounding = 6.0.into();
    ctx.set_visuals(visuals);
}

impl eframe::App for KskApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // GIAO DIỆN ĐĂNG NHẬP NẾU CHƯA ĐĂNG NHẬP
        if !self.logged_in {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(80.0);
                    ui.heading(egui::RichText::new("🏥 HỆ THỐNG KHÁM SỨC KHỎE ĐỊNH KỲ").size(24.0).strong().color(egui::Color32::from_rgb(14, 165, 233)));
                    ui.label(egui::RichText::new("Chuẩn Quyết định 3176/QĐ-BYT & Quyết định 1613/BYT").size(14.0).weak());
                    ui.add_space(24.0);

                    egui::Frame::group(ui.style())
                        .fill(if self.dark_mode { egui::Color32::from_rgb(30, 41, 59) } else { egui::Color32::from_rgb(248, 250, 252) })
                        .inner_margin(24.0)
                        .rounding(8.0)
                        .show(ui, |ui| {
                            ui.set_max_width(340.0);
                            ui.strong("Tài khoản:");
                            ui.add(egui::TextEdit::singleline(&mut self.login_user).desired_width(320.0));
                            ui.add_space(8.0);

                            ui.strong("Mật khẩu:");
                            ui.add(egui::TextEdit::singleline(&mut self.login_pass).password(true).desired_width(320.0));
                            ui.add_space(16.0);

                            if ui.button(egui::RichText::new("  Đăng Nhập  ").size(15.0).strong()).clicked() {
                                let query = "SELECT id, username, role FROM users WHERE username = ?1 AND password = ?2";
                                let pass_hashed = hash_pass(&self.login_pass);
                                let found = {
                                    let mut stmt = self.conn.prepare(query).unwrap();
                                    stmt.query_row(params![self.login_user, self.login_pass], |r| {
                                        Ok((r.get::<_, String>(1)?, r.get::<_, String>(2)?))
                                    }).or_else(|_| {
                                        stmt.query_row(params![self.login_user, pass_hashed], |r| {
                                            Ok((r.get::<_, String>(1)?, r.get::<_, String>(2)?))
                                        })
                                    })
                                };

                                match found {
                                    Ok((u, r)) => {
                                        self.current_user = u;
                                        self.current_role = r;
                                        self.logged_in = true;
                                        self.status_msg = format!("Xin chào {}, vai trò: {}!", self.current_user, self.current_role);
                                        self.new_record_with_defaults();
                                    }
                                    Err(_) => {
                                        self.status_msg = "LỖI: Sai tên đăng nhập hoặc mật khẩu!".into();
                                    }
                                }
                            }

                            if !self.status_msg.is_empty() {
                                ui.add_space(8.0);
                                ui.colored_label(egui::Color32::RED, &self.status_msg);
                            }
                        });
                });
            });
            return;
        }

        // TOP PANEL: THANH TIÊU ĐỀ, USER INFO, NÚT GIAO DIỆN & TAB CHÍNH
        egui::TopBottomPanel::top("top_nav").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.heading(egui::RichText::new("🏥 KSK ĐỊNH KỲ").strong().color(egui::Color32::from_rgb(14, 165, 233)));
                ui.label(egui::RichText::new("QĐ 3176/QĐ-BYT & QĐ 1613/BYT").weak().size(12.0));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Đăng Xuất").clicked() {
                        self.logged_in = false;
                        self.login_pass.clear();
                        self.current_user.clear();
                        self.current_role.clear();
                    }

                    if ui.button(if self.dark_mode { "☀ Sáng" } else { "🌙 Tối" }).clicked() {
                        self.dark_mode = !self.dark_mode;
                        apply_theme(ctx, self.dark_mode);
                    }

                    ui.colored_label(
                        egui::Color32::from_rgb(14, 165, 233),
                        format!("👤 {} ({})", self.current_user, self.current_role)
                    );
                });
            });

            ui.separator();

            // CÁC TAB CHÍNH TOÀN ỨNG DỤNG
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.active_tab, 0, "📝 Phiếu Nhập Liệu");
                ui.selectable_value(&mut self.active_tab, 1, format!("📋 Hồ Sơ ({})", self.records.len()));
                ui.selectable_value(&mut self.active_tab, 2, "📚 Danh Mục & Quy Tắc");
                if self.current_role == "admin" {
                    ui.selectable_value(&mut self.active_tab, 3, "👥 Quản Lý Users");
                    ui.selectable_value(&mut self.active_tab, 4, "⚙ Cài Đặt Hệ Thống");
                }
            });
            ui.add_space(4.0);
        });

        // BOTTOM PANEL: STATUS BAR
        egui::TopBottomPanel::bottom("bottom_status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(&self.status_msg).size(12.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.weak("App KSK QĐ 3176 v0.4.0 — Google Antigravity");
                });
            });
        });

        // CENTRAL PANEL NỘI DUNG CHÍNH THEO TAB ĐANG CHỌN
        egui::CentralPanel::default().show(ctx, |ui| {
            // TAB 3: QUẢN LÝ USERS (ADMIN)
            if self.active_tab == 3 {
                ui.heading("Quản Lý Người Dùng & Phân Quyền");
                ui.separator();

                egui::Grid::new("grid_add_user").num_columns(4).spacing([12.0, 8.0]).show(ui, |ui| {
                    ui.label("Tên đăng nhập mới:"); ui.text_edit_singleline(&mut self.new_username);
                    ui.label("Mật khẩu:"); ui.text_edit_singleline(&mut self.new_password);
                    ui.end_row();

                    ui.label("Vai trò:");
                    egui::ComboBox::from_id_source("cb_new_role").selected_text(&self.new_user_role).show_ui(ui, |ui| {
                        let roles = [
                            ("admin", "Quản trị viên (Admin)"),
                            ("reception", "Tiếp đón / Hành chính"),
                            ("bs_theluc", "Bác sĩ Thể lực"),
                            ("bs_noi", "Bác sĩ Nội khoa"),
                            ("bs_ngoai", "Bác sĩ Ngoại khoa"),
                            ("bs_dalieu", "Bác sĩ Da liễu"),
                            ("bs_san", "Bác sĩ Sản phụ khoa"),
                            ("bs_mat", "Bác sĩ Mắt"),
                            ("bs_tmh", "Bác sĩ Tai Mũi Họng"),
                            ("bs_rhm", "Bác sĩ Răng Hàm Mặt"),
                            ("bs_cls", "Bác sĩ Cận Lâm Sàng"),
                            ("bs_ketluan", "Bác sĩ Kết Luận"),
                        ];
                        for (r_code, r_label) in roles {
                            ui.selectable_value(&mut self.new_user_role, r_code.to_string(), r_label);
                        }
                    });

                    if ui.button("➕ Thêm User").clicked() {
                        if !self.new_username.trim().is_empty() && !self.new_password.trim().is_empty() {
                            let pass_h = hash_pass(&self.new_password);
                            let _ = self.conn.execute(
                                "INSERT INTO users (username, password, role) VALUES (?1, ?2, ?3)",
                                params![self.new_username, pass_h, self.new_user_role],
                            );
                            self.new_username.clear();
                            self.new_password.clear();
                            self.reload_users();
                            self.status_msg = "Đã thêm người dùng thành công!".into();
                        }
                    }
                    ui.end_row();
                });

                ui.separator();
                ui.strong("DANH SÁCH TÀI KHOẢN NGƯỜI DÙNG:");
                let mut del_user_id = None;
                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("grid_users_table").striped(true).min_col_width(120.0).spacing([12.0, 6.0]).show(ui, |ui| {
                        ui.strong("ID"); ui.strong("Tên Đăng Nhập"); ui.strong("Vai Trò"); ui.strong("Hành Động"); ui.end_row();
                        for u in &self.user_list {
                            ui.label(u.id.to_string());
                            ui.label(&u.username);
                            ui.label(&u.role);
                            if u.username != "admin" && ui.button("Xóa").clicked() {
                                del_user_id = Some(u.id);
                            } else {
                                ui.label("");
                            }
                            ui.end_row();
                        }
                    });
                });
                if let Some(id) = del_user_id {
                    let _ = self.conn.execute("DELETE FROM users WHERE id = ?", params![id]);
                    self.reload_users();
                    self.status_msg = "Đã xóa tài khoản người dùng!".into();
                }
                return;
            }

            // TAB 4: CÀI ĐẶT HỆ THỐNG
            if self.active_tab == 4 {
                ui.heading("Cài Đặt Hệ Thống & Tham Số Mặc Định");
                ui.separator();

                egui::Grid::new("grid_settings").num_columns(2).spacing([16.0, 10.0]).show(ui, |ui| {
                    ui.label("Mã CSKCB mặc định:"); ui.text_edit_singleline(&mut self.setting_cskcb); ui.end_row();
                    ui.label("Mã Định Danh GTIN CSKCB:"); ui.text_edit_singleline(&mut self.setting_gtin); ui.end_row();
                    ui.label("Mã Tỉnh Nơi Ở Mặc Định:"); ui.text_edit_singleline(&mut self.setting_prov); ui.end_row();
                    ui.label("Mã Xã Nơi Ở Mặc Định:"); ui.text_edit_singleline(&mut self.setting_ward); ui.end_row();
                });

                ui.add_space(10.0);
                if ui.button("💾 Lưu Cài Đặt").clicked() {
                    self.config.default_cskcb = self.setting_cskcb.clone();
                    self.config.default_gtin_cskcb = self.setting_gtin.clone();
                    self.config.default_province_code = self.setting_prov.clone();
                    self.config.default_ward_code = self.setting_ward.clone();
                    if let Ok(_) = self.config.save() {
                        self.status_msg = "Đã lưu cài đặt cấu hình thành công!".into();
                    }
                }
                return;
            }

            // TAB 2: QUẢN LÝ DANH MỤC & BỘ QUY TẮC QĐ 1613
            if self.active_tab == 2 {
                ui.horizontal(|ui| {
                    if ui.selectable_label(self.cur_cat_tab != "qd1613", "📚 Danh Mục QĐ 3176").clicked() {
                        self.cur_cat_tab = "priority".into();
                    }
                    if ui.selectable_label(self.cur_cat_tab == "qd1613", "⚖ Bộ Quy Tắc Phân Loại QĐ 1613/BYT").clicked() {
                        self.cur_cat_tab = "qd1613".into();
                    }
                });
                ui.separator();

                if self.cur_cat_tab == "qd1613" {
                    ui.horizontal(|ui| {
                        ui.heading("Bộ Quy Tắc Phân Loại Bệnh Tật & Sức Khỏe (QĐ 1613/BYT)");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("🔄 Khôi Phục Quy Tắc Gốc (QĐ 1613)").clicked() {
                                self.reset_qd1613_to_default();
                                self.reload_qd1613();
                                self.status_msg = "Đã khôi phục toàn bộ 499 quy tắc phân loại gốc theo QĐ 1613/BYT!".into();
                            }
                        });
                    });

                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(format!("📊 Thống kê: {} Chuyên khoa | {} Bệnh tật | {} Quy tắc phân loại", self.qd_specialties.len(), self.qd_diseases.len(), self.qd_classifications.len()));
                    });

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label("Lọc theo chuyên khoa:");
                        let cur_spec_label = if self.qd_rule_filter_spec == "all" {
                            "Tất cả chuyên khoa".to_string()
                        } else {
                            self.qd_specialties.iter().find(|s| s.id == self.qd_rule_filter_spec).map(|s| s.name.clone()).unwrap_or("Tất cả".to_string())
                        };

                        egui::ComboBox::from_id_source("cb_filter_spec")
                            .width(220.0)
                            .selected_text(cur_spec_label)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.qd_rule_filter_spec, "all".into(), "Tất cả chuyên khoa");
                                for s in &self.qd_specialties {
                                    ui.selectable_value(&mut self.qd_rule_filter_spec, s.id.clone(), &s.name);
                                }
                            });

                        ui.add_space(12.0);
                        ui.label("Tìm kiếm:");
                        ui.add(egui::TextEdit::singleline(&mut self.qd_rule_search).hint_text("Nhập tên bệnh hoặc tiêu chuẩn..."));
                    });

                    ui.separator();

                    let kw = self.qd_rule_search.to_lowercase();
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        egui::Grid::new("grid_qd1613_rules").striped(true).min_col_width(70.0).spacing([12.0, 6.0]).show(ui, |ui| {
                            ui.strong("Mã Tiêu Chuẩn");
                            ui.strong("Chuyên Khoa");
                            ui.strong("Bệnh Tật");
                            ui.strong("Tiêu Chuẩn / Phân Loại");
                            ui.strong("Điểm Phân Loại");
                            ui.strong("Quy Tắc Tham Chiếu");
                            ui.end_row();

                            let mut filtered_items = Vec::new();
                            for c in &self.qd_classifications {
                                let disease = self.qd_diseases.iter().find(|d| d.id == c.disease_id);
                                let spec_id = disease.map(|d| d.spec_id.as_str()).unwrap_or("");
                                if self.qd_rule_filter_spec != "all" && spec_id != self.qd_rule_filter_spec {
                                    continue;
                                }
                                let d_name = disease.map(|d| d.name.as_str()).unwrap_or("");
                                let spec = self.qd_specialties.iter().find(|s| s.id == spec_id);
                                let s_name = spec.map(|s| s.name.as_str()).unwrap_or("");

                                if !kw.is_empty() && !c.name.to_lowercase().contains(&kw) && !d_name.to_lowercase().contains(&kw) && !c.id.to_lowercase().contains(&kw) {
                                    continue;
                                }
                                filtered_items.push((c.id.clone(), s_name.to_string(), d_name.to_string(), c.name.clone(), c.score, c.ref_rule.clone(), c.ref_disease_id.is_some()));
                            }

                            for (cid, s_name, d_name, c_name, score, rule, is_dyn) in filtered_items.iter().take(200) {
                                ui.label(cid);
                                ui.label(s_name);
                                ui.strong(d_name);
                                ui.label(c_name);
                                if let Some(sc) = score {
                                    ui_score_badge(ui, *sc, *is_dyn);
                                } else {
                                    ui.colored_label(egui::Color32::from_rgb(180, 83, 9), "Tham chiếu *");
                                }
                                ui.colored_label(egui::Color32::GRAY, rule.as_deref().unwrap_or(""));
                                ui.end_row();
                            }
                        });
                    });
                    return;
                }

                // SUB-TAB DANH MỤC QĐ 3176
                let cat_tabs = [
                    ("priority", "1. Đối tượng (16 nhóm)"),
                    ("payment", "2. Nguồn chi trả"),
                    ("province", "3. Tỉnh / TP"),
                    ("ward", "4. Xã / Phường"),
                    ("ethnicity", "5. Dân tộc"),
                    ("occupation", "6. Nghề nghiệp"),
                    ("icd10", "7. Mã ICD-10"),
                ];

                ui.horizontal_wrapped(|ui| {
                    for (t_code, t_title) in cat_tabs {
                        if ui.selectable_label(self.cur_cat_tab == t_code, t_title).clicked() {
                            self.cur_cat_tab = t_code.into();
                        }
                    }
                });

                ui.separator();

                let filtered_cats: Vec<CatalogRecord> = self.catalogs.iter().filter(|c| c.cat_type == self.cur_cat_tab).cloned().collect();
                ui.label(format!("Tổng số bản ghi danh mục '{}': {}", self.cur_cat_tab, filtered_cats.len()));

                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("grid_cat_display").striped(true).min_col_width(120.0).spacing([12.0, 6.0]).show(ui, |ui| {
                        ui.strong("Mã"); ui.strong("Tên Danh Mục"); ui.strong("Mã Cha"); ui.end_row();
                        for item in filtered_cats {
                            ui.label(&item.code);
                            ui.label(&item.name);
                            ui.label(&item.parent_code);
                            ui.end_row();
                        }
                    });
                });
                return;
            }

            // TAB 1: DANH SÁCH HỒ SƠ ĐÃ LƯU
            if self.active_tab == 1 {
                ui.horizontal(|ui| {
                    ui.heading(format!("Danh Sách Hồ Sơ Khám Sức Khỏe ({})", self.records.len()));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("📥 Xuất File CSV Chuẩn QĐ 3176 (97 Cột)").clicked() {
                            self.export_csv();
                        }
                    });
                });
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("grid_rec_list").striped(true).min_col_width(100.0).spacing([14.0, 8.0]).show(ui, |ui| {
                        ui.strong("ID"); ui.strong("Họ và Tên"); ui.strong("Số CCCD"); ui.strong("Ngày Vào"); ui.strong("Trạng Thái"); ui.strong("Thao Tác"); ui.end_row();
                        for r in &self.records {
                            ui.label(r.id.to_string());
                            ui.strong(&r.ho_ten);
                            ui.label(&r.so_cccd);
                            ui.label(&r.ngay_vao);
                            if r.status == 1 {
                                ui.colored_label(egui::Color32::from_rgb(220, 140, 0), "🔒 Đã Khóa");
                            } else {
                                ui.colored_label(egui::Color32::GREEN, "📝 Đang Soạn");
                            }

                            if ui.button("Mở Xem / Chỉnh Sửa").clicked() {
                                self.current_id = Some(r.id);
                                self.current_status = r.status;
                                self.form = r.data.clone();
                                self.active_tab = 0;
                            }
                            ui.end_row();
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
                        self.new_record_with_defaults();
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

            // MENU 4 PHÂN HỆ CHÍNH: HÀNH CHÍNH, TIỀN SỬ, KHÁM BỆNH, KẾT LUẬN
            ui.horizontal_wrapped(|ui| {
                let main_tabs = [
                    (0, "📋 1. HÀNH CHÍNH", "hchinh"),
                    (1, "🩺 2. TIỀN SỬ", "hchinh"),
                    (2, "🏥 3. KHÁM BỆNH", "noi"),
                    (3, "⚖ 4. KẾT LUẬN", "ketluan"),
                ];

                for (idx, title, role_key) in main_tabs {
                    let can_edit = self.can_edit_dept(role_key) || (idx == 2 && self.current_role != "reception" && self.current_role != "client") || self.current_role == "admin";
                    let mut btn_text = egui::RichText::new(title).size(15.0);
                    if can_edit {
                        btn_text = btn_text.strong().color(egui::Color32::from_rgb(14, 165, 233));
                    }
                    if ui.selectable_label(self.dept_tab == idx, btn_text).clicked() {
                        self.dept_tab = idx;
                    }
                }
            });

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                match self.dept_tab {
                    // I. HÀNH CHÍNH
                    0 => {
                        let enabled = self.can_edit_dept("hchinh");
                        ui.add_enabled_ui(enabled, |ui| {
                            egui::Grid::new("grid_hc_detail").num_columns(4).spacing([16.0, 9.0]).show(ui, |ui| {
                                ui.strong("1. Họ và tên *:"); ui.text_edit_singleline(&mut self.form.ho_ten);
                                ui.strong("2. Giới tính *:");
                                egui::ComboBox::from_id_source("cb_gt").selected_text(if self.form.gioi_tinh == 1 { "1 - Nam" } else { "2 - Nữ" }).show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.form.gioi_tinh, 1, "1 - Nam");
                                    ui.selectable_value(&mut self.form.gioi_tinh, 2, "2 - Nữ");
                                });
                                ui.end_row();

                                ui.strong("3. Ngày sinh *:");
                                date_input_picker(ui, "dp_ngay_sinh", &mut self.form.ngay_sinh);

                                ui.label("4. Dân tộc:");
                                let eths = Self::cat_items(&self.catalogs, "ethnicity", None);
                                search_combo(ui, "cb_eth", &mut self.form.ma_dan_toc, &eths);
                                ui.end_row();

                                ui.strong("5. Số CMND/CCCD/ĐD *:"); ui.text_edit_singleline(&mut self.form.so_cccd);
                                ui.label("6. Ngày cấp CCCD:");
                                date_input_picker(ui, "dp_ngaycap_cccd", &mut self.form.ngaycap_cccd);
                                ui.end_row();

                                ui.label("7. Nơi cấp CCCD:"); ui.text_edit_singleline(&mut self.form.noicap_cccd);
                                ui.label("8. Nghề nghiệp:");
                                let occs = Self::cat_items(&self.catalogs, "occupation", None);
                                search_combo(ui, "cb_occ", &mut self.form.ma_nghe_nghiep, &occs);
                                ui.end_row();

                                // NƠI Ở HIỆN TẠI (ĐẶT LIÊN TIẾP, KHÔNG BỌC KHUNG)
                                ui.strong("9. Tỉnh / TP nơi ở *:");
                                let provs = Self::cat_items(&self.catalogs, "province", None);
                                let prov_changed = search_combo(ui, "cb_prov", &mut self.form.matinh_cu_tru, &provs);
                                if prov_changed {
                                    if let Some(first_w) = self.catalogs.iter().find(|c| c.cat_type == "ward" && c.parent_code == self.form.matinh_cu_tru) {
                                        self.form.maxa_cu_tru = first_w.code.clone();
                                    }
                                }

                                ui.strong("10. Xã / Phường nơi ở *:");
                                let wards = Self::cat_items(&self.catalogs, "ward", Some(&self.form.matinh_cu_tru));
                                search_combo(ui, "cb_ward", &mut self.form.maxa_cu_tru, &wards);
                                ui.end_row();

                                ui.strong("11. Nơi ở hiện tại (số nhà, thôn/xóm):");
                                ui.text_edit_singleline(&mut self.form.dia_chi);

                                ui.label("12. Điện thoại:"); ui.text_edit_singleline(&mut self.form.dien_thoai);
                                ui.end_row();

                                ui.label("13. Nhóm máu:"); ui.text_edit_singleline(&mut self.form.nhom_mau);
                                ui.label("14. Mã Y Tế / Lượt khám:"); ui.text_edit_singleline(&mut self.form.ma_lk);
                                ui.end_row();

                                ui.strong("15. Ngày khám KSK *:");
                                date_input_picker(ui, "dp_ngay_vao", &mut self.form.ngay_vao);

                                ui.label("16. Đối tượng khám:");
                                let pris = Self::cat_items(&self.catalogs, "priority", None);
                                search_combo(ui, "cb_pri", &mut self.form.doi_tuong, &pris);
                                ui.end_row();

                                ui.label("17. Nguồn chi trả:");
                                let pays = Self::cat_items(&self.catalogs, "payment", None);
                                let mut pay_s = self.form.nguon_chi_tra.to_string();
                                if search_combo(ui, "cb_pay", &mut pay_s, &pays) {
                                    if let Ok(n) = pay_s.parse::<i32>() { self.form.nguon_chi_tra = n; }
                                }
                                ui.label("18. Loại hình KCB:"); ui.label("15 - KSK định kỳ");
                                ui.end_row();
                            });

                            ui.add_space(8.0);
                            ui.label("Nơi làm việc, học tập hiện tại:");
                            ui.add(egui::TextEdit::multiline(&mut self.form.noi_lam_viec_hoc_tap).desired_rows(2).desired_width(ui.available_width()));
                            ui.add_space(4.0);
                            ui.label("Lý do khám sức khỏe (LY_DO_VV):");
                            ui.add(egui::TextEdit::multiline(&mut self.form.ly_do_vv).desired_rows(2).desired_width(ui.available_width()));
                        });
                    }

                    // III. TIỀN SỬ BỆNH TẬT
                    1 => {
                        let enabled = self.can_edit_dept("hchinh");
                        ui.add_enabled_ui(enabled, |ui| {
                            // 1. TIỀN SỬ GIA ĐÌNH
                            ui.heading("1. Tiền sử gia đình:");
                            ui.horizontal(|ui| {
                                ui.strong("Có ai trong gia đình mắc một trong các bệnh: truyền nhiễm, tim mạch, ĐTĐ, lao, hen, ung thư, động kinh, tâm thần:");
                                Self::ui_radio_yn(ui, &mut self.form.tsgd_mac_benh);
                            });

                            if self.form.tsgd_mac_benh == 1 {
                                ui.add_space(4.0);
                                egui::Frame::group(ui.style()).fill(if self.dark_mode { egui::Color32::from_rgb(30, 41, 59) } else { egui::Color32::from_rgb(240, 249, 255) }).show(ui, |ui| {
                                    ui.strong("👉 Tiền sử bệnh, tật của gia đình (Mã ICD-10, chọn nhiều):");
                                    let icds = Self::cat_items(&self.catalogs, "icd10", None);
                                    multi_icd(ui, "icd_tsgd", &mut self.form.tsgd_ma_benh, &icds);
                                });
                            }

                            ui.separator();
                            ui.heading("2. Tiền sử bản thân:");
                            ui.horizontal(|ui| {
                                // CỘT 1 (STT 1 -> 11)
                                ui.vertical(|ui| {
                                    ui.set_width(ui.available_width() / 2.0 - 15.0);
                                    egui::Grid::new("grid_tsbt_col1").num_columns(3).spacing([8.0, 6.0]).show(ui, |ui| {
                                        ui.strong("STT"); ui.strong("Tiền sử bệnh lý"); ui.strong("Chọn"); ui.end_row();

                                        ui.label("1"); ui.label("Có bệnh hay bị thương trong 5 năm qua"); Self::ui_radio_yn(ui, &mut self.form.tsbt_benh_trong_5_nam_qua); ui.end_row();
                                        ui.label("2"); ui.label("Có bệnh thần kinh hay bị thương ở đầu"); Self::ui_radio_yn(ui, &mut self.form.tsbt_benh_than_kinh); ui.end_row();
                                        ui.label("3"); ui.label("Bệnh mắt hoặc giảm thị lực (trừ trường hợp đeo kính thuốc)"); Self::ui_radio_yn(ui, &mut self.form.tsbt_benh_mat); ui.end_row();
                                        ui.label("4"); ui.label("Bệnh ở tai, giảm sức nghe hoặc thăng bằng"); Self::ui_radio_yn(ui, &mut self.form.tsbt_benh_tai); ui.end_row();
                                        ui.label("5"); ui.label("Bệnh ở tim, hoặc nhồi máu cơ tim, các bệnh tim mạch khác"); Self::ui_radio_yn(ui, &mut self.form.tsbt_benh_tim); ui.end_row();
                                        ui.label("6"); ui.label("Phẫu thuật can thiệp tim - mạch (thay van, bắc cầu nối, tạo hình mạch, máy tạo nhịp tim, đặt stent, ghép tim)"); Self::ui_radio_yn(ui, &mut self.form.tsbt_phau_thuat_tim); ui.end_row();
                                        ui.label("7"); ui.label("Tăng huyết áp"); Self::ui_radio_yn(ui, &mut self.form.tsbt_tang_huyet_ap); ui.end_row();
                                        ui.label("8"); ui.label("Khó thở"); Self::ui_radio_yn(ui, &mut self.form.tsbt_kho_tho); ui.end_row();
                                        ui.label("9"); ui.label("Bệnh phổi, hen, khí phế thũng, viêm phế quản mạn tính"); Self::ui_radio_yn(ui, &mut self.form.tsbt_benh_phoi); ui.end_row();
                                        ui.label("10"); ui.label("Bệnh thận, lọc máu"); Self::ui_radio_yn(ui, &mut self.form.tsbt_benh_than); ui.end_row();
                                        ui.label("11"); ui.label("Nghiện rượu, bia"); Self::ui_radio_yn(ui, &mut self.form.tsbt_nghien_ruou); ui.end_row();
                                    });
                                });

                                ui.separator();

                                // CỘT 2 (STT 12 -> 22)
                                ui.vertical(|ui| {
                                    egui::Grid::new("grid_tsbt_col2").num_columns(3).spacing([8.0, 6.0]).show(ui, |ui| {
                                        ui.strong("STT"); ui.strong("Tiền sử bệnh lý"); ui.strong("Chọn"); ui.end_row();

                                        ui.label("12"); ui.label("Đái tháo đường hoặc kiểm soát tăng đường huyết"); Self::ui_radio_yn(ui, &mut self.form.tsbt_dai_thao_duong); ui.end_row();
                                        ui.label("13"); ui.label("Bệnh tâm thần"); Self::ui_radio_yn(ui, &mut self.form.tsbt_benh_tam_than); ui.end_row();
                                        ui.label("14"); ui.label("Mất ý thức, rối loạn ý thức"); Self::ui_radio_yn(ui, &mut self.form.tsbt_mat_y_thuc); ui.end_row();
                                        ui.label("15"); ui.label("Ngất, chóng mặt"); Self::ui_radio_yn(ui, &mut self.form.tsbt_ngat); ui.end_row();
                                        ui.label("16"); ui.label("Bệnh tiêu hóa"); Self::ui_radio_yn(ui, &mut self.form.tsbt_benh_tieu_hoa); ui.end_row();
                                        ui.label("17"); ui.label("Rối loạn giấc ngủ, ngừng thở khi ngủ, ngủ rũ ban ngày, ngáy to"); Self::ui_radio_yn(ui, &mut self.form.tsbt_roi_loan_giac_ngu); ui.end_row();
                                        ui.label("18"); ui.label("Tai biến mạch máu não hoặc liệt"); Self::ui_radio_yn(ui, &mut self.form.tsbt_tai_bien); ui.end_row();
                                        ui.label("19"); ui.label("Bệnh hoặc tổn thương cột sống"); Self::ui_radio_yn(ui, &mut self.form.tsbt_benh_cot_song); ui.end_row();
                                        ui.label("20"); ui.label("Sử dụng rượu thường xuyên, liên tục"); Self::ui_radio_yn(ui, &mut self.form.tsbt_ruou_thuong_xuyen); ui.end_row();
                                        ui.label("21"); ui.label("Sử dụng ma túy và chất gây nghiện"); Self::ui_radio_yn(ui, &mut self.form.tsbt_ma_tuy); ui.end_row();
                                        ui.label("22"); ui.label("Bệnh khác (ghi rõ)"); Self::ui_radio_yn(ui, &mut self.form.tsbt_benh_khac); ui.end_row();
                                    });

                                    if self.form.tsbt_benh_khac == 1 {
                                        ui.add_space(4.0);
                                        ui.horizontal(|ui| {
                                            ui.label("Tên bệnh khác:");
                                            ui.add(egui::TextEdit::singleline(&mut self.form.tsbt_ma_benh_khac).hint_text("Ghi cụ thể tên bệnh đang mắc hoặc đã mắc (nếu có)").desired_width(280.0));
                                        });
                                    }
                                });
                            });

                            ui.separator();
                            ui.heading("3. Các câu hỏi khác (nếu có):");

                            // a. Hiện tại có đang điều trị bệnh gì không?
                            ui.horizontal(|ui| {
                                ui.strong("a. Hiện tại có đang điều trị bệnh gì không?");
                                Self::ui_radio_yn(ui, &mut self.form.tsbt_dang_dieu_tri_benh);
                            });
                            if self.form.tsbt_dang_dieu_tri_benh == 1 {
                                egui::Grid::new("grid_dang_dt").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
                                    ui.label("Tên bệnh đang điều trị (Mã bệnh ICD-10):");
                                    let icds_dt = Self::cat_items(&self.catalogs, "icd10", None);
                                    multi_icd(ui, "icd_tsbt_dt", &mut self.form.tsbt_ma_benh, &icds_dt);
                                    ui.end_row();

                                    ui.label("Thuốc đang sử dụng và liều lượng:");
                                    ui.add(egui::TextEdit::singleline(&mut self.form.tsbt_ten_thuoc_lieu_luong).hint_text("Nhập tên thuốc đang dùng").desired_width(320.0));
                                    ui.end_row();
                                });
                            }

                            // b. Tiền sử thai sản (CHỈ HIỂN THỊ KHI LÀ NỮ: gioi_tinh == 2)
                            if self.form.gioi_tinh == 2 {
                                ui.add_space(6.0);
                                ui.horizontal(|ui| {
                                    ui.strong("b. Tiền sử thai sản:");
                                    Self::ui_radio_yn(ui, &mut self.form.tsbt_thai_san);
                                });
                                if self.form.tsbt_thai_san == 1 {
                                    egui::Grid::new("grid_thai_san").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
                                        ui.label("Cụ thể tên bệnh thai sản (Mã bệnh ICD-10):");
                                        let icds_ts = Self::cat_items(&self.catalogs, "icd10", None);
                                        multi_icd(ui, "icd_tsbt_ts", &mut self.form.tsbt_ma_benh_thai_san, &icds_ts);
                                        ui.end_row();

                                        ui.label("Thuốc đang sử dụng điều trị thai sản:");
                                        ui.add(egui::TextEdit::singleline(&mut self.form.tsbt_ten_thuoc_thai_san).hint_text("Nhập thuốc đang sử dụng").desired_width(320.0));
                                        ui.end_row();
                                    });
                                }
                            } else {
                                self.form.tsbt_thai_san = 0;
                            }
                        });
                    }

                    // 2: KHÁM BỆNH (GỒM THỂ LỰC, CÁC KHOA LÂM SÀNG VÀ CẬN LÂM SÀNG)
                    2 => {
                        ui.horizontal_wrapped(|ui| {
                            let sub_depts = [
                                (0, "🏃 Thể Lực", "theluc"),
                                (1, "🩺 Nội Khoa", "noi"),
                                (2, "🔪 Ngoại Khoa", "ngoai"),
                                (3, "🧴 Da Liễu", "dalieu"),
                                (4, "🌸 Sản Phụ Khoa", "san"),
                                (5, "👁 Mắt", "mat"),
                                (6, "👂 Tai Mũi Họng", "tmh"),
                                (7, "🦷 Răng Hàm Mặt", "rhm"),
                                (8, "🔬 Cận Lâm Sàng", "cls"),
                            ];
                            for (s_idx, s_title, s_role) in sub_depts {
                                let can_edit = self.can_edit_dept(s_role) || self.current_role == "admin";
                                let mut text = egui::RichText::new(s_title).size(14.0);
                                if can_edit {
                                    text = text.strong().color(egui::Color32::from_rgb(16, 185, 129));
                                }
                                if ui.selectable_label(self.kham_sub_tab == s_idx, text).clicked() {
                                    self.kham_sub_tab = s_idx;
                                }
                            }
                        });
                        ui.separator();

                        match self.kham_sub_tab {
                            // THỂ LỰC
                            0 => {
                                let enabled = self.can_edit_dept("theluc") || self.current_role == "admin";
                                ui.add_enabled_ui(enabled, |ui| {
                                    ui.heading("Khám Thể Lực & Phân Loại Chỉ Số Thể Trạng");
                                    ui.add_space(6.0);
                                    egui::Grid::new("grid_tl").num_columns(4).spacing([16.0, 10.0]).show(ui, |ui| {
                                        ui.strong("Chiều cao (cm):");
                                        if ui.text_edit_singleline(&mut self.form.chieu_cao).changed() {
                                            if let (Ok(h), Ok(w)) = (self.form.chieu_cao.parse::<f64>(), self.form.can_nang.parse::<f64>()) {
                                                if h > 0.0 { self.form.chi_so_bmi = format!("{:.2}", w / ((h/100.0)*(h/100.0))); }
                                            }
                                            self.update_global_health_score();
                                        }

                                        ui.strong("Cân nặng (kg):");
                                        if ui.text_edit_singleline(&mut self.form.can_nang).changed() {
                                            if let (Ok(h), Ok(w)) = (self.form.chieu_cao.parse::<f64>(), self.form.can_nang.parse::<f64>()) {
                                                if h > 0.0 { self.form.chi_so_bmi = format!("{:.2}", w / ((h/100.0)*(h/100.0))); }
                                            }
                                            self.update_global_health_score();
                                        }
                                        ui.end_row();

                                        ui.strong("Chỉ số BMI (Tự tính):");
                                        ui.label(egui::RichText::new(&self.form.chi_so_bmi).strong().color(egui::Color32::from_rgb(14, 165, 233)));

                                        ui.strong("Mạch (lần/phút):");
                                        if ui.text_edit_singleline(&mut self.form.mach).changed() {
                                            self.sync_specialty_result("CK_NOI_TIM_MACH");
                                            self.update_global_health_score();
                                        }
                                        ui.end_row();

                                        ui.strong("Huyết áp (mmHg):");
                                        if ui.text_edit_singleline(&mut self.form.huyet_ap).changed() {
                                            self.sync_specialty_result("CK_NOI_TIM_MACH");
                                            self.update_global_health_score();
                                        }

                                        ui.strong("Phân loại thể lực (1-5):");
                                        ui.horizontal(|ui| {
                                            for i in 1..=5 {
                                                if ui.radio_value(&mut self.form.kham_the_luc_pl, i, format!("Loại {}", i)).clicked() {
                                                    self.update_global_health_score();
                                                }
                                            }
                                        });
                                        ui.end_row();
                                    });
                                });
                            }

                            // NỘI KHOA (GỒM 8 CHUYÊN KHOA CON THEO QĐ 1613/BYT)
                            1 => {
                                let enabled = self.can_edit_dept("noi") || self.current_role == "admin";
                                ui.add_enabled_ui(enabled, |ui| {
                                    let noi_subtabs = [
                                        (0, "CK_NOI_TIM_MACH", "❤️ Tim Mạch"),
                                        (1, "CK_NOI_HO_HAP", "🫁 Hô Hấp"),
                                        (2, "CK_NOI_TIEU_HOA", "🥣 Tiêu Hóa"),
                                        (3, "CK_NOI_TIET_NIEU_SD", "💧 Thận - TN - SD"),
                                        (4, "CK_NOI_THAN_KINH", "🧠 Thần Kinh"),
                                        (5, "CK_NOI_TAM_THAN", "🧘 Tâm Thần"),
                                        (6, "CK_NOI_NOITIET", "🧪 Nội Tiết"),
                                        (7, "CK_NOI_CO_XUONG_KHOP", "🦴 Cơ Xương Khớp"),
                                    ];

                                    ui.horizontal_wrapped(|ui| {
                                        for (idx, _, label) in &noi_subtabs {
                                            if ui.selectable_label(self.noi_khoa_sub_tab == *idx, *label).clicked() {
                                                self.noi_khoa_sub_tab = *idx;
                                            }
                                        }
                                    });
                                    ui.separator();

                                    let (_, cur_spec_id, _) = noi_subtabs[self.noi_khoa_sub_tab];
                                    self.render_qd1613_specialty(ui, cur_spec_id, Some("Nội khoa"));
                                });
                            }

                            // NGOẠI KHOA
                            2 => {
                                let enabled = self.can_edit_dept("ngoai") || self.current_role == "admin";
                                ui.add_enabled_ui(enabled, |ui| {
                                    self.render_qd1613_specialty(ui, "CK_NGOAI", None);
                                });
                            }

                            // DA LIỄU
                            3 => {
                                let enabled = self.can_edit_dept("dalieu") || self.current_role == "admin";
                                ui.add_enabled_ui(enabled, |ui| {
                                    self.render_qd1613_specialty(ui, "CK_DA_LIEU", None);
                                });
                            }

                            // SẢN PHỤ KHOA
                            4 => {
                                let enabled = self.can_edit_dept("san") || self.current_role == "admin";
                                ui.add_enabled_ui(enabled, |ui| {
                                    self.render_qd1613_specialty(ui, "CK_PHU_SAN", None);
                                });
                            }

                            // MẮT
                            5 => {
                                let enabled = self.can_edit_dept("mat") || self.current_role == "admin";
                                ui.add_enabled_ui(enabled, |ui| {
                                    self.render_qd1613_specialty(ui, "CK_MAT", None);
                                });
                            }

                            // TAI MŨI HỌNG
                            6 => {
                                let enabled = self.can_edit_dept("tmh") || self.current_role == "admin";
                                ui.add_enabled_ui(enabled, |ui| {
                                    self.render_qd1613_specialty(ui, "CK_TMH", None);
                                });
                            }

                            // RĂNG HÀM MẶT
                            7 => {
                                let enabled = self.can_edit_dept("rhm") || self.current_role == "admin";
                                ui.add_enabled_ui(enabled, |ui| {
                                    self.render_qd1613_specialty(ui, "CK_RHM", None);
                                });
                            }

                            // CẬN LÂM SÀNG
                            8 => {
                                let enabled = self.can_edit_dept("cls") || self.current_role == "admin";
                                ui.add_enabled_ui(enabled, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.heading("Khám Cận Lâm Sàng & Kết Quả Xét Nghiệm / CĐHA");
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui.button("➕ Thêm Dòng Chỉ Số").clicked() {
                                                self.form.cls_list.push(ClsItem::default());
                                            }
                                        });
                                    });
                                    ui.separator();

                                    let mut del_idx = None;
                                    egui::Grid::new("grid_cls_table").striped(true).min_col_width(80.0).spacing([8.0, 6.0]).show(ui, |ui| {
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
                            _ => {}
                        }
                    }

                    // 3: KẾT LUẬN TOÀN DIỆN
                    3 => {
                        let enabled = self.can_edit_dept("ketluan") || self.current_role == "admin";
                        ui.add_enabled_ui(enabled, |ui| {
                            ui.heading("Kết Luận Phân Loại Sức Khỏe Toàn Diện");
                            ui.add_space(6.0);

                            self.update_global_health_score();

                            egui::Frame::group(ui.style())
                                .fill(if self.dark_mode { egui::Color32::from_rgb(30, 41, 59) } else { egui::Color32::from_rgb(240, 249, 255) })
                                .stroke(egui::Stroke::new(2.0_f32, egui::Color32::from_rgb(56, 189, 248)))
                                .inner_margin(16.0)
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.strong(egui::RichText::new("⚖ PHÂN LOẠI SỨC KHỎE TỔNG HỢP:").size(15.0));
                                        ui_score_badge(ui, self.form.phan_loai_sk, false);
                                        ui.strong(format!("Tự động tính từ các chuyên khoa: LOẠI {}", self.form.phan_loai_sk));
                                    });

                                    ui.add_space(8.0);
                                    ui.horizontal(|ui| {
                                        ui.label("Chọn mức phân loại sức khỏe chung:");
                                        let pl_options = [
                                            (1, "Loại I (Rất khỏe)"),
                                            (2, "Loại II (Khỏe)"),
                                            (3, "Loại III (Trung bình)"),
                                            (4, "Loại IV (Yếu)"),
                                            (5, "Loại V (Rất yếu)"),
                                        ];
                                        for (pl_num, pl_text) in pl_options {
                                            ui.radio_value(&mut self.form.phan_loai_sk, pl_num, pl_text);
                                        }
                                    });
                                });

                            ui.add_space(10.0);
                            ui.strong("BẢNG TỔNG HỢP KẾT LUẬN CỦA TẤT CẢ CÁC CHUYÊN KHOA:");
                            ui.add_space(4.0);

                            let summary_list = [
                                ("Thể lực", &self.form.chi_so_bmi, self.form.kham_the_luc_pl),
                                ("Tuần hoàn", &self.form.noi_khoa_tuan_hoan, self.form.noi_khoa_tuan_hoan_pl),
                                ("Hô hấp", &self.form.noi_khoa_ho_hap, self.form.noi_khoa_ho_hap_pl),
                                ("Tiêu hóa", &self.form.noi_khoa_tieu_hoa, self.form.noi_khoa_tieu_hoa_pl),
                                ("Thận - Tiết niệu - SD", &self.form.noi_khoa_than_tn_sd, self.form.noi_khoa_than_tn_sd_pl),
                                ("Nội tiết", &self.form.noi_khoa_noi_tiet, self.form.noi_khoa_noi_tiet_pl),
                                ("Cơ - Xương - Khớp", &self.form.noi_khoa_co_xuong_khop, self.form.noi_khoa_co_xuong_khop_pl),
                                ("Thần kinh", &self.form.noi_khoa_than_kinh, self.form.noi_khoa_than_kinh_pl),
                                ("Tâm thần", &self.form.noi_khoa_tam_than, self.form.noi_khoa_tam_than_pl),
                                ("Ngoại khoa", &self.form.ket_qua_kham_ngoai_khoa, self.form.kham_ngoai_khoa_pl),
                                ("Da liễu", &self.form.ket_qua_kham_da_lieu, self.form.kham_da_lieu_pl),
                                ("Sản phụ khoa", &self.form.ket_qua_kham_san_phu_khoa, self.form.kham_san_phu_khoa_pl),
                                ("Mắt", &self.form.benh_khac_mat, self.form.kham_mat_pl),
                                ("Tai Mũi Họng", &self.form.benh_khac_tai_mui_hong, self.form.kham_tai_mui_hong_pl),
                                ("Răng Hàm Mặt", &self.form.benh_khac_rang_ham_mat, self.form.kham_rang_ham_mat_pl),
                            ];

                            egui::Grid::new("grid_summary_all_specs").striped(true).min_col_width(120.0).spacing([14.0, 8.0]).show(ui, |ui| {
                                ui.strong("STT"); ui.strong("Chuyên Khoa"); ui.strong("Phân Loại"); ui.strong("Kết Luận / Bệnh Lý Ghi Nhận"); ui.end_row();
                                for (idx, (s_name, s_concl, s_sc)) in summary_list.iter().enumerate() {
                                    ui.label((idx + 1).to_string());
                                    ui.strong(*s_name);
                                    ui_score_badge(ui, *s_sc, false);
                                    if *s_sc > 1 {
                                        ui.colored_label(egui::Color32::from_rgb(220, 38, 38), egui::RichText::new(*s_concl).strong());
                                    } else {
                                        ui.label(*s_concl);
                                    }
                                    ui.end_row();
                                }
                            });

                            ui.add_space(12.0);
                            ui.strong("Kết luận tình trạng bệnh / Đủ sức khỏe (KET_LUAN_BENH):");
                            ui.add(egui::TextEdit::singleline(&mut self.form.ket_luan_benh).desired_width(ui.available_width()));

                            ui.add_space(6.0);
                            ui.strong("Các bệnh tật (nếu có) / Hướng điều trị tiếp theo (CAC_BENH_TAT_NEU_CO):");
                            ui.add(egui::TextEdit::multiline(&mut self.form.cac_benh_tat_neu_co).desired_rows(3).desired_width(ui.available_width()));
                        });
                    }
                    _ => {}
                }
            });
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([1024.0, 680.0])
            .with_title("Hệ Thống Khám Sức Khỏe Định Kỳ - QĐ 3176/QĐ-BYT & QĐ 1613/BYT"),
        ..Default::default()
    };
    eframe::run_native(
        "Hệ Thống Khám Sức Khỏe Định Kỳ",
        native_options,
        Box::new(|cc| Box::new(KskApp::new(cc))),
    )
}
