use rusqlite::{params, Connection};
use crate::models::{UserRecord, RoleRecord};

pub fn load_users(conn: &Connection) -> Result<Vec<UserRecord>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT id, username, COALESCE(full_name, ''), password, role, COALESCE(is_active, 1) FROM users ORDER BY id ASC")?;
    let rows = stmt.query_map([], |r| {
        let is_active_int: i32 = r.get(5)?;
        Ok(UserRecord {
            id: r.get(0)?,
            username: r.get(1)?,
            full_name: r.get(2)?,
            password: r.get(3)?,
            role: r.get(4)?,
            is_active: is_active_int != 0,
        })
    })?;
    let mut list = Vec::new();
    for row in rows {
        list.push(row?);
    }
    Ok(list)
}

pub fn save_user(conn: &Connection, user: &UserRecord) -> Result<(), rusqlite::Error> {
    if user.id == 0 {
        conn.execute(
            "INSERT INTO users (username, full_name, password, role, is_active) VALUES (?, ?, ?, ?, ?)",
            params![user.username, user.full_name, user.password, user.role, if user.is_active { 1 } else { 0 }],
        )?;
    } else {
        conn.execute(
            "UPDATE users SET username = ?, full_name = ?, password = ?, role = ?, is_active = ? WHERE id = ?",
            params![user.username, user.full_name, user.password, user.role, if user.is_active { 1 } else { 0 }, user.id],
        )?;
    }
    Ok(())
}

pub fn delete_user(conn: &Connection, id: i64) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM users WHERE id = ?", params![id])?;
    Ok(())
}

pub fn load_roles(conn: &Connection) -> Result<Vec<RoleRecord>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT code, name, description, is_system FROM roles ORDER BY code ASC")?;
    let role_rows = stmt.query_map([], |r| {
        let is_sys_int: i32 = r.get(3)?;
        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, is_sys_int != 0))
    })?;

    let mut roles = Vec::new();
    for r in role_rows {
        let (code, name, description, is_system) = r?;
        let mut p_stmt = conn.prepare("SELECT perm_key FROM role_permissions WHERE role_code = ?")?;
        let perms = p_stmt.query_map(params![code], |pr| pr.get::<_, String>(0))?
            .filter_map(|res| res.ok())
            .collect();

        roles.push(RoleRecord {
            code,
            name,
            description,
            is_system,
            permissions: perms,
        });
    }
    Ok(roles)
}

pub fn save_role(conn: &Connection, role: &RoleRecord) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO roles (code, name, description, is_system) VALUES (?, ?, ?, ?)
         ON CONFLICT(code) DO UPDATE SET name = excluded.name, description = excluded.description",
        params![role.code, role.name, role.description, if role.is_system { 1 } else { 0 }],
    )?;

    // Update permissions
    conn.execute("DELETE FROM role_permissions WHERE role_code = ?", params![role.code])?;
    for p in &role.permissions {
        conn.execute("INSERT INTO role_permissions (role_code, perm_key) VALUES (?, ?)", params![role.code, p])?;
    }
    Ok(())
}

pub fn delete_role(conn: &Connection, code: &str) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM role_permissions WHERE role_code = ?", params![code])?;
    conn.execute("DELETE FROM roles WHERE code = ?", params![code])?;
    Ok(())
}
