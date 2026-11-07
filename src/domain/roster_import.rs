use std::fs;
use std::path::Path;
use crate::config::AppConfig;
use crate::models::{CompanyRecord, ContractRecord, KskRecordPayload, RecordItem};

pub fn export_roster_template_csv(output_path: &Path) -> Result<(), String> {
    let mut wtr = csv::WriterBuilder::new().from_writer(vec![]);
    wtr.write_record([
        "STT",
        "HO_TEN",
        "GIOI_TINH (1=Nam, 2=Nu)",
        "NGAY_SINH (DD/MM/YYYY)",
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
        "TSGD_MAC_BENH (0/1)",
        "TSGD_MA_BENH",
        "TSBT_TANG_HUYET_AP (0/1)",
        "TSBT_DAI_THAO_DUONG (0/1)",
        "TSBT_BENH_KHAC",
    ])
    .map_err(|e| e.to_string())?;

    wtr.write_record([
        "1",
        "NGUYỄN VĂN AN",
        "1",
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
        "0",
        "",
        "0",
        "0",
        "",
    ])
    .map_err(|e| e.to_string())?;

    let bytes = wtr.into_inner().map_err(|e| e.to_string())?;
    // Prepend UTF-8 BOM for Excel compatibility
    let mut out = vec![0xEF, 0xBB, 0xBF];
    out.extend_from_slice(&bytes);
    fs::write(output_path, out).map_err(|e| e.to_string())
}

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

    for (idx, result) in rdr.records().enumerate() {
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
        p.ma_cskcb = config.default_cskcb.clone();
        p.ma_gtin_cskcb = config.default_gtin_cskcb.clone();
        p.ngay_vao = if !contract.ngay_bat_dau.trim().is_empty() {
            contract.ngay_bat_dau.clone()
        } else {
            now_date.clone()
        };

        p.ho_ten = ho_ten.clone();
        let gt_str = row.get(2).unwrap_or("1").trim().to_lowercase();
        p.gioi_tinh = if gt_str == "2" || gt_str.contains("nữ") || gt_str.contains("nu") {
            2
        } else {
            1
        };
        p.ngay_sinh = row.get(3).unwrap_or("").trim().to_string();
        p.so_cccd = row.get(4).unwrap_or("").trim().to_string();
        p.ngaycap_cccd = row.get(5).unwrap_or("").trim().to_string();
        let noicap = row.get(6).unwrap_or("").trim();
        if !noicap.is_empty() {
            p.noicap_cccd = noicap.to_string();
        }
        p.dien_thoai = row.get(7).unwrap_or("").trim().to_string();
        p.dia_chi = row.get(8).unwrap_or("").trim().to_string();
        let matinh = row.get(9).unwrap_or("").trim();
        p.matinh_cu_tru = if matinh.is_empty() { config.default_province_code.clone() } else { matinh.to_string() };
        let maxa = row.get(10).unwrap_or("").trim();
        p.maxa_cu_tru = if maxa.is_empty() { config.default_ward_code.clone() } else { maxa.to_string() };
        let dantoc = row.get(11).unwrap_or("").trim();
        if !dantoc.is_empty() {
            p.ma_dan_toc = dantoc.to_string();
        }
        let nghenghiep = row.get(12).unwrap_or("").trim();
        if !nghenghiep.is_empty() {
            p.ma_nghe_nghiep = nghenghiep.to_string();
        }
        p.bo_phan = row.get(13).unwrap_or("").trim().to_string();
        let nhommau = row.get(14).unwrap_or("").trim();
        if !nhommau.is_empty() {
            p.nhom_mau = nhommau.to_string();
        }

        p.noi_lam_viec_hoc_tap = company.ten_cong_ty.clone();
        p.doi_tuong = contract.doi_tuong_mac_dinh.clone();
        p.nguon_chi_tra = contract.nguon_chi_tra_mac_dinh;
        p.ly_do_vv = "Khám sức khỏe định kỳ".into();

        if row.get(15).unwrap_or("0").trim() == "1" {
            p.tsgd_mac_benh = 1;
            p.tsgd_ma_benh = row.get(16).unwrap_or("").trim().to_string();
        }
        if row.get(17).unwrap_or("0").trim() == "1" {
            p.tsbt_tang_huyet_ap = 1;
        }
        if row.get(18).unwrap_or("0").trim() == "1" {
            p.tsbt_dai_thao_duong = 1;
        }
        let benh_khac = row.get(19).unwrap_or("").trim();
        if !benh_khac.is_empty() {
            p.tsbt_benh_khac = 1;
            p.tsbt_ma_benh_khac = benh_khac.to_string();
        }

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

        let _ = idx;
    }

    Ok(out)
}
