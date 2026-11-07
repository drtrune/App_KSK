use rusqlite::{params, Connection};
use crate::models::{KskRecordPayload, RecordItem};

pub fn load_records(conn: &Connection) -> Result<Vec<RecordItem>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, ho_ten, so_cccd, ngay_vao, status, json_data, contract_id, ma_y_te, so_luu_tru FROM records ORDER BY id DESC",
    )?;
    let rows = stmt.query_map([], |r| {
        let id: i64 = r.get(0)?;
        let json_str: String = r.get(5)?;
        let mut payload: KskRecordPayload = serde_json::from_str(&json_str).unwrap_or_default();
        let col_contract_id: Option<i64> = r.get(6).ok().flatten();
        let col_ma_y_te: String = r.get::<_, Option<String>>(7)?.unwrap_or_default();
        let col_so_luu_tru: String = r.get::<_, Option<String>>(8)?.unwrap_or_default();

        if payload.contract_id.is_none() {
            payload.contract_id = col_contract_id;
        }
        if !col_ma_y_te.is_empty() {
            payload.ma_y_te = col_ma_y_te;
        } else if payload.ma_y_te.is_empty() {
            payload.ma_y_te = format!("YT{:05}", id);
        }
        if !col_so_luu_tru.is_empty() {
            payload.so_luu_tru = col_so_luu_tru;
        }

        Ok(RecordItem {
            id,
            ho_ten: r.get::<_, Option<String>>(1)?.unwrap_or_default(),
            so_cccd: r.get::<_, Option<String>>(2)?.unwrap_or_default(),
            ngay_vao: r.get::<_, Option<String>>(3)?.unwrap_or_default(),
            status: r.get(4)?,
            contract_id: payload.contract_id,
            ma_y_te: payload.ma_y_te.clone(),
            so_luu_tru: payload.so_luu_tru.clone(),
            data: payload,
        })
    })?;

    let mut list = Vec::new();
    for row in rows {
        list.push(row?);
    }
    Ok(list)
}

pub fn save_record(conn: &Connection, item: &RecordItem) -> Result<i64, rusqlite::Error> {
    let mut data = item.data.clone();
    data.sync_quick_cls_to_list();
    if data.ma_y_te.trim().is_empty() {
        data.ma_y_te = if !item.ma_y_te.trim().is_empty() {
            item.ma_y_te.clone()
        } else {
            format!("YT{}", chrono::Local::now().format("%y%m%d%H%M%S"))
        };
    }

    let json_data = serde_json::to_string(&data).unwrap_or_default();
    if item.id == 0 {
        conn.execute(
            "INSERT INTO records (ho_ten, so_cccd, ngay_vao, status, json_data, contract_id, ma_y_te, so_luu_tru) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                data.ho_ten,
                data.so_cccd,
                data.ngay_vao,
                item.status,
                json_data,
                data.contract_id,
                data.ma_y_te,
                data.so_luu_tru
            ],
        )?;
        Ok(conn.last_insert_rowid())
    } else {
        conn.execute(
            "UPDATE records SET ho_ten = ?, so_cccd = ?, ngay_vao = ?, status = ?, json_data = ?, contract_id = ?, ma_y_te = ?, so_luu_tru = ? WHERE id = ?",
            params![
                data.ho_ten,
                data.so_cccd,
                data.ngay_vao,
                item.status,
                json_data,
                data.contract_id,
                data.ma_y_te,
                data.so_luu_tru,
                item.id
            ],
        )?;
        Ok(item.id)
    }
}

pub fn delete_record(conn: &Connection, id: i64) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM attachments WHERE owner_type = 'record' AND owner_id = ?", params![id])?;
    conn.execute("DELETE FROM records WHERE id = ?", params![id])?;
    Ok(())
}
