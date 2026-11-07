use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct CompanyRecord {
    pub id: i64,
    pub ma_cong_ty: String,
    pub ten_cong_ty: String,
    pub ma_so_thue: String,
    pub dia_chi: String,
    pub nguoi_lien_he: String,
    pub dien_thoai: String,
    pub ghi_chu: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ContractRecord {
    pub id: i64,
    pub company_id: i64,
    pub so_hop_dong: String,
    pub ten_dot_kham: String,
    pub ngay_ky: String,
    pub ngay_bat_dau: String,
    pub ngay_ket_thuc: String,
    pub so_luong_du_kien: i32,
    pub doi_tuong_mac_dinh: String,
    pub nguon_chi_tra_mac_dinh: i32,
    pub trang_thai: String, // "Đang chuẩn bị", "Đang khám", "Đã hoàn thành"
    pub ghi_chu: String,
}

impl Default for ContractRecord {
    fn default() -> Self {
        let now = chrono::Local::now().format("%d/%m/%Y").to_string();
        Self {
            id: 0,
            company_id: 0,
            so_hop_dong: "".into(),
            ten_dot_kham: "".into(),
            ngay_ky: now.clone(),
            ngay_bat_dau: now.clone(),
            ngay_ket_thuc: now,
            so_luong_du_kien: 50,
            doi_tuong_mac_dinh: "13".into(), // Người lao động
            nguon_chi_tra_mac_dinh: 4,       // Người sử dụng lao động
            trang_thai: "Đang khám".into(),
            ghi_chu: "".into(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AttachmentRecord {
    pub id: i64,
    pub owner_type: String, // "contract" | "record"
    pub owner_id: i64,
    pub file_name: String,
    pub file_path: String,
    pub doc_type: String, // "Hợp đồng", "Phiếu khám Mẫu 03", "Kết quả CLS", "Tài liệu khác"
    pub uploaded_at: String,
}
