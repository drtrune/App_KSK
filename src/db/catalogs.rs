use rusqlite::{params, Connection};
use crate::models::CatalogRecord;

pub fn load_catalogs(conn: &Connection) -> Result<Vec<CatalogRecord>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT id, cat_type, code, name, COALESCE(parent_code, ''), COALESCE(sort_order, 0), COALESCE(is_active, 1) FROM catalogs ORDER BY cat_type, sort_order ASC, code ASC")?;
    let rows = stmt.query_map([], |r| {
        let is_act: i32 = r.get(6)?;
        Ok(CatalogRecord {
            id: r.get(0)?,
            cat_type: r.get(1)?,
            code: r.get(2)?,
            name: r.get(3)?,
            parent_code: r.get(4)?,
            sort_order: r.get(5)?,
            is_active: is_act != 0,
        })
    })?;

    let mut list = Vec::new();
    for row in rows {
        list.push(row?);
    }
    Ok(list)
}

pub fn save_catalog(conn: &Connection, item: &CatalogRecord) -> Result<(), rusqlite::Error> {
    if item.id == 0 {
        conn.execute(
            "INSERT INTO catalogs (cat_type, code, name, parent_code, sort_order, is_active)
             VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT(cat_type, code) DO UPDATE SET name = excluded.name, parent_code = excluded.parent_code, sort_order = excluded.sort_order, is_active = excluded.is_active",
            params![item.cat_type, item.code, item.name, item.parent_code, item.sort_order, if item.is_active { 1 } else { 0 }],
        )?;
    } else {
        conn.execute(
            "UPDATE catalogs SET code = ?, name = ?, parent_code = ?, sort_order = ?, is_active = ? WHERE id = ?",
            params![item.code, item.name, item.parent_code, item.sort_order, if item.is_active { 1 } else { 0 }, item.id],
        )?;
    }
    Ok(())
}

pub fn delete_catalog(conn: &Connection, id: i64) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM catalogs WHERE id = ?", params![id])?;
    Ok(())
}

