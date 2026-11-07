pub mod hanh_chinh;
pub mod ket_luan;
pub mod kham;
pub mod quick_entry;
pub mod tien_su;

use std::collections::HashSet;
use egui::{Color32, RichText, Ui, Vec2};
use crate::auth::permissions::Perm;
use crate::models::catalog::CatalogRecord;
use crate::models::company::AttachmentRecord;
use crate::models::qd1613::{QdClassification, QdDisease, QdSpecialty};
use crate::models::record::KskRecordPayload;
use crate::ui::pages::record_form::quick_entry::{QuickEntryAction, QuickEntryView};
use crate::ui::widgets::badge::ui_status_badge;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormStep {
    HanhChinh,
    TienSu,
    KhamBenh,
    KetLuan,
    NhapNhanh1Cua,
}

pub enum FormAction {
    None,
    BackToList,
    Save,
    ToggleLock,
    AssignArchiveAndSave,
    PrintMau03PreExam,
    PrintMau03Full,
    AttachScanFile,
    OpenAttachment(String),
    DeleteAttachment(i64),
}

pub struct RecordFormView;

impl RecordFormView {
    #[allow(clippy::too_many_arguments)]
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
        attachments: &[AttachmentRecord],
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

                ui.add_space(6.0);
                let title = match record_id {
                    Some(id) => format!(
                        "HỒ SƠ: {} (Mã YT: {} | ID: {})",
                        if form.ho_ten.is_empty() { "Chưa đặt tên" } else { &form.ho_ten },
                        &form.ma_y_te,
                        id
                    ),
                    None => format!("TẠO HỒ SƠ MỚI (Mã YT: {})", &form.ma_y_te),
                };
                ui.heading(RichText::new(title).strong().size(15.0));

                ui.add_space(6.0);
                ui_status_badge(ui, *is_locked);

                if !form.so_luu_tru.trim().is_empty() {
                    egui::Frame::none()
                        .fill(Color32::from_rgb(16, 185, 129))
                        .rounding(4.0)
                        .inner_margin(egui::Margin::symmetric(8.0, 2.0))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(format!("Số: {}", form.so_luu_tru))
                                    .color(Color32::WHITE)
                                    .strong()
                                    .size(11.5),
                            );
                        });
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if user_perms.contains(&Perm::RecordConfirm) || user_perms.contains(&Perm::RecordUnlock) {
                        let lock_txt = if *is_locked { "🔓 Mở khóa" } else { "🔒 Khóa" };
                        if ui.button(RichText::new(lock_txt).size(11.5)).clicked() {
                            action = FormAction::ToggleLock;
                        }
                    }

                    if !*is_locked && user_perms.contains(&Perm::RecordCreate) {
                        let save_btn = egui::Button::new(RichText::new("💾 Lưu hồ sơ").strong().size(12.0).color(Color32::WHITE))
                            .fill(Color32::from_rgb(14, 165, 233))
                            .rounding(6.0)
                            .min_size(Vec2::new(95.0, 28.0));
                        if ui.add(save_btn).clicked() {
                            action = FormAction::Save;
                        }
                    }

                    if ui
                        .button(RichText::new("🖨 In Mẫu 03 [Đầy đủ]").size(11.5))
                        .on_hover_text("Xuất phiếu Word Mẫu 03 có cả Số lưu trữ & Kết quả khám")
                        .clicked()
                    {
                        action = FormAction::PrintMau03Full;
                    }

                    if ui
                        .button(RichText::new("🖨 In Mẫu 03 + Barcode [Trước khám]").size(11.5).color(Color32::from_rgb(14, 165, 233)))
                        .on_hover_text("Trộn phiếu Word Mẫu 03 có sẵn Hành chính + Mã vạch tiếp nhận căn phải (Để trống Số lưu trữ & phần khám để in phát cho người khám)")
                        .clicked()
                    {
                        action = FormAction::PrintMau03PreExam;
                    }
                });
            });

            ui.add_space(6.0);

            // Step Bar
            ui.horizontal(|ui| {
                let steps = [
                    (FormStep::HanhChinh, "1. HÀNH CHÍNH", Perm::FormHanhChinh),
                    (FormStep::TienSu, "2. TIỀN SỬ", Perm::FormTienSu),
                    (FormStep::KhamBenh, "3. KHÁM CHUYÊN KHOA", Perm::ExamTheLuc),
                    (FormStep::KetLuan, "4. KẾT LUẬN & SỐ LƯU TRỮ", Perm::FormKetLuan),
                    (FormStep::NhapNhanh1Cua, "⚡ 5. NHẬP NHANH 1 CỬA (MẪU 03) & SCAN", Perm::FormKetLuan),
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

                    let btn = egui::Button::new(RichText::new(label).strong().size(12.0).color(text_color))
                        .fill(btn_bg)
                        .rounding(6.0)
                        .min_size(Vec2::new(130.0, 28.0));

                    if ui.add(btn).clicked() {
                        *current_step = st;
                    }
                }
            });

            ui.add_space(6.0);
            ui.separator();
            ui.add_space(6.0);

            // Render active step
            let mut sub_action = QuickEntryAction::None;
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
                    sub_action = ket_luan::KetLuanView::show(
                        ui,
                        form,
                        record_id,
                        attachments,
                        user_perms,
                        *is_locked,
                        dark_mode,
                    );
                }
                FormStep::NhapNhanh1Cua => {
                    sub_action = QuickEntryView::show(
                        ui,
                        form,
                        record_id,
                        attachments,
                        *is_locked,
                        dark_mode,
                    );
                }
            }

            match sub_action {
                QuickEntryAction::None => {}
                QuickEntryAction::AssignArchiveAndSave => action = FormAction::AssignArchiveAndSave,
                QuickEntryAction::AttachScanFile => action = FormAction::AttachScanFile,
                QuickEntryAction::OpenAttachment(p) => action = FormAction::OpenAttachment(p),
                QuickEntryAction::DeleteAttachment(id) => action = FormAction::DeleteAttachment(id),
            }
        });

        action
    }
}
