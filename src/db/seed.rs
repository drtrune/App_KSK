use rusqlite::{params, Connection};
use crate::models::{CatalogRecord, Qd1613CatalogData, RoleRecord, UserRecord};
use crate::auth::permissions::Perm;
use crate::auth::hash_password;

pub fn seed_all(conn: &Connection, qd1613_json: &str) -> Result<(), rusqlite::Error> {
    seed_roles(conn)?;
    seed_admin_user(conn)?;
    seed_catalogs(conn)?;
    seed_qd1613(conn, qd1613_json)?;
    Ok(())
}

pub fn seed_roles(conn: &Connection) -> Result<(), rusqlite::Error> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM roles", [], |r| r.get(0)).unwrap_or(0);
    if count > 0 {
        return Ok(());
    }

    let default_roles = [
        ("admin", "Quản trị viên (Admin)", "Toàn quyền hệ thống", true, vec![
            Perm::RecordView, Perm::RecordCreate, Perm::RecordDelete, Perm::RecordExport, Perm::RecordConfirm, Perm::RecordUnlock,
            Perm::FormHanhChinh, Perm::FormTienSu, Perm::ExamTheLuc, Perm::ExamNoi, Perm::ExamNgoai, Perm::ExamDaLieu, Perm::ExamSan,
            Perm::ExamMat, Perm::ExamTmh, Perm::ExamRhm, Perm::ExamCls, Perm::FormKetLuan,
            Perm::CatalogManage, Perm::Qd1613Manage, Perm::UserManage, Perm::RoleManage, Perm::SettingsManage,
        ]),
        ("reception", "Tiếp đón / Hành chính", "Nhập thông tin hành chính, tiền sử", true, vec![
            Perm::RecordView, Perm::RecordCreate, Perm::RecordExport, Perm::FormHanhChinh, Perm::FormTienSu,
        ]),
        ("bs_theluc", "Bác sĩ Thể lực", "Khám & phân loại thể lực", true, vec![
            Perm::RecordView, Perm::ExamTheLuc,
        ]),
        ("bs_noi", "Bác sĩ Nội khoa", "Khám 8 chuyên khoa nội", true, vec![
            Perm::RecordView, Perm::ExamNoi,
        ]),
        ("bs_ngoai", "Bác sĩ Ngoại khoa", "Khám chuyên khoa ngoại", true, vec![
            Perm::RecordView, Perm::ExamNgoai,
        ]),
        ("bs_dalieu", "Bác sĩ Da liễu", "Khám chuyên khoa da liễu", true, vec![
            Perm::RecordView, Perm::ExamDaLieu,
        ]),
        ("bs_san", "Bác sĩ Sản phụ khoa", "Khám chuyên khoa sản phụ khoa", true, vec![
            Perm::RecordView, Perm::ExamSan,
        ]),
        ("bs_mat", "Bác sĩ Mắt", "Khám mắt & đo thị lực", true, vec![
            Perm::RecordView, Perm::ExamMat,
        ]),
        ("bs_tmh", "Bác sĩ Tai Mũi Họng", "Khám TMH & đo sức nghe", true, vec![
            Perm::RecordView, Perm::ExamTmh,
        ]),
        ("bs_rhm", "Bác sĩ Răng Hàm Mặt", "Khám răng hàm mặt", true, vec![
            Perm::RecordView, Perm::ExamRhm,
        ]),
        ("bs_cls", "Bác sĩ Cận Lâm Sàng", "Nhập kết quả xét nghiệm / CĐHA", true, vec![
            Perm::RecordView, Perm::ExamCls,
        ]),
        ("bs_ketluan", "Bác sĩ Kết Luận", "Kết luận phân loại sức khỏe & ký hồ sơ", true, vec![
            Perm::RecordView, Perm::RecordExport, Perm::RecordConfirm, Perm::FormKetLuan,
        ]),
        ("client", "Người khám bệnh", "Tự nhập thông tin hành chính", true, vec![
            Perm::RecordView, Perm::RecordCreate, Perm::FormHanhChinh, Perm::FormTienSu,
        ]),
    ];

    for (code, name, desc, is_sys, perms) in default_roles {
        let role = RoleRecord {
            code: code.to_string(),
            name: name.to_string(),
            description: desc.to_string(),
            is_system: is_sys,
            permissions: perms.into_iter().map(|p| p.as_str().to_string()).collect(),
        };
        crate::db::users::save_role(conn, &role)?;
    }

    Ok(())
}

pub fn seed_admin_user(conn: &Connection) -> Result<(), rusqlite::Error> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM users", [], |r| r.get(0)).unwrap_or(0);
    if count == 0 {
        let admin = UserRecord {
            id: 0,
            username: "admin".to_string(),
            full_name: "Quản trị viên hệ thống".to_string(),
            password: hash_password("admin123"),
            role: "admin".to_string(),
            is_active: true,
        };
        crate::db::users::save_user(conn, &admin)?;
    }
    Ok(())
}

pub fn seed_catalogs(conn: &Connection) -> Result<(), rusqlite::Error> {
    let seeds = [
        // 1. Đối tượng theo đúng 16 mã QĐ 3176
        ("priority", "1", "1 - Người cao tuổi", "", 1),
        ("priority", "2", "2 - Người khuyết tật", "", 2),
        ("priority", "3", "3 - Người thuộc hộ nghèo, cận nghèo", "", 3),
        ("priority", "4", "4 - Người có công", "", 4),
        ("priority", "5", "5 - Người mắc bệnh mạn tính", "", 5),
        ("priority", "6", "6 - Vùng đồng bào DTTS & miền núi", "", 6),
        ("priority", "7", "7 - Vùng ĐK KT-XH khó khăn, đặc biệt khó khăn", "", 7),
        ("priority", "8", "8 - Người sống tại xã đảo", "", 8),
        ("priority", "9", "9 - Người sống tại đặc khu", "", 9),
        ("priority", "10", "10 - Trẻ em trong cơ sở mầm non", "", 10),
        ("priority", "11", "11 - Học sinh cơ sở GD phổ thông", "", 11),
        ("priority", "12", "12 - Sinh viên", "", 12),
        ("priority", "13", "13 - Người lao động", "", 13),
        ("priority", "14", "14 - Người lao động không chính thức", "", 14),
        ("priority", "15", "15 - Người chưa có BHYT", "", 15),
        ("priority", "16", "16 - Các đối tượng khác", "", 16),

        // 2. Nguồn chi trả theo QĐ 3176
        ("payment", "1", "1 - Ngân sách Trung ương", "", 1),
        ("payment", "2", "2 - Ngân sách Địa phương", "", 2),
        ("payment", "3", "3 - Quỹ Bảo hiểm y tế", "", 3),
        ("payment", "4", "4 - Người sử dụng lao động", "", 4),
        ("payment", "5", "5 - Xã hội hóa", "", 5),
        ("payment", "9", "9 - Khác", "", 6),

        // 3. Toàn bộ 63 Tỉnh/Thành phố Việt Nam
        ("province", "01", "01 - TP. Hà Nội", "", 1),
        ("province", "02", "02 - Tỉnh Hà Giang", "", 2),
        ("province", "04", "04 - Tỉnh Cao Bằng", "", 3),
        ("province", "06", "06 - Tỉnh Bắc Kạn", "", 4),
        ("province", "08", "08 - Tỉnh Tuyên Quang", "", 5),
        ("province", "10", "10 - Tỉnh Lào Cai", "", 6),
        ("province", "11", "11 - Tỉnh Điện Biên", "", 7),
        ("province", "12", "12 - Tỉnh Lai Châu", "", 8),
        ("province", "14", "14 - Tỉnh Sơn La", "", 9),
        ("province", "15", "15 - Tỉnh Yên Bái", "", 10),
        ("province", "17", "17 - Tỉnh Hòa Bình", "", 11),
        ("province", "19", "19 - Tỉnh Thái Nguyên", "", 12),
        ("province", "20", "20 - Tỉnh Lạng Sơn", "", 13),
        ("province", "22", "22 - Tỉnh Quảng Ninh", "", 14),
        ("province", "24", "24 - Tỉnh Bắc Giang", "", 15),
        ("province", "25", "25 - Tỉnh Phú Thọ", "", 16),
        ("province", "26", "26 - Tỉnh Vĩnh Phúc", "", 17),
        ("province", "27", "27 - Tỉnh Bắc Ninh", "", 18),
        ("province", "30", "30 - Tỉnh Hải Dương", "", 19),
        ("province", "31", "31 - TP. Hải Phòng", "", 20),
        ("province", "33", "33 - Tỉnh Hưng Yên", "", 21),
        ("province", "34", "34 - Tỉnh Thái Bình", "", 22),
        ("province", "35", "35 - Tỉnh Hà Nam", "", 23),
        ("province", "36", "36 - Tỉnh Nam Định", "", 24),
        ("province", "37", "37 - Tỉnh Ninh Bình", "", 25),
        ("province", "38", "38 - Tỉnh Thanh Hóa", "", 26),
        ("province", "40", "40 - Tỉnh Nghệ An", "", 27),
        ("province", "42", "42 - Tỉnh Hà Tĩnh", "", 28),
        ("province", "44", "44 - Tỉnh Quảng Bình", "", 29),
        ("province", "45", "45 - Tỉnh Quảng Trị", "", 30),
        ("province", "46", "46 - Tỉnh Thừa Thiên Huế", "", 31),
        ("province", "48", "48 - TP. Đà Nẵng", "", 32),
        ("province", "49", "49 - Tỉnh Quảng Nam", "", 33),
        ("province", "51", "51 - Tỉnh Quảng Ngãi", "", 34),
        ("province", "52", "52 - Tỉnh Bình Định", "", 35),
        ("province", "54", "54 - Tỉnh Phú Yên", "", 36),
        ("province", "56", "56 - Tỉnh Khánh Hòa", "", 37),
        ("province", "58", "58 - Tỉnh Ninh Thuận", "", 38),
        ("province", "60", "60 - Tỉnh Bình Thuận", "", 39),
        ("province", "62", "62 - Tỉnh Kon Tum", "", 40),
        ("province", "64", "64 - Tỉnh Gia Lai", "", 41),
        ("province", "66", "66 - Tỉnh Đắk Lắk", "", 42),
        ("province", "67", "67 - Tỉnh Đắk Nông", "", 43),
        ("province", "68", "68 - Tỉnh Lâm Đồng", "", 44),
        ("province", "70", "70 - Tỉnh Bình Phước", "", 45),
        ("province", "72", "72 - Tỉnh Tây Ninh", "", 46),
        ("province", "74", "74 - Tỉnh Bình Dương", "", 47),
        ("province", "75", "75 - Tỉnh Đồng Nai", "", 48),
        ("province", "77", "77 - Tỉnh Bà Rịa - Vũng Tàu", "", 49),
        ("province", "79", "79 - TP. Hồ Chí Minh", "", 50),
        ("province", "80", "80 - Tỉnh Long An", "", 51),
        ("province", "82", "82 - Tỉnh Tiền Giang", "", 52),
        ("province", "83", "83 - Tỉnh Bến Tre", "", 53),
        ("province", "84", "84 - Tỉnh Trà Vinh", "", 54),
        ("province", "86", "86 - Tỉnh Vĩnh Long", "", 55),
        ("province", "87", "87 - Tỉnh Đồng Tháp", "", 56),
        ("province", "89", "89 - Tỉnh An Giang", "", 57),
        ("province", "91", "91 - Tỉnh Kiên Giang", "", 58),
        ("province", "92", "92 - TP. Cần Thơ", "", 59),
        ("province", "93", "93 - Tỉnh Hậu Giang", "", 60),
        ("province", "94", "94 - Tỉnh Sóc Trăng", "", 61),
        ("province", "95", "95 - Tỉnh Bạc Liêu", "", 62),
        ("province", "96", "96 - Tỉnh Cà Mau", "", 63),

        // 4. Xã/Phường mẫu (Bình Định 52, Gia Lai 64, Hà Nội 01, TP.HCM 79, Đà Nẵng 48)
        ("ward", "21517", "21517 - Phường Lê Lợi", "52", 1),
        ("ward", "21520", "21520 - Phường Trần Hưng Đạo", "52", 2),
        ("ward", "21523", "21523 - Phường Ngô Mây", "52", 3),
        ("ward", "21526", "21526 - Phường Nguyễn Văn Cừ", "52", 4),
        ("ward", "21529", "21529 - Phường Ghềnh Ráng", "52", 5),
        ("ward", "21532", "21532 - Phường Nhơn Phú", "52", 6),
        ("ward", "21535", "21535 - Phường Nhơn Bình", "52", 7),
        ("ward", "21538", "21538 - Phường Đống Đa", "52", 8),
        ("ward", "21541", "21541 - Phường Thị Nại", "52", 9),
        ("ward", "21544", "21544 - Phường Bùi Thị Xuân", "52", 10),
        ("ward", "21550", "21550 - Thị xã An Nhơn", "52", 11),
        ("ward", "21553", "21553 - Huyện Tuy Phước", "52", 12),
        ("ward", "21556", "21556 - Huyện Phù Cát", "52", 13),
        ("ward", "21559", "21559 - Huyện Hoài Nhơn", "52", 14),

        ("ward", "23992", "23992 - Phường Hoa Lư", "64", 1),
        ("ward", "23995", "23995 - Phường Tây Sơn", "64", 2),
        ("ward", "23998", "23998 - Phường Hội Thương", "64", 3),
        ("ward", "24001", "24001 - Phường Diên Hồng", "64", 4),
        ("ward", "24004", "24004 - Phường Thống Nhất", "64", 5),
        ("ward", "24007", "24007 - Phường Phù Đổng", "64", 6),
        ("ward", "24010", "24010 - Phường Chi Lăng", "64", 7),
        ("ward", "24013", "24013 - Xã Trà Đa", "64", 8),
        ("ward", "24016", "24016 - Xã Biển Hồ", "64", 9),
        ("ward", "24020", "24020 - Thị xã An Khê", "64", 10),
        ("ward", "24025", "24025 - Thị xã Ayun Pa", "64", 11),

        ("ward", "00001", "00001 - Phường Phúc Xá", "01", 1),
        ("ward", "00004", "00004 - Phường Trúc Bạch", "01", 2),
        ("ward", "00007", "00007 - Phường Hàng Mã", "01", 3),
        ("ward", "00010", "00010 - Phường Hàng Bạc", "01", 4),
        ("ward", "00015", "00015 - Phường Cầu Giấy", "01", 5),
        ("ward", "00020", "00020 - Phường Dịch Vọng", "01", 6),
        ("ward", "00025", "00025 - Phường Kim Mã", "01", 7),
        ("ward", "00030", "00030 - Phường Chương Mỹ", "01", 8),

        ("ward", "26734", "26734 - Phường Bến Nghé", "79", 1),
        ("ward", "26737", "26737 - Phường Bến Thành", "79", 2),
        ("ward", "26740", "26740 - Phường Cầu Ông Lãnh", "79", 3),
        ("ward", "26743", "26743 - Phường Tân Định", "79", 4),
        ("ward", "26746", "26746 - Phường Đa Kao", "79", 5),
        ("ward", "26750", "26750 - Phường Thảo Điền", "79", 6),
        ("ward", "26755", "26755 - Phường An Phú", "79", 7),

        ("ward", "20194", "20194 - Phường Hải Châu 1", "48", 1),
        ("ward", "20197", "20197 - Phường Thạch Thang", "48", 2),
        ("ward", "20200", "20200 - Phường Hòa Cường Bắc", "48", 3),

        // 5. Dân tộc mẫu
        ("ethnicity", "01", "01 - Kinh", "", 1),
        ("ethnicity", "02", "02 - Tày", "", 2),
        ("ethnicity", "03", "03 - Thái", "", 3),
        ("ethnicity", "10", "10 - Gia Rai", "", 4),
        ("ethnicity", "11", "11 - Ba Na", "", 5),

        // 6. Nghề nghiệp mẫu
        ("occupation", "22", "22 - Kỹ sư, Chuyên viên", "", 1),
        ("occupation", "52", "52 - Nhân viên bán hàng", "", 2),
        ("occupation", "71", "71 - Công nhân xây dựng", "", 3),
        ("occupation", "99", "99 - Khác", "", 4),

        // 7. ICD-10 cơ bản cho tiền sử
        ("icd10", "I10", "I10 - Bệnh tăng huyết áp vô căn", "", 1),
        ("icd10", "E11", "E11 - Đái tháo đường không phụ thuộc insulin", "", 2),
        ("icd10", "J45", "J45 - Hen phế quản", "", 3),
        ("icd10", "A15", "A15 - Lao đường hô hấp", "", 4),
        ("icd10", "G40", "G40 - Động kinh", "", 5),

        // 8. Nhóm máu (bổ sung danh mục định nghĩa)
        ("blood_type", "A", "Nhóm máu A", "", 1),
        ("blood_type", "B", "Nhóm máu B", "", 2),
        ("blood_type", "AB", "Nhóm máu AB", "", 3),
        ("blood_type", "O", "Nhóm máu O", "", 4),

        // 9. Lý do khám
        ("exam_reason", "KSK_DK", "Khám sức khỏe định kỳ", "", 1),
        ("exam_reason", "KSK_TD", "Khám tuyển dụng", "", 2),
        ("exam_reason", "KSK_HT", "Khám sức khỏe học tập", "", 3),

        // 10. Loại hình KCB (theo QĐ 1804)
        ("kcb_type", "15", "15 - Khám sức khỏe định kỳ", "", 1),
        ("kcb_type", "01", "01 - Khám bệnh ngoại trú", "", 2),

        // 11. Giới tính
        ("gender", "1", "1 - Nam", "", 1),
        ("gender", "2", "2 - Nữ", "", 2),

        // 12. Nơi cấp CCCD
        ("id_issuer", "C06", "Cục Cảnh sát QLHCVTTXH", "", 1),

        // 13. Kết luận mẫu
        ("conclusion_sample", "DU_SK", "Đủ sức khỏe làm việc", "", 1),
        ("conclusion_sample", "DU_SK_DK", "Đủ sức khỏe học tập", "", 2),
    ];

    for s in seeds {
        let item = CatalogRecord {
            id: 0,
            cat_type: s.0.to_string(),
            code: s.1.to_string(),
            name: s.2.to_string(),
            parent_code: s.3.to_string(),
            sort_order: s.4,
            is_active: true,
        };
        crate::db::catalogs::save_catalog(conn, &item)?;
    }

    Ok(())
}

pub fn seed_qd1613(conn: &Connection, qd1613_json: &str) -> Result<(), rusqlite::Error> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM qd1613_specialties", [], |r| r.get(0)).unwrap_or(0);
    if count == 0 {
        if let Ok(data) = serde_json::from_str::<Qd1613CatalogData>(qd1613_json) {
            let _ = conn.execute("DELETE FROM qd1613_classifications", []);
            let _ = conn.execute("DELETE FROM qd1613_diseases", []);
            let _ = conn.execute("DELETE FROM qd1613_specialties", []);

            let mut stmt_s = conn.prepare("INSERT INTO qd1613_specialties (id, name, parent_id) VALUES (?, ?, ?)")?;
            for s in data.specialties {
                let _ = stmt_s.execute(params![s.id, s.name, s.parent_id]);
            }

            let mut stmt_d = conn.prepare("INSERT INTO qd1613_diseases (id, name, spec_id) VALUES (?, ?, ?)")?;
            for d in data.diseases {
                let _ = stmt_d.execute(params![d.id, d.name, d.spec_id]);
            }

            let mut stmt_c = conn.prepare(
                "INSERT INTO qd1613_classifications (id, disease_id, name, score, ref_rule, ref_disease_id, modifier)
                 VALUES (?, ?, ?, ?, ?, ?, ?)"
            )?;
            for c in data.classifications {
                let _ = stmt_c.execute(params![c.id, c.disease_id, c.name, c.score, c.ref_rule, c.ref_disease_id, c.modifier]);
            }
        }
    }
    Ok(())
}

