pub mod the_luc;
pub mod specialty_simple;
pub mod specialty_catalog;
pub mod cls;

use std::collections::HashSet;
use egui::{Color32, RichText, Ui};
use crate::auth::permissions::Perm;
use crate::models::qd1613::{QdClassification, QdDisease, QdSpecialty};
use crate::models::record::KskRecordPayload;

pub struct KhamView;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecialtySubTab {
    TheLuc,
    // Nội khoa subtabs
    NoiTuanHoan,
    NoiHoHap,
    NoiTieuHoa,
    NoiThanTietNieu,
    NoiNoiTiet,
    NoiCoXuongKhop,
    NoiThanKinh,
    NoiTamThan,
    // Other specialties
    NgoaiKhoa,
    DaLieu,
    SanPhuKhoa,
    Mat,
    TaiMuiHong,
    RangHamMat,
    CanLamSang,
}

impl KhamView {
    pub fn show(
        ui: &mut Ui,
        form: &mut KskRecordPayload,
        active_subtab: &mut SpecialtySubTab,
        simple_mode: &mut bool,
        expanded_diseases: &mut HashSet<String>,
        specialties: &[QdSpecialty],
        diseases: &[QdDisease],
        classifications: &[QdClassification],
        user_perms: &[Perm],
        is_locked: bool,
        dark_mode: bool,
    ) {
        ui.vertical(|ui| {
            // Mode toggle & Department tabs
            ui.horizontal(|ui| {
                ui.label(RichText::new("Chế độ khám:").size(12.0).color(Color32::from_rgb(100, 116, 139)));
                let mode_btn_txt = if *simple_mode { "⚡ Chế độ: Đơn giản" } else { "⚕ Chế độ: QĐ 1613" };
                if ui.button(RichText::new(mode_btn_txt).strong().size(11.5).color(Color32::from_rgb(14, 165, 233))).on_hover_text("Chuyển đổi giữa chế độ nhập nhanh và chế độ phân loại chi tiết QĐ 1613").clicked() {
                    *simple_mode = !*simple_mode;
                }
            });

            ui.add_space(6.0);

            // Subtab navigation bar
            ui.horizontal_wrapped(|ui| {
                let mut tab_btn = |ui: &mut Ui, target: SpecialtySubTab, label: &str, perm: Option<Perm>| {
                    let has_perm = match perm {
                        Some(p) => user_perms.contains(&p),
                        None => true,
                    };

                    let is_active = *active_subtab == target;
                    let text_color = if is_active {
                        Color32::from_rgb(14, 165, 233)
                    } else if !has_perm {
                        Color32::from_rgb(156, 163, 175)
                    } else if dark_mode {
                        Color32::from_rgb(226, 232, 240)
                    } else {
                        Color32::from_rgb(51, 65, 85)
                    };

                    let btn_bg = if is_active {
                        if dark_mode { Color32::from_rgb(30, 41, 59) } else { Color32::from_rgb(224, 242, 254) }
                    } else {
                        Color32::TRANSPARENT
                    };

                    let btn = egui::Button::new(RichText::new(label).size(12.0).color(text_color).strong())
                        .fill(btn_bg)
                        .rounding(4.0);

                    if ui.add(btn).clicked() {
                        *active_subtab = target;
                    }
                };

                tab_btn(ui, SpecialtySubTab::TheLuc, "🏃 Thể Lực", Some(Perm::ExamTheLuc));
                tab_btn(ui, SpecialtySubTab::NoiTuanHoan, "❤️ Tuần Hoàn", Some(Perm::ExamNoi));
                tab_btn(ui, SpecialtySubTab::NoiHoHap, "🫁 Hô Hấp", Some(Perm::ExamNoi));
                tab_btn(ui, SpecialtySubTab::NoiTieuHoa, "🍽️ Tiêu Hóa", Some(Perm::ExamNoi));
                tab_btn(ui, SpecialtySubTab::NoiThanTietNieu, "💧 Thận - TN", Some(Perm::ExamNoi));
                tab_btn(ui, SpecialtySubTab::NoiNoiTiet, "🧬 Nội Tiết", Some(Perm::ExamNoi));
                tab_btn(ui, SpecialtySubTab::NoiCoXuongKhop, "🦴 Cơ Xương Khớp", Some(Perm::ExamNoi));
                tab_btn(ui, SpecialtySubTab::NoiThanKinh, "🧠 Thần Kinh", Some(Perm::ExamNoi));
                tab_btn(ui, SpecialtySubTab::NoiTamThan, "🧘 Tâm Thần", Some(Perm::ExamNoi));
                tab_btn(ui, SpecialtySubTab::NgoaiKhoa, "🔪 Ngoại Khoa", Some(Perm::ExamNgoai));
                tab_btn(ui, SpecialtySubTab::DaLieu, "🩹 Da Liễu", Some(Perm::ExamDaLieu));
                tab_btn(ui, SpecialtySubTab::SanPhuKhoa, "🌸 Sản Phụ Khoa", Some(Perm::ExamSan));
                tab_btn(ui, SpecialtySubTab::Mat, "👁️ Mắt", Some(Perm::ExamMat));
                tab_btn(ui, SpecialtySubTab::TaiMuiHong, "👂 Tai Mũi Họng", Some(Perm::ExamTmh));
                tab_btn(ui, SpecialtySubTab::RangHamMat, "🦷 Răng Hàm Mặt", Some(Perm::ExamRhm));
                tab_btn(ui, SpecialtySubTab::CanLamSang, "🧪 Cận Lâm Sàng", Some(Perm::ExamCls));
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Active view rendering
            match active_subtab {
                SpecialtySubTab::TheLuc => {
                    the_luc::TheLucView::show(ui, form, is_locked);
                }
                SpecialtySubTab::CanLamSang => {
                    cls::ClsView::show(ui, form, is_locked);
                }
                other => {
                    let (spec_id, spec_name, qd_code) = match other {
                        SpecialtySubTab::NoiTuanHoan => ("tuan_hoan", "Tuần hoàn", "CK_NOI_TIM_MACH"),
                        SpecialtySubTab::NoiHoHap => ("ho_hap", "Hô hấp", "CK_NOI_HO_HAP"),
                        SpecialtySubTab::NoiTieuHoa => ("tieu_hoa", "Tiêu hóa", "CK_NOI_TIEU_HOA"),
                        SpecialtySubTab::NoiThanTietNieu => ("than_tiet_nieu", "Thận - Tiết niệu - Sinh dục", "CK_NOI_TIET_NIEU_SD"),
                        SpecialtySubTab::NoiNoiTiet => ("noi_tiet", "Nội tiết", "CK_NOI_NOITIET"),
                        SpecialtySubTab::NoiCoXuongKhop => ("co_xuong_khop", "Cơ - Xương - Khớp", "CK_NOI_CO_XUONG_KHOP"),
                        SpecialtySubTab::NoiThanKinh => ("than_kinh", "Thần kinh", "CK_NOI_THAN_KINH"),
                        SpecialtySubTab::NoiTamThan => ("tam_than", "Tâm thần", "CK_NOI_TAM_THAN"),
                        SpecialtySubTab::NgoaiKhoa => ("ngoai_khoa", "Ngoại khoa", "CK_NGOAI"),
                        SpecialtySubTab::DaLieu => ("da_lieu", "Da liễu", "CK_DA_LIEU"),
                        SpecialtySubTab::SanPhuKhoa => ("san_phu_khoa", "Sản phụ khoa", "CK_PHU_SAN"),
                        SpecialtySubTab::Mat => ("mat", "Mắt", "CK_MAT"),
                        SpecialtySubTab::TaiMuiHong => ("tai_mui_hong", "Tai Mũi Họng", "CK_TMH"),
                        SpecialtySubTab::RangHamMat => ("rang_ham_mat", "Răng Hàm Mặt", "CK_RHM"),
                        _ => ("ngoai_khoa", "Ngoại khoa", "CK_NGOAI"),
                    };

                    if *simple_mode {
                        specialty_simple::SpecialtySimpleView::show(ui, form, spec_id, spec_name, is_locked);
                    } else {
                        specialty_catalog::SpecialtyCatalogView::show(
                            ui,
                            form,
                            qd_code,
                            spec_name,
                            specialties,
                            diseases,
                            classifications,
                            expanded_diseases,
                            is_locked,
                            dark_mode,
                        );
                    }
                }
            }
        });
    }
}
