use rusqlite::{params, Connection};
use crate::models::{QdSpecialty, QdDisease, QdClassification};

pub fn load_qd1613(conn: &Connection) -> Result<(Vec<QdSpecialty>, Vec<QdDisease>, Vec<QdClassification>), rusqlite::Error> {
    let mut s_stmt = conn.prepare("SELECT id, name, parent_id FROM qd1613_specialties ORDER BY id ASC")?;
    let specialties = s_stmt.query_map([], |r| {
        Ok(QdSpecialty {
            id: r.get(0)?,
            name: r.get(1)?,
            parent_id: r.get(2)?,
        })
    })?.filter_map(|r| r.ok()).collect();

    let mut d_stmt = conn.prepare("SELECT id, name, spec_id FROM qd1613_diseases ORDER BY id ASC")?;
    let diseases = d_stmt.query_map([], |r| {
        Ok(QdDisease {
            id: r.get(0)?,
            name: r.get(1)?,
            spec_id: r.get(2)?,
        })
    })?.filter_map(|r| r.ok()).collect();

    let mut c_stmt = conn.prepare("SELECT id, disease_id, name, score, ref_rule, ref_disease_id, modifier FROM qd1613_classifications ORDER BY id ASC")?;
    let classifications = c_stmt.query_map([], |r| {
        Ok(QdClassification {
            id: r.get(0)?,
            disease_id: r.get(1)?,
            name: r.get(2)?,
            score: r.get(3)?,
            ref_rule: r.get(4)?,
            ref_disease_id: r.get(5)?,
            modifier: r.get(6)?,
        })
    })?.filter_map(|r| r.ok()).collect();

    Ok((specialties, diseases, classifications))
}

pub fn save_classification(conn: &Connection, c: &QdClassification) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO qd1613_classifications (id, disease_id, name, score, ref_rule, ref_disease_id, modifier)
         VALUES (?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET name = excluded.name, score = excluded.score, ref_rule = excluded.ref_rule",
        params![c.id, c.disease_id, c.name, c.score, c.ref_rule, c.ref_disease_id, c.modifier],
    )?;
    Ok(())
}

pub fn delete_classification(conn: &Connection, id: &str) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM qd1613_classifications WHERE id = ?", params![id])?;
    Ok(())
}
