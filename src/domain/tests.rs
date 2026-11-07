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

        let cfg = AppConfig::default();
        let test_docx = PathBuf::from("target/test_mau03_qr.docx");
        export_record_to_mau03_docx(&rec, &cfg, &test_docx, true).expect("mau03 docx");
        assert!(test_docx.exists());

        let test_xlsm = PathBuf::from("target/test_import_ksk.xlsm");
        export_to_xlsm_file(&[rec], &test_xlsm).expect("xlsm export");
        assert!(test_xlsm.exists());
    }
}
