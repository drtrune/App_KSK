pub mod hanh_chinh;
pub mod tien_su;
pub mod kham;
pub mod ket_luan;

use std::collections::HashSet;
use egui::{Color32, RichText, Ui, Vec2};
use crate::auth::permissions::Perm;
use crate::models::catalog::CatalogRecord;
use crate::models::qd1613::{QdClassification, QdDisease, QdSpecialty};
use crate::models::record::KskRecordPayload;
use crate::ui::widgets::badge::ui_status_badge;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormStep {
    HanhChinh,
    TienSu,
    KhamBenh,
    KetLuan,
}

pub enum FormAction {
    None,
    BackToList,
    Save,
    ToggleLock,
}

pub struct RecordFormView;

impl RecordFormView {
    pub fn show(
        ui: &mut Ui,
        form: &mut KskRecordPayload,
        record_id: Option<i64>,
        is_locked: &mut bool,
        current_step: &mut FormStep,
        kham_subtab: &mut kham::SpecialtySubTab,
        simple_mode: &mut bool,
        expanded_diseases: &mut HashSet<String>,
        catalogs: &[CatalogRecord],
        specialties: &[QdSpecialty],
        diseases: &[QdDisease],
        classifications: &[QdClassification],
        user_perms: &[Perm],
        dark_mode: bool,
    ) -> FormAction {
        let mut action = FormAction::None;

        ui.vertical(|ui| {
            // Top Toolbar
            ui.horizontal(|ui| {
                if ui.button(RichText::new("« Quay lại danh sách").size(12.0)).clicked() {
                    action = FormAction::BackToList;
                }

                ui.add_space(8.0);
                let title = match record_id {
                    Some(id) => format!("HỒ SƠ KHÁM: {} (ID: {})", if form.ho_ten.is_empty() { "Chưa đặt tên" } else { &form.ho_ten }, id),
                    None => "TẠO HỒ SƠ KHÁM MỚI".to_string(),
                };
                ui.heading(RichText::new(title).strong().size(16.0));

                ui.add_space(8.0);
                ui_status_badge(ui, *is_locked);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if user_perms.contains(&Perm::RecordConfirm) || user_perms.contains(&Perm::RecordUnlock) {
                        let lock_txt = if *is_locked { "🔓 Mở khóa hồ sơ" } else { "🔒 Khóa hồ sơ" };
                        if ui.button(RichText::new(lock_txt).size(12.0)).clicked() {
                            action = FormAction::ToggleLock;
                        }
                    }

                    if !*is_locked && user_perms.contains(&Perm::RecordCreate) {
                        let save_btn = egui::Button::new(RichText::new("💾 Lưu hồ sơ").strong().size(12.5).color(Color32::WHITE))
                            .fill(Color32::from_rgb(14, 165, 233))
                            .rounding(6.0)
                            .min_size(Vec2::new(100.0, 30.0));
                        if ui.add(save_btn).clicked() {
                            action = FormAction::Save;
                        }
                    }
                });
            });

            ui.add_space(8.0);

            // Step Bar
            ui.horizontal(|ui| {
                let steps = [
                    (FormStep::HanhChinh, "1. HÀNH CHÍNH", Perm::FormHanhChinh),
                    (FormStep::TienSu, "2. TIỀN SỬ", Perm::FormTienSu),
                    (FormStep::KhamBenh, "3. KHÁM BỆNH", Perm::ExamTheLuc),
                    (FormStep::KetLuan, "4. KẾT LUẬN", Perm::FormKetLuan),
                ];

                for (st, label, _p) in steps {
                    let is_current = *current_step == st;
                    let text_color = if is_current {
                        Color32::from_rgb(14, 165, 233)
                    } else if dark_mode {
                        Color32::from_rgb(148, 163, 184)
                    } else {
                        Color32::from_rgb(71, 85, 105)
                    };

                    let btn_bg = if is_current {
                        if dark_mode { Color32::from_rgb(30, 41, 59) } else { Color32::from_rgb(224, 242, 254) }
                    } else {
                        Color32::TRANSPARENT
                    };

                    let btn = egui::Button::new(RichText::new(label).strong().size(12.5).color(text_color))
                        .fill(btn_bg)
                        .rounding(6.0)
                        .min_size(Vec2::new(140.0, 30.0));

                    if ui.add(btn).clicked() {
                        *current_step = st;
                    }
                }
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Render active step
            match current_step {
                FormStep::HanhChinh => {
                    hanh_chinh::HanhChinhView::show(ui, form, *is_locked, catalogs);
                }
                FormStep::TienSu => {
                    tien_su::TienSuView::show(ui, form, *is_locked, catalogs);
                }
                FormStep::KhamBenh => {
                    kham::KhamView::show(
                        ui,
                        form,
                        kham_subtab,
                        simple_mode,
                        expanded_diseases,
                        specialties,
                        diseases,
                        classifications,
                        user_perms,
                        *is_locked,
                        dark_mode,
                    );
                }
                FormStep::KetLuan => {
                    ket_luan::KetLuanView::show(ui, form, user_perms, *is_locked, dark_mode);
                }
            }
        });

        action
    }
}
