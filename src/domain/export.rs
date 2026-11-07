use crate::models::RecordItem;

pub fn export_records_to_csv(records: &[RecordItem]) -> Result<String, String> {
    let mut wtr = csv::WriterBuilder::new().from_writer(vec![]);
    let headers = [
        // I. Hành chính
        "HO_TEN", "GIOI_TINH", "NGAY_SINH", "MA_DAN_TOC", "SO_CCCD", "NGAYCAP_CCCD", "NOICAP_CCCD",
        "DIA_CHI", "MATINH_CU_TRU", "MAXA_CU_TRU", "MA_NGHE_NGHIEP", "NOI_LAM_VIEC_HOC_TAP",
        "DIEN_THOAI", "NHOM_MAU", "LY_DO_VV",
        // II. Thông tin chung lần khám
        "MA_LK", "MA_CSKCB", "MA_GTIN_CSKCB", "DOI_TUONG", "NGUON_CHI_TRA", "MA_LOAI_KCB", "NGAY_VAO",
        // III. Tiền sử
        "TSGD_MAC_BENH", "TSGD_MA_BENH", "TSBT_BENH_TRONG_5_NAM_QUA", "TSBT_BENH_THAN_KINH",
        "TSBT_BENH_MAT", "TSBT_BENH_TAI", "TSBT_BENH_TIM", "TSBT_PHAU_THUAT_TIM", "TSBT_TANG_HUYET_AP",
        "TSBT_KHO_THO", "TSBT_BENH_PHOI", "TSBT_BENH_THAN", "TSBT_NGHIEN_RUOU", "TSBT_DAI_THAO_DUONG",
        "TSBT_BENH_TAM_THAN", "TSBT_MAT_Y_THUC", "TSBT_NGAT", "TSBT_BENH_TIEU_HOA", "TSBT_ROI_LOAN_GIAC_NGU",
        "TSBT_TAI_BIEN", "TSBT_BENH_COT_SONG", "TSBT_RUOU_THUONG_XUYEN", "TSBT_MA_TUY", "TSBT_BENH_KHAC",
        "TSBT_MA_BENH_KHAC", "TSBT_DANG_DIEU_TRI_BENH", "TSBT_MA_BENH", "TSBT_TEN_THUOC_LIEU_LUONG",
        "TSBT_THAI_SAN", "TSBT_MA_BENH_THAI_SAN", "TSBT_TEN_THUOC_THAI_SAN",
        // IV. Khám thể lực
        "CHIEU_CAO", "CAN_NANG", "CHI_SO_BMI", "MACH", "HUYET_AP", "KHAM_THE_LUC_PL",
        // V. Khám lâm sàng
        "NOI_KHOA_TUAN_HOAN", "NOI_KHOA_TUAN_HOAN_PL", "NOI_KHOA_HO_HAP", "NOI_KHOA_HO_HAP_PL",
        "NOI_KHOA_TIEU_HOA", "NOI_KHOA_TIEU_HOA_PL", "NOI_KHOA_THAN_TN_SD", "NOI_KHOA_THAN_TN_SD_PL",
        "NOI_KHOA_NOI_TIET", "NOI_KHOA_NOI_TIET_PL", "NOI_KHOA_CO_XUONG_KHOP", "NOI_KHOA_CO_XUONG_KHOP_PL",
        "NOI_KHOA_THAN_KINH", "NOI_KHOA_THAN_KINH_PL", "NOI_KHOA_TAM_THAN", "NOI_KHOA_TAM_THAN_PL",
        "KET_QUA_KHAM_NGOAI_KHOA", "KHAM_NGOAI_KHOA_PL", "KET_QUA_KHAM_DA_LIEU", "KHAM_DA_LIEU_PL",
        "KET_QUA_KHAM_SAN_PHU_KHOA", "KHAM_SAN_PHU_KHOA_PL", "KHONG_KINH_MAT_PHAI", "KHONG_KINH_MAT_TRAI",
        "CO_KINH_MAT_PHAI", "CO_KINH_MAT_TRAI", "BENH_KHAC_MAT", "KHAM_MAT_PL", "TAI_TRAI_NOI_THUONG",
        "TAI_TRAI_NOI_THAM", "TAI_PHAI_NOI_THUONG", "TAI_PHAI_NOI_THAM", "BENH_KHAC_TAI_MUI_HONG",
        "KHAM_TAI_MUI_HONG_PL", "HAM_TREN", "HAM_DUOI", "BENH_KHAC_RANG_HAM_MAT", "KHAM_RANG_HAM_MAT_PL",
        // VI. Kết luận
        "PHAN_LOAI_SK", "KET_LUAN_BENH", "CAC_BENH_TAT_NEU_CO",
    ];

    wtr.write_record(&headers).map_err(|e| e.to_string())?;

    for item in records {
        let d = &item.data;
        let row: Vec<String> = vec![
            d.ho_ten.clone(), d.gioi_tinh.to_string(), d.ngay_sinh.clone(), d.ma_dan_toc.clone(),
            d.so_cccd.clone(), d.ngaycap_cccd.clone(), d.noicap_cccd.clone(), d.dia_chi.clone(),
            d.matinh_cu_tru.clone(), d.maxa_cu_tru.clone(), d.ma_nghe_nghiep.clone(),
            d.noi_lam_viec_hoc_tap.clone(), d.dien_thoai.clone(), d.nhom_mau.clone(), d.ly_do_vv.clone(),
            d.ma_lk.clone(), d.ma_cskcb.clone(), d.ma_gtin_cskcb.clone(), d.doi_tuong.clone(),
            d.nguon_chi_tra.to_string(), d.ma_loai_kcb.clone(), d.ngay_vao.clone(),
            d.tsgd_mac_benh.to_string(), d.tsgd_ma_benh.clone(), d.tsbt_benh_trong_5_nam_qua.to_string(),
            d.tsbt_benh_than_kinh.to_string(), d.tsbt_benh_mat.to_string(), d.tsbt_benh_tai.to_string(),
            d.tsbt_benh_tim.to_string(), d.tsbt_phau_thuat_tim.to_string(), d.tsbt_tang_huyet_ap.to_string(),
            d.tsbt_kho_tho.to_string(), d.tsbt_benh_phoi.to_string(), d.tsbt_benh_than.to_string(),
            d.tsbt_nghien_ruou.to_string(), d.tsbt_dai_thao_duong.to_string(), d.tsbt_benh_tam_than.to_string(),
            d.tsbt_mat_y_thuc.to_string(), d.tsbt_ngat.to_string(), d.tsbt_benh_tieu_hoa.to_string(),
            d.tsbt_roi_loan_giac_ngu.to_string(), d.tsbt_tai_bien.to_string(), d.tsbt_benh_cot_song.to_string(),
            d.tsbt_ruou_thuong_xuyen.to_string(), d.tsbt_ma_tuy.to_string(), d.tsbt_benh_khac.to_string(),
            d.tsbt_ma_benh_khac.clone(), d.tsbt_dang_dieu_tri_benh.to_string(), d.tsbt_ma_benh.clone(),
            d.tsbt_ten_thuoc_lieu_luong.clone(), d.tsbt_thai_san.to_string(), d.tsbt_ma_benh_thai_san.clone(),
            d.tsbt_ten_thuoc_thai_san.clone(), d.chieu_cao.clone(), d.can_nang.clone(), d.chi_so_bmi.clone(),
            d.mach.clone(), d.huyet_ap.clone(), d.kham_the_luc_pl.to_string(), d.noi_khoa_tuan_hoan.clone(),
            d.noi_khoa_tuan_hoan_pl.to_string(), d.noi_khoa_ho_hap.clone(), d.noi_khoa_ho_hap_pl.to_string(),
            d.noi_khoa_tieu_hoa.clone(), d.noi_khoa_tieu_hoa_pl.to_string(), d.noi_khoa_than_tn_sd.clone(),
            d.noi_khoa_than_tn_sd_pl.to_string(), d.noi_khoa_noi_tiet.clone(), d.noi_khoa_noi_tiet_pl.to_string(),
            d.noi_khoa_co_xuong_khop.clone(), d.noi_khoa_co_xuong_khop_pl.to_string(),
            d.noi_khoa_than_kinh.clone(), d.noi_khoa_than_kinh_pl.to_string(), d.noi_khoa_tam_than.clone(),
            d.noi_khoa_tam_than_pl.to_string(), d.ket_qua_kham_ngoai_khoa.clone(),
            d.kham_ngoai_khoa_pl.to_string(), d.ket_qua_kham_da_lieu.clone(), d.kham_da_lieu_pl.to_string(),
            d.ket_qua_kham_san_phu_khoa.clone(), d.kham_san_phu_khoa_pl.to_string(),
            d.khong_kinh_mat_phai.clone(), d.khong_kinh_mat_trai.clone(), d.co_kinh_mat_phai.clone(),
            d.co_kinh_mat_trai.clone(), d.benh_khac_mat.clone(), d.kham_mat_pl.to_string(),
            d.tai_trai_noi_thuong.clone(), d.tai_trai_noi_tham.clone(), d.tai_phai_noi_thuong.clone(),
            d.tai_phai_noi_tham.clone(), d.benh_khac_tai_mui_hong.clone(),
            d.kham_tai_mui_hong_pl.to_string(), d.ham_tren.clone(), d.ham_duoi.clone(),
            d.benh_khac_rang_ham_mat.clone(), d.kham_rang_ham_mat_pl.to_string(),
            d.phan_loai_sk.to_string(), d.ket_luan_benh.clone(), d.cac_benh_tat_neu_co.clone(),
        ];
        wtr.write_record(&row).map_err(|e| e.to_string())?;
    }

    let bytes = wtr.into_inner().map_err(|e| e.to_string())?;
    String::from_utf8(bytes).map_err(|e| e.to_string())
}

