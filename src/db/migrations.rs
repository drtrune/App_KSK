use rusqlite::Connection;

pub fn run_migrations(conn: &Connection) -> Result<(), rusqlite::Error> {
    let version: i32 = conn.query_row("PRAGMA user_version;", [], |r| r.get(0))?;

    if version < 1 {
        conn.execute_batch(
            r#"
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
            "#,
        )?;

        let _ = conn.execute("ALTER TABLE users ADD COLUMN full_name TEXT DEFAULT ''", []);
        let _ = conn.execute("ALTER TABLE users ADD COLUMN is_active INTEGER DEFAULT 1", []);
        let _ = conn.execute("ALTER TABLE catalogs ADD COLUMN sort_order INTEGER DEFAULT 0", []);
        let _ = conn.execute("ALTER TABLE catalogs ADD COLUMN is_active INTEGER DEFAULT 1", []);
    }

    if version < 2 {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS companies (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ma_cong_ty TEXT NOT NULL,
                ten_cong_ty TEXT NOT NULL,
                ma_so_thue TEXT DEFAULT '',
                dia_chi TEXT DEFAULT '',
                nguoi_lien_he TEXT DEFAULT '',
                dien_thoai TEXT DEFAULT '',
                ghi_chu TEXT DEFAULT ''
            );

            CREATE TABLE IF NOT EXISTS contracts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                company_id INTEGER NOT NULL,
                so_hop_dong TEXT NOT NULL,
                ten_dot_kham TEXT NOT NULL,
                ngay_ky TEXT DEFAULT '',
                ngay_bat_dau TEXT DEFAULT '',
                ngay_ket_thuc TEXT DEFAULT '',
                so_luong_du_kien INTEGER DEFAULT 0,
                doi_tuong_mac_dinh TEXT DEFAULT '13',
                nguon_chi_tra_mac_dinh INTEGER DEFAULT 4,
                trang_thai TEXT DEFAULT 'Đang khám',
                ghi_chu TEXT DEFAULT ''
            );

            CREATE TABLE IF NOT EXISTS attachments (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                owner_type TEXT NOT NULL,
                owner_id INTEGER NOT NULL,
                file_name TEXT NOT NULL,
                file_path TEXT NOT NULL,
                doc_type TEXT DEFAULT '',
                uploaded_at TEXT DEFAULT ''
            );

            PRAGMA user_version = 2;
            "#,
        )?;

        let _ = conn.execute("ALTER TABLE records ADD COLUMN contract_id INTEGER", []);
        let _ = conn.execute("ALTER TABLE records ADD COLUMN ma_y_te TEXT DEFAULT ''", []);
        let _ = conn.execute("ALTER TABLE records ADD COLUMN so_luu_tru TEXT DEFAULT ''", []);
    }

    if version < 3 {
        // Cập nhật bổ sung đầy đủ 63 Tỉnh/TP và Xã/Phường chuẩn Việt Nam
        crate::db::seed::seed_catalogs(conn)?;
        conn.execute("PRAGMA user_version = 3;", [])?;
    }

    Ok(())
}
