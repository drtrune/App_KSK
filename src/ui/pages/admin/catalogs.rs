use egui::{Color32, RichText, Ui, Vec2};
use crate::models::catalog::{CatalogRecord, CatalogTypeInfo};

pub struct AdminCatalogsPage;

pub enum CatalogAction {
    None,
    Save(CatalogRecord),
    Delete(i64),
    ToggleActive(i64, bool),
}

pub struct CatalogEditState {
    pub is_open: bool,
    pub id: Option<i64>,
    pub cat_type: String,
    pub code: String,
    pub name: String,
    pub parent_code: String,
    pub sort_order: i32,
    pub is_active: bool,
}

impl Default for CatalogEditState {
    fn default() -> Self {
        Self {
            is_open: false,
            id: None,
            cat_type: "priority".into(),
            code: "".into(),
            name: "".into(),
            parent_code: "".into(),
            sort_order: 0,
            is_active: true,
        }
    }
}

impl AdminCatalogsPage {
    pub fn show(
        ui: &mut Ui,
        catalogs: &[CatalogRecord],
        selected_type: &mut String,
        search_query: &mut String,
        edit_state: &mut CatalogEditState,
        dark_mode: bool,
    ) -> CatalogAction {
        let mut action = CatalogAction::None;
        let all_types = CatalogTypeInfo::all();

        ui.vertical(|ui| {
            // Header
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.heading(RichText::new("QUẢN LÝ DANH MỤC DÙNG CHUNG").size(18.0).strong());
                    ui.label(RichText::new("Định nghĩa, thêm mới, sửa, xóa và cấu hình các danh mục hệ thống.")
                        .size(12.0)
                        .color(Color32::from_rgb(100, 116, 139)));
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let add_btn = egui::Button::new(RichText::new("➕  Thêm danh mục mới").strong().size(12.0).color(Color32::WHITE))
                        .fill(Color32::from_rgb(14, 165, 233))
                        .rounding(6.0)
                        .min_size(Vec2::new(140.0, 30.0));

                    if ui.add(add_btn).clicked() {
                        edit_state.is_open = true;
                        edit_state.id = None;
                        edit_state.cat_type = selected_type.clone();
                        edit_state.code = "".into();
                        edit_state.name = "".into();
                        edit_state.parent_code = "".into();
                        edit_state.sort_order = 0;
                        edit_state.is_active = true;
                    }
                });
            });

            ui.add_space(10.0);

            // Catalog Type Selector Pills
            ui.horizontal_wrapped(|ui| {
                for t in all_types {
                    let is_sel = *selected_type == t.cat_type;
                    let text = format!("{} ({})", t.title, catalogs.iter().filter(|c| c.cat_type == t.cat_type).count());
                    let btn_bg = if is_sel {
                        if dark_mode { Color32::from_rgb(30, 41, 59) } else { Color32::from_rgb(224, 242, 254) }
                    } else {
                        Color32::TRANSPARENT
                    };
                    let text_color = if is_sel {
                        Color32::from_rgb(14, 165, 233)
                    } else if dark_mode {
                        Color32::from_rgb(203, 213, 225)
                    } else {
                        Color32::from_rgb(71, 85, 105)
                    };

                    let btn = egui::Button::new(RichText::new(text).size(12.0).color(text_color).strong())
                        .fill(btn_bg)
                        .rounding(4.0);

                    if ui.add(btn).clicked() {
                        *selected_type = t.cat_type.to_string();
                    }
                }
            });

            ui.add_space(8.0);

            // Search filter
            ui.horizontal(|ui| {
                ui.label(RichText::new("Tìm kiếm:").size(12.0));
                ui.add(egui::TextEdit::singleline(search_query).hint_text("Tìm mã hoặc tên danh mục...").desired_width(260.0));
                if !search_query.is_empty() && ui.button("✖").clicked() {
                    search_query.clear();
                }
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(6.0);

            // Filter items
            let q = search_query.trim().to_lowercase();
            let items: Vec<&CatalogRecord> = catalogs
                .iter()
                .filter(|c| c.cat_type == *selected_type)
                .filter(|c| q.is_empty() || c.code.to_lowercase().contains(&q) || c.name.to_lowercase().contains(&q))
                .collect();

            // Table of items
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    egui::Grid::new("grid_catalogs_table")
                        .striped(true)
                        .spacing([16.0, 8.0])
                        .min_col_width(60.0)
                        .show(ui, |ui| {
                            ui.label(RichText::new("STT").strong().size(11.5).color(Color32::from_rgb(100, 116, 139)));
                            ui.label(RichText::new("Mã code").strong().size(11.5).color(Color32::from_rgb(100, 116, 139)));
                            ui.label(RichText::new("Tên hiển thị").strong().size(11.5).color(Color32::from_rgb(100, 116, 139)));
                            ui.label(RichText::new("Mã cha").strong().size(11.5).color(Color32::from_rgb(100, 116, 139)));
                            ui.label(RichText::new("Thứ tự").strong().size(11.5).color(Color32::from_rgb(100, 116, 139)));
                            ui.label(RichText::new("Trạng thái").strong().size(11.5).color(Color32::from_rgb(100, 116, 139)));
                            ui.label(RichText::new("Thao tác").strong().size(11.5).color(Color32::from_rgb(100, 116, 139)));
                            ui.end_row();

                            for (idx, it) in items.iter().enumerate() {
                                ui.label((idx + 1).to_string());
                                ui.label(RichText::new(&it.code).strong().monospace().size(12.0));
                                ui.label(RichText::new(&it.name).size(12.0));
                                ui.label(if it.parent_code.is_empty() { "-" } else { &it.parent_code });
                                ui.label(it.sort_order.to_string());

                                let (st_txt, st_col) = if it.is_active {
                                    ("Đang dùng", Color32::from_rgb(16, 185, 129))
                                } else {
                                    ("Tạm ẩn", Color32::from_rgb(156, 163, 175))
                                };
                                ui.colored_label(st_col, st_txt);

                                ui.horizontal(|ui| {
                                    if ui.button(RichText::new("Sửa").size(11.0).color(Color32::from_rgb(14, 165, 233))).clicked() {
                                        edit_state.is_open = true;
                                        edit_state.id = Some(it.id);
                                        edit_state.cat_type = it.cat_type.clone();
                                        edit_state.code = it.code.clone();
                                        edit_state.name = it.name.clone();
                                        edit_state.parent_code = it.parent_code.clone();
                                        edit_state.sort_order = it.sort_order;
                                        edit_state.is_active = it.is_active;
                                    }

                                    let toggle_txt = if it.is_active { "Ẩn" } else { "Hiện" };
                                    if ui.button(RichText::new(toggle_txt).size(11.0)).clicked() {
                                        action = CatalogAction::ToggleActive(it.id, !it.is_active);
                                    }

                                    if ui.button(RichText::new("Xóa").size(11.0).color(Color32::from_rgb(239, 68, 68))).clicked() {
                                        action = CatalogAction::Delete(it.id);
                                    }
                                });

                                ui.end_row();
                            }
                        });
                });
        });

        // Edit Modal / Window
        if edit_state.is_open {
            egui::Window::new(if edit_state.id.is_some() { "Chỉnh sửa mục danh mục" } else { "Thêm mới mục danh mục" })
                .collapsible(false)
                .resizable(false)
                .min_width(380.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new(format!("Loại danh mục: {}", edit_state.cat_type)).size(12.0).color(Color32::from_rgb(100, 116, 139)));
                        ui.add_space(8.0);

                        ui.label(RichText::new("Mã code (*):").strong().size(12.0));
                        ui.add(egui::TextEdit::singleline(&mut edit_state.code).desired_width(360.0));

                        ui.add_space(6.0);
                        ui.label(RichText::new("Tên hiển thị (*):").strong().size(12.0));
                        ui.add(egui::TextEdit::singleline(&mut edit_state.name).desired_width(360.0));

                        ui.add_space(6.0);
                        ui.label(RichText::new("Mã cha (nếu có, VD mã Tỉnh cho Xã):").size(12.0));
                        ui.add(egui::TextEdit::singleline(&mut edit_state.parent_code).desired_width(360.0));

                        ui.add_space(6.0);
                        ui.label(RichText::new("Thứ tự hiển thị:").size(12.0));
                        ui.add(egui::DragValue::new(&mut edit_state.sort_order));

                        ui.add_space(6.0);
                        ui.checkbox(&mut edit_state.is_active, "Đang sử dụng (Kích hoạt)");

                        ui.add_space(14.0);
                        ui.horizontal(|ui| {
                            let btn = egui::Button::new(RichText::new("Lưu danh mục").strong().color(Color32::WHITE))
                                .fill(Color32::from_rgb(14, 165, 233));
                            if ui.add(btn).clicked() {
                                if !edit_state.code.trim().is_empty() && !edit_state.name.trim().is_empty() {
                                    action = CatalogAction::Save(CatalogRecord {
                                        id: edit_state.id.unwrap_or(0),
                                        cat_type: edit_state.cat_type.clone(),
                                        code: edit_state.code.trim().to_string(),
                                        name: edit_state.name.trim().to_string(),
                                        parent_code: edit_state.parent_code.trim().to_string(),
                                        sort_order: edit_state.sort_order,
                                        is_active: edit_state.is_active,
                                    });
                                    edit_state.is_open = false;
                                }
                            }

                            if ui.button("Hủy").clicked() {
                                edit_state.is_open = false;
                            }
                        });
                    });
                });
        }

        action
    }
}
