use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::path::Path;
use barcoders::sym::code128::Code128;
use image::{ImageBuffer, Luma};
use qrcode::QrCode;
use zip::write::FileOptions;
use zip::{ZipArchive, ZipWriter};
use crate::config::AppConfig;
use crate::domain::calc_age;
use crate::models::RecordItem;

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Tạo ảnh PNG Mã vạch 1D Code 128 cho Mã tiếp nhận (`ma_lk`)
pub fn generate_barcode_png_bytes(ma_lk: &str) -> Result<Vec<u8>, String> {
    let clean_code = ma_lk.trim();
    let safe_code: String = clean_code
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    let final_code = if safe_code.is_empty() {
        "LK000001".to_string()
    } else {
        safe_code
    };

    // Chuẩn Code 128 Set B (ký tự bắt đầu: \u{0181})
    let payload = format!("\u{0181}{}", final_code);
    let bc = Code128::new(&payload).map_err(|e| format!("Lỗi tạo Barcode Code 128: {:?}", e))?;
    let encoded: Vec<u8> = bc.encode();

    let module_width = 2u32;
    let quiet_zone_modules = 15u32;
    let total_modules = encoded.len() as u32 + quiet_zone_modules * 2;
    let img_width = total_modules * module_width;
    let img_height = 50u32;

    let mut img: ImageBuffer<Luma<u8>, Vec<u8>> =
        ImageBuffer::from_pixel(img_width, img_height, Luma([255u8]));

    for (i, &bit) in encoded.iter().enumerate() {
        if bit == 1 {
            let start_x = (i as u32 + quiet_zone_modules) * module_width;
            for x in start_x..(start_x + module_width) {
                for y in 0..img_height {
                    img.put_pixel(x, y, Luma([0u8]));
                }
            }
        }
    }

    let mut png_bytes = Vec::new();
    let dyn_img = image::DynamicImage::ImageLuma8(img);
    dyn_img
        .write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(png_bytes)
}

/// Tạo ảnh PNG Mã QR cho hồ sơ khám (`KSK|<ma_y_te>|<so_cccd>|<ho_ten>`)
pub fn generate_qr_png_bytes(ma_y_te: &str, so_cccd: &str, ho_ten: &str) -> Result<Vec<u8>, String> {
    let payload = format!("KSK|{}|{}|{}", ma_y_te.trim(), so_cccd.trim(), ho_ten.trim());
    let code = QrCode::new(payload.as_bytes()).map_err(|e| e.to_string())?;
    let matrix = code.to_colors();
    let width = code.width() as u32;
    let scale = 6u32;
    let quiet = 2u32;
    let img_size = (width + quiet * 2) * scale;

    let mut img: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::from_pixel(img_size, img_size, Luma([255u8]));
    for y in 0..width {
        for x in 0..width {
            let is_dark = matrix[(y * width + x) as usize] == qrcode::Color::Dark;
            if is_dark {
                let px_x = (x + quiet) * scale;
                let px_y = (y + quiet) * scale;
                for dy in 0..scale {
                    for dx in 0..scale {
                        img.put_pixel(px_x + dx, px_y + dy, Luma([0u8]));
                    }
                }
            }
        }
    }

    let mut png_bytes = Vec::new();
    let dyn_img = image::DynamicImage::ImageLuma8(img);
    dyn_img
        .write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(png_bytes)
}

/// Thay thế nội dung text của một đoạn văn `<w:p ... w14:paraId="PARA_ID" ...>...</w:p>`
/// giữ nguyên `<w:pPr>...</w:pPr>`.
fn replace_para_by_id(doc_xml: &str, para_id: &str, new_runs_xml: &str) -> String {
    let needle = format!("w14:paraId=\"{}\"", para_id);
    let Some(id_pos) = doc_xml.find(&needle) else {
        return doc_xml.to_string();
    };
    let Some(p_start) = doc_xml[..id_pos].rfind("<w:p ") else {
        return doc_xml.to_string();
    };
    let Some(p_open_end_rel) = doc_xml[p_start..].find('>') else {
        return doc_xml.to_string();
    };
    let p_open_end = p_start + p_open_end_rel + 1;

    let Some(p_end_rel) = doc_xml[p_open_end..].find("</w:p>") else {
        return doc_xml.to_string();
    };
    let p_end = p_open_end + p_end_rel;

    let inner = &doc_xml[p_open_end..p_end];
    let ppr = if let (Some(s), Some(e)) = (inner.find("<w:pPr>"), inner.find("</w:pPr>")) {
        &inner[s..e + "</w:pPr>".len()]
    } else {
        ""
    };

    let mut out = String::with_capacity(doc_xml.len() + new_runs_xml.len());
    out.push_str(&doc_xml[..p_open_end]);
    out.push_str(ppr);
    out.push_str(new_runs_xml);
    out.push_str(&doc_xml[p_end..]);
    out
}

fn run_text(text: &str, bold: bool) -> String {
    if bold {
        format!(
            "<w:r><w:rPr><w:b/><w:bCs/></w:rPr><w:t xml:space=\"preserve\">{}</w:t></w:r>",
            xml_escape(text)
        )
    } else {
        format!(
            "<w:r><w:t xml:space=\"preserve\">{}</w:t></w:r>",
            xml_escape(text)
        )
    }
}

fn run_checkbox(checked: bool) -> String {
    let sym = if checked { "☑" } else { "☐" };
    format!(
        "<w:r><w:rPr><w:rFonts w:ascii=\"Segoe UI Symbol\" w:hAnsi=\"Segoe UI Symbol\" w:cs=\"Segoe UI Symbol\"/></w:rPr><w:t>{}</w:t></w:r>",
        sym
    )
}

#[allow(dead_code)]
fn inline_qr_drawing_xml(rel_id: &str) -> String {
    // 720000 EMUs = ~2.0 cm x 2.0 cm crisp QR code in header
    format!(
        "<w:r><w:drawing><wp:inline distT=\"0\" distB=\"0\" distL=\"0\" distR=\"0\">\
        <wp:extent cx=\"760000\" cy=\"760000\"/>\
        <wp:effectExtent l=\"0\" t=\"0\" r=\"0\" b=\"0\"/>\
        <wp:docPr id=\"99\" name=\"QR_KSK\"/>\
        <wp:cNvGraphicFramePr><a:graphicFrameLocks xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" noChangeAspect=\"1\"/></wp:cNvGraphicFramePr>\
        <a:graphic xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\">\
        <a:graphicData uri=\"http://schemas.openxmlformats.org/drawingml/2006/picture\">\
        <pic:pic xmlns:pic=\"http://schemas.openxmlformats.org/drawingml/2006/picture\">\
        <pic:nvPicPr><pic:cNvPr id=\"99\" name=\"qr.png\"/><pic:cNvPicPr/></pic:nvPicPr>\
        <pic:blipFill><a:blip r:embed=\"{}\"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill>\
        <pic:spPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"760000\" cy=\"760000\"/></a:xfrm><a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom></pic:spPr>\
        </pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r>",
        rel_id
    )
}

/// Xuất phiếu khám Mẫu 03 (.docx) cho 1 hồ sơ:
/// - `pre_exam_only = true`: Điền sẵn Hành chính + Tiền sử + Barcode tiếp nhận (Header căn phải), để trống `Số: ....../GKSK` và phần khám để bác sĩ ghi tay.
/// - `pre_exam_only = false`: Điền đầy đủ cả Số lưu trữ + Kết quả Thể lực, Lâm sàng, Cận lâm sàng, Kết luận.
pub fn export_record_to_mau03_docx(
    record: &RecordItem,
    config: &AppConfig,
    output_path: &Path,
    pre_exam_only: bool,
) -> Result<(), String> {
    let template_path = Path::new("DATA/Mẫu 03 Giấy KSK và KSK định kỳ dùng cho người từ đủ 18 tuổi trở lên.docx");
    if !template_path.exists() {
        return Err("Không tìm thấy file mẫu DATA/Mẫu 03 Giấy KSK và KSK định kỳ dùng cho người từ đủ 18 tuổi trở lên.docx".into());
    }

    let d = &record.data;
    let ma_tiep_nhan = if !d.ma_lk.trim().is_empty() {
        d.ma_lk.trim()
    } else if !d.ma_y_te.trim().is_empty() {
        d.ma_y_te.trim()
    } else {
        &format!("LK{:06}", record.id.max(1))
    };

    let barcode_png = generate_barcode_png_bytes(ma_tiep_nhan)?;

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

        if name == "[Content_Types].xml" {
            let mut s = String::from_utf8_lossy(&buf).to_string();
            if !s.contains("Extension=\"png\"") {
                s = s.replace(
                    "<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">",
                    "<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"><Default Extension=\"png\" ContentType=\"image/png\"/>",
                );
            }
            if !s.contains("PartName=\"/word/header1.xml\"") {
                s = s.replace(
                    "</Types>",
                    "<Override PartName=\"/word/header1.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml\"/></Types>",
                );
            }
            writer.write_all(s.as_bytes()).map_err(|e| e.to_string())?;
        } else if name == "word/_rels/document.xml.rels" {
            let mut s = String::from_utf8_lossy(&buf).to_string();
            if !s.contains("Target=\"header1.xml\"") {
                s = s.replace(
                    "</Relationships>",
                    "<Relationship Id=\"rIdHeader1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/header\" Target=\"header1.xml\"/></Relationships>",
                );
            }
            writer.write_all(s.as_bytes()).map_err(|e| e.to_string())?;
        } else if name == "word/document.xml" {
            let s = String::from_utf8_lossy(&buf).to_string();
            let merged = merge_document_xml(&s, record, ma_tiep_nhan, config, pre_exam_only);
            writer.write_all(merged.as_bytes()).map_err(|e| e.to_string())?;
        } else {
            writer.write_all(&buf).map_err(|e| e.to_string())?;
        }
    }

    // 1. Ghi ảnh Barcode 1D Code 128
    writer.start_file("word/media/barcode_tiepnhan.png", options).map_err(|e| e.to_string())?;
    writer.write_all(&barcode_png).map_err(|e| e.to_string())?;

    // 2. Ghi relationship cho Header
    writer.start_file("word/_rels/header1.xml.rels", options).map_err(|e| e.to_string())?;
    let header_rels = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
<Relationship Id=\"rIdBarcode\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/image\" Target=\"media/barcode_tiepnhan.png\"/>\
</Relationships>";
    writer.write_all(header_rels.as_bytes()).map_err(|e| e.to_string())?;

    // 3. Ghi file word/header1.xml (Barcode 1D + Mã tiếp nhận căn phải)
    writer.start_file("word/header1.xml", options).map_err(|e| e.to_string())?;
    let header_xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
<w:hdr xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\" \
xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" \
xmlns:wp=\"http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing\">\
<w:p><w:pPr><w:pStyle w:val=\"Header\"/><w:spacing w:before=\"0\" w:after=\"0\"/><w:jc w:val=\"right\"/></w:pPr>\
<w:r><w:drawing><wp:inline xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" xmlns:pic=\"http://schemas.openxmlformats.org/drawingml/2006/picture\">\
<wp:extent cx=\"1645920\" cy=\"320040\"/>\
<wp:docPr id=\"1\" name=\"Barcode\"/>\
<wp:cNvGraphicFramePr><a:graphicFrameLocks noChangeAspect=\"1\"/></wp:cNvGraphicFramePr>\
<a:graphic><a:graphicData uri=\"http://schemas.openxmlformats.org/drawingml/2006/picture\">\
<pic:pic><pic:nvPicPr><pic:cNvPr id=\"0\" name=\"barcode.png\"/><pic:cNvPicPr/></pic:nvPicPr>\
<pic:blipFill><a:blip r:embed=\"rIdBarcode\"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill>\
<pic:spPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"1645920\" cy=\"320040\"/></a:xfrm><a:prstGeom prst=\"rect\"/></pic:spPr>\
</pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>\
<w:p><w:pPr><w:spacing w:before=\"0\" w:after=\"0\"/><w:jc w:val=\"right\"/></w:pPr>\
<w:r><w:rPr><w:rFonts w:ascii=\"Times New Roman\" w:hAnsi=\"Times New Roman\"/><w:sz w:val=\"17\"/><w:szCs w:val=\"17\"/></w:rPr>\
<w:t xml:space=\"preserve\">Mã tiếp nhận: {}</w:t></w:r></w:p></w:hdr>",
        xml_escape(ma_tiep_nhan)
    );
    writer.write_all(header_xml.as_bytes()).map_err(|e| e.to_string())?;

    writer.finish().map_err(|e| e.to_string())?;
    Ok(())
}

fn merge_document_xml(
    doc_xml: &str,
    record: &RecordItem,
    _ma_tiep_nhan: &str,
    config: &AppConfig,
    pre_exam_only: bool,
) -> String {
    let d = &record.data;
    let mut xml = doc_xml.to_string();

    // Đảm bảo sectPr trỏ đến Header chứa Barcode tiếp nhận
    if let Some(pos) = xml.rfind("<w:sectPr") {
        if let Some(open_rel) = xml[pos..].find('>') {
            let open_end = pos + open_rel + 1;
            if !xml.contains("w:headerReference") {
                xml.insert_str(open_end, "<w:headerReference w:type=\"default\" r:id=\"rIdHeader1\"/>");
            }
        }
    }

    // 0. Dòng Số lưu trữ (paraId="56EA532B") - Căn chỉnh chuẩn công văn, không bị chèn hình QR méo dòng
    let so_gksk_txt = if pre_exam_only || d.so_luu_tru.trim().is_empty() {
        format!("Số: ……………/{}", config.archive_suffix)
    } else {
        format!("Số: {}", d.so_luu_tru)
    };
    let header_no_runs = format!(
        "<w:r><w:tab/></w:r>{}",
        run_text(&so_gksk_txt, true)
    );
    xml = replace_para_by_id(&xml, "56EA532B", &header_no_runs);

    // 1. Họ và tên (76857D19)
    let p1 = format!(
        "{}{}{}",
        run_text("1. ", false),
        run_text("Họ và tên (viết chữ in hoa): ", true),
        run_text(&d.ho_ten.to_uppercase(), true)
    );
    xml = replace_para_by_id(&xml, "76857D19", &p1);

    // 2. Giới tính (7A1D182C)
    let p2 = format!(
        "{}{}{}{}{}",
        run_text("2. Giới tính: ", true),
        run_checkbox(d.gioi_tinh == 1),
        run_text(" Nam     ", false),
        run_checkbox(d.gioi_tinh == 2),
        run_text(" Nữ", false),
    );
    xml = replace_para_by_id(&xml, "7A1D182C", &p2);

    // 3. Ngày sinh & Tuổi (493545FF)
    let age_str = calc_age(&d.ngay_sinh).map(|a| a.to_string()).unwrap_or_else(|| "....".into());
    let p3 = format!(
        "{}{}{}{}",
        run_text("3. Ngày tháng năm sinh: ", true),
        run_text(if d.ngay_sinh.is_empty() { "……/……/…………" } else { &d.ngay_sinh }, false),
        run_text("   (Tuổi: ", false),
        run_text(&format!("{})", age_str), true)
    );
    xml = replace_para_by_id(&xml, "493545FF", &p3);

    // 4. Số CCCD (1BCB24C6)
    let p4 = format!(
        "{}{}",
        run_text("4. Số CCCD/Hộ chiếu/Mã định danh: ", true),
        run_text(&d.so_cccd, true)
    );
    xml = replace_para_by_id(&xml, "1BCB24C6", &p4);

    // 5. Ngày cấp & Nơi cấp (5B4E914D)
    let p5 = format!(
        "{}{}{}{}",
        run_text("5. Cấp ngày: ", true),
        run_text(if d.ngaycap_cccd.is_empty() { "……/……/…………" } else { &d.ngaycap_cccd }, false),
        run_text("     Nơi cấp: ", true),
        run_text(&d.noicap_cccd, false)
    );
    xml = replace_para_by_id(&xml, "5B4E914D", &p5);

    // 6. Dân tộc & 7. Đối tượng (70291AD9)
    let dt_txt = if d.ma_dan_toc == "01" || d.ma_dan_toc == "1" { "Kinh" } else { &d.ma_dan_toc };
    let p6 = format!(
        "{}{}{}{}",
        run_text("6. Dân tộc: ", true),
        run_text(dt_txt, false),
        run_text("          7. Đối tượng: ", true),
        run_text(&d.doi_tuong, false)
    );
    xml = replace_para_by_id(&xml, "70291AD9", &p6);

    // 8. Nguồn chi trả & 9. Nhóm máu (73F7FBE7)
    let nct_txt = match d.nguon_chi_tra {
        1 => "Ngân sách TW",
        2 => "Ngân sách ĐP",
        3 => "Quỹ BHYT",
        4 => "Người sử dụng lao động",
        5 => "Xã hội hóa",
        _ => "Khác",
    };
    let p8 = format!(
        "{}{}{}{}{}",
        run_text("8. Nguồn chi trả: ", true),
        run_text(nct_txt, false),
        run_text("          9. Nhóm máu: ", true),
        run_text(&d.nhom_mau, false),
        run_text(&format!("     Điện thoại: {}", d.dien_thoai), false),
    );
    xml = replace_para_by_id(&xml, "73F7FBE7", &p8);

    // 10. Nơi ở hiện tại (717F0AE8)
    let p10 = format!(
        "{}{}",
        run_text("10. Nơi ở hiện tại: ", true),
        run_text(
            &format!(
                "{} (Mã Tỉnh: {}, Mã Xã: {})",
                if d.dia_chi.is_empty() { "............................" } else { &d.dia_chi },
                d.matinh_cu_tru,
                d.maxa_cu_tru
            ),
            false
        )
    );
    xml = replace_para_by_id(&xml, "717F0AE8", &p10);

    // 11. Nghề nghiệp (31D253D4)
    let p11 = format!("{}{}", run_text("11. Nghề nghiệp: ", true), run_text(&d.ma_nghe_nghiep, false));
    xml = replace_para_by_id(&xml, "31D253D4", &p11);

    // 12. Nơi làm việc, học tập (6CE6B511)
    let nlv = if d.bo_phan.trim().is_empty() {
        d.noi_lam_viec_hoc_tap.clone()
    } else {
        format!("{} ({})", d.noi_lam_viec_hoc_tap, d.bo_phan)
    };
    let p12 = format!("{}{}", run_text("12. Nơi làm việc, học tập: ", true), run_text(&nlv, true));
    xml = replace_para_by_id(&xml, "6CE6B511", &p12);

    // 13. Lý do khám sức khỏe (69BD5132)
    let p13 = format!("{}{}", run_text("13. Lý do khám sức khỏe: ", true), run_text(&d.ly_do_vv, false));
    xml = replace_para_by_id(&xml, "69BD5132", &p13);

    // Tiền sử gia đình (78FED6C5)
    let p_tsgd = format!(
        "{}{}{}{}{}",
        run_text("a) ", false),
        run_checkbox(d.tsgd_mac_benh == 0),
        run_text(" Không       b) ", false),
        run_checkbox(d.tsgd_mac_benh == 1),
        run_text(&format!(" Có; Nếu “có”, đề nghị ghi cụ thể tên bệnh: {}", d.tsgd_ma_benh), false),
    );
    xml = replace_para_by_id(&xml, "78FED6C5", &p_tsgd);

    // Tiền sử bản thân 22 mục (Cột Có / Không)
    let tsbt_pairs = [
        (d.tsbt_benh_trong_5_nam_qua, "4241F7A3", "77C60C2B"),
        (d.tsbt_benh_than_kinh, "138A1E4C", "34B8FDFC"),
        (d.tsbt_benh_mat, "76ED44D5", "07768E20"),
        (d.tsbt_benh_tai, "25AA0C29", "07A14921"),
        (d.tsbt_benh_tim, "5128823E", "1A196EA0"),
        (d.tsbt_phau_thuat_tim, "65F7C8EE", "6DE44373"),
        (d.tsbt_tang_huyet_ap, "2CE6B889", "2A422A1A"),
        (d.tsbt_kho_tho, "1A6ACF63", "0EA2BF71"),
        (d.tsbt_benh_phoi, "5911F751", "2149B4D2"),
        (d.tsbt_benh_than, "7BD3BC7D", "04F2F004"),
        (d.tsbt_nghien_ruou, "5861C9A9", "3A822AE4"),
        (d.tsbt_dai_thao_duong, "7571611D", "19A79AD6"),
        (d.tsbt_benh_tam_than, "2C391F77", "2678E017"),
        (d.tsbt_mat_y_thuc, "0811960D", "25B625C6"),
        (d.tsbt_ngat, "1D1D41A0", "066B5279"),
        (d.tsbt_benh_tieu_hoa, "18BBB76B", "747852B1"),
        (d.tsbt_roi_loan_giac_ngu, "58D4EFA1", "7BF9D378"),
        (d.tsbt_tai_bien, "075961AE", "0B04F5AE"),
        (d.tsbt_benh_cot_song, "63A2B9EA", "74C344B6"),
        (d.tsbt_ruou_thuong_xuyen, "0CE88B9B", "5040673D"),
        (d.tsbt_ma_tuy, "605F59F1", "450A6F62"),
        (d.tsbt_benh_khac, "263462E7", "42C30385"),
    ];
    for (val, yes_id, no_id) in tsbt_pairs {
        xml = replace_para_by_id(&xml, yes_id, &run_checkbox(val == 1));
        xml = replace_para_by_id(&xml, no_id, &run_checkbox(val == 0));
    }

    if !d.tsbt_ma_benh_khac.trim().is_empty() {
        xml = replace_para_by_id(
            &xml,
            "54F3C0B2",
            &run_text(&format!("Bệnh khác (ghi rõ): {}", d.tsbt_ma_benh_khac), false),
        );
    }

    // Câu 3a & 3b
    xml = replace_para_by_id(&xml, "521E9DDD", &format!("{}{}", run_checkbox(d.tsbt_dang_dieu_tri_benh == 0), run_text(" Không", false)));
    xml = replace_para_by_id(
        &xml,
        "7B9B041E",
        &format!(
            "{}{}",
            run_checkbox(d.tsbt_dang_dieu_tri_benh == 1),
            run_text(
                &format!(
                    " Có (Tên bệnh, thuốc đang dùng): {} {}",
                    d.tsbt_ma_benh, d.tsbt_ten_thuoc_lieu_luong
                ),
                false
            )
        ),
    );
    xml = replace_para_by_id(&xml, "04F9F521", &format!("{}{}", run_checkbox(d.tsbt_thai_san == 0), run_text(" Không", false)));
    xml = replace_para_by_id(
        &xml,
        "043D9AC7",
        &format!(
            "{}{}",
            run_checkbox(d.tsbt_thai_san == 1),
            run_text(
                &format!(
                    " Có (Tên bệnh thai sản, thuốc): {} {}",
                    d.tsbt_ma_benh_thai_san, d.tsbt_ten_thuoc_thai_san
                ),
                false
            )
        ),
    );

    // Nếu xuất bản đầy đủ sau khi khám (pre_exam_only == false), điền cả Thể lực, Lâm sàng, CLS, Kết luận
    if !pre_exam_only {
        xml = replace_para_by_id(
            &xml,
            "0CFFD94B",
            &run_text(
                &format!(
                    "- Chiều cao: {} cm;   - Cân nặng: {} kg;   - Chỉ số BMI: {}",
                    d.chieu_cao, d.can_nang, d.chi_so_bmi
                ),
                false,
            ),
        );
        xml = replace_para_by_id(
            &xml,
            "4E3FBB3B",
            &run_text(
                &format!("- Mạch: {} lần/phút;   - Huyết áp: {} mmHg", d.mach, d.huyet_ap),
                false,
            ),
        );
        xml = replace_para_by_id(
            &xml,
            "7DFEADFB",
            &run_text(&format!("- Phân loại thể lực: Loại {}", d.kham_the_luc_pl), true),
        );
        xml = replace_para_by_id(
            &xml,
            "71DF01BD",
            &run_text(
                &format!(
                    "1. Tình trạng sức khỏe; mắc các bệnh, tật (nếu có): {} ({})",
                    d.ket_luan_benh,
                    if d.cac_benh_tat_neu_co.trim().is_empty() { "Không" } else { &d.cac_benh_tat_neu_co }
                ),
                true,
            ),
        );
        xml = replace_para_by_id(
            &xml,
            "668D7EE4",
            &run_text(
                &format!("2. Phân loại sức khỏe (theo quy định của Bộ Y tế): Loại {}", d.phan_loai_sk),
                true,
            ),
        );
    }

    xml
}

