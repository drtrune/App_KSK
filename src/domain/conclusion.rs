use crate::models::{KskRecordPayload, QdClassification, QdDisease};
use crate::domain::scoring::{
    calc_blood_pressure_score, calc_hearing_from_distances, calc_pulse_score, calc_vision_score,
    calc_classification_score,
};
use crate::domain::dates::calculate_age;

pub fn sync_specialty_result(form: &mut KskRecordPayload, spec_id: &str, qd_diseases: &[QdDisease], qd_classifications: &[QdClassification]) {
    let base_vision = calc_vision_score(&form.khong_kinh_mat_phai, &form.khong_kinh_mat_trai);
    let has_glasses = !form.co_kinh_mat_phai.trim().is_empty() || !form.co_kinh_mat_trai.trim().is_empty();
    let eff_vision = if has_glasses {
        calc_vision_score(&form.co_kinh_mat_phai, &form.co_kinh_mat_trai)
    } else {
        base_vision
    };
    let (base_hearing, hearing_desc) = calc_hearing_from_distances(
        &form.tai_phai_noi_thuong, &form.tai_trai_noi_thuong,
        &form.tai_phai_noi_tham, &form.tai_trai_noi_tham,
    );
    let age = calculate_age(&form.ngay_sinh);
    let (bp_score, bp_desc) = calc_blood_pressure_score(&form.huyet_ap, age);
    let (pulse_score, pulse_desc) = calc_pulse_score(&form.mach);

    let diseases: Vec<QdDisease> = qd_diseases.iter().filter(|d| d.spec_id == spec_id).cloned().collect();
    let mut abnormal_items: Vec<(String, i32)> = Vec::new();
    let mut max_sc = 1;

    let extra = form.specialty_extras.entry(spec_id.to_string()).or_default();
    let auto_overrides = extra.auto_overrides.clone();
    let manual_rows = extra.manual_rows.clone();
    let final_note = extra.final_note.clone();

    // 1. Kiểm tra các bệnh tự động riêng cho từng chuyên khoa
    if spec_id == "CK_MAT" {
        if eff_vision > max_sc { max_sc = eff_vision; }
        if eff_vision > 1 || has_glasses {
            let override_text = auto_overrides.get("BT_MAT_THI_LUC").cloned().unwrap_or_else(|| {
                if has_glasses {
                    format!("Không kính: MP {}/10, MT {}/10 | Có kính: MP {}/10, MT {}/10", form.khong_kinh_mat_phai, form.khong_kinh_mat_trai, form.co_kinh_mat_phai, form.co_kinh_mat_trai)
                } else {
                    format!("Thị lực không kính MP {}/10, MT {}/10", form.khong_kinh_mat_phai, form.khong_kinh_mat_trai)
                }
            });
            abnormal_items.push((format!("Thị lực: {}", override_text), eff_vision));
        }
    } else if spec_id == "CK_TMH" {
        if base_hearing > max_sc { max_sc = base_hearing; }
        if base_hearing > 1 {
            let override_text = auto_overrides.get("BT_TMH_TAI_TRONG").cloned().unwrap_or(hearing_desc);
            abnormal_items.push((format!("Sức nghe: {}", override_text), base_hearing));
        }
    } else if spec_id == "CK_NOI_TIM_MACH" {
        if bp_score > max_sc { max_sc = bp_score; }
        if bp_score > 1 {
            let override_text = auto_overrides.get("BT_TM_HUYET_AP").cloned().unwrap_or(bp_desc);
            abnormal_items.push((format!("Huyết áp: {}", override_text), bp_score));
        }
        if pulse_score > max_sc { max_sc = pulse_score; }
        if pulse_score > 1 {
            let override_text = auto_overrides.get("BT_TM_MACH").cloned().unwrap_or(pulse_desc);
            abnormal_items.push((format!("Tần số tim: {}", override_text), pulse_score));
        }
    }

    // 2. Kiểm tra các bệnh được chọn bằng TICKBOX (Checkbox)
    for d in &diseases {
        // Bỏ qua các mã bệnh đã tính tự động ở trên
        if d.id == "BT_MAT_THI_LUC" || d.id == "BT_TMH_TAI_TRONG" || d.id == "BT_TM_HUYET_AP" || d.id == "BT_TM_MACH" {
            continue;
        }

        let clss: Vec<QdClassification> = qd_classifications.iter().filter(|c| c.disease_id == d.id).cloned().collect();
        if clss.is_empty() { continue; }

        for c in &clss {
            if form.selected_items.contains(&c.id) {
                let sc = calc_classification_score(c, base_vision, base_hearing);
                if sc > max_sc { max_sc = sc; }
                if sc > 1 {
                    let text = if let Some(custom) = auto_overrides.get(&c.id) {
                        if !custom.trim().is_empty() { custom.clone() } else { c.name.clone() }
                    } else {
                        c.name.clone()
                    };
                    abnormal_items.push((format!("{}: {}", d.name, text), sc));
                }
            }
        }
    }

    // 3. Dòng nhập tay
    for mr in &manual_rows {
        if mr.classification > max_sc {
            max_sc = mr.classification;
        }
        if !mr.manifestation.trim().is_empty() {
            abnormal_items.push((mr.manifestation.clone(), mr.classification));
        }
    }

    let summary = if max_sc == 1 && abnormal_items.is_empty() {
        "Bình thường".to_string()
    } else {
        let mut parts: Vec<String> = abnormal_items.into_iter().map(|(txt, _)| txt).collect();
        if !final_note.trim().is_empty() {
            parts.push(format!("Ghi chú: {}", final_note.trim()));
        }
        parts.join("; ")
    };

    match spec_id {
        "CK_NOI_TIM_MACH" => {
            form.noi_khoa_tuan_hoan = summary;
            form.noi_khoa_tuan_hoan_pl = max_sc;
        }
        "CK_NOI_HO_HAP" => {
            form.noi_khoa_ho_hap = summary;
            form.noi_khoa_ho_hap_pl = max_sc;
        }
        "CK_NOI_TIEU_HOA" => {
            form.noi_khoa_tieu_hoa = summary;
            form.noi_khoa_tieu_hoa_pl = max_sc;
        }
        "CK_NOI_TIET_NIEU_SD" => {
            form.noi_khoa_than_tn_sd = summary;
            form.noi_khoa_than_tn_sd_pl = max_sc;
        }
        "CK_NOI_THAN_KINH" => {
            form.noi_khoa_than_kinh = summary;
            form.noi_khoa_than_kinh_pl = max_sc;
        }
        "CK_NOI_TAM_THAN" => {
            form.noi_khoa_tam_than = summary;
            form.noi_khoa_tam_than_pl = max_sc;
        }
        "CK_NOI_NOITIET" => {
            form.noi_khoa_noi_tiet = summary;
            form.noi_khoa_noi_tiet_pl = max_sc;
        }
        "CK_NOI_CO_XUONG_KHOP" => {
            form.noi_khoa_co_xuong_khop = summary;
            form.noi_khoa_co_xuong_khop_pl = max_sc;
        }
        "CK_NGOAI" => {
            form.ket_qua_kham_ngoai_khoa = summary;
            form.kham_ngoai_khoa_pl = max_sc;
        }
        "CK_DA_LIEU" => {
            form.ket_qua_kham_da_lieu = summary;
            form.kham_da_lieu_pl = max_sc;
        }
        "CK_PHU_SAN" => {
            form.ket_qua_kham_san_phu_khoa = summary;
            form.kham_san_phu_khoa_pl = max_sc;
        }
        "CK_MAT" => {
            form.benh_khac_mat = summary;
            form.kham_mat_pl = max_sc;
        }
        "CK_TMH" => {
            form.benh_khac_tai_mui_hong = summary;
            form.kham_tai_mui_hong_pl = max_sc;
        }
        "CK_RHM" => {
            form.benh_khac_rang_ham_mat = summary;
            form.kham_rang_ham_mat_pl = max_sc;
        }
        _ => {}
    }

    update_global_health_score(form);
}

pub fn update_global_health_score(form: &mut KskRecordPayload) {
    let scores = [
        form.kham_the_luc_pl,
        form.noi_khoa_tuan_hoan_pl,
        form.noi_khoa_ho_hap_pl,
        form.noi_khoa_tieu_hoa_pl,
        form.noi_khoa_than_tn_sd_pl,
        form.noi_khoa_noi_tiet_pl,
        form.noi_khoa_co_xuong_khop_pl,
        form.noi_khoa_than_kinh_pl,
        form.noi_khoa_tam_than_pl,
        form.kham_ngoai_khoa_pl,
        form.kham_da_lieu_pl,
        form.kham_san_phu_khoa_pl,
        form.kham_mat_pl,
        form.kham_tai_mui_hong_pl,
        form.kham_rang_ham_mat_pl,
    ];
    let max_val = scores.into_iter().max().unwrap_or(1).clamp(1, 5);
    form.phan_loai_sk = max_val;
}

