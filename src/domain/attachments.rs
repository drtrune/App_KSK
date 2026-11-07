use std::fs;
use std::path::{Path, PathBuf};
use crate::config::AppConfig;
use crate::domain::docx_merge::export_record_to_mau03_docx;
use crate::domain::export::{
    export_cls_to_csv, export_records_to_csv, export_records_to_xml, export_to_xlsm_file,
};
use crate::models::{AttachmentRecord, CompanyRecord, ContractRecord, RecordItem};

pub fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

pub fn store_attachment_file(
    src_path: &Path,
    owner_type: &str, // "contract" | "record"
    owner_id: i64,
    doc_type: &str,
    prefix_label: &str,
) -> Result<AttachmentRecord, String> {
    let dir = PathBuf::from(format!("attachments/{}/{}", owner_type, owner_id));
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let orig_name = src_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("scan_file.pdf");

    let safe_prefix = sanitize_filename(prefix_label);
    let dest_name = if safe_prefix.is_empty() {
        sanitize_filename(orig_name)
    } else {
        format!("{}_{}", safe_prefix, sanitize_filename(orig_name))
    };

    let dest_path = dir.join(&dest_name);
    fs::copy(src_path, &dest_path).map_err(|e| format!("Không thể sao chép file scan: {}", e))?;

    Ok(AttachmentRecord {
        id: 0,
        owner_type: owner_type.to_string(),
        owner_id,
        file_name: dest_name,
        file_path: dest_path.to_string_lossy().to_string(),
        doc_type: doc_type.to_string(),
        uploaded_at: chrono::Local::now().format("%d/%m/%Y %H:%M").to_string(),
    })
}

pub fn open_path_in_os(path_str: &str) {
    let p = Path::new(path_str);
    if p.exists() {
        let _ = std::process::Command::new("explorer").arg(p).spawn();
    }
}

/// Đóng gói Trọn Bộ Hồ Sơ Đoàn (Hợp đồng Scan + Hồ sơ Scan cá nhân + Phiếu Mẫu 03 + Excel/XML BYT)
pub fn export_contract_full_dossier(
    base_dir: &Path,
    company: &CompanyRecord,
    contract: &ContractRecord,
    contract_records: &[RecordItem],
    all_attachments: &[AttachmentRecord],
    config: &AppConfig,
) -> Result<PathBuf, String> {
    let folder_name = sanitize_filename(&format!(
        "BoHoSo_{}_{}",
        company.ma_cong_ty,
        contract.so_hop_dong
    ));
    let root = base_dir.join(folder_name);
    let dir_hd = root.join("01_HopDong_PhuLuc");
    let dir_scan = root.join("02_HoSoScan_CaNhan");
    let dir_docx = root.join("03_PhieuKham_Mau03");
    let dir_byt = root.join("04_DuLieu_BYT_XML_Excel");

    fs::create_dir_all(&dir_hd).map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir_scan).map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir_docx).map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir_byt).map_err(|e| e.to_string())?;

    // 1. Copy các file scan Hợp đồng
    for att in all_attachments
        .iter()
        .filter(|a| a.owner_type == "contract" && a.owner_id == contract.id)
    {
        let src = Path::new(&att.file_path);
        if src.exists() {
            let _ = fs::copy(src, dir_hd.join(&att.file_name));
        }
    }

    // 2. Copy các file scan Phiếu khám cá nhân & xuất phiếu Mẫu 03 Word
    for (idx, rec) in contract_records.iter().enumerate() {
        let num_tag = if !rec.data.so_luu_tru.trim().is_empty() {
            sanitize_filename(&rec.data.so_luu_tru)
        } else {
            sanitize_filename(&rec.data.ma_y_te)
        };
        let person_tag = sanitize_filename(&format!(
            "{:03}_{}_{}",
            idx + 1,
            num_tag,
            rec.ho_ten.to_uppercase()
        ));

        for att in all_attachments
            .iter()
            .filter(|a| a.owner_type == "record" && a.owner_id == rec.id)
        {
            let src = Path::new(&att.file_path);
            if src.exists() {
                let dst_name = format!("{}_{}", person_tag, att.file_name);
                let _ = fs::copy(src, dir_scan.join(dst_name));
            }
        }

        let docx_path = dir_docx.join(format!("{}_Mau03.docx", person_tag));
        let pre_exam_only = rec.status == 0 && rec.data.so_luu_tru.trim().is_empty();
        let _ = export_record_to_mau03_docx(rec, config, &docx_path, pre_exam_only);
    }

    // 3. Xuất bộ dữ liệu BYT (Excel .xlsm + CSV + XML)
    let xlsm_path = dir_byt.join("Import_KSK_Tren_18.xlsm");
    let _ = export_to_xlsm_file(contract_records, &xlsm_path);

    if let Ok(csv_main) = export_records_to_csv(contract_records) {
        let mut bom = vec![0xEF, 0xBB, 0xBF];
        bom.extend_from_slice(csv_main.as_bytes());
        let _ = fs::write(dir_byt.join("Sheet_Tren_18.csv"), bom);
    }

    if let Ok(csv_cls) = export_cls_to_csv(contract_records) {
        let mut bom = vec![0xEF, 0xBB, 0xBF];
        bom.extend_from_slice(csv_cls.as_bytes());
        let _ = fs::write(dir_byt.join("Sheet_DM_CanLamSang.csv"), bom);
    }

    let xml_data = export_records_to_xml(contract_records);
    let _ = fs::write(dir_byt.join("HoSo_KSK_QD3176.xml"), xml_data.as_bytes());

    Ok(root)
}

