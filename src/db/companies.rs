use rusqlite::{params, Connection};
use crate::models::{AttachmentRecord, CompanyRecord, ContractRecord};

pub fn load_companies(conn: &Connection) -> Result<Vec<CompanyRecord>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, ma_cong_ty, ten_cong_ty, ma_so_thue, dia_chi, nguoi_lien_he, dien_thoai, ghi_chu FROM companies ORDER BY id DESC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(CompanyRecord {
            id: r.get(0)?,
            ma_cong_ty: r.get(1)?,
            ten_cong_ty: r.get(2)?,
            ma_so_thue: r.get::<_, Option<String>>(3)?.unwrap_or_default(),
            dia_chi: r.get::<_, Option<String>>(4)?.unwrap_or_default(),
            nguoi_lien_he: r.get::<_, Option<String>>(5)?.unwrap_or_default(),
            dien_thoai: r.get::<_, Option<String>>(6)?.unwrap_or_default(),
            ghi_chu: r.get::<_, Option<String>>(7)?.unwrap_or_default(),
        })
    })?;
    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}

pub fn save_company(conn: &Connection, item: &CompanyRecord) -> Result<i64, rusqlite::Error> {
    if item.id == 0 {
        conn.execute(
            "INSERT INTO companies (ma_cong_ty, ten_cong_ty, ma_so_thue, dia_chi, nguoi_lien_he, dien_thoai, ghi_chu) VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![
                item.ma_cong_ty,
                item.ten_cong_ty,
                item.ma_so_thue,
                item.dia_chi,
                item.nguoi_lien_he,
                item.dien_thoai,
                item.ghi_chu
            ],
        )?;
        Ok(conn.last_insert_rowid())
    } else {
        conn.execute(
            "UPDATE companies SET ma_cong_ty = ?, ten_cong_ty = ?, ma_so_thue = ?, dia_chi = ?, nguoi_lien_he = ?, dien_thoai = ?, ghi_chu = ? WHERE id = ?",
            params![
                item.ma_cong_ty,
                item.ten_cong_ty,
                item.ma_so_thue,
                item.dia_chi,
                item.nguoi_lien_he,
                item.dien_thoai,
                item.ghi_chu,
                item.id
            ],
        )?;
        Ok(item.id)
    }
}

pub fn delete_company(conn: &Connection, id: i64) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM contracts WHERE company_id = ?", params![id])?;
    conn.execute("DELETE FROM companies WHERE id = ?", params![id])?;
    Ok(())
}

pub fn load_contracts(conn: &Connection) -> Result<Vec<ContractRecord>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, company_id, so_hop_dong, ten_dot_kham, ngay_ky, ngay_bat_dau, ngay_ket_thuc, so_luong_du_kien, doi_tuong_mac_dinh, nguon_chi_tra_mac_dinh, trang_thai, ghi_chu FROM contracts ORDER BY id DESC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(ContractRecord {
            id: r.get(0)?,
            company_id: r.get(1)?,
            so_hop_dong: r.get(2)?,
            ten_dot_kham: r.get(3)?,
            ngay_ky: r.get::<_, Option<String>>(4)?.unwrap_or_default(),
            ngay_bat_dau: r.get::<_, Option<String>>(5)?.unwrap_or_default(),
            ngay_ket_thuc: r.get::<_, Option<String>>(6)?.unwrap_or_default(),
            so_luong_du_kien: r.get::<_, Option<i32>>(7)?.unwrap_or(0),
            doi_tuong_mac_dinh: r.get::<_, Option<String>>(8)?.unwrap_or_else(|| "13".into()),
            nguon_chi_tra_mac_dinh: r.get::<_, Option<i32>>(9)?.unwrap_or(4),
            trang_thai: r.get::<_, Option<String>>(10)?.unwrap_or_else(|| "Đang khám".into()),
            ghi_chu: r.get::<_, Option<String>>(11)?.unwrap_or_default(),
        })
    })?;
    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}

pub fn save_contract(conn: &Connection, item: &ContractRecord) -> Result<i64, rusqlite::Error> {
    if item.id == 0 {
        conn.execute(
            "INSERT INTO contracts (company_id, so_hop_dong, ten_dot_kham, ngay_ky, ngay_bat_dau, ngay_ket_thuc, so_luong_du_kien, doi_tuong_mac_dinh, nguon_chi_tra_mac_dinh, trang_thai, ghi_chu) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                item.company_id,
                item.so_hop_dong,
                item.ten_dot_kham,
                item.ngay_ky,
                item.ngay_bat_dau,
                item.ngay_ket_thuc,
                item.so_luong_du_kien,
                item.doi_tuong_mac_dinh,
                item.nguon_chi_tra_mac_dinh,
                item.trang_thai,
                item.ghi_chu
            ],
        )?;
        Ok(conn.last_insert_rowid())
    } else {
        conn.execute(
            "UPDATE contracts SET company_id = ?, so_hop_dong = ?, ten_dot_kham = ?, ngay_ky = ?, ngay_bat_dau = ?, ngay_ket_thuc = ?, so_luong_du_kien = ?, doi_tuong_mac_dinh = ?, nguon_chi_tra_mac_dinh = ?, trang_thai = ?, ghi_chu = ? WHERE id = ?",
            params![
                item.company_id,
                item.so_hop_dong,
                item.ten_dot_kham,
                item.ngay_ky,
                item.ngay_bat_dau,
                item.ngay_ket_thuc,
                item.so_luong_du_kien,
                item.doi_tuong_mac_dinh,
                item.nguon_chi_tra_mac_dinh,
                item.trang_thai,
                item.ghi_chu,
                item.id
            ],
        )?;
        Ok(item.id)
    }
}

pub fn delete_contract(conn: &Connection, id: i64) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM attachments WHERE owner_type = 'contract' AND owner_id = ?", params![id])?;
    conn.execute("UPDATE records SET contract_id = NULL WHERE contract_id = ?", params![id])?;
    conn.execute("DELETE FROM contracts WHERE id = ?", params![id])?;
    Ok(())
}

pub fn load_attachments(conn: &Connection) -> Result<Vec<AttachmentRecord>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, owner_type, owner_id, file_name, file_path, doc_type, uploaded_at FROM attachments ORDER BY id DESC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(AttachmentRecord {
            id: r.get(0)?,
            owner_type: r.get(1)?,
            owner_id: r.get(2)?,
            file_name: r.get(3)?,
            file_path: r.get(4)?,
            doc_type: r.get::<_, Option<String>>(5)?.unwrap_or_default(),
            uploaded_at: r.get::<_, Option<String>>(6)?.unwrap_or_default(),
        })
    })?;
    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}

pub fn add_attachment(conn: &Connection, item: &AttachmentRecord) -> Result<i64, rusqlite::Error> {
    conn.execute(
        "INSERT INTO attachments (owner_type, owner_id, file_name, file_path, doc_type, uploaded_at) VALUES (?, ?, ?, ?, ?, ?)",
        params![
            item.owner_type,
            item.owner_id,
            item.file_name,
            item.file_path,
            item.doc_type,
            item.uploaded_at
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn delete_attachment(conn: &Connection, id: i64) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM attachments WHERE id = ?", params![id])?;
    Ok(())
}
