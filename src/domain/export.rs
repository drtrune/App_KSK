use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use zip::write::FileOptions;
use zip::{ZipArchive, ZipWriter};
use crate::models::RecordItem;

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn pl_display(pl: i32) -> &'static str {
    match pl {
        1 => "Loại 1",
        2 => "Loại 2",
        3 => "Loại 3",
        4 => "Loại 4",
        5 => "Loại 5",
        _ => "",
    }
}

fn co_khong_display(val: i32) -> &'static str {
    if val == 1 { "Có" } else { "Không" }
}

fn doi_tuong_display(code: &str) -> &'static str {
    match code.trim() {
        "1" => "Người cao tuổi",
        "2" => "Người khuyết tật",
        "3" => "Người thuộc hộ nghèo, cận nghèo",
        "4" => "Người có công",
        "5" => "Người mắc bệnh mạn tính",
        "6" => "Người sống tại vùng đồng bào dân tộc thiểu số và miền núi",
        "7" => "Người sống tại vùng có điều kiện kinh tế - xã hội khó khăn, đặc biệt khó khăn",
        "8" => "Người sống tại xã đảo",
        "9" => "Người sống tại đặc khu",
        "10" => "Trẻ em trong cơ sở giáo dục mầm non",
        "11" => "Học sinh trong các cơ sở giáo dục phổ thông",
        "12" => "Sinh viên",
        "13" => "Người lao động",
        "14" => "Người lao động không chính thức",
        "15" => "Người chưa có Bảo hiểm y tế",
        _ => "Các đối tượng khác",
    }
}

fn nguon_chi_tra_display(code: i32) -> &'static str {
    match code {
        1 => "Ngân sách Trung ương",
        2 => "Ngân sách Địa phương",
        3 => "Quỹ Bảo hiểm y tế",
        4 => "Người sử dụng lao động",
        5 => "Xã hội hóa",
        _ => "Khác",
    }
}

/// Xuất CSV chuẩn 100% các cột biến của sheet `Trên 18` trong `Import_KSK_Tren 18.xlsm`
pub fn export_records_to_csv(records: &[RecordItem]) -> Result<String, String> {
    let mut wtr = csv::WriterBuilder::new().from_writer(vec![]);
    let headers = [
        "TT", "MA_CSKCB", "MA_GTIN_CSKCB", "NGAY_VAO", "HO_TEN", "GIOI_TINH", "NGAY_SINH",
        "MA_DAN_TOC", "SO_CCCD", "NGAYCAP_CCCD", "NOICAP_CCCD", "MATINH_CU_TRU", "MAXA_CU_TRU",
        "DIA_CHI", "MA_NGHE_NGHIEP", "NHOM_MAU", "DOI_TUONG", "NGUON_CHI_TRA",
        "NOI_LAM_VIEC_HOC_TAP", "DIEN_THOAI", "LY_DO_VV", "MA_LOAI_KCB",
        "TSGD_MAC_BENH", "TSGD_MA_BENH",
        "TSBT_BENH_TRONG_5_NAM_QUA", "TSBT_BENH_THAN_KINH", "TSBT_BENH_MAT", "TSBT_BENH_TAI",
        "TSBT_BENH_TIM", "TSBT_PHAU_THUAT_TIM", "TSBT_TANG_HUYET_AP", "TSBT_KHO_THO",
        "TSBT_BENH_PHOI", "TSBT_BENH_THAN", "TSBT_NGHIEN_RUOU", "TSBT_DAI_THAO_DUONG",
        "TSBT_BENH_TAM_THAN", "TSBT_MAT_Y_THUC", "TSBT_NGAT", "TSBT_BENH_TIEU_HOA",
        "TSBT_ROI_LOAN_GIAC_NGU", "TSBT_TAI_BIEN", "TSBT_BENH_COT_SONG", "TSBT_RUOU_THUONG_XUYEN",
        "TSBT_MA_TUY", "TSBT_BENH_KHAC", "TSBT_MA_BENH_KHAC",
        "TSBT_DANG_DIEU_TRI_BENH", "TSBT_MA_BENH", "TSBT_TEN_THUOC_LIEU_LUONG",
        "TSBT_THAI_SAN", "TSBT_MA_BENH_THAI_SAN", "TSBT_TEN_THUOC_THAI_SAN",
        "CHIEU_CAO", "CAN_NANG", "CHI_SO_BMI", "MACH", "HUYET_AP", "KHAM_THE_LUC_PL",
        "NOI_KHOA_TUAN_HOAN", "NOI_KHOA_TUAN_HOAN_PL",
        "NOI_KHOA_HO_HAP", "NOI_KHOA_HO_HAP_PL",
        "NOI_KHOA_TIEU_HOA", "NOI_KHOA_TIEU_HOA_PL",
        "NOI_KHOA_THAN_TN_SD", "NOI_KHOA_THAN_TIETNIEU_PL",
        "NOI_KHOA_NOI_TIET", "NOI_KHOA_NOI_TIET_PL",
        "NOI_KHOA_CO_XUONG_KHOP", "NOI_KHOA_CO_XUONG_KHOP_PL",
        "NOI_KHOA_THAN_KINH", "NOI_KHOA_THAN_KINH_PL",
        "NOI_KHOA_TAM_THAN", "NOI_KHOA_TAM_THAN_PL",
        "KET_QUA_KHAM_NGOAI_KHOA", "KHAM_NGOAI_KHOA_PL",
        "KET_QUA_KHAM_DA_LIEU", "KHAM_DA_LIEU_PL",
        "KET_QUA_KHAM_SAN_PHU_KHOA", "KHAM_SAN_PHU_KHOA_PL",
        "KHONG_KINH_MAT_PHAI", "KHONG_KINH_MAT_TRAI", "CO_KINH_MAT_PHAI", "CO_KINH_MAT_TRAI",
        "BENH_KHAC_MAT", "KHAM_MAT_PL",
        "TAI_TRAI_NOI_THUONG", "TAI_TRAI_NOI_THAM", "TAI_PHAI_NOI_THUONG", "TAI_PHAI_NOI_THAM",
        "BENH_KHAC_TAI_MUI_HONG", "KHAM_TAI_MUI_HONG_PL",
        "HAM_TREN", "HAM_DUOI", "BENH_KHAC_RANG_HAM_MAT", "KHAM_RANG_HAM_MAT_PL",
        "PHAN_LOAI_SK", "KET_LUAN_BENH", "CAC_BENH_TAT_NEU_CO",
        "SO_LUU_TRU", "MA_Y_TE",
    ];

    wtr.write_record(&headers).map_err(|e| e.to_string())?;

    for (idx, item) in records.iter().enumerate() {
        let d = &item.data;
        let row: Vec<String> = vec![
            (idx + 1).to_string(),
            d.ma_cskcb.clone(), d.ma_gtin_cskcb.clone(), d.ngay_vao.clone(),
            d.ho_ten.to_uppercase(), d.gioi_tinh.to_string(), d.ngay_sinh.clone(),
            d.ma_dan_toc.clone(), d.so_cccd.clone(), d.ngaycap_cccd.clone(), d.noicap_cccd.clone(),
            d.matinh_cu_tru.clone(), d.maxa_cu_tru.clone(), d.dia_chi.clone(),
            d.ma_nghe_nghiep.clone(), d.nhom_mau.clone(), d.doi_tuong.clone(),
            d.nguon_chi_tra.to_string(), d.noi_lam_viec_hoc_tap.clone(), d.dien_thoai.clone(),
            d.ly_do_vv.clone(), d.ma_loai_kcb.clone(),
            d.tsgd_mac_benh.to_string(), d.tsgd_ma_benh.clone(),
            d.tsbt_benh_trong_5_nam_qua.to_string(), d.tsbt_benh_than_kinh.to_string(),
            d.tsbt_benh_mat.to_string(), d.tsbt_benh_tai.to_string(),
            d.tsbt_benh_tim.to_string(), d.tsbt_phau_thuat_tim.to_string(),
            d.tsbt_tang_huyet_ap.to_string(), d.tsbt_kho_tho.to_string(),
            d.tsbt_benh_phoi.to_string(), d.tsbt_benh_than.to_string(),
            d.tsbt_nghien_ruou.to_string(), d.tsbt_dai_thao_duong.to_string(),
            d.tsbt_benh_tam_than.to_string(), d.tsbt_mat_y_thuc.to_string(),
            d.tsbt_ngat.to_string(), d.tsbt_benh_tieu_hoa.to_string(),
            d.tsbt_roi_loan_giac_ngu.to_string(), d.tsbt_tai_bien.to_string(),
            d.tsbt_benh_cot_song.to_string(), d.tsbt_ruou_thuong_xuyen.to_string(),
            d.tsbt_ma_tuy.to_string(), d.tsbt_benh_khac.to_string(), d.tsbt_ma_benh_khac.clone(),
            d.tsbt_dang_dieu_tri_benh.to_string(), d.tsbt_ma_benh.clone(), d.tsbt_ten_thuoc_lieu_luong.clone(),
            d.tsbt_thai_san.to_string(), d.tsbt_ma_benh_thai_san.clone(), d.tsbt_ten_thuoc_thai_san.clone(),
            d.chieu_cao.clone(), d.can_nang.clone(), d.chi_so_bmi.clone(),
            d.mach.clone(), d.huyet_ap.clone(), d.kham_the_luc_pl.to_string(),
            d.noi_khoa_tuan_hoan.clone(), d.noi_khoa_tuan_hoan_pl.to_string(),
            d.noi_khoa_ho_hap.clone(), d.noi_khoa_ho_hap_pl.to_string(),
            d.noi_khoa_tieu_hoa.clone(), d.noi_khoa_tieu_hoa_pl.to_string(),
            d.noi_khoa_than_tn_sd.clone(), d.noi_khoa_than_tn_sd_pl.to_string(),
            d.noi_khoa_noi_tiet.clone(), d.noi_khoa_noi_tiet_pl.to_string(),
            d.noi_khoa_co_xuong_khop.clone(), d.noi_khoa_co_xuong_khop_pl.to_string(),
            d.noi_khoa_than_kinh.clone(), d.noi_khoa_than_kinh_pl.to_string(),
            d.noi_khoa_tam_than.clone(), d.noi_khoa_tam_than_pl.to_string(),
            d.ket_qua_kham_ngoai_khoa.clone(), d.kham_ngoai_khoa_pl.to_string(),
            d.ket_qua_kham_da_lieu.clone(), d.kham_da_lieu_pl.to_string(),
            d.ket_qua_kham_san_phu_khoa.clone(), d.kham_san_phu_khoa_pl.to_string(),
            d.khong_kinh_mat_phai.clone(), d.khong_kinh_mat_trai.clone(),
            d.co_kinh_mat_phai.clone(), d.co_kinh_mat_trai.clone(),
            d.benh_khac_mat.clone(), d.kham_mat_pl.to_string(),
            d.tai_trai_noi_thuong.clone(), d.tai_trai_noi_tham.clone(),
            d.tai_phai_noi_thuong.clone(), d.tai_phai_noi_tham.clone(),
            d.benh_khac_tai_mui_hong.clone(), d.kham_tai_mui_hong_pl.to_string(),
            d.ham_tren.clone(), d.ham_duoi.clone(),
            d.benh_khac_rang_ham_mat.clone(), d.kham_rang_ham_mat_pl.to_string(),
            d.phan_loai_sk.to_string(), d.ket_luan_benh.clone(), d.cac_benh_tat_neu_co.clone(),
            d.so_luu_tru.clone(), d.ma_y_te.clone(),
        ];
        wtr.write_record(&row).map_err(|e| e.to_string())?;
    }

    let bytes = wtr.into_inner().map_err(|e| e.to_string())?;
    String::from_utf8(bytes).map_err(|e| e.to_string())
}

/// Xuất CSV cho sheet `DM_CanLamSang` (9 cột chuẩn)
pub fn export_cls_to_csv(records: &[RecordItem]) -> Result<String, String> {
    let mut wtr = csv::WriterBuilder::new().from_writer(vec![]);
    wtr.write_record(["TT", "HO_TEN", "SO_CCCD", "MA_CHI_SO", "MA_DICH_VU", "DON_VI_DO", "GIA_TRI", "MO_TA", "KET_LUAN"])
        .map_err(|e| e.to_string())?;

    let mut tt = 1;
    for rec in records {
        let mut d = rec.data.clone();
        d.sync_quick_cls_to_list();
        for cls in &d.cls_list {
            wtr.write_record([
                &tt.to_string(),
                &d.ho_ten.to_uppercase(),
                &d.so_cccd,
                &cls.ma_chi_so,
                &cls.ma_dich_vu,
                &cls.don_vi_do,
                &cls.gia_tri,
                &cls.mo_ta,
                &cls.ket_luan,
            ])
            .map_err(|e| e.to_string())?;
            tt += 1;
        }
    }

    let bytes = wtr.into_inner().map_err(|e| e.to_string())?;
    String::from_utf8(bytes).map_err(|e| e.to_string())
}

/// Xuất trực tiếp XML theo chuẩn QĐ 3176/QĐ-BYT (Bao gồm cả DSACH_CLS)
pub fn export_records_to_xml(records: &[RecordItem]) -> String {
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<DANH_SACH_HOSO_KSK>\n");

    for (idx, item) in records.iter().enumerate() {
        let mut d = item.data.clone();
        d.sync_quick_cls_to_list();

        out.push_str(&format!("  <HOSO_KSK STT=\"{}\">\n", idx + 1));
        out.push_str(&format!("    <SO_LUU_TRU>{}</SO_LUU_TRU>\n", xml_escape(&d.so_luu_tru)));
        out.push_str(&format!("    <MA_Y_TE>{}</MA_Y_TE>\n", xml_escape(&d.ma_y_te)));
        out.push_str(&format!("    <MA_LK>{}</MA_LK>\n", xml_escape(&d.ma_lk)));
        out.push_str(&format!("    <MA_CSKCB>{}</MA_CSKCB>\n", xml_escape(&d.ma_cskcb)));
        out.push_str(&format!("    <MA_GTIN_CSKCB>{}</MA_GTIN_CSKCB>\n", xml_escape(&d.ma_gtin_cskcb)));
        out.push_str(&format!("    <NGAY_VAO>{}</NGAY_VAO>\n", xml_escape(&d.ngay_vao)));
        out.push_str(&format!("    <HO_TEN>{}</HO_TEN>\n", xml_escape(&d.ho_ten.to_uppercase())));
        out.push_str(&format!("    <GIOI_TINH>{}</GIOI_TINH>\n", d.gioi_tinh));
        out.push_str(&format!("    <NGAY_SINH>{}</NGAY_SINH>\n", xml_escape(&d.ngay_sinh)));
        out.push_str(&format!("    <MA_DAN_TOC>{}</MA_DAN_TOC>\n", xml_escape(&d.ma_dan_toc)));
        out.push_str(&format!("    <SO_CCCD>{}</SO_CCCD>\n", xml_escape(&d.so_cccd)));
        out.push_str(&format!("    <NGAYCAP_CCCD>{}</NGAYCAP_CCCD>\n", xml_escape(&d.ngaycap_cccd)));
        out.push_str(&format!("    <NOICAP_CCCD>{}</NOICAP_CCCD>\n", xml_escape(&d.noicap_cccd)));
        out.push_str(&format!("    <MATINH_CU_TRU>{}</MATINH_CU_TRU>\n", xml_escape(&d.matinh_cu_tru)));
        out.push_str(&format!("    <MAXA_CU_TRU>{}</MAXA_CU_TRU>\n", xml_escape(&d.maxa_cu_tru)));
        out.push_str(&format!("    <DIA_CHI>{}</DIA_CHI>\n", xml_escape(&d.dia_chi)));
        out.push_str(&format!("    <MA_NGHE_NGHIEP>{}</MA_NGHE_NGHIEP>\n", xml_escape(&d.ma_nghe_nghiep)));
        out.push_str(&format!("    <NHOM_MAU>{}</NHOM_MAU>\n", xml_escape(&d.nhom_mau)));
        out.push_str(&format!("    <DOI_TUONG>{}</DOI_TUONG>\n", xml_escape(&d.doi_tuong)));
        out.push_str(&format!("    <NGUON_CHI_TRA>{}</NGUON_CHI_TRA>\n", d.nguon_chi_tra));
        out.push_str(&format!("    <NOI_LAM_VIEC_HOC_TAP>{}</NOI_LAM_VIEC_HOC_TAP>\n", xml_escape(&d.noi_lam_viec_hoc_tap)));
        out.push_str(&format!("    <DIEN_THOAI>{}</DIEN_THOAI>\n", xml_escape(&d.dien_thoai)));
        out.push_str(&format!("    <LY_DO_VV>{}</LY_DO_VV>\n", xml_escape(&d.ly_do_vv)));
        out.push_str(&format!("    <MA_LOAI_KCB>{}</MA_LOAI_KCB>\n", xml_escape(&d.ma_loai_kcb)));
        out.push_str(&format!("    <TSGD_MAC_BENH>{}</TSGD_MAC_BENH>\n", d.tsgd_mac_benh));
        out.push_str(&format!("    <TSGD_MA_BENH>{}</TSGD_MA_BENH>\n", xml_escape(&d.tsgd_ma_benh)));
        out.push_str(&format!("    <TSBT_BENH_TRONG_5_NAM_QUA>{}</TSBT_BENH_TRONG_5_NAM_QUA>\n", d.tsbt_benh_trong_5_nam_qua));
        out.push_str(&format!("    <TSBT_BENH_THAN_KINH>{}</TSBT_BENH_THAN_KINH>\n", d.tsbt_benh_than_kinh));
        out.push_str(&format!("    <TSBT_BENH_MAT>{}</TSBT_BENH_MAT>\n", d.tsbt_benh_mat));
        out.push_str(&format!("    <TSBT_BENH_TAI>{}</TSBT_BENH_TAI>\n", d.tsbt_benh_tai));
        out.push_str(&format!("    <TSBT_BENH_TIM>{}</TSBT_BENH_TIM>\n", d.tsbt_benh_tim));
        out.push_str(&format!("    <TSBT_PHAU_THUAT_TIM>{}</TSBT_PHAU_THUAT_TIM>\n", d.tsbt_phau_thuat_tim));
        out.push_str(&format!("    <TSBT_TANG_HUYET_AP>{}</TSBT_TANG_HUYET_AP>\n", d.tsbt_tang_huyet_ap));
        out.push_str(&format!("    <TSBT_KHO_THO>{}</TSBT_KHO_THO>\n", d.tsbt_kho_tho));
        out.push_str(&format!("    <TSBT_BENH_PHOI>{}</TSBT_BENH_PHOI>\n", d.tsbt_benh_phoi));
        out.push_str(&format!("    <TSBT_BENH_THAN>{}</TSBT_BENH_THAN>\n", d.tsbt_benh_than));
        out.push_str(&format!("    <TSBT_NGHIEN_RUOU>{}</TSBT_NGHIEN_RUOU>\n", d.tsbt_nghien_ruou));
        out.push_str(&format!("    <TSBT_DAI_THAO_DUONG>{}</TSBT_DAI_THAO_DUONG>\n", d.tsbt_dai_thao_duong));
        out.push_str(&format!("    <TSBT_BENH_TAM_THAN>{}</TSBT_BENH_TAM_THAN>\n", d.tsbt_benh_tam_than));
        out.push_str(&format!("    <TSBT_MAT_Y_THUC>{}</TSBT_MAT_Y_THUC>\n", d.tsbt_mat_y_thuc));
        out.push_str(&format!("    <TSBT_NGAT>{}</TSBT_NGAT>\n", d.tsbt_ngat));
        out.push_str(&format!("    <TSBT_BENH_TIEU_HOA>{}</TSBT_BENH_TIEU_HOA>\n", d.tsbt_benh_tieu_hoa));
        out.push_str(&format!("    <TSBT_ROI_LOAN_GIAC_NGU>{}</TSBT_ROI_LOAN_GIAC_NGU>\n", d.tsbt_roi_loan_giac_ngu));
        out.push_str(&format!("    <TSBT_TAI_BIEN>{}</TSBT_TAI_BIEN>\n", d.tsbt_tai_bien));
        out.push_str(&format!("    <TSBT_BENH_COT_SONG>{}</TSBT_BENH_COT_SONG>\n", d.tsbt_benh_cot_song));
        out.push_str(&format!("    <TSBT_RUOU_THUONG_XUYEN>{}</TSBT_RUOU_THUONG_XUYEN>\n", d.tsbt_ruou_thuong_xuyen));
        out.push_str(&format!("    <TSBT_MA_TUY>{}</TSBT_MA_TUY>\n", d.tsbt_ma_tuy));
        out.push_str(&format!("    <TSBT_BENH_KHAC>{}</TSBT_BENH_KHAC>\n", d.tsbt_benh_khac));
        out.push_str(&format!("    <TSBT_MA_BENH_KHAC>{}</TSBT_MA_BENH_KHAC>\n", xml_escape(&d.tsbt_ma_benh_khac)));
        out.push_str(&format!("    <TSBT_DANG_DIEU_TRI_BENH>{}</TSBT_DANG_DIEU_TRI_BENH>\n", d.tsbt_dang_dieu_tri_benh));
        out.push_str(&format!("    <TSBT_MA_BENH>{}</TSBT_MA_BENH>\n", xml_escape(&d.tsbt_ma_benh)));
        out.push_str(&format!("    <TSBT_TEN_THUOC_LIEU_LUONG>{}</TSBT_TEN_THUOC_LIEU_LUONG>\n", xml_escape(&d.tsbt_ten_thuoc_lieu_luong)));
        out.push_str(&format!("    <TSBT_THAI_SAN>{}</TSBT_THAI_SAN>\n", d.tsbt_thai_san));
        out.push_str(&format!("    <TSBT_MA_BENH_THAI_SAN>{}</TSBT_MA_BENH_THAI_SAN>\n", xml_escape(&d.tsbt_ma_benh_thai_san)));
        out.push_str(&format!("    <TSBT_TEN_THUOC_THAI_SAN>{}</TSBT_TEN_THUOC_THAI_SAN>\n", xml_escape(&d.tsbt_ten_thuoc_thai_san)));
        out.push_str(&format!("    <CHIEU_CAO>{}</CHIEU_CAO>\n", xml_escape(&d.chieu_cao)));
        out.push_str(&format!("    <CAN_NANG>{}</CAN_NANG>\n", xml_escape(&d.can_nang)));
        out.push_str(&format!("    <CHI_SO_BMI>{}</CHI_SO_BMI>\n", xml_escape(&d.chi_so_bmi)));
        out.push_str(&format!("    <MACH>{}</MACH>\n", xml_escape(&d.mach)));
        out.push_str(&format!("    <HUYET_AP>{}</HUYET_AP>\n", xml_escape(&d.huyet_ap)));
        out.push_str(&format!("    <KHAM_THE_LUC_PL>{}</KHAM_THE_LUC_PL>\n", d.kham_the_luc_pl));
        out.push_str(&format!("    <NOI_KHOA_TUAN_HOAN>{}</NOI_KHOA_TUAN_HOAN>\n", xml_escape(&d.noi_khoa_tuan_hoan)));
        out.push_str(&format!("    <NOI_KHOA_TUAN_HOAN_PL>{}</NOI_KHOA_TUAN_HOAN_PL>\n", d.noi_khoa_tuan_hoan_pl));
        out.push_str(&format!("    <NOI_KHOA_HO_HAP>{}</NOI_KHOA_HO_HAP>\n", xml_escape(&d.noi_khoa_ho_hap)));
        out.push_str(&format!("    <NOI_KHOA_HO_HAP_PL>{}</NOI_KHOA_HO_HAP_PL>\n", d.noi_khoa_ho_hap_pl));
        out.push_str(&format!("    <NOI_KHOA_TIEU_HOA>{}</NOI_KHOA_TIEU_HOA>\n", xml_escape(&d.noi_khoa_tieu_hoa)));
        out.push_str(&format!("    <NOI_KHOA_TIEU_HOA_PL>{}</NOI_KHOA_TIEU_HOA_PL>\n", d.noi_khoa_tieu_hoa_pl));
        out.push_str(&format!("    <NOI_KHOA_THAN_TN_SD>{}</NOI_KHOA_THAN_TN_SD>\n", xml_escape(&d.noi_khoa_than_tn_sd)));
        out.push_str(&format!("    <NOI_KHOA_THAN_TIETNIEU_PL>{}</NOI_KHOA_THAN_TIETNIEU_PL>\n", d.noi_khoa_than_tn_sd_pl));
        out.push_str(&format!("    <NOI_KHOA_NOI_TIET>{}</NOI_KHOA_NOI_TIET>\n", xml_escape(&d.noi_khoa_noi_tiet)));
        out.push_str(&format!("    <NOI_KHOA_NOI_TIET_PL>{}</NOI_KHOA_NOI_TIET_PL>\n", d.noi_khoa_noi_tiet_pl));
        out.push_str(&format!("    <NOI_KHOA_CO_XUONG_KHOP>{}</NOI_KHOA_CO_XUONG_KHOP>\n", xml_escape(&d.noi_khoa_co_xuong_khop)));
        out.push_str(&format!("    <NOI_KHOA_CO_XUONG_KHOP_PL>{}</NOI_KHOA_CO_XUONG_KHOP_PL>\n", d.noi_khoa_co_xuong_khop_pl));
        out.push_str(&format!("    <NOI_KHOA_THAN_KINH>{}</NOI_KHOA_THAN_KINH>\n", xml_escape(&d.noi_khoa_than_kinh)));
        out.push_str(&format!("    <NOI_KHOA_THAN_KINH_PL>{}</NOI_KHOA_THAN_KINH_PL>\n", d.noi_khoa_than_kinh_pl));
        out.push_str(&format!("    <NOI_KHOA_TAM_THAN>{}</NOI_KHOA_TAM_THAN>\n", xml_escape(&d.noi_khoa_tam_than)));
        out.push_str(&format!("    <NOI_KHOA_TAM_THAN_PL>{}</NOI_KHOA_TAM_THAN_PL>\n", d.noi_khoa_tam_than_pl));
        out.push_str(&format!("    <KET_QUA_KHAM_NGOAI_KHOA>{}</KET_QUA_KHAM_NGOAI_KHOA>\n", xml_escape(&d.ket_qua_kham_ngoai_khoa)));
        out.push_str(&format!("    <KHAM_NGOAI_KHOA_PL>{}</KHAM_NGOAI_KHOA_PL>\n", d.kham_ngoai_khoa_pl));
        out.push_str(&format!("    <KET_QUA_KHAM_DA_LIEU>{}</KET_QUA_KHAM_DA_LIEU>\n", xml_escape(&d.ket_qua_kham_da_lieu)));
        out.push_str(&format!("    <KHAM_DA_LIEU_PL>{}</KHAM_DA_LIEU_PL>\n", d.kham_da_lieu_pl));
        out.push_str(&format!("    <KET_QUA_KHAM_SAN_PHU_KHOA>{}</KET_QUA_KHAM_SAN_PHU_KHOA>\n", xml_escape(&d.ket_qua_kham_san_phu_khoa)));
        out.push_str(&format!("    <KHAM_SAN_PHU_KHOA_PL>{}</KHAM_SAN_PHU_KHOA_PL>\n", d.kham_san_phu_khoa_pl));
        out.push_str(&format!("    <KHONG_KINH_MAT_PHAI>{}</KHONG_KINH_MAT_PHAI>\n", xml_escape(&d.khong_kinh_mat_phai)));
        out.push_str(&format!("    <KHONG_KINH_MAT_TRAI>{}</KHONG_KINH_MAT_TRAI>\n", xml_escape(&d.khong_kinh_mat_trai)));
        out.push_str(&format!("    <CO_KINH_MAT_PHAI>{}</CO_KINH_MAT_PHAI>\n", xml_escape(&d.co_kinh_mat_phai)));
        out.push_str(&format!("    <CO_KINH_MAT_TRAI>{}</CO_KINH_MAT_TRAI>\n", xml_escape(&d.co_kinh_mat_trai)));
        out.push_str(&format!("    <BENH_KHAC_MAT>{}</BENH_KHAC_MAT>\n", xml_escape(&d.benh_khac_mat)));
        out.push_str(&format!("    <KHAM_MAT_PL>{}</KHAM_MAT_PL>\n", d.kham_mat_pl));
        out.push_str(&format!("    <TAI_TRAI_NOI_THUONG>{}</TAI_TRAI_NOI_THUONG>\n", xml_escape(&d.tai_trai_noi_thuong)));
        out.push_str(&format!("    <TAI_TRAI_NOI_THAM>{}</TAI_TRAI_NOI_THAM>\n", xml_escape(&d.tai_trai_noi_tham)));
        out.push_str(&format!("    <TAI_PHAI_NOI_THUONG>{}</TAI_PHAI_NOI_THUONG>\n", xml_escape(&d.tai_phai_noi_thuong)));
        out.push_str(&format!("    <TAI_PHAI_NOI_THAM>{}</TAI_PHAI_NOI_THAM>\n", xml_escape(&d.tai_phai_noi_tham)));
        out.push_str(&format!("    <BENH_KHAC_TAI_MUI_HONG>{}</BENH_KHAC_TAI_MUI_HONG>\n", xml_escape(&d.benh_khac_tai_mui_hong)));
        out.push_str(&format!("    <KHAM_TAI_MUI_HONG_PL>{}</KHAM_TAI_MUI_HONG_PL>\n", d.kham_tai_mui_hong_pl));
        out.push_str(&format!("    <HAM_TREN>{}</HAM_TREN>\n", xml_escape(&d.ham_tren)));
        out.push_str(&format!("    <HAM_DUOI>{}</HAM_DUOI>\n", xml_escape(&d.ham_duoi)));
        out.push_str(&format!("    <BENH_KHAC_RANG_HAM_MAT>{}</BENH_KHAC_RANG_HAM_MAT>\n", xml_escape(&d.benh_khac_rang_ham_mat)));
        out.push_str(&format!("    <KHAM_RANG_HAM_MAT_PL>{}</KHAM_RANG_HAM_MAT_PL>\n", d.kham_rang_ham_mat_pl));

        out.push_str("    <DSACH_CLS>\n");
        for cls in &d.cls_list {
            out.push_str("      <CHI_TIET_CLS>\n");
            out.push_str(&format!("        <MA_CHI_SO>{}</MA_CHI_SO>\n", xml_escape(&cls.ma_chi_so)));
            out.push_str(&format!("        <MA_DICH_VU>{}</MA_DICH_VU>\n", xml_escape(&cls.ma_dich_vu)));
            out.push_str(&format!("        <DON_VI_DO>{}</DON_VI_DO>\n", xml_escape(&cls.don_vi_do)));
            out.push_str(&format!("        <GIA_TRI>{}</GIA_TRI>\n", xml_escape(&cls.gia_tri)));
            out.push_str(&format!("        <MO_TA>{}</MO_TA>\n", xml_escape(&cls.mo_ta)));
            out.push_str(&format!("        <KET_LUAN>{}</KET_LUAN>\n", xml_escape(&cls.ket_luan)));
            out.push_str("      </CHI_TIET_CLS>\n");
        }
        out.push_str("    </DSACH_CLS>\n");

        out.push_str(&format!("    <PHAN_LOAI_SK>{}</PHAN_LOAI_SK>\n", d.phan_loai_sk));
        out.push_str(&format!("    <KET_LUAN_BENH>{}</KET_LUAN_BENH>\n", xml_escape(&d.ket_luan_benh)));
        out.push_str(&format!("    <CAC_BENH_TAT_NEU_CO>{}</CAC_BENH_TAT_NEU_CO>\n", xml_escape(&d.cac_benh_tat_neu_co)));
        out.push_str("  </HOSO_KSK>\n");
    }

    out.push_str("</DANH_SACH_HOSO_KSK>\n");
    out
}

/// Ghi trực tiếp dữ liệu vào bản sao của file mẫu `DATA/Import_KSK_Tren 18.xlsm`
/// (Giữ nguyên Macro VBA, công thức và cả 2 sheet `Trên 18` + `DM_CanLamSang`)
pub fn export_to_xlsm_file(records: &[RecordItem], output_path: &Path) -> Result<(), String> {
    let template_path = Path::new("DATA/Import_KSK_Tren 18.xlsm");
    if !template_path.exists() {
        return Err("Không tìm thấy file mẫu DATA/Import_KSK_Tren 18.xlsm".into());
    }

    let src_file = File::open(template_path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(src_file).map_err(|e| e.to_string())?;

    let dst_file = File::create(output_path).map_err(|e| e.to_string())?;
    let mut writer = ZipWriter::new(dst_file);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().to_string();
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf).map_err(|e| e.to_string())?;

        writer.start_file(&name, options).map_err(|e| e.to_string())?;

        if name == "xl/worksheets/sheet6.xml" {
            let xml_str = String::from_utf8_lossy(&buf);
            let updated = inject_sheet6_rows(&xml_str, records);
            writer.write_all(updated.as_bytes()).map_err(|e| e.to_string())?;
        } else if name == "xl/worksheets/sheet7.xml" {
            let xml_str = String::from_utf8_lossy(&buf);
            let updated = inject_sheet7_rows(&xml_str, records);
            writer.write_all(updated.as_bytes()).map_err(|e| e.to_string())?;
        } else {
            writer.write_all(&buf).map_err(|e| e.to_string())?;
        }
    }

    writer.finish().map_err(|e| e.to_string())?;
    Ok(())
}

fn cell_inline(col: &str, row: usize, val: &str) -> String {
    if val.is_empty() {
        format!("<c r=\"{}{}\"/>", col, row)
    } else {
        format!(
            "<c r=\"{}{}\" t=\"inlineStr\"><is><t>{}</t></is></c>",
            col,
            row,
            xml_escape(val)
        )
    }
}

fn inject_sheet6_rows(orig_xml: &str, records: &[RecordItem]) -> String {
    // Keep rows 1, 2, 3 and replace rows >= 4 before </sheetData>
    let Some(sd_start) = orig_xml.find("<sheetData>") else {
        return orig_xml.to_string();
    };
    let Some(sd_end) = orig_xml.find("</sheetData>") else {
        return orig_xml.to_string();
    };
    let sheet_data = &orig_xml[sd_start..sd_end];
    // Find end of row 3
    let prefix_end = if let Some(r4_pos) = sheet_data.find("<row r=\"4\"") {
        sd_start + r4_pos
    } else {
        sd_end
    };

    let mut out = String::with_capacity(orig_xml.len() + records.len() * 2048);
    out.push_str(&orig_xml[..prefix_end]);

    for (idx, item) in records.iter().enumerate() {
        let r = idx + 4;
        let d = &item.data;
        let gt_txt = match d.gioi_tinh { 2 => "Nữ", 3 => "Chưa xác định", _ => "Nam" };
        out.push_str(&format!("<row r=\"{}\">", r));
        let cols: &[(&str, String)] = &[
            ("A", (idx + 1).to_string()),
            ("B", d.ma_cskcb.clone()),
            ("C", d.ma_gtin_cskcb.clone()),
            ("D", d.ngay_vao.clone()),
            ("E", d.ho_ten.to_uppercase()),
            ("F", gt_txt.into()),
            ("G", d.ngay_sinh.clone()),
            ("H", d.ma_dan_toc.clone()),
            ("I", d.so_cccd.clone()),
            ("J", d.ngaycap_cccd.clone()),
            ("K", d.noicap_cccd.clone()),
            ("L", d.matinh_cu_tru.clone()),
            ("M", d.matinh_cu_tru.clone()),
            ("N", d.maxa_cu_tru.clone()),
            ("O", d.maxa_cu_tru.clone()),
            ("P", d.dia_chi.clone()),
            ("Q", d.ma_nghe_nghiep.clone()),
            ("R", d.nhom_mau.clone()),
            ("S", doi_tuong_display(&d.doi_tuong).into()),
            ("T", nguon_chi_tra_display(d.nguon_chi_tra).into()),
            ("U", d.noi_lam_viec_hoc_tap.clone()),
            ("V", d.dien_thoai.clone()),
            ("W", d.ly_do_vv.clone()),
            ("X", d.ma_loai_kcb.clone()),
            ("Y", co_khong_display(d.tsgd_mac_benh).into()),
            ("Z", d.tsgd_ma_benh.clone()),
            ("AA", co_khong_display(d.tsbt_benh_trong_5_nam_qua).into()),
            ("AB", co_khong_display(d.tsbt_benh_than_kinh).into()),
            ("AC", co_khong_display(d.tsbt_benh_mat).into()),
            ("AD", co_khong_display(d.tsbt_benh_tai).into()),
            ("AE", co_khong_display(d.tsbt_benh_tim).into()),
            ("AF", co_khong_display(d.tsbt_phau_thuat_tim).into()),
            ("AG", co_khong_display(d.tsbt_tang_huyet_ap).into()),
            ("AH", co_khong_display(d.tsbt_kho_tho).into()),
            ("AI", co_khong_display(d.tsbt_benh_phoi).into()),
            ("AJ", co_khong_display(d.tsbt_benh_than).into()),
            ("AK", co_khong_display(d.tsbt_nghien_ruou).into()),
            ("AL", co_khong_display(d.tsbt_dai_thao_duong).into()),
            ("AM", co_khong_display(d.tsbt_benh_tam_than).into()),
            ("AN", co_khong_display(d.tsbt_mat_y_thuc).into()),
            ("AO", co_khong_display(d.tsbt_ngat).into()),
            ("AP", co_khong_display(d.tsbt_benh_tieu_hoa).into()),
            ("AQ", co_khong_display(d.tsbt_roi_loan_giac_ngu).into()),
            ("AR", co_khong_display(d.tsbt_tai_bien).into()),
            ("AS", co_khong_display(d.tsbt_benh_cot_song).into()),
            ("AT", co_khong_display(d.tsbt_ruou_thuong_xuyen).into()),
            ("AU", co_khong_display(d.tsbt_ma_tuy).into()),
            ("AV", co_khong_display(d.tsbt_benh_khac).into()),
            ("AW", d.tsbt_ma_benh_khac.clone()),
            ("AX", co_khong_display(d.tsbt_dang_dieu_tri_benh).into()),
            ("AY", d.tsbt_ma_benh.clone()),
            ("AZ", d.tsbt_ten_thuoc_lieu_luong.clone()),
            ("BA", co_khong_display(d.tsbt_thai_san).into()),
            ("BB", d.tsbt_ma_benh_thai_san.clone()),
            ("BC", d.tsbt_ten_thuoc_thai_san.clone()),
            ("BD", d.chieu_cao.clone()),
            ("BE", d.can_nang.clone()),
            ("BF", d.chi_so_bmi.clone()),
            ("BG", d.mach.clone()),
            ("BH", d.huyet_ap.clone()),
            ("BI", pl_display(d.kham_the_luc_pl).into()),
            ("BJ", d.noi_khoa_tuan_hoan.clone()),
            ("BK", pl_display(d.noi_khoa_tuan_hoan_pl).into()),
            ("BL", d.noi_khoa_ho_hap.clone()),
            ("BM", pl_display(d.noi_khoa_ho_hap_pl).into()),
            ("BN", d.noi_khoa_tieu_hoa.clone()),
            ("BO", pl_display(d.noi_khoa_tieu_hoa_pl).into()),
            ("BP", d.noi_khoa_than_tn_sd.clone()),
            ("BQ", pl_display(d.noi_khoa_than_tn_sd_pl).into()),
            ("BR", d.noi_khoa_noi_tiet.clone()),
            ("BS", pl_display(d.noi_khoa_noi_tiet_pl).into()),
            ("BT", d.noi_khoa_co_xuong_khop.clone()),
            ("BU", pl_display(d.noi_khoa_co_xuong_khop_pl).into()),
            ("BV", d.noi_khoa_than_kinh.clone()),
            ("BW", pl_display(d.noi_khoa_than_kinh_pl).into()),
            ("BX", d.noi_khoa_tam_than.clone()),
            ("BY", pl_display(d.noi_khoa_tam_than_pl).into()),
            ("BZ", d.ket_qua_kham_ngoai_khoa.clone()),
            ("CA", pl_display(d.kham_ngoai_khoa_pl).into()),
            ("CB", d.ket_qua_kham_da_lieu.clone()),
            ("CC", pl_display(d.kham_da_lieu_pl).into()),
            ("CD", d.ket_qua_kham_san_phu_khoa.clone()),
            ("CE", pl_display(d.kham_san_phu_khoa_pl).into()),
            ("CF", d.khong_kinh_mat_phai.clone()),
            ("CG", d.khong_kinh_mat_trai.clone()),
            ("CH", d.co_kinh_mat_phai.clone()),
            ("CI", d.co_kinh_mat_trai.clone()),
            ("CJ", d.benh_khac_mat.clone()),
            ("CK", pl_display(d.kham_mat_pl).into()),
            ("CL", d.tai_trai_noi_thuong.clone()),
            ("CM", d.tai_trai_noi_tham.clone()),
            ("CN", d.tai_phai_noi_thuong.clone()),
            ("CO", d.tai_phai_noi_tham.clone()),
            ("CP", d.benh_khac_tai_mui_hong.clone()),
            ("CQ", pl_display(d.kham_tai_mui_hong_pl).into()),
            ("CR", d.ham_tren.clone()),
            ("CS", d.ham_duoi.clone()),
            ("CT", d.benh_khac_rang_ham_mat.clone()),
            ("CU", pl_display(d.kham_rang_ham_mat_pl).into()),
            ("CW", pl_display(d.phan_loai_sk).into()),
            ("CX", d.ket_luan_benh.clone()),
            ("CY", d.cac_benh_tat_neu_co.clone()),
        ];
        for (c, v) in cols {
            out.push_str(&cell_inline(c, r, v));
        }
        out.push_str("</row>");
    }

    out.push_str(&orig_xml[sd_end..]);
    out
}

fn inject_sheet7_rows(orig_xml: &str, records: &[RecordItem]) -> String {
    let Some(sd_start) = orig_xml.find("<sheetData>") else {
        return orig_xml.to_string();
    };
    let Some(sd_end) = orig_xml.find("</sheetData>") else {
        return orig_xml.to_string();
    };
    let sheet_data = &orig_xml[sd_start..sd_end];
    let prefix_end = if let Some(r3_pos) = sheet_data.find("<row r=\"3\"") {
        sd_start + r3_pos
    } else {
        sd_end
    };

    let mut out = String::with_capacity(orig_xml.len() + records.len() * 512);
    out.push_str(&orig_xml[..prefix_end]);

    let mut row_num = 3usize;
    let mut tt = 1usize;
    for rec in records {
        let mut d = rec.data.clone();
        d.sync_quick_cls_to_list();
        for cls in &d.cls_list {
            out.push_str(&format!("<row r=\"{}\">", row_num));
            out.push_str(&cell_inline("A", row_num, &tt.to_string()));
            out.push_str(&cell_inline("B", row_num, &d.ho_ten.to_uppercase()));
            out.push_str(&cell_inline("C", row_num, &d.so_cccd));
            out.push_str(&cell_inline("D", row_num, &cls.ma_chi_so));
            out.push_str(&cell_inline("E", row_num, &cls.ma_dich_vu));
            out.push_str(&cell_inline("F", row_num, &cls.don_vi_do));
            out.push_str(&cell_inline("G", row_num, &cls.gia_tri));
            out.push_str(&cell_inline("H", row_num, &cls.mo_ta));
            out.push_str(&cell_inline("I", row_num, &cls.ket_luan));
            out.push_str("</row>");
            row_num += 1;
            tt += 1;
        }
    }

    out.push_str(&orig_xml[sd_end..]);
    out
}
