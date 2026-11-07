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
use crate::models::{CatalogRecord, RecordItem};

pub fn resolve_doi_tuong_name(code: &str, catalogs: Option<&[CatalogRecord]>) -> String {
    let trimmed = code.trim();
    if let Some(cats) = catalogs {
        if let Some(c) = cats.iter().find(|c| c.cat_type == "priority" && c.code == trimmed) {
            let name = c.name.trim();
            if let Some(idx) = name.find(" - ") {
                return name[idx + 3..].trim().to_string();
            }
            return name.to_string();
        }
    }
    match trimmed {
        "1" => "Người cao tuổi",
        "2" => "Người khuyết tật",
        "3" => "Người thuộc hộ nghèo, cận nghèo",
        "4" => "Người có công",
        "5" => "Người mắc bệnh mạn tính",
        "6" => "Vùng đồng bào DTTS & miền núi",
        "7" => "Vùng ĐK KT-XH khó khăn",
        "8" => "Người sống tại xã đảo",
        "9" => "Người sống tại đặc khu",
        "10" => "Trẻ em mầm non",
        "11" => "Học sinh phổ thông",
        "12" => "Sinh viên",
        "13" => "Người lao động",
        "14" => "Người lao động không chính thức",
        "15" => "Người chưa có BHYT",
        "16" | "99" => "Các đối tượng khác",
        other => {
            if other.chars().any(|c| c.is_alphabetic()) {
                other
            } else {
                "Người lao động"
            }
        }
    }
    .to_string()
}

pub fn resolve_dan_toc_name(code: &str, catalogs: Option<&[CatalogRecord]>) -> String {
    let trimmed = code.trim();
    if let Some(cats) = catalogs {
        if let Some(c) = cats.iter().find(|c| c.cat_type == "ethnicity" && c.code == trimmed) {
            let name = c.name.trim();
            if let Some(idx) = name.find(" - ") {
                return name[idx + 3..].trim().to_string();
            }
            return name.to_string();
        }
    }
    match trimmed {
        "01" | "1" => "Kinh",
        "02" | "2" => "Tày",
        "03" | "3" => "Thái",
        "04" | "4" => "Mường",
        "05" | "5" => "H'Mông",
        "06" | "6" => "Dao",
        "10" => "Gia Rai",
        "11" => "Ba Na",
        "12" => "Ê Đê",
        "13" => "Chăm",
        "14" => "Sán Dìu",
        "15" => "Hoa",
        other => {
            if other.chars().any(|c| c.is_alphabetic()) {
                other
            } else {
                "Kinh"
            }
        }
    }
    .to_string()
}

pub fn resolve_occupation_name(code: &str, catalogs: Option<&[CatalogRecord]>) -> String {
    let trimmed = code.trim();
    if let Some(cats) = catalogs {
        if let Some(c) = cats.iter().find(|c| c.cat_type == "occupation" && c.code == trimmed) {
            let name = c.name.trim();
            if let Some(idx) = name.find(" - ") {
                return name[idx + 3..].trim().to_string();
            }
            return name.to_string();
        }
    }
    match trimmed {
        "22" => "Kỹ sư, Chuyên viên",
        "52" => "Nhân viên bán hàng",
        "71" => "Công nhân xây dựng",
        "83" => "Lái xe",
        "99" => "Khác",
        other => {
            if other.chars().any(|c| c.is_alphabetic()) {
                other
            } else if other.is_empty() {
                "Người lao động"
            } else {
                "Người lao động"
            }
        }
    }
    .to_string()
}

pub fn resolve_province_name(code: &str, catalogs: Option<&[CatalogRecord]>) -> String {
    let trimmed = code.trim();
    if let Some(cats) = catalogs {
        if let Some(c) = cats.iter().find(|c| c.cat_type == "province" && c.code == trimmed) {
            let name = c.name.trim();
            if let Some(idx) = name.find(" - ") {
                return name[idx + 3..].trim().to_string();
            }
            return name.to_string();
        }
    }
    match trimmed {
        "01" => "Hà Nội",
        "02" => "Hà Giang",
        "04" => "Cao Bằng",
        "06" => "Bắc Kạn",
        "08" => "Tuyên Quang",
        "10" => "Lào Cai",
        "11" => "Điện Biên",
        "12" => "Lai Châu",
        "14" => "Sơn La",
        "15" => "Yên Bái",
        "17" => "Hòa Bình",
        "19" => "Thái Nguyên",
        "20" => "Lạng Sơn",
        "22" => "Quảng Ninh",
        "24" => "Bắc Giang",
        "25" => "Phú Thọ",
        "26" => "Vĩnh Phúc",
        "27" => "Bắc Ninh",
        "30" => "Hải Dương",
        "31" => "Hải Phòng",
        "33" => "Hưng Yên",
        "34" => "Thái Bình",
        "35" => "Hà Nam",
        "36" => "Nam Định",
        "37" => "Ninh Bình",
        "38" => "Thanh Hóa",
        "40" => "Nghệ An",
        "42" => "Hà Tĩnh",
        "44" => "Quảng Bình",
        "45" => "Quảng Trị",
        "46" => "Thừa Thiên Huế",
        "48" => "Đà Nẵng",
        "49" => "Quảng Nam",
        "51" => "Quảng Ngãi",
        "52" => "Bình Định",
        "54" => "Phú Yên",
        "56" => "Khánh Hòa",
        "58" => "Ninh Thuận",
        "60" => "Bình Thuận",
        "62" => "Kon Tum",
        "64" => "Gia Lai",
        "66" => "Đắk Lắk",
        "67" => "Đắk Nông",
        "68" => "Lâm Đồng",
        "70" => "Bình Phước",
        "72" => "Tây Ninh",
        "74" => "Bình Dương",
        "75" => "Đồng Nai",
        "77" => "Bà Rịa - Vũng Tàu",
        "79" => "TP. Hồ Chí Minh",
        "80" => "Long An",
        "82" => "Tiền Giang",
        "83" => "Bến Tre",
        "84" => "Trà Vinh",
        "86" => "Vĩnh Long",
        "87" => "Đồng Tháp",
        "89" => "An Giang",
        "91" => "Kiên Giang",
        "92" => "Cần Thơ",
        "93" => "Hậu Giang",
        "94" => "Sóc Trăng",
        "95" => "Bạc Liêu",
        "96" => "Cà Mau",
        other => {
            if other.chars().any(|c| c.is_alphabetic()) {
                other
            } else {
                ""
            }
        }
    }
    .to_string()
}

pub fn resolve_ward_name(code: &str, catalogs: Option<&[CatalogRecord]>) -> String {
    let trimmed = code.trim();
    if let Some(cats) = catalogs {
        if let Some(c) = cats.iter().find(|c| c.cat_type == "ward" && c.code == trimmed) {
            let name = c.name.trim();
            if let Some(idx) = name.find(" - ") {
                return name[idx + 3..].trim().to_string();
            }
            return name.to_string();
        }
    }
    if trimmed.chars().any(|c| c.is_alphabetic()) {
        trimmed.to_string()
    } else {
        "".to_string()
    }
}

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
    // Giữ lại các ký tự ASCII hiển thị hợp lệ cho Code 128 (chữ, số, dấu chấm '.', gạch ngang '-', gạch dưới '_', gạch chéo '/', v.v.)
    let safe_code: String = clean_code
        .chars()
        .filter(|c| c.is_ascii_graphic() || *c == ' ')
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

fn run_br() -> &'static str {
    "<w:r><w:br/></w:r>"
}

fn parse_vision_val(s: &str) -> Option<i32> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    let num_str = if let Some(idx) = t.find('/') {
        &t[..idx]
    } else {
        t
    };
    if let Ok(v) = num_str.trim().parse::<i32>() {
        if (0..=10).contains(&v) {
            return Some(v);
        }
    }
    if let Ok(f) = num_str.trim().parse::<f64>() {
        let v = f.round() as i32;
        if (0..=10).contains(&v) {
            return Some(v);
        }
    }
    None
}

/// Sinh dãy 11 ô tick OMR Thị lực: 0, 1, 2, ..., 10 (/10)
fn run_omr_vision_boxes(label: &str, selected: Option<i32>, is_pre_exam: bool) -> String {
    let mut out = String::new();
    out.push_str(&run_text(label, true));
    for v in 0..=10 {
        let checked = if is_pre_exam {
            false
        } else {
            selected == Some(v)
        };
        out.push_str(&run_checkbox(checked));
        out.push_str(&run_text(&format!("{} ", v), false));
    }
    out.push_str(&run_text("/10", true));
    out
}

fn is_hearing_thuong_match(val: &str, target_idx: usize) -> bool {
    let v = val.trim().to_lowercase();
    if v.is_empty() {
        return false;
    }
    match target_idx {
        0 => v.contains(">5") || v.contains("> 5"), // >5m
        1 => (v == "5" || v == "5m" || v == "=5m" || v == "= 5m") && !v.contains(">") && !v.contains("0.5"), // 5m
        2 => v == "4" || v == "4m", // 4m
        3 => v == "3" || v == "3m", // 3m
        4 => v == "2" || v == "2m", // 2m
        5 => (v == "1" || v == "1m") && !v.contains("0.1"), // 1m
        6 => v.contains("0.5") || v.contains("0,5"), // 0.5m
        7 => v.contains("sat") || v.contains("sát"), // Sát tai
        8 => v.contains("diec") || v.contains("điếc"), // Điếc
        _ => false,
    }
}

fn is_hearing_tham_match(val: &str, target_idx: usize) -> bool {
    let v = val.trim().to_lowercase();
    if v.is_empty() {
        return false;
    }
    match target_idx {
        0 => v.contains(">0.5") || v.contains("> 0.5") || v.contains(">0,5") || v.contains("> 0,5"), // >0.5m
        1 => (v == "0.5" || v == "0.5m" || v == "0,5" || v == "0,5m" || v == "=0.5m" || v == "= 0.5m") && !v.contains(">"), // 0.5m
        2 => v.contains("0.4") || v.contains("0,4"), // 0.4m
        3 => v.contains("0.3") || v.contains("0,3"), // 0.3m
        4 => v.contains("0.2") || v.contains("0,2"), // 0.2m
        5 => (v.contains("0.1") || v.contains("0,1")) && !v.contains("0.05"), // 0.1m
        6 => v.contains("0.05") || v.contains("0,05"), // 0.05m
        7 => v.contains("sat") || v.contains("sát"), // Sát tai
        8 => v.contains("diec") || v.contains("điếc"), // Điếc
        _ => false,
    }
}

/// Sinh dãy 9 ô tick OMR Thính lực nói thường: >5m, 5m, 4m, 3m, 2m, 1m, 0.5m, Sát tai, Điếc
fn run_omr_hearing_thuong_boxes(val: &str, is_pre_exam: bool) -> String {
    let labels = [">5m", "5m", "4m", "3m", "2m", "1m", "0.5m", "Sát tai", "Điếc"];
    let mut out = String::new();
    for (i, lbl) in labels.iter().enumerate() {
        let checked = if is_pre_exam {
            false
        } else {
            is_hearing_thuong_match(val, i)
        };
        out.push_str(&run_checkbox(checked));
        out.push_str(&run_text(&format!("{} ", lbl), false));
    }
    out
}

/// Sinh dãy 9 ô tick OMR Thính lực nói thầm: >0.5m, 0.5m, 0.4m, 0.3m, 0.2m, 0.1m, 0.05m, Sát tai, Điếc
fn run_omr_hearing_tham_boxes(val: &str, is_pre_exam: bool) -> String {
    let labels = [">0.5m", "0.5m", "0.4m", "0.3m", "0.2m", "0.1m", "0.05m", "Sát tai", "Điếc"];
    let mut out = String::new();
    for (i, lbl) in labels.iter().enumerate() {
        let checked = if is_pre_exam {
            false
        } else {
            is_hearing_tham_match(val, i)
        };
        out.push_str(&run_checkbox(checked));
        out.push_str(&run_text(&format!("{} ", lbl), false));
    }
    out
}

/// Sinh dãy 5 ô tick OMR Phân loại chuyên khoa: [ I ] [ II ] [ III ] [ IV ] [ V ]
fn run_omr_classification_boxes(score: i32, is_pre_exam: bool) -> String {
    let mut out = String::new();
    let labels = ["I", "II", "III", "IV", "V"];
    for (i, lbl) in labels.iter().enumerate() {
        let checked = if is_pre_exam {
            false
        } else {
            score == (i as i32 + 1)
        };
        out.push_str(&run_checkbox(checked));
        out.push_str(&run_text(&format!(" {}   ", lbl), true));
    }
    out
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
/// - `pre_exam_only = true`: Điền sẵn Hành chính + Barcode tiếp nhận (Header căn phải), ĐỂ TRỐNG toàn bộ ô Tiền sử (☐ Có, ☐ Không) để người khám tự kê khai bằng bút, để trống `Số: ....../GKSK` và phần khám để bác sĩ ghi tay.
/// - `pre_exam_only = false`: Điền đầy đủ cả Số lưu trữ + Kết quả Thể lực, Lâm sàng, Cận lâm sàng, Kết luận.
pub fn export_record_to_mau03_docx(
    record: &RecordItem,
    config: &AppConfig,
    catalogs: Option<&[CatalogRecord]>,
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
            let merged = merge_document_xml(&s, record, ma_tiep_nhan, config, catalogs, pre_exam_only);
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
    catalogs: Option<&[CatalogRecord]>,
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

    // 6. Dân tộc & 7. Đối tượng (70291AD9) - Xuất VALUE chữ rõ ràng, không để mã số
    let dt_txt = resolve_dan_toc_name(&d.ma_dan_toc, catalogs);
    let doi_tuong_txt = resolve_doi_tuong_name(&d.doi_tuong, catalogs);
    let p6 = format!(
        "{}{}{}{}",
        run_text("6. Dân tộc: ", true),
        run_text(&dt_txt, false),
        run_text("          7. Đối tượng: ", true),
        run_text(&doi_tuong_txt, false)
    );
    xml = replace_para_by_id(&xml, "70291AD9", &p6);

    // 8. Nguồn chi trả & 9. Nhóm máu (73F7FBE7)
    let nct_txt = match d.nguon_chi_tra {
        1 => "Ngân sách Trung ương",
        2 => "Ngân sách Địa phương",
        3 => "Quỹ Bảo hiểm y tế",
        4 => "Người sử dụng lao động",
        5 => "Xã hội hóa",
        _ => "Khác",
    };
    let p8 = format!(
        "{}{}{}{}{}",
        run_text("8. Nguồn chi trả: ", true),
        run_text(nct_txt, false),
        run_text("          9. Nhóm máu: ", true),
        run_text(if d.nhom_mau.is_empty() { "……" } else { &d.nhom_mau }, false),
        run_text(&format!("     Điện thoại: {}", if d.dien_thoai.is_empty() { "………………" } else { &d.dien_thoai }), false),
    );
    xml = replace_para_by_id(&xml, "73F7FBE7", &p8);

    // 10. Nơi ở hiện tại (717F0AE8) - Trình bày đúng chuẩn: Tỉnh/thành, Phường/xã, Số nhà/thôn/xóm (Không để mã số)
    let tinh_name = resolve_province_name(&d.matinh_cu_tru, catalogs);
    let xa_name = resolve_ward_name(&d.maxa_cu_tru, catalogs);
    let tinh_str = if tinh_name.is_empty() { "……………" } else { &tinh_name };
    let xa_str = if xa_name.is_empty() { "……………" } else { &xa_name };
    let dia_chi_str = if d.dia_chi.is_empty() { "……………" } else { &d.dia_chi };

    let p10 = format!(
        "{}{}{}{}{}{}",
        run_text("10. Nơi ở hiện tại: Tỉnh/thành: ", true),
        run_text(&format!("{}    ", tinh_str), false),
        run_text("Phường/xã: ", true),
        run_text(&format!("{}    ", xa_str), false),
        run_text("Số nhà/thôn/xóm: ", true),
        run_text(dia_chi_str, false),
    );
    xml = replace_para_by_id(&xml, "717F0AE8", &p10);

    // 11. Nghề nghiệp (31D253D4) - Xuất VALUE tên nghề nghiệp, không để mã số
    let nghe_nghiep_txt = resolve_occupation_name(&d.ma_nghe_nghiep, catalogs);
    let p11 = format!("{}{}", run_text("11. Nghề nghiệp: ", true), run_text(&nghe_nghiep_txt, false));
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
    // Nếu in trước khám (pre_exam_only): để trống cả 2 ô Có / Không để người khám tự kê khai bằng bút
    let p_tsgd = if pre_exam_only {
        format!(
            "{}{}{}{}{}",
            run_text("a) ", false),
            run_checkbox(false),
            run_text(" Không       b) ", false),
            run_checkbox(false),
            run_text(" Có; Nếu “có”, đề nghị ghi cụ thể tên bệnh: ……………………………………………………", false),
        )
    } else {
        format!(
            "{}{}{}{}{}",
            run_text("a) ", false),
            run_checkbox(d.tsgd_mac_benh == 0),
            run_text(" Không       b) ", false),
            run_checkbox(d.tsgd_mac_benh == 1),
            run_text(
                &format!(
                    " Có; Nếu “có”, đề nghị ghi cụ thể tên bệnh: {}",
                    if d.tsgd_mac_benh == 1 { &d.tsgd_ma_benh } else { "" }
                ),
                false
            ),
        )
    };
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
    if pre_exam_only {
        // Trước khám: Giữ nguyên ô trống ☐ cho cả 2 cột Có và Không để người đi khám tự kê khai
        for (_val, yes_id, no_id) in tsbt_pairs {
            xml = replace_para_by_id(&xml, yes_id, &run_checkbox(false));
            xml = replace_para_by_id(&xml, no_id, &run_checkbox(false));
        }
        xml = replace_para_by_id(
            &xml,
            "54F3C0B2",
            &run_text("Bệnh khác (ghi rõ): …………………………………………………………………", false),
        );
    } else {
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
    }

    // Câu 3a & 3b (Đang điều trị bệnh & Thai sản)
    if pre_exam_only {
        xml = replace_para_by_id(&xml, "521E9DDD", &format!("{}{}", run_checkbox(false), run_text(" Không", false)));
        xml = replace_para_by_id(
            &xml,
            "7B9B041E",
            &format!("{}{}", run_checkbox(false), run_text(" Có (Nếu có, xin hãy liệt kê tên bệnh, các thuốc đang dùng và liều lượng): ……………………………………………………", false)),
        );
        xml = replace_para_by_id(&xml, "04F9F521", &format!("{}{}", run_checkbox(false), run_text(" Không", false)));
        xml = replace_para_by_id(
            &xml,
            "043D9AC7",
            &format!("{}{}", run_checkbox(false), run_text(" Có (Nếu có, xin hãy liệt kê tên bệnh, các thuốc đang dùng và liều lượng): ……………………………………………………", false)),
        );
    } else {
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
                        if d.tsbt_dang_dieu_tri_benh == 1 { &d.tsbt_ma_benh } else { "" },
                        if d.tsbt_dang_dieu_tri_benh == 1 { &d.tsbt_ten_thuoc_lieu_luong } else { "" }
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
                        if d.tsbt_thai_san == 1 { &d.tsbt_ma_benh_thai_san } else { "" },
                        if d.tsbt_thai_san == 1 { &d.tsbt_ten_thuoc_thai_san } else { "" }
                    ),
                    false
                )
            ),
        );
    }

    // Khám lâm sàng - Thị lực (11 ô OMR: 0..=10/10)
    let p_khong_kinh = format!(
        "{}{}{}{}{}",
        run_text("Kết quả khám thị lực:  Không kính: ", true),
        run_br(),
        run_omr_vision_boxes("  Mắt phải: ", parse_vision_val(&d.khong_kinh_mat_phai), pre_exam_only),
        run_br(),
        run_omr_vision_boxes("  Mắt trái:  ", parse_vision_val(&d.khong_kinh_mat_trai), pre_exam_only),
    );
    xml = replace_para_by_id(&xml, "67E0EE1A", &p_khong_kinh);

    let p_co_kinh = format!(
        "{}{}{}{}{}",
        run_text("Có kính: ", true),
        run_br(),
        run_omr_vision_boxes("  Mắt phải: ", parse_vision_val(&d.co_kinh_mat_phai), pre_exam_only),
        run_br(),
        run_omr_vision_boxes("  Mắt trái:  ", parse_vision_val(&d.co_kinh_mat_trai), pre_exam_only),
    );
    xml = replace_para_by_id(&xml, "3B1B5834", &p_co_kinh);

    let p_benh_mat = if pre_exam_only {
        run_text("Các bệnh về mắt (nếu có):…………………………………………………….", false)
    } else {
        run_text(
            &format!(
                "Các bệnh về mắt (nếu có): {}",
                if d.benh_khac_mat.trim().is_empty() { "Không" } else { &d.benh_khac_mat }
            ),
            false,
        )
    };
    xml = replace_para_by_id(&xml, "176B4C85", &p_benh_mat);

    let p_pl_mat = format!(
        "{}{}",
        run_text("Phân loại: ", true),
        run_omr_classification_boxes(d.kham_mat_pl, pre_exam_only),
    );
    xml = replace_para_by_id(&xml, "1344604A", &p_pl_mat);

    // Khám lâm sàng - Thính lực (Nói thường 9 mức, Nói thầm 9 mức OMR)
    let p_tai_trai = format!(
        "{}{}{}{}",
        run_text("Tai trái:   - Nói thường: ", true),
        run_omr_hearing_thuong_boxes(&d.tai_trai_noi_thuong, pre_exam_only),
        run_br(),
        format!(
            "{}{}",
            run_text("                - Nói thầm:    ", true),
            run_omr_hearing_tham_boxes(&d.tai_trai_noi_tham, pre_exam_only)
        ),
    );
    xml = replace_para_by_id(&xml, "7A0DA69A", &p_tai_trai);

    let p_tai_phai = format!(
        "{}{}{}{}",
        run_text("Tai phải:  - Nói thường: ", true),
        run_omr_hearing_thuong_boxes(&d.tai_phai_noi_thuong, pre_exam_only),
        run_br(),
        format!(
            "{}{}",
            run_text("                - Nói thầm:    ", true),
            run_omr_hearing_tham_boxes(&d.tai_phai_noi_tham, pre_exam_only)
        ),
    );
    xml = replace_para_by_id(&xml, "78BD496B", &p_tai_phai);

    let p_benh_tmh = if pre_exam_only {
        run_text("Các bệnh về tai mũi họng (nếu có): …………………………………………………", false)
    } else {
        run_text(
            &format!(
                "Các bệnh về tai mũi họng (nếu có): {}",
                if d.benh_khac_tai_mui_hong.trim().is_empty() { "Không" } else { &d.benh_khac_tai_mui_hong }
            ),
            false,
        )
    };
    xml = replace_para_by_id(&xml, "01C9BD32", &p_benh_tmh);

    let p_pl_tmh = format!(
        "{}{}",
        run_text("Phân loại: ", true),
        run_omr_classification_boxes(d.kham_tai_mui_hong_pl, pre_exam_only),
    );
    xml = replace_para_by_id(&xml, "28538091", &p_pl_tmh);

    // Khám lâm sàng - Răng Hàm Mặt
    let p_ham_tren = if pre_exam_only {
        run_text("Hàm trên: ……………………………………………………………………", false)
    } else {
        run_text(&format!("Hàm trên: {}", if d.ham_tren.trim().is_empty() { "Bình thường" } else { &d.ham_tren }), false)
    };
    xml = replace_para_by_id(&xml, "3DF4DB07", &p_ham_tren);

    let p_ham_duoi = if pre_exam_only {
        run_text("Hàm dưới: …………………………………………………………………..", false)
    } else {
        run_text(&format!("Hàm dưới: {}", if d.ham_duoi.trim().is_empty() { "Bình thường" } else { &d.ham_duoi }), false)
    };
    xml = replace_para_by_id(&xml, "758363D9", &p_ham_duoi);

    let p_benh_rhm = if pre_exam_only {
        run_text("Các bệnh về răng hàm mặt (nếu có):…………………………………………", false)
    } else {
        run_text(
            &format!(
                "Các bệnh về răng hàm mặt (nếu có): {}",
                if d.benh_khac_rang_ham_mat.trim().is_empty() { "Không" } else { &d.benh_khac_rang_ham_mat }
            ),
            false,
        )
    };
    xml = replace_para_by_id(&xml, "731CB55C", &p_benh_rhm);

    let p_pl_rhm = format!(
        "{}{}",
        run_text("Phân loại: ", true),
        run_omr_classification_boxes(d.kham_rang_ham_mat_pl, pre_exam_only),
    );
    xml = replace_para_by_id(&xml, "68A87DC9", &p_pl_rhm);

    // Khám lâm sàng - Ngoại khoa, Da liễu, Sản phụ khoa
    let p_ngoai_kham = if pre_exam_only {
        run_text("Ngoại khoa: ……………………………………………………………", false)
    } else {
        run_text(
            &format!(
                "Ngoại khoa: {}",
                if d.ket_qua_kham_ngoai_khoa.trim().is_empty() { "Bình thường" } else { &d.ket_qua_kham_ngoai_khoa }
            ),
            false,
        )
    };
    xml = replace_para_by_id(&xml, "7F3DA171", &p_ngoai_kham);
    let p_pl_ngoai = format!("{}{}", run_text("Phân loại: ", true), run_omr_classification_boxes(d.kham_ngoai_khoa_pl, pre_exam_only));
    xml = replace_para_by_id(&xml, "3E29774F", &p_pl_ngoai);

    let p_da_lieu_kham = if pre_exam_only {
        run_text("Da liễu: …………………………………………………………………", false)
    } else {
        run_text(
            &format!(
                "Da liễu: {}",
                if d.ket_qua_kham_da_lieu.trim().is_empty() { "Bình thường" } else { &d.ket_qua_kham_da_lieu }
            ),
            false,
        )
    };
    xml = replace_para_by_id(&xml, "5699913B", &p_da_lieu_kham);
    let p_pl_da_lieu = format!("{}{}", run_text("Phân loại: ", true), run_omr_classification_boxes(d.kham_da_lieu_pl, pre_exam_only));
    xml = replace_para_by_id(&xml, "4152D987", &p_pl_da_lieu);

    let p_san_kham = if pre_exam_only {
        run_text("Sản phụ khoa: ……………………………………………………………………….", false)
    } else {
        run_text(
            &format!(
                "Sản phụ khoa: {}",
                if d.ket_qua_kham_san_phu_khoa.trim().is_empty() { "Bình thường" } else { &d.ket_qua_kham_san_phu_khoa }
            ),
            false,
        )
    };
    xml = replace_para_by_id(&xml, "6C57BBE7", &p_san_kham);
    let p_pl_san = format!("{}{}", run_text("Phân loại: ", true), run_omr_classification_boxes(d.kham_san_phu_khoa_pl, pre_exam_only));
    xml = replace_para_by_id(&xml, "0A910D0F", &p_pl_san);

    // Khám lâm sàng - 8 chuyên khoa Nội khoa (Col 2: Khám + Phân loại OMR)
    if !pre_exam_only && !d.noi_khoa_tuan_hoan.trim().is_empty() {
        xml = replace_para_by_id(&xml, "38818360", &run_text(&d.noi_khoa_tuan_hoan, false));
    }
    xml = replace_para_by_id(&xml, "028B9A01", &run_omr_classification_boxes(d.noi_khoa_tuan_hoan_pl, pre_exam_only));

    if !pre_exam_only && !d.noi_khoa_ho_hap.trim().is_empty() {
        xml = replace_para_by_id(&xml, "4145B678", &run_text(&d.noi_khoa_ho_hap, false));
    }
    xml = replace_para_by_id(&xml, "245846FE", &run_omr_classification_boxes(d.noi_khoa_ho_hap_pl, pre_exam_only));

    if !pre_exam_only && !d.noi_khoa_tieu_hoa.trim().is_empty() {
        xml = replace_para_by_id(&xml, "781D6744", &run_text(&d.noi_khoa_tieu_hoa, false));
    }
    xml = replace_para_by_id(&xml, "7B69C077", &run_omr_classification_boxes(d.noi_khoa_tieu_hoa_pl, pre_exam_only));

    if !pre_exam_only && !d.noi_khoa_than_tn_sd.trim().is_empty() {
        xml = replace_para_by_id(&xml, "1129AE75", &run_text(&d.noi_khoa_than_tn_sd, false));
    }
    xml = replace_para_by_id(&xml, "104430BC", &run_omr_classification_boxes(d.noi_khoa_than_tn_sd_pl, pre_exam_only));

    if !pre_exam_only && !d.noi_khoa_noi_tiet.trim().is_empty() {
        xml = replace_para_by_id(&xml, "5AC60B19", &run_text(&d.noi_khoa_noi_tiet, false));
    }
    xml = replace_para_by_id(&xml, "6A5150A6", &run_omr_classification_boxes(d.noi_khoa_noi_tiet_pl, pre_exam_only));

    if !pre_exam_only && !d.noi_khoa_co_xuong_khop.trim().is_empty() {
        xml = replace_para_by_id(&xml, "114067FA", &run_text(&d.noi_khoa_co_xuong_khop, false));
    }
    xml = replace_para_by_id(&xml, "0FE1C4A1", &run_omr_classification_boxes(d.noi_khoa_co_xuong_khop_pl, pre_exam_only));

    if !pre_exam_only && !d.noi_khoa_than_kinh.trim().is_empty() {
        xml = replace_para_by_id(&xml, "72EF6D71", &run_text(&d.noi_khoa_than_kinh, false));
    }
    xml = replace_para_by_id(&xml, "72BEAC19", &run_omr_classification_boxes(d.noi_khoa_than_kinh_pl, pre_exam_only));

    if !pre_exam_only && !d.noi_khoa_tam_than.trim().is_empty() {
        xml = replace_para_by_id(&xml, "3B53584E", &run_text(&d.noi_khoa_tam_than, false));
    }
    xml = replace_para_by_id(&xml, "7AC5B481", &run_omr_classification_boxes(d.noi_khoa_tam_than_pl, pre_exam_only));

    // Phân loại thể lực (7DFEADFB) - Có 5 ô tick OMR I..V
    let p_pl_the_luc = format!(
        "{}{}",
        run_text("- Phân loại thể lực: ", true),
        run_omr_classification_boxes(d.kham_the_luc_pl, pre_exam_only)
    );
    xml = replace_para_by_id(&xml, "7DFEADFB", &p_pl_the_luc);

    // Phân loại sức khỏe chung (668D7EE4) - Có 5 ô tick OMR I..V
    let p_pl_sk_chung = format!(
        "{}{}",
        run_text("2. Phân loại sức khỏe (theo quy định của Bộ Y tế): ", true),
        run_omr_classification_boxes(d.phan_loai_sk, pre_exam_only)
    );
    xml = replace_para_by_id(&xml, "668D7EE4", &p_pl_sk_chung);

    // Nếu xuất bản đầy đủ sau khi khám (pre_exam_only == false), điền thông số Thể lực & Kết luận bệnh
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
    }

    xml
}

