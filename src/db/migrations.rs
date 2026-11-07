use rusqlite::Connection;

pub fn run_migrations(conn: &Connection) -> Result<(), rusqlite::Error> {
    // Check current user_version
    let version: i32 = conn.query_row("PRAGMA user_version;", [], |r| r.get(0))?;

    if version < 1 {
        conn.execute_batch(
            r#"
            -- 1. Base legacy tables if not existing
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT UNIQUE NOT NULL,
                full_name TEXT DEFAULT '',
                password TEXT NOT NULL,
                role TEXT NOT NULL,
                is_active INTEGER DEFAULT 1
            );

            CREATE TABLE IF NOT EXISTS roles (
                code TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT DEFAULT '',
                is_system INTEGER DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS role_permissions (
                role_code TEXT NOT NULL,
                perm_key TEXT NOT NULL,
                PRIMARY KEY (role_code, perm_key)
            );

            CREATE TABLE IF NOT EXISTS catalogs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                cat_type TEXT NOT NULL,
                code TEXT NOT NULL,
                name TEXT NOT NULL,
                parent_code TEXT DEFAULT '',
                sort_order INTEGER DEFAULT 0,
                is_active INTEGER DEFAULT 1,
                UNIQUE(cat_type, code)
            );

            CREATE TABLE IF NOT EXISTS records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ho_ten TEXT,
                so_cccd TEXT,
                ngay_vao TEXT,
                status INTEGER DEFAULT 0,
                json_data TEXT
            );

            CREATE TABLE IF NOT EXISTS qd1613_specialties (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                parent_id TEXT
            );

            CREATE TABLE IF NOT EXISTS qd1613_diseases (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                spec_id TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS qd1613_classifications (
                id TEXT PRIMARY KEY,
                disease_id TEXT NOT NULL,
                name TEXT NOT NULL,
                score INTEGER,
                ref_rule TEXT,
                ref_disease_id TEXT,
                modifier TEXT
            );

            PRAGMA user_version = 1;
            "#
        )?;

        // Ensure columns exist if migrating from old database without them
        let _ = conn.execute("ALTER TABLE users ADD COLUMN full_name TEXT DEFAULT ''", []);
        let _ = conn.execute("ALTER TABLE users ADD COLUMN is_active INTEGER DEFAULT 1", []);
        let _ = conn.execute("ALTER TABLE catalogs ADD COLUMN sort_order INTEGER DEFAULT 0", []);
        let _ = conn.execute("ALTER TABLE catalogs ADD COLUMN is_active INTEGER DEFAULT 1", []);
    }

    Ok(())
}
