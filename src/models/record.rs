use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ManualConclusionRow {
    pub classification: i32,
    pub classification_id: String,
    pub manifestation: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SpecialtyConclusionExtra {
    pub manual_rows: Vec<ManualConclusionRow>,
    pub auto_overrides: HashMap<String, String>, // disease_id -> custom manifestation text
    pub final_note: String,
}

// 1 Dòng Cận Lâm Sàng (Mục VI & Sheet DM_CanLamSang)
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct ClsItem {
    pub ma_dich_vu: String,
    pub ma_chi_so: String,
    pub gia_tri: String,
    pub don_vi_do: String,
    pub mo_ta: String,
    pub ket_luan: String,
}

// CẤU TRÚC ĐẦY ĐỦ 100% CÁC TRƯỜNG DỮ LIỆU THEO QĐ 3176/QĐ-BYT & MẪU 03
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct KskRecordPayload {
    // 0. Quản lý Hồ sơ, Đoàn khám, Mã Y Tế (QR) & Số Lưu Trữ (Số GKSK)
    #[serde(default)]
    pub contract_id: Option<i64>,
    #[serde(default)]
    pub ma_y_te: String, // Mã định danh trên QR Code (sinh ngay khi tạo/import)
    #[serde(default)]
    pub so_luu_tru: String, // Số: ....../GKSK-... (chỉ cấp khi Kết luận sau khám)
    #[serde(default)]
    pub bo_phan: String, // Phòng ban / Phân xưởng (nếu khám đoàn)

    // I. Hành chính
    pub ho_ten: String,
    pub gioi_tinh: i32, // 1: Nam, 2: Nữ, 3: Chưa xác định
    pub ngay_sinh: String,
    pub ma_dan_toc: String,
    pub so_cccd: String,
    pub ngaycap_cccd: String,
    pub noicap_cccd: String,
    pub dia_chi: String,
    pub matinh_cu_tru: String,
    pub maxa_cu_tru: String,
    pub ma_nghe_nghiep: String,
    pub noi_lam_viec_hoc_tap: String,
    pub dien_thoai: String,
    pub nhom_mau: String,
    pub ly_do_vv: String,

    // II. Thông tin chung lần khám
    pub ma_lk: String,
    pub ma_cskcb: String,
    pub ma_gtin_cskcb: String,
    pub doi_tuong: String, // Mã 1 -> 16
    pub nguon_chi_tra: i32, // 1, 2, 3, 4, 5, 9
    pub ma_loai_kcb: String, // Theo QĐ 1804 (15: KSK định kỳ)
    pub ngay_vao: String,

    // III. Tiền sử bệnh tật
    // A. Gia đình
    pub tsgd_mac_benh: i32, // 0: Không, 1: Có
    pub tsgd_ma_benh: String, // Mã ICD-10 hoặc tên bệnh phân cách bằng ;
    // B. Bản thân (22 chỉ số)
    pub tsbt_benh_trong_5_nam_qua: i32,
    pub tsbt_benh_than_kinh: i32,
    pub tsbt_benh_mat: i32,
    pub tsbt_benh_tai: i32,
    pub tsbt_benh_tim: i32,
    pub tsbt_phau_thuat_tim: i32,
    pub tsbt_tang_huyet_ap: i32,
    pub tsbt_kho_tho: i32,
    pub tsbt_benh_phoi: i32,
    pub tsbt_benh_than: i32,
    pub tsbt_nghien_ruou: i32,
    pub tsbt_dai_thao_duong: i32,
    pub tsbt_benh_tam_than: i32,
    pub tsbt_mat_y_thuc: i32,
    pub tsbt_ngat: i32,
    pub tsbt_benh_tieu_hoa: i32,
    pub tsbt_roi_loan_giac_ngu: i32,
    pub tsbt_tai_bien: i32,
    pub tsbt_benh_cot_song: i32,
    pub tsbt_ruou_thuong_xuyen: i32,
    pub tsbt_ma_tuy: i32,
    pub tsbt_benh_khac: i32,
    pub tsbt_ma_benh_khac: String,
    pub tsbt_dang_dieu_tri_benh: i32,
    pub tsbt_ma_benh: String,
    pub tsbt_ten_thuoc_lieu_luong: String,
    pub tsbt_thai_san: i32,
    pub tsbt_ma_benh_thai_san: String,
    pub tsbt_ten_thuoc_thai_san: String,

    // IV. Khám thể lực
    pub chieu_cao: String,
    pub can_nang: String,
    pub chi_so_bmi: String,
    pub mach: String,
    pub huyet_ap: String,
    pub kham_the_luc_pl: i32,

    // V. Khám lâm sàng
    // A. Nội khoa
    pub noi_khoa_tuan_hoan: String,
    pub noi_khoa_tuan_hoan_pl: i32,
    pub noi_khoa_ho_hap: String,
    pub noi_khoa_ho_hap_pl: i32,
    pub noi_khoa_tieu_hoa: String,
    pub noi_khoa_tieu_hoa_pl: i32,
    pub noi_khoa_than_tn_sd: String,
    pub noi_khoa_than_tn_sd_pl: i32, // Mã biến Excel: NOI_KHOA_THAN_TIETNIEU_PL
    pub noi_khoa_noi_tiet: String,
    pub noi_khoa_noi_tiet_pl: i32,
    pub noi_khoa_co_xuong_khop: String,
    pub noi_khoa_co_xuong_khop_pl: i32,
    pub noi_khoa_than_kinh: String,
    pub noi_khoa_than_kinh_pl: i32,
    pub noi_khoa_tam_than: String,
    pub noi_khoa_tam_than_pl: i32,
    // B. Ngoại khoa
    pub ket_qua_kham_ngoai_khoa: String,
    pub kham_ngoai_khoa_pl: i32,
    // C. Da liễu
    pub ket_qua_kham_da_lieu: String,
    pub kham_da_lieu_pl: i32,
    // D. Sản phụ khoa
    pub ket_qua_kham_san_phu_khoa: String,
    pub kham_san_phu_khoa_pl: i32,
    // E. Mắt
    pub khong_kinh_mat_phai: String,
    pub khong_kinh_mat_trai: String,
    pub co_kinh_mat_phai: String,
    pub co_kinh_mat_trai: String,
    pub benh_khac_mat: String,
    pub kham_mat_pl: i32,
    // F. Tai Mũi Họng
    pub tai_trai_noi_thuong: String,
    pub tai_trai_noi_tham: String,
    pub tai_phai_noi_thuong: String,
    pub tai_phai_noi_tham: String,
    pub benh_khac_tai_mui_hong: String,
    pub kham_tai_mui_hong_pl: i32,
    // G. Răng Hàm Mặt
    pub ham_tren: String,
    pub ham_duoi: String,
    pub benh_khac_rang_ham_mat: String,
    pub kham_rang_ham_mat_pl: i32,

    // VI. Khám cận lâm sàng (Các trường nhanh theo Mẫu 03 + Danh sách chi tiết DM_CanLamSang)
    #[serde(default)]
    pub cls_huyet_hoc: String,
    #[serde(default)]
    pub cls_duong_mau: String,
    #[serde(default)]
    pub cls_ure: String,
    #[serde(default)]
    pub cls_creatinin: String,
    #[serde(default)]
    pub cls_asat_got: String,
    #[serde(default)]
    pub cls_alat_gpt: String,
    #[serde(default)]
    pub cls_nuoc_tieu: String,
    #[serde(default)]
    pub cls_xq_tim_phoi: String,
    #[serde(default)]
    pub cls_khac_co: i32,
    #[serde(default)]
    pub cls_khac_mo_ta: String,
    #[serde(default)]
    pub cls_dinh_ky_ket_qua: String,
    pub cls_list: Vec<ClsItem>,

    // VII. Kết luận
    pub phan_loai_sk: i32,
    pub ket_luan_benh: String,
    pub cac_benh_tat_neu_co: String,

    // Lựa chọn động theo QĐ 1613/BYT
    #[serde(default)]
    pub selected_items: HashSet<String>,
    #[serde(default)]
    pub selected_classifications: HashMap<String, String>,
    #[serde(default)]
    pub specialty_extras: HashMap<String, SpecialtyConclusionExtra>,
    #[serde(default)]
    pub dominant_hand: String,
    #[serde(default)]
    pub voice_hearing: i32,
    #[serde(default)]
    pub audiometer_hearing: i32,
}

impl KskRecordPayload {
    /// Đồng bộ các chỉ số nhanh trên Mẫu 03 sang danh sách `cls_list` (Sheet `DM_CanLamSang`)
    pub fn sync_quick_cls_to_list(&mut self) {
        let quick_defs = [
            ("XN_HH", "Tổng phân tích tế bào máu ngoại vi", &self.cls_huyet_hoc, ""),
            ("S01", "Đường máu (Glucose)", &self.cls_duong_mau, "mmol/l"),
            ("S02", "Urê máu", &self.cls_ure, "mmol/l"),
            ("S03", "Creatinin máu", &self.cls_creatinin, "µmol/l"),
            ("S04", "ASAT (GOT)", &self.cls_asat_got, "U/L"),
            ("S05", "ALAT (GPT)", &self.cls_alat_gpt, "U/L"),
            ("XN_NT", "Tổng phân tích nước tiểu", &self.cls_nuoc_tieu, ""),
            ("CDHA_XQ", "X-Quang tim phổi thẳng", &self.cls_xq_tim_phoi, ""),
        ];

        for (code, name, val, unit) in quick_defs {
            let trimmed = val.trim();
            if let Some(existing) = self.cls_list.iter_mut().find(|c| c.ma_dich_vu == code || c.ma_chi_so == name) {
                if !trimmed.is_empty() {
                    existing.gia_tri = trimmed.to_string();
                }
            } else if !trimmed.is_empty() {
                self.cls_list.push(ClsItem {
                    ma_dich_vu: code.to_string(),
                    ma_chi_so: name.to_string(),
                    gia_tri: trimmed.to_string(),
                    don_vi_do: unit.to_string(),
                    mo_ta: trimmed.to_string(),
                    ket_luan: "Bình thường".to_string(),
                });
            }
        }
    }

    /// Đặt toàn bộ kết quả khám Lâm sàng & Thể lực về Mặc định Bình thường (Loại I) để nhập 1 cửa siêu tốc
    pub fn reset_clinical_to_normal(&mut self) {
        if self.chieu_cao.trim().is_empty() { self.chieu_cao = "165".into(); }
        if self.can_nang.trim().is_empty() { self.can_nang = "60".into(); }
        if self.chi_so_bmi.trim().is_empty() { self.chi_so_bmi = "22.04".into(); }
        if self.mach.trim().is_empty() { self.mach = "75".into(); }
        if self.huyet_ap.trim().is_empty() { self.huyet_ap = "120/80".into(); }
        self.kham_the_luc_pl = 1;

        self.noi_khoa_tuan_hoan = "Bình thường".into();
        self.noi_khoa_tuan_hoan_pl = 1;
        self.noi_khoa_ho_hap = "Bình thường".into();
        self.noi_khoa_ho_hap_pl = 1;
        self.noi_khoa_tieu_hoa = "Bình thường".into();
        self.noi_khoa_tieu_hoa_pl = 1;
        self.noi_khoa_than_tn_sd = "Bình thường".into();
        self.noi_khoa_than_tn_sd_pl = 1;
        self.noi_khoa_noi_tiet = "Bình thường".into();
        self.noi_khoa_noi_tiet_pl = 1;
        self.noi_khoa_co_xuong_khop = "Bình thường".into();
        self.noi_khoa_co_xuong_khop_pl = 1;
        self.noi_khoa_than_kinh = "Bình thường".into();
        self.noi_khoa_than_kinh_pl = 1;
        self.noi_khoa_tam_than = "Bình thường".into();
        self.noi_khoa_tam_than_pl = 1;

        self.ket_qua_kham_ngoai_khoa = "Bình thường".into();
        self.kham_ngoai_khoa_pl = 1;
        self.ket_qua_kham_da_lieu = "Bình thường".into();
        self.kham_da_lieu_pl = 1;
        self.ket_qua_kham_san_phu_khoa = "Bình thường".into();
        self.kham_san_phu_khoa_pl = 1;

        self.khong_kinh_mat_phai = "10/10".into();
        self.khong_kinh_mat_trai = "10/10".into();
        self.benh_khac_mat = "Bình thường".into();
        self.kham_mat_pl = 1;

        self.tai_trai_noi_thuong = "5".into();
        self.tai_trai_noi_tham = "0.5".into();
        self.tai_phai_noi_thuong = "5".into();
        self.tai_phai_noi_tham = "0.5".into();
        self.benh_khac_tai_mui_hong = "Bình thường".into();
        self.kham_tai_mui_hong_pl = 1;

        self.ham_tren = "Bình thường".into();
        self.ham_duoi = "Bình thường".into();
        self.benh_khac_rang_ham_mat = "Bình thường".into();
        self.kham_rang_ham_mat_pl = 1;

        if self.cls_huyet_hoc.trim().is_empty() { self.cls_huyet_hoc = "Bình thường".into(); }
        if self.cls_nuoc_tieu.trim().is_empty() { self.cls_nuoc_tieu = "Bình thường".into(); }
        if self.cls_xq_tim_phoi.trim().is_empty() { self.cls_xq_tim_phoi = "Tim phổi bình thường".into(); }

        self.phan_loai_sk = 1;
        self.ket_luan_benh = "Đủ sức khỏe làm việc".into();
        self.cac_benh_tat_neu_co = "Không".into();
    }
}

impl Default for KskRecordPayload {
    fn default() -> Self {
        let now = chrono::Local::now();
        let ts = now.format("%Y%m%d%H%M%S").to_string();
        Self {
            contract_id: None,
            ma_y_te: format!("YT{}", ts),
            so_luu_tru: "".into(), // Chưa cấp số lưu trữ khi mới tạo
            bo_phan: "".into(),
            ho_ten: "".into(),
            gioi_tinh: 1, // Nam
            ngay_sinh: "".into(),
            ma_dan_toc: "01".into(),
            so_cccd: "".into(),
            ngaycap_cccd: "".into(),
            noicap_cccd: "Cục Cảnh sát QLHCVTTXH".into(),
            dia_chi: "".into(),
            matinh_cu_tru: "52".into(),
            maxa_cu_tru: "21550".into(),
            ma_nghe_nghiep: "22".into(),
            noi_lam_viec_hoc_tap: "".into(),
            dien_thoai: "".into(),
            nhom_mau: "O".into(),
            ly_do_vv: "Khám sức khỏe định kỳ".into(),
            ma_lk: format!("LK{}", ts),
            ma_cskcb: "52208".into(),
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
            khong_kinh_mat_phai: "10/10".into(),
            khong_kinh_mat_trai: "10/10".into(),
            co_kinh_mat_phai: "".into(),
            co_kinh_mat_trai: "".into(),
            benh_khac_mat: "Bình thường".into(),
            kham_mat_pl: 1,
            tai_trai_noi_thuong: "5".into(),
            tai_trai_noi_tham: "0.5".into(),
            tai_phai_noi_thuong: "5".into(),
            tai_phai_noi_tham: "0.5".into(),
            benh_khac_tai_mui_hong: "Bình thường".into(),
            kham_tai_mui_hong_pl: 1,
            ham_tren: "Bình thường".into(),
            ham_duoi: "Bình thường".into(),
            benh_khac_rang_ham_mat: "Bình thường".into(),
            kham_rang_ham_mat_pl: 1,
            cls_huyet_hoc: "Bình thường".into(),
            cls_duong_mau: "".into(),
            cls_ure: "".into(),
            cls_creatinin: "".into(),
            cls_asat_got: "".into(),
            cls_alat_gpt: "".into(),
            cls_nuoc_tieu: "Bình thường".into(),
            cls_xq_tim_phoi: "Tim phổi bình thường".into(),
            cls_khac_co: 0,
            cls_khac_mo_ta: "".into(),
            cls_dinh_ky_ket_qua: "".into(),
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

#[derive(Clone, Debug)]
pub struct RecordItem {
    pub id: i64,
    pub ho_ten: String,
    pub so_cccd: String,
    pub ngay_vao: String,
    pub status: i32, // 0: Đang soạn (Chưa kết luận), 1: Đã kết luận & Cấp số lưu trữ
    pub contract_id: Option<i64>,
    pub ma_y_te: String,
    pub so_luu_tru: String,
    pub data: KskRecordPayload,
}
