use eframe::egui;
use crate::models::catalog::CatalogRecord;

pub struct SearchCombo;

impl SearchCombo {
    pub fn show(
        ui: &mut egui::Ui,
        id_source: &str,
        selected_name: &mut String,
        catalogs: &[CatalogRecord],
        catalog_type: &str,
        width: f32,
        hint: &str,
    ) -> bool {
        let items: Vec<(String, String)> = catalogs
            .iter()
            .filter(|c| c.cat_type == catalog_type && c.is_active)
            .map(|c| (c.code.clone(), c.name.clone()))
            .collect();

        search_combo_impl(ui, id_source, selected_name, &items, width, hint)
    }

    pub fn show_filtered(
        ui: &mut egui::Ui,
        id_source: &str,
        selected_name: &mut String,
        catalogs: &[CatalogRecord],
        catalog_type: &str,
        parent_code: Option<&str>,
        width: f32,
        hint: &str,
    ) -> bool {
        let items: Vec<(String, String)> = catalogs
            .iter()
            .filter(|c| c.cat_type == catalog_type && c.is_active)
            .filter(|c| match parent_code {
                Some(p) => c.parent_code == p,
                None => true,
            })
            .map(|c| (c.code.clone(), c.name.clone()))
            .collect();

        search_combo_impl(ui, id_source, selected_name, &items, width, hint)
    }
}

fn search_combo_impl(
    ui: &mut egui::Ui,
    id_source: &str,
    selected_val: &mut String,
    items: &[(String, String)],
    width: f32,
    hint: &str,
) -> bool {
    let mut changed = false;
    let selected_label = items.iter().find(|(code, name)| code == selected_val || name == selected_val)
        .map(|(_, name)| name.clone())
        .unwrap_or_else(|| {
            if selected_val.trim().is_empty() { hint.to_string() } else { selected_val.clone() }
        });

    let search_id = ui.make_persistent_id(format!("{}_search", id_source));
    let mut search_kw = ui.data_mut(|d| d.get_temp::<String>(search_id).unwrap_or_default());

    egui::ComboBox::from_id_source(id_source)
        .width(width)
        .selected_text(selected_label)
        .show_ui(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("🔍");
                if ui.add(egui::TextEdit::singleline(&mut search_kw).hint_text("Tìm nhanh...")).changed() {
                    ui.data_mut(|d| d.insert_temp(search_id, search_kw.clone()));
                }
            });
            ui.separator();

            let kw_lower = search_kw.to_lowercase();
            egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                for (code, name) in items {
                    if kw_lower.is_empty() || code.to_lowercase().contains(&kw_lower) || name.to_lowercase().contains(&kw_lower) {
                        let is_sel = selected_val == code || selected_val == name;
                        if ui.selectable_label(is_sel, format!("{}: {}", code, name)).clicked() {
                            *selected_val = name.clone();
                            changed = true;
                        }
                    }
                }
            });
        });
    changed
}

// BỘ CHỌN NHIỀU MÃ ICD-10 CÁCH NHAU BẰNG DẤU CHẤM PHẨY
pub fn multi_icd(ui: &mut egui::Ui, id_source: &str, target_val: &mut String, items: &[(String, String)]) {
    ui.horizontal(|ui| {
        ui.add(egui::TextEdit::singleline(target_val).desired_width(220.0).hint_text("Mã ICD-10 (nối bằng ;)"));
        let mut temp_code = String::new();
        if search_combo_impl(ui, id_source, &mut temp_code, items, 180.0, "Chọn ICD-10...") && !temp_code.is_empty() {
            let mut list: Vec<String> = target_val.split(';')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if !list.contains(&temp_code) {
                list.push(temp_code);
                *target_val = list.join("; ");
            }
        }
    });
}
