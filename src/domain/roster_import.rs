use std::fs;
use std::path::Path;
use crate::config::AppConfig;
use crate::models::{CompanyRecord, ContractRecord, KskRecordPayload, RecordItem};

/// Xuất tệp mẫu CSV danh sách nhân viên khám đoàn.
/// CHỈ gồm các trường hành chính của đối tượng khám.
/// Các trường mặc định (mã CSKCB, nơi làm việc/công ty, đối tượng, nguồn chi trả, ngày khám...)
/// và tiền sử bệnh không cần đưa vào file import vì được hệ thống tự động thiết lập.
pub fn export_roster_template_csv(output_path: &Path) -> Result<(), String> {
    let mut wtr = csv::WriterBuilder::new().from_writer(vec![]);
    wtr.write_record([
        "STT",
        "HO_TEN",
        "GIOI_TINH",
        "NGAY_SINH",
        "SO_CCCD",
        "NGAYCAP_CCCD",
        "NOICAP_CCCD",
        "DIEN_THOAI",
        "DIA_CHI",
        "MATINH_CU_TRU",
        "MAXA_CU_TRU",
        "MA_DAN_TOC",
        "MA_NGHE_NGHIEP",
        "BO_PHAN",
        "NHOM_MAU",
    ])
    .map_err(|e| e.to_string())?;

    wtr.write_record([
        "1",
        "NGUYỄN VĂN AN",
        "Nam",
        "15/05/1990",
        "052090001234",
        "10/06/2021",
        "Cục Cảnh sát QLHCVTTXH",
        "0905123456",
        "123 Nguyễn Tất Thành",
        "52",
        "21550",
        "01",
        "22",
        "Phòng Kỹ Thuật",
        "O",
    ])
    .map_err(|e| e.to_string())?;

    wtr.write_record([
        "2",
        "TRẦN THỊ BÍCH",
        "Nữ",
        "20/10/1995",
        "052195005678",
        "15/12/2022",
        "Cục Cảnh sát QLHCVTTXH",
        "0912345678",
        "45 Lê Duẩn",
        "52",
        "21550",
        "01",
        "52",
        "Phòng Kế Toán",
        "A",
    ])
    .map_err(|e| e.to_string())?;

    let bytes = wtr.into_inner().map_err(|e| e.to_string())?;
    // Prepend UTF-8 BOM for Excel compatibility
    let mut out = vec![0xEF, 0xBB, 0xBF];
    out.extend_from_slice(&bytes);
    fs::write(output_path, out).map_err(|e| e.to_string())
}

/// Import danh sách đoàn khám từ CSV:
/// - Chỉ import các trường thuộc phần hành chính.
/// - Các dữ liệu mặc định:
///   + Mã CSKCB, Mã GTIN CSKCB: lấy từ cài đặt hệ thống (config)
///   + Nơi làm việc, học tập: tự động gán tên Công ty thuộc Hợp đồng
///   + Ngày khám: lấy từ ngày bắt đầu đợt khám của Hợp đồng
///   + Đối tượng, nguồn chi trả: lấy từ Hợp đồng
/// - KHÔNG import tiền sử bệnh (tiền sử để mặc định trống/chưa bệnh để bác sĩ/người khám ghi nhận).
pub fn import_roster_from_csv(
    csv_path: &Path,
    company: &CompanyRecord,
    contract: &ContractRecord,
    config: &AppConfig,
    existing_count: usize,
) -> Result<Vec<RecordItem>, String> {
    let raw = fs::read_to_string(csv_path).map_err(|e| e.to_string())?;
    let clean = raw.trim_start_matches('\u{feff}');
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(clean.as_bytes());

    let mut out = Vec::new();
    let now_date = chrono::Local::now().format("%d/%m/%Y").to_string();

    for (_idx, result) in rdr.records().enumerate() {
        let row = result.map_err(|e| e.to_string())?;
        let ho_ten = row.get(1).unwrap_or("").trim().to_uppercase();
        if ho_ten.is_empty() {
            continue;
        }

        let seq = existing_count + out.len() + 1;
        let mut p = KskRecordPayload::default();
        p.contract_id = Some(contract.id);
        p.ma_y_te = format!("HD{:02}-{:04}", contract.id.max(1), seq);
        p.so_luu_tru = "".into(); // Chưa cấp số lưu trữ trước khi khám
        p.ma_lk = format!("LK{}-{:04}", contract.id.max(1), seq);

        // Các dữ liệu cố định / mặc định từ cấu hình hệ thống & hợp đồng
        p.ma_cskcb = config.default_cskcb.clone();
        p.ma_gtin_cskcb = config.default_gtin_cskcb.clone();
        p.ngay_vao = if !contract.ngay_bat_dau.trim().is_empty() {
            contract.ngay_bat_dau.clone()
        } else {
            now_date.clone()
        };
        p.noi_lam_viec_hoc_tap = company.ten_cong_ty.clone();
        p.doi_tuong = contract.doi_tuong_mac_dinh.clone();
        p.nguon_chi_tra = contract.nguon_chi_tra_mac_dinh;
        p.ly_do_vv = "Khám sức khỏe định kỳ".into();

        // 1. Họ tên
        p.ho_ten = ho_ten.clone();

        // 2. Giới tính (1: Nam, 2: Nữ)
        let gt_str = row.get(2).unwrap_or("1").trim().to_lowercase();
        p.gioi_tinh = if gt_str == "2" || gt_str.contains("nữ") || gt_str.contains("nu") {
            2
        } else {
            1
        };

        // 3. Ngày sinh (DD/MM/YYYY)
        p.ngay_sinh = row.get(3).unwrap_or("").trim().to_string();

        // 4. Số CCCD
        p.so_cccd = row.get(4).unwrap_or("").trim().to_string();

        // 5. Ngày cấp & Nơi cấp CCCD
        p.ngaycap_cccd = row.get(5).unwrap_or("").trim().to_string();
        let noicap = row.get(6).unwrap_or("").trim();
        if !noicap.is_empty() {
            p.noicap_cccd = noicap.to_string();
        }

        // 6. Điện thoại
        p.dien_thoai = row.get(7).unwrap_or("").trim().to_string();

        // 7. Địa chỉ nơi ở hiện tại
        p.dia_chi = row.get(8).unwrap_or("").trim().to_string();

        // 8. Tỉnh / Xã cư trú
        let matinh = row.get(9).unwrap_or("").trim();
        p.matinh_cu_tru = if matinh.is_empty() {
            config.default_province_code.clone()
        } else {
            matinh.to_string()
        };
        let maxa = row.get(10).unwrap_or("").trim();
        p.maxa_cu_tru = if maxa.is_empty() {
            config.default_ward_code.clone()
        } else {
            maxa.to_string()
        };

        // 9. Dân tộc
        let dantoc = row.get(11).unwrap_or("").trim();
        if !dantoc.is_empty() {
            p.ma_dan_toc = dantoc.to_string();
        }

        // 10. Nghề nghiệp
        let nghenghiep = row.get(12).unwrap_or("").trim();
        if !nghenghiep.is_empty() {
            p.ma_nghe_nghiep = nghenghiep.to_string();
        }

        // 11. Bộ phận / Phân xưởng
        p.bo_phan = row.get(13).unwrap_or("").trim().to_string();

        // 12. Nhóm máu (nếu có)
        let nhommau = row.get(14).unwrap_or("").trim();
        if !nhommau.is_empty() {
            p.nhom_mau = nhommau.to_string();
        }

        // Tiền sử bệnh: để nguyên mặc định (không import tiền sử)
        p.tsgd_mac_benh = 0;
        p.tsgd_ma_benh.clear();
        p.tsbt_benh_trong_5_nam_qua = 0;
        p.tsbt_benh_than_kinh = 0;
        p.tsbt_benh_mat = 0;
        p.tsbt_benh_tai = 0;
        p.tsbt_benh_tim = 0;
        p.tsbt_phau_thuat_tim = 0;
        p.tsbt_tang_huyet_ap = 0;
        p.tsbt_kho_tho = 0;
        p.tsbt_benh_phoi = 0;
        p.tsbt_benh_than = 0;
        p.tsbt_nghien_ruou = 0;
        p.tsbt_dai_thao_duong = 0;
        p.tsbt_benh_tam_than = 0;
        p.tsbt_mat_y_thuc = 0;
        p.tsbt_ngat = 0;
        p.tsbt_benh_tieu_hoa = 0;
        p.tsbt_roi_loan_giac_ngu = 0;
        p.tsbt_tai_bien = 0;
        p.tsbt_benh_cot_song = 0;
        p.tsbt_ruou_thuong_xuyen = 0;
        p.tsbt_ma_tuy = 0;
        p.tsbt_benh_khac = 0;
        p.tsbt_ma_benh_khac.clear();
        p.tsbt_dang_dieu_tri_benh = 0;
        p.tsbt_ma_benh.clear();
        p.tsbt_ten_thuoc_lieu_luong.clear();
        p.tsbt_thai_san = 0;
        p.tsbt_ma_benh_thai_san.clear();
        p.tsbt_ten_thuoc_thai_san.clear();

        out.push(RecordItem {
            id: 0,
            ho_ten: p.ho_ten.clone(),
            so_cccd: p.so_cccd.clone(),
            ngay_vao: p.ngay_vao.clone(),
            status: 0,
            contract_id: Some(contract.id),
            ma_y_te: p.ma_y_te.clone(),
            so_luu_tru: "".into(),
            data: p,
        });
    }

    Ok(out)
}
