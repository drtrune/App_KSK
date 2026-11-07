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

// TỰ ĐỘNG PHÂN LOẠI THỊ LỰC (QĐ 1613/BYT - thang 0..10)
pub fn calc_vision_score(right_s: &str, left_s: &str) -> i32 {
    let parse_v = |s: &str| -> f64 {
        let t = s.trim();
        if t.contains('/') {
            if let Some(first) = t.split('/').next() {
                return first.trim().parse::<f64>().unwrap_or(10.0);
            }
        }
        t.parse::<f64>().unwrap_or(10.0)
    };
    let r = parse_v(right_s).min(10.0).max(0.0);
    let l = parse_v(left_s).min(10.0).max(0.0);
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

pub fn parse_hearing_dist_thuong(s: &str) -> f64 {
    let t = s.trim().to_lowercase();
    if t.contains("diec") || t.contains("điếc") {
        return 0.0;
    }
    if t.contains("sat") || t.contains("sát") {
        return 0.2; // sát tai / cách 1 gang tay sát vành tai
    }
    if t.contains(">5") || t.contains("> 5") {
        return 6.0;
    }
    if t.contains("0.5") || t.contains("0,5") {
        return 0.5;
    }
    let clean: String = t.chars().filter(|c| c.is_ascii_digit() || *c == '.' || *c == ',').collect();
    let clean = clean.replace(',', ".");
    clean.parse::<f64>().unwrap_or(5.0)
}

pub fn parse_hearing_dist_tham(s: &str) -> f64 {
    let t = s.trim().to_lowercase();
    if t.contains("diec") || t.contains("điếc") {
        return 0.0;
    }
    if t.contains("sat") || t.contains("sát") {
        return 0.01;
    }
    if t.contains(">0.5") || t.contains("> 0.5") || t.contains(">0,5") || t.contains("> 0,5") {
        return 0.6;
    }
    let clean: String = t.chars().filter(|c| c.is_ascii_digit() || *c == '.' || *c == ',').collect();
    let clean = clean.replace(',', ".");
    clean.parse::<f64>().unwrap_or(0.5)
}

fn score_single_ear(d_thuong: f64, d_tham: f64) -> i32 {
    let s_thuong = if d_thuong >= 5.0 {
        1
    } else if d_thuong >= 4.0 {
        2
    } else if d_thuong >= 1.0 {
        3
    } else if d_thuong >= 0.5 {
        4
    } else {
        5
    };

    let s_tham = if d_tham >= 0.5 {
        1
    } else if d_tham >= 0.4 {
        2
    } else if d_tham >= 0.1 {
        3
    } else if d_tham >= 0.05 {
        4
    } else {
        5
    };

    s_thuong.max(s_tham)
}

// TỰ ĐỘNG PHÂN LOẠI THÍNH LỰC THEO BẢNG TÍNH THIẾU HỤT SỨC NGHE SƠ BỘ (BYT)
pub fn calc_hearing_from_distances(
    r_thuong: &str, l_thuong: &str,
    r_tham: &str, l_tham: &str,
) -> (i32, String) {
    let r_th = parse_hearing_dist_thuong(r_thuong);
    let l_th = parse_hearing_dist_thuong(l_thuong);
    let r_tm = parse_hearing_dist_tham(r_tham);
    let l_tm = parse_hearing_dist_tham(l_tham);

    let score_r = score_single_ear(r_th, r_tm);
    let score_l = score_single_ear(l_th, l_tm);
    let final_score = score_r.max(score_l);

    let desc = match final_score {
        1 => "Sức nghe tốt (Nói thường >= 5m, Nói thầm >= 0.5m)",
        2 => "Thiếu hụt sức nghe sơ bộ (Nói thường 4m, Nói thầm 0.4m)",
        3 => "Thiếu hụt sức nghe 35% - 65% (Nói thường 1m - 3m, Nói thầm 0.1m - 0.3m)",
        4 => "Thiếu hụt sức nghe 65% - 85% (Nói thường 0.5m, Nói thầm 0.05m)",
        _ => "Thiếu hụt sức nghe 85% - 100% (Sát tai / Điếc)",
    };

    (final_score, desc.to_string())
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

