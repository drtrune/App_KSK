use egui::{Color32, RichText, Ui, Vec2};
use crate::models::{AttachmentRecord, CompanyRecord, ContractRecord, RecordItem};
use crate::ui::widgets::badge::ui_score_badge;

#[derive(Default, Clone)]
pub struct CompanyUiState {
    pub selected_company_id: Option<i64>,
    pub selected_contract_id: Option<i64>,
    pub editing_company: CompanyRecord,
    pub show_company_form: bool,
    pub editing_contract: ContractRecord,
    pub show_contract_form: bool,
    pub search_emp: String,
}

pub enum CompanyPageAction {
    None,
    SaveCompany(CompanyRecord),
    DeleteCompany(i64),
    SaveContract(ContractRecord),
    DeleteContract(i64),
    AttachContractScan(i64),
    OpenAttachment(String),
    DeleteAttachment(i64),
    DownloadRosterTemplate,
    ImportRosterCsv(i64),
    NewRecordForContract(i64),
    BatchPrintMau03PreExam(i64),
    ExportContractXlsm(i64),
    ExportFullDossierPackage(i64),
    OpenRecordFastEntry(i64),
    PrintSingleMau03(i64),
    AttachRecordScan(i64),
}

pub struct CompaniesPage;

impl CompaniesPage {
    pub fn show(
        ui: &mut Ui,
        companies: &[CompanyRecord],
        contracts: &[ContractRecord],
        records: &[RecordItem],
        attachments: &[AttachmentRecord],
        state: &mut CompanyUiState,
        dark_mode: bool,
    ) -> CompanyPageAction {
        let mut action = CompanyPageAction::None;

        if state.selected_company_id.is_none() {
            if let Some(first) = companies.first() {
                state.selected_company_id = Some(first.id);
            }
        }

        let card_bg = if dark_mode { Color32::from_rgb(30, 41, 59) } else { Color32::WHITE };
        let border_color = if dark_mode { Color32::from_rgb(51, 65, 85) } else { Color32::from_rgb(226, 232, 240) };

        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.heading(RichText::new("🏢 QUẢN LÝ CÔNG TY, HỢP ĐỒNG KHÁM ĐOÀN & BỘ HỒ SƠ LƯU TRỮ").size(17.5).strong());
                    ui.label(
                        RichText::new("Quy trình khép kín: Tạo Công ty & Scan Hợp đồng -> Import danh sách & In Mẫu 03 kèm QR (chưa cấp số) -> Quét QR nhập 1 cửa & Cấp Số Lưu Trữ -> Scan phiếu & Đóng gói bộ hồ sơ.")
                            .size(11.5)
                            .color(Color32::from_rgb(100, 116, 139)),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button(RichText::new("➕ Thêm Công ty mới").strong().color(Color32::WHITE))
                        .clicked()
                    {
                        state.editing_company = CompanyRecord::default();
                        state.show_company_form = true;
                    }
                    if ui.button("📄 Tải mẫu Excel/CSV Danh sách Đoàn").clicked() {
                        action = CompanyPageAction::DownloadRosterTemplate;
                    }
                });
            });

            ui.add_space(8.0);

            // Form Thêm/Sửa Công ty (nếu bật)
            if state.show_company_form {
                egui::Frame::none()
                    .fill(card_bg)
                    .stroke(egui::Stroke::new(1.5_f32, Color32::from_rgb(14, 165, 233)))
                    .rounding(8.0)
                    .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new(if state.editing_company.id == 0 { "THÊM CÔNG TY / ĐƠN VỊ MỚI" } else { "CẬP NHẬT CÔNG TY" }).strong());
                        ui.horizontal_wrapped(|ui| {
                            ui.label("Mã Cty:");
                            ui.add(egui::TextEdit::singleline(&mut state.editing_company.ma_cong_ty).desired_width(90.0).hint_text("VD: CTY01"));
                            ui.label("Tên Công ty (*):");
                            ui.add(egui::TextEdit::singleline(&mut state.editing_company.ten_cong_ty).desired_width(240.0));
                            ui.label("Mã số thuế:");
                            ui.add(egui::TextEdit::singleline(&mut state.editing_company.ma_so_thue).desired_width(110.0));
                            ui.label("Điện thoại:");
                            ui.add(egui::TextEdit::singleline(&mut state.editing_company.dien_thoai).desired_width(110.0));
                        });
                        ui.horizontal_wrapped(|ui| {
                            ui.label("Địa chỉ:");
                            ui.add(egui::TextEdit::singleline(&mut state.editing_company.dia_chi).desired_width(300.0));
                            ui.label("Người liên hệ:");
                            ui.add(egui::TextEdit::singleline(&mut state.editing_company.nguoi_lien_he).desired_width(160.0));
                            if ui.button(RichText::new("💾 Lưu Công ty").strong().color(Color32::from_rgb(16, 185, 129))).clicked() {
                                action = CompanyPageAction::SaveCompany(state.editing_company.clone());
                                state.show_company_form = false;
                            }
                            if ui.button("Đóng").clicked() {
                                state.show_company_form = false;
                            }
                        });
                    });
                ui.add_space(6.0);
            }

            // Hàng chọn Công ty & Hợp đồng
            egui::Frame::none()
                .fill(card_bg)
                .stroke(egui::Stroke::new(1.0_f32, border_color))
                .rounding(8.0)
                .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new("1. Chọn Công ty:").strong().size(12.5));
                        for comp in companies {
                            let is_sel = state.selected_company_id == Some(comp.id);
                            if ui.selectable_label(is_sel, format!("🏢 {} ({})", comp.ten_cong_ty, comp.ma_cong_ty)).clicked() {
                                state.selected_company_id = Some(comp.id);
                                state.selected_contract_id = contracts.iter().find(|c| c.company_id == comp.id).map(|c| c.id);
                            }
                        }
                        if let Some(cid) = state.selected_company_id {
                            if let Some(comp) = companies.iter().find(|c| c.id == cid) {
                                if ui.small_button("✏ Sửa Cty").clicked() {
                                    state.editing_company = comp.clone();
                                    state.show_company_form = true;
                                }
                                if ui.small_button(RichText::new("🗑").color(Color32::from_rgb(239, 68, 68))).clicked() {
                                    action = CompanyPageAction::DeleteCompany(comp.id);
                                }
                            }
                        }
                    });

                    if let Some(cid) = state.selected_company_id {
                        ui.add_space(4.0);
                        ui.separator();
                        ui.add_space(4.0);
                        ui.horizontal_wrapped(|ui| {
                            ui.label(RichText::new("2. Chọn Hợp đồng / Đợt khám:").strong().size(12.5));
                            let comp_contracts: Vec<&ContractRecord> = contracts.iter().filter(|c| c.company_id == cid).collect();
                            if state.selected_contract_id.is_none() {
                                state.selected_contract_id = comp_contracts.first().map(|c| c.id);
                            }
                            for ct in comp_contracts {
                                let is_sel = state.selected_contract_id == Some(ct.id);
                                if ui.selectable_label(is_sel, format!("📑 HĐ: {} - {}", ct.so_hop_dong, ct.ten_dot_kham)).clicked() {
                                    state.selected_contract_id = Some(ct.id);
                                }
                            }
                            if ui.button(RichText::new("➕ Tạo Hợp đồng / Đợt khám mới").strong().color(Color32::from_rgb(14, 165, 233))).clicked() {
                                let mut new_ct = ContractRecord::default();
                                new_ct.company_id = cid;
                                state.editing_contract = new_ct;
                                state.show_contract_form = true;
                            }
                        });
                    }
                });

            // Form Thêm/Sửa Hợp đồng (nếu bật)
            if state.show_contract_form {
                ui.add_space(6.0);
                egui::Frame::none()
                    .fill(card_bg)
                    .stroke(egui::Stroke::new(1.5_f32, Color32::from_rgb(16, 185, 129)))
                    .rounding(8.0)
                    .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new("THÔNG TIN HỢP ĐỒNG KHÁM SỨC KHỎE ĐOÀN").strong());
                        ui.horizontal_wrapped(|ui| {
                            ui.label("Số Hợp đồng (*):");
                            ui.add(egui::TextEdit::singleline(&mut state.editing_contract.so_hop_dong).desired_width(120.0).hint_text("VD: 12/2026/HĐ-KSK"));
                            ui.label("Tên đợt khám (*):");
                            ui.add(egui::TextEdit::singleline(&mut state.editing_contract.ten_dot_kham).desired_width(220.0).hint_text("VD: Khám định kỳ đợt 1/2026"));
                            ui.label("Ngày khám:");
                            ui.add(egui::TextEdit::singleline(&mut state.editing_contract.ngay_bat_dau).desired_width(90.0));
                            ui.label("Số lượng dự kiến:");
                            let mut sl_str = state.editing_contract.so_luong_du_kien.to_string();
                            if ui.add(egui::TextEdit::singleline(&mut sl_str).desired_width(60.0)).changed() {
                                state.editing_contract.so_luong_du_kien = sl_str.parse().unwrap_or(0);
                            }
                            if ui.button(RichText::new("💾 Lưu Hợp đồng").strong().color(Color32::from_rgb(16, 185, 129))).clicked() {
                                action = CompanyPageAction::SaveContract(state.editing_contract.clone());
                                state.show_contract_form = false;
                            }
                            if ui.button("Đóng").clicked() {
                                state.show_contract_form = false;
                            }
                        });
                    });
            }

            ui.add_space(8.0);

            // CHI TIẾT HỢP ĐỒNG ĐANG CHỌN
            if let Some(ct_id) = state.selected_contract_id {
                if let Some(ct) = contracts.iter().find(|c| c.id == ct_id) {
                    let ct_records: Vec<&RecordItem> = records.iter().filter(|r| r.contract_id == Some(ct.id)).collect();
                    let concluded_count = ct_records.iter().filter(|r| !r.data.so_luu_tru.trim().is_empty()).count();
                    let scanned_count = ct_records
                        .iter()
                        .filter(|r| attachments.iter().any(|a| a.owner_type == "record" && a.owner_id == r.id))
                        .count();
                    let ct_atts: Vec<&AttachmentRecord> = attachments
                        .iter()
                        .filter(|a| a.owner_type == "contract" && a.owner_id == ct.id)
                        .collect();

                    // Khối Tiến độ & Các nút Thao tác Đoàn
                    egui::Frame::none()
                        .fill(card_bg)
                        .stroke(egui::Stroke::new(1.0_f32, border_color))
                        .rounding(8.0)
                        .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                        .show(ui, |ui| {
                            ui.horizontal_wrapped(|ui| {
                                ui.label(RichText::new(format!("📑 HỢP ĐỒNG: {} ({})", ct.so_hop_dong, ct.ten_dot_kham)).strong().size(14.0).color(Color32::from_rgb(14, 165, 233)));
                                ui.label(RichText::new(format!(
                                    "|  Tổng danh sách: {} người  |  Đã cấp Số lưu trữ: {}/{}  |  Đã Scan phiếu: {}/{}",
                                    ct_records.len(),
                                    concluded_count,
                                    ct_records.len(),
                                    scanned_count,
                                    ct_records.len()
                                )).strong().size(12.0));

                                if ui.small_button("✏ Sửa HĐ").clicked() {
                                    state.editing_contract = ct.clone();
                                    state.show_contract_form = true;
                                }
                            });

                            ui.add_space(6.0);

                            // Dòng File Scan Hợp đồng
                            ui.horizontal_wrapped(|ui| {
                                ui.label(RichText::new("📎 Scan Hợp đồng & Phụ lục:").strong().size(12.0));
                                if ui.button(RichText::new("➕ Đính kèm file Scan Hợp đồng").color(Color32::from_rgb(14, 165, 233))).clicked() {
                                    action = CompanyPageAction::AttachContractScan(ct.id);
                                }
                                for att in &ct_atts {
                                    ui.group(|ui| {
                                        ui.label(RichText::new(&att.file_name).size(11.5));
                                        if ui.small_button("👁 Mở").clicked() {
                                            action = CompanyPageAction::OpenAttachment(att.file_path.clone());
                                        }
                                        if ui.small_button("✖").clicked() {
                                            action = CompanyPageAction::DeleteAttachment(att.id);
                                        }
                                    });
                                }
                            });

                            ui.add_space(8.0);
                            ui.separator();
                            ui.add_space(6.0);

                            // Hàng nút Nghiệp vụ Khám Đoàn (Trước khám & Sau khám)
                            ui.horizontal_wrapped(|ui| {
                                if ui.button(RichText::new("📥 1. Import Danh sách Nhân viên (CSV)").strong().color(Color32::from_rgb(14, 165, 233))).clicked() {
                                    action = CompanyPageAction::ImportRosterCsv(ct.id);
                                }
                                if ui.button("➕ Thêm lẻ 1 người vào Đoàn").clicked() {
                                    action = CompanyPageAction::NewRecordForContract(ct.id);
                                }
                                if ui
                                    .button(RichText::new("🖨 2. Trộn & Xuất Phiếu Mẫu 03 (.docx + Barcode) Cả Đoàn [Trước Khám]").strong().color(Color32::from_rgb(217, 119, 6)))
                                    .on_hover_text("Xuất hàng loạt phiếu khám Word Mẫu 03 đã điền Hành chính + Mã vạch tiếp nhận (chưa có Số lưu trữ) để phát cho nhân viên đi khám")
                                    .clicked()
                                {
                                    action = CompanyPageAction::BatchPrintMau03PreExam(ct.id);
                                }
                                if ui
                                    .button(RichText::new("📊 3. Xuất Import_KSK_Tren_18.xlsm (Đoàn)").strong().color(Color32::from_rgb(16, 185, 129)))
                                    .clicked()
                                {
                                    action = CompanyPageAction::ExportContractXlsm(ct.id);
                                }
                                let pkg_btn = egui::Button::new(
                                    RichText::new("📦 4. Đóng gói Trọn Bộ Hồ Sơ Đoàn (HĐ + Scan + Mẫu 03 + XML/Excel)")
                                        .strong()
                                        .color(Color32::WHITE),
                                )
                                .fill(Color32::from_rgb(16, 185, 129))
                                .rounding(6.0)
                                .min_size(Vec2::new(260.0, 28.0));
                                if ui.add(pkg_btn).clicked() {
                                    action = CompanyPageAction::ExportFullDossierPackage(ct.id);
                                }
                            });
                        });

                    ui.add_space(8.0);

                    // Bảng danh sách nhân viên trong Hợp đồng
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Tìm nhân viên trong đoàn:").size(12.0));
                        ui.add(egui::TextEdit::singleline(&mut state.search_emp).desired_width(260.0).hint_text("Nhập Họ tên, Mã tiếp nhận, CCCD, Số lưu trữ..."));
                    });
                    ui.add_space(4.0);

                    let q = state.search_emp.trim().to_lowercase();
                    egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                        egui::Grid::new("contract_employees_grid")
                            .striped(true)
                            .spacing([12.0, 8.0])
                            .show(ui, |ui| {
                                ui.label(RichText::new("STT").strong());
                                ui.label(RichText::new("Mã Tiếp Nhận").strong());
                                ui.label(RichText::new("Số Lưu Trữ (GKSK)").strong());
                                ui.label(RichText::new("Họ và Tên").strong());
                                ui.label(RichText::new("GT").strong());
                                ui.label(RichText::new("Ngày sinh").strong());
                                ui.label(RichText::new("Số CCCD").strong());
                                ui.label(RichText::new("Bộ phận").strong());
                                ui.label(RichText::new("Loại SK").strong());
                                ui.label(RichText::new("File Scan").strong());
                                ui.label(RichText::new("Thao tác").strong());
                                ui.end_row();

                                for (idx, r) in ct_records.iter().filter(|r| {
                                    q.is_empty()
                                        || r.ho_ten.to_lowercase().contains(&q)
                                        || r.data.ma_lk.to_lowercase().contains(&q)
                                        || r.data.ma_y_te.to_lowercase().contains(&q)
                                        || r.data.so_luu_tru.to_lowercase().contains(&q)
                                        || r.so_cccd.to_lowercase().contains(&q)
                                }).enumerate() {
                                    ui.label((idx + 1).to_string());
                                    let ma_tn = if !r.data.ma_lk.is_empty() { &r.data.ma_lk } else { &r.data.ma_y_te };
                                    ui.label(RichText::new(ma_tn).monospace().strong());
                                    if r.data.so_luu_tru.trim().is_empty() {
                                        ui.label(RichText::new("Chưa cấp số").color(Color32::from_rgb(217, 119, 6)));
                                    } else {
                                        ui.label(RichText::new(&r.data.so_luu_tru).monospace().strong().color(Color32::from_rgb(5, 150, 105)));
                                    }
                                    ui.label(RichText::new(&r.ho_ten).strong());
                                    ui.label(if r.data.gioi_tinh == 2 { "Nữ" } else { "Nam" });
                                    ui.label(&r.data.ngay_sinh);
                                    ui.label(&r.so_cccd);
                                    ui.label(&r.data.bo_phan);
                                    ui_score_badge(ui, r.data.phan_loai_sk, false);

                                    let rec_scans: Vec<&AttachmentRecord> = attachments
                                        .iter()
                                        .filter(|a| a.owner_type == "record" && a.owner_id == r.id)
                                        .collect();
                                    if rec_scans.is_empty() {
                                        ui.label(RichText::new("Chưa scan").size(11.0).color(Color32::from_rgb(148, 163, 184)));
                                    } else if let Some(first_scan) = rec_scans.first() {
                                        if ui.small_button(format!("👁 {} file", rec_scans.len())).clicked() {
                                            action = CompanyPageAction::OpenAttachment(first_scan.file_path.clone());
                                        }
                                    }

                                    ui.horizontal(|ui| {
                                        if ui.button(RichText::new("⚡ Nhập 1 cửa & Cấp số").size(11.0).strong().color(Color32::from_rgb(14, 165, 233))).clicked() {
                                            action = CompanyPageAction::OpenRecordFastEntry(r.id);
                                        }
                                        if ui.button(RichText::new("🖨 Mẫu 03").size(11.0)).clicked() {
                                            action = CompanyPageAction::PrintSingleMau03(r.id);
                                        }
                                        if ui.button(RichText::new("📎 Đính kèm Scan").size(11.0)).clicked() {
                                            action = CompanyPageAction::AttachRecordScan(r.id);
                                        }
                                    });
                                    ui.end_row();
                                }
                            });
                    });
                }
            }
        });

        action
    }
}
