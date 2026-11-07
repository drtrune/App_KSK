use rusqlite::{params, Connection};
use crate::models::RecordItem;

pub fn load_records(conn: &Connection) -> Result<Vec<RecordItem>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT id, ho_ten, so_cccd, ngay_vao, status, json_data FROM records ORDER BY id DESC")?;
    let rows = stmt.query_map([], |r| {
        let json_str: String = r.get(5)?;
        let payload = serde_json::from_str(&json_str).unwrap_or_default();
        Ok(RecordItem {
            id: r.get(0)?,
            ho_ten: r.get::<_, Option<String>>(1)?.unwrap_or_default(),
            so_cccd: r.get::<_, Option<String>>(2)?.unwrap_or_default(),
            ngay_vao: r.get::<_, Option<String>>(3)?.unwrap_or_default(),
            status: r.get(4)?,
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
    let json_data = serde_json::to_string(&item.data).unwrap_or_default();
    if item.id == 0 {
        conn.execute(
            "INSERT INTO records (ho_ten, so_cccd, ngay_vao, status, json_data) VALUES (?, ?, ?, ?, ?)",
            params![item.ho_ten, item.so_cccd, item.ngay_vao, item.status, json_data],
        )?;
        Ok(conn.last_insert_rowid())
    } else {
        conn.execute(
            "UPDATE records SET ho_ten = ?, so_cccd = ?, ngay_vao = ?, status = ?, json_data = ? WHERE id = ?",
            params![item.ho_ten, item.so_cccd, item.ngay_vao, item.status, json_data, item.id],
        )?;
        Ok(item.id)
    }
}

pub fn delete_record(conn: &Connection, id: i64) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM records WHERE id = ?", params![id])?;
    Ok(())
}
