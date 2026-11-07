use crate::models::QdClassification;

pub fn calc_bmi(chieu_cao_cm: &str, can_nang_kg: &str) -> Option<f64> {
    let h: f64 = chieu_cao_cm.trim().replace(',', ".").parse().ok()?;
    let w: f64 = can_nang_kg.trim().replace(',', ".").parse().ok()?;
    if h > 30.0 && w > 5.0 {
        let hm = h / 100.0;
        Some(w / (hm * hm))
    } else {
        None
    }
}

// TÁCH VÀ PHÂN TÍCH HUYẾT ÁP (SYSTOLIC / DIASTOLIC)
pub fn parse_blood_pressure(s: &str) -> (f64, f64) {
    let cleaned = s.replace(' ', "");
    let parts: Vec<&str> = cleaned.split('/').collect();
    if parts.len() == 2 {
        let sys = parts[0].parse::<f64>().unwrap_or(120.0);
        let dia = parts[1].parse::<f64>().unwrap_or(80.0);
        return (sys, dia);
    }
    (120.0, 80.0)
}

// TỰ ĐỘNG PHÂN LOẠI HUYẾT ÁP THEO ĐỘ TUỔI (QĐ 1613/BYT)
pub fn calc_blood_pressure_score(bp_str: &str, age: u32) -> (i32, String) {
    let (sys, dia) = parse_blood_pressure(bp_str);
    if age < 30 {
        if sys >= 110.0 && sys <= 125.0 && dia >= 65.0 && dia <= 85.0 {
            (1, format!("< 30 tuổi: {}/{} mmHg (Bình thường)", sys as i32, dia as i32))
        } else if sys <= 135.0 && dia < 90.0 && dia >= 60.0 {
            (2, format!("< 30 tuổi: {}/{} mmHg (Hơi cao/thấp)", sys as i32, dia as i32))
        } else if sys <= 140.0 && dia <= 95.0 {
            (3, format!("< 30 tuổi: {}/{} mmHg (Tăng HA giới hạn)", sys as i32, dia as i32))
        } else if sys < 100.0 || dia < 60.0 {
            (4, format!("< 30 tuổi: {}/{} mmHg (Hạ huyết áp)", sys as i32, dia as i32))
        } else {
            (4, format!("< 30 tuổi: {}/{} mmHg (Tăng huyết áp)", sys as i32, dia as i32))
        }
    } else {
        if sys < 140.0 && dia < 90.0 {
            (1, format!(">= 30 tuổi: {}/{} mmHg (Bình thường)", sys as i32, dia as i32))
        } else if sys <= 145.0 && dia <= 95.0 {
            (3, format!(">= 30 tuổi: {}/{} mmHg (Tăng HA giới hạn)", sys as i32, dia as i32))
        } else {
            (4, format!(">= 30 tuổi: {}/{} mmHg (Tăng huyết áp)", sys as i32, dia as i32))
        }
    }
}

// TỰ ĐỘNG PHÂN LOẠI TẦN SỐ TIM / MẠCH (QĐ 1613/BYT)
pub fn calc_pulse_score(pulse_str: &str) -> (i32, String) {
    let p: f64 = pulse_str.trim().parse().unwrap_or(75.0);
    if p >= 60.0 && p <= 75.0 {
        (1, format!("Tần số {} lần/phút (Bình thường)", p as i32))
    } else if p > 75.0 && p <= 85.0 {
        (2, format!("Tần số {} lần/phút (Hơi nhanh)", p as i32))
    } else if (p > 85.0 && p <= 95.0) || (p >= 55.0 && p < 60.0) {
        (3, format!("Tần số {} lần/phút (Nhanh/Chậm nhẹ)", p as i32))
    } else {
        (4, format!("Tần số {} lần/phút (Rất nhanh/Rất chậm)", p as i32))
    }
}

// TỰ ĐỘNG PHÂN LOẠI THỊ LỰC (QĐ 1613/BYT)
pub fn calc_vision_score(right_s: &str, left_s: &str) -> i32 {
    let r: f64 = right_s.trim().parse::<f64>().unwrap_or(10.0).min(10.0_f64).max(0.0_f64);
    let l: f64 = left_s.trim().parse::<f64>().unwrap_or(10.0).min(10.0_f64).max(0.0_f64);
    let max_v = r.max(l);
    let sum_v = r + l;

    if max_v >= 9.99 && sum_v >= 18.99 {
        1
    } else if max_v >= 8.99 && sum_v >= 15.99 {
        2
    } else if max_v >= 6.99 && sum_v >= 13.99 {
        3
    } else if max_v >= 5.99 && sum_v >= 10.99 {
        4
    } else {
        5
    }
}

// TỰ ĐỘNG PHÂN LOẠI THÍNH LỰC TỪ KHOẢNG CÁCH NÓI THƯỜNG (5M) VÀ NÓI THẦM (QĐ 1613/BYT)
pub fn calc_hearing_from_distances(
    r_thuong: &str, l_thuong: &str,
    r_tham: &str, l_tham: &str,
) -> (i32, String) {
    let rt = r_tham.trim().parse::<f64>().unwrap_or(5.0);
    let lt = l_tham.trim().parse::<f64>().unwrap_or(5.0);
    let rth = r_thuong.trim().parse::<f64>().unwrap_or(5.0);
    let lth = l_thuong.trim().parse::<f64>().unwrap_or(5.0);

    let min_tham = rt.min(lt);
    let max_tham = rt.max(lt);
    let min_thuong = rth.min(lth);

    if min_tham >= 4.9 && min_thuong >= 4.9 {
        (1, "Nói thầm 5m cả hai tai (Bình thường)".into())
    } else if (max_tham >= 3.9 && min_tham >= 1.9) || min_thuong >= 4.0 {
        (2, "Nói thầm 1 bên 4m, bên kia 2m".into())
    } else if (max_tham >= 2.9 && min_tham >= 0.9) || min_thuong >= 3.0 {
        (3, "Nói thầm 1 bên 3m, bên kia 1m".into())
    } else if (max_tham >= 1.9 && min_tham >= 0.4) || min_thuong >= 2.0 {
        (4, "Nói thầm 1 bên 2m, bên kia 0.5m".into())
    } else {
        (5, "Nói thầm 1 bên 1m, bên kia < 0.1m".into())
    }
}

pub fn calc_classification_score(
    c: &QdClassification,
    base_vision: i32,
    base_hearing: i32,
) -> i32 {
    if let Some(s) = c.score {
        return s;
    }
    if let Some(ref ref_did) = c.ref_disease_id {
        if ref_did == "BT_MAT_THI_LUC" {
            let mut sc = base_vision;
            if let Some(ref m) = c.modifier {
                if m.contains("+1") {
                    sc = (sc + 1).min(5);
                } else if m.contains("+2") {
                    sc = (sc + 2).min(5);
                }
            }
            return sc;
        } else if ref_did == "BT_TMH_TAI_TRONG" {
            let mut sc = base_hearing;
            if let Some(ref m) = c.modifier {
                if m.contains("+1") {
                    sc = (sc + 1).min(5);
                }
            }
            return sc;
        }
    }
    1
}

