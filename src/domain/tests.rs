#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::config::AppConfig;
    use crate::domain::dates::*;
    use crate::domain::docx_merge::*;
    use crate::domain::export::*;
    use crate::domain::scoring::*;
    use crate::models::{KskRecordPayload, RecordItem};

    #[test]
    fn test_age_calculation() {
        assert_eq!(is_valid_date(29, 2, 2024), true);
        assert_eq!(is_valid_date(29, 2, 2023), false);
        assert_eq!(parse_dmy("15/08/1995"), Some((15, 8, 1995)));
        assert_eq!(parse_dmy("invalid"), None);
    }

    #[test]
    fn test_blood_pressure() {
        let (score1, _) = calc_blood_pressure_score("120/80", 25);
        assert_eq!(score1, 1);
        let (score2, _) = calc_blood_pressure_score("150/100", 25);
        assert_eq!(score2, 4);

        let (score3, _) = calc_blood_pressure_score("130/85", 40);
        assert_eq!(score3, 1);
        let (score4, _) = calc_blood_pressure_score("142/92", 40);
        assert_eq!(score4, 3);
    }

    #[test]
    fn test_vision_scoring() {
        assert_eq!(calc_vision_score("10", "10"), 1);
        assert_eq!(calc_vision_score("9", "8"), 2);
        assert_eq!(calc_vision_score("7", "7"), 3);
        assert_eq!(calc_vision_score("5", "5"), 5);
    }

    #[test]
    fn test_hearing_scoring() {
        let (sc1, _) = calc_hearing_from_distances("5", "5", "5", "5");
        assert_eq!(sc1, 1);
        let (sc2, _) = calc_hearing_from_distances("4", "4", "2", "4");
        assert_eq!(sc2, 2);
    }

    #[test]
    fn test_export_xml_and_csv_and_mau03_qr() {
        let mut p = KskRecordPayload::default();
        p.ho_ten = "NGUYỄN VĂN KIỂM TRA".into();
        p.so_cccd = "052090009999".into();
        p.ma_y_te = "HD01-0001".into();
        p.ma_lk = "LK.2026.03.01.0001".into();
        p.cls_duong_mau = "5.3".into();
        p.sync_quick_cls_to_list();

        let rec = RecordItem {
            id: 1,
            ho_ten: p.ho_ten.clone(),
            so_cccd: p.so_cccd.clone(),
            ngay_vao: p.ngay_vao.clone(),
            status: 0,
            contract_id: Some(1),
            ma_y_te: p.ma_y_te.clone(),
            so_luu_tru: "".into(),
            data: p,
        };

        let csv_main = export_records_to_csv(&[rec.clone()]).expect("csv main");
        assert!(csv_main.contains("NOI_KHOA_THAN_TIETNIEU_PL"));
        assert!(csv_main.contains("NGUYỄN VĂN KIỂM TRA"));

        let csv_cls = export_cls_to_csv(&[rec.clone()]).expect("csv cls");
        assert!(csv_cls.contains("MA_CHI_SO"));
        assert!(csv_cls.contains("5.3"));

        let xml = export_records_to_xml(&[rec.clone()]);
        assert!(xml.contains("<NOI_KHOA_THAN_TIETNIEU_PL>1</NOI_KHOA_THAN_TIETNIEU_PL>"));
        assert!(xml.contains("<DSACH_CLS>"));

        let qr = generate_qr_png_bytes(&rec.ma_y_te, &rec.so_cccd, &rec.ho_ten).expect("qr png");
        assert!(qr.len() > 100);

        let barcode = generate_barcode_png_bytes(&rec.data.ma_lk).expect("barcode png");
        assert!(barcode.len() > 100);

        // Kiểm tra barcode chứa dấu chấm '.', gạch chéo '/', gạch ngang '-'
        let bc_with_dots = generate_barcode_png_bytes("LK.2026.03.01-0001/A.B").expect("barcode with dots");
        assert!(bc_with_dots.len() > 100);

        let cfg = AppConfig::default();
        let test_docx_pre = PathBuf::from("target/test_mau03_barcode_pre.docx");
        export_record_to_mau03_docx(&rec, &cfg, None, &test_docx_pre, true).expect("mau03 docx pre");
        assert!(test_docx_pre.exists());

        // Kiểm tra bản đầy đủ sau khám
        let mut rec_full = rec.clone();
        rec_full.data.khong_kinh_mat_phai = "10/10".into();
        rec_full.data.khong_kinh_mat_trai = "9/10".into();
        rec_full.data.kham_mat_pl = 1;
        rec_full.data.tai_phai_noi_thuong = ">5m".into();
        rec_full.data.tai_phai_noi_tham = ">0.5m".into();
        rec_full.data.kham_tai_mui_hong_pl = 1;
        rec_full.data.noi_khoa_tuan_hoan_pl = 1;
        rec_full.data.phan_loai_sk = 1;

        let test_docx_full = PathBuf::from("target/test_mau03_barcode_full.docx");
        export_record_to_mau03_docx(&rec_full, &cfg, None, &test_docx_full, false).expect("mau03 docx full");
        assert!(test_docx_full.exists());

        let test_xlsm = PathBuf::from("target/test_import_ksk.xlsm");
        export_to_xlsm_file(&[rec], &test_xlsm).expect("xlsm export");
        assert!(test_xlsm.exists());
    }

    #[test]
    fn test_date_auto_slash() {
        use crate::ui::widgets::date_picker::format_date_input_auto_slash;
        // Gõ 2 số ngày -> tự chèn /
        assert_eq!(format_date_input_auto_slash("1", "15"), "15/");
        // Người dùng tự bấm nút / khi đang có 15/ -> không tự chèn thêm //
        assert_eq!(format_date_input_auto_slash("15/", "15//"), "15/");
        // Gõ tiếp tháng
        assert_eq!(format_date_input_auto_slash("15/", "15/0"), "15/0");
        assert_eq!(format_date_input_auto_slash("15/0", "15/08"), "15/08/");
        // Người dùng tự bấm nút / khi đang có 15/08/ -> không nhân đôi
        assert_eq!(format_date_input_auto_slash("15/08/", "15/08//"), "15/08/");
        // Gõ năm
        assert_eq!(format_date_input_auto_slash("15/08/", "15/08/1990"), "15/08/1990");
        // Backspace xóa lùi -> không ép chèn lại
        assert_eq!(format_date_input_auto_slash("15/", "15"), "15");
        assert_eq!(format_date_input_auto_slash("15", "1"), "1");
    }

    #[test]
    fn test_egui_text_state() {
        use eframe::egui;
        let ccursor = egui::text::CCursor::new(5);
        let range = egui::text_selection::CCursorRange::one(ccursor);
        assert_eq!(range.primary.index, 5);
    }

    #[test]
    fn test_catalog_ward_filter() {
        use crate::models::catalog::CatalogRecord;

        let cats = vec![
            CatalogRecord { id: 1, cat_type: "province".into(), code: "52".into(), name: "52 - Tỉnh Bình Định".into(), parent_code: "".into(), sort_order: 1, is_active: true },
            CatalogRecord { id: 2, cat_type: "province".into(), code: "64".into(), name: "64 - Tỉnh Gia Lai".into(), parent_code: "".into(), sort_order: 2, is_active: true },
            CatalogRecord { id: 3, cat_type: "ward".into(), code: "21517".into(), name: "21517 - Phường Lê Lợi".into(), parent_code: "52".into(), sort_order: 1, is_active: true },
            CatalogRecord { id: 4, cat_type: "ward".into(), code: "23992".into(), name: "23992 - Phường Hoa Lư".into(), parent_code: "64".into(), sort_order: 1, is_active: true },
        ];

        // Lọc khi truyền "52"
        let w_52: Vec<_> = cats.iter().filter(|c| c.cat_type == "ward" && c.parent_code == "52").collect();
        assert_eq!(w_52.len(), 1);
        assert_eq!(w_52[0].code, "21517");

        // Lọc khi truyền "64 - Tỉnh Gia Lai"
        let p_str = "64 - Tỉnh Gia Lai";
        let clean_p = if let Some(idx) = p_str.find(" - ") { &p_str[..idx] } else { p_str };
        let w_64: Vec<_> = cats.iter().filter(|c| c.cat_type == "ward" && c.parent_code == clean_p).collect();
        assert_eq!(w_64.len(), 1);
        assert_eq!(w_64[0].code, "23992");
    }
}
