use eframe::egui;
use chrono::Datelike;
use crate::domain::dates::{is_valid_date, parse_dmy};

pub struct DatePicker;

impl DatePicker {
    pub fn show(ui: &mut egui::Ui, id_source: &str, date_str: &mut String) -> bool {
        date_input_picker(ui, id_source, date_str)
    }
}

pub fn date_input_picker(ui: &mut egui::Ui, id_source: &str, date_str: &mut String) -> bool {
    let mut changed = false;
    let mut digits_only = String::new();
    for ch in date_str.chars() {
        if ch.is_ascii_digit() {
            digits_only.push(ch);
            if digits_only.len() == 8 { break; }
        }
    }

    let mut is_error = false;
    if !date_str.is_empty() {
        if let Some((d, m, y)) = parse_dmy(date_str) {
            if !is_valid_date(d, m, y) { is_error = true; }
        } else if date_str.len() >= 10 {
            is_error = true;
        }
    }

    let cal_id = ui.make_persistent_id(format!("{}_cal_state", id_source));
    let mut view_date: (u32, u32) = ui.data_mut(|d| {
        *d.get_temp_mut_or_insert_with(cal_id, || {
            let now = chrono::Local::now();
            (now.month(), now.year() as u32)
        })
    });

    ui.horizontal(|ui| {
        let mut text_edit = egui::TextEdit::singleline(date_str).desired_width(115.0);
        if is_error {
            text_edit = text_edit.text_color(egui::Color32::from_rgb(220, 38, 38));
        }

        let resp = ui.add(text_edit);
        if resp.changed() {
            let mut formatted = String::new();
            let mut d_count = 0;
            for ch in date_str.chars() {
                if ch.is_ascii_digit() && d_count < 8 {
                    formatted.push(ch);
                    d_count += 1;
                    if (d_count == 2 || d_count == 4) && d_count < digits_only.len() {
                        formatted.push('/');
                    }
                }
            }
            if !formatted.is_empty() {
                *date_str = formatted;
            }
            changed = true;
        }

        // Warning hint
        if is_error {
            ui.label(egui::RichText::new("⚠ Sai định dạng").size(11.0).color(egui::Color32::from_rgb(220, 38, 38)));
        }

        // Date picker popup button
        let popup_id = ui.make_persistent_id(format!("{}_popup", id_source));
        let btn_resp = ui.button("📅");
        if btn_resp.clicked() {
            ui.memory_mut(|m| m.toggle_popup(popup_id));
        }

        egui::popup_below_widget(ui, popup_id, &btn_resp, |ui| {
            ui.set_min_width(220.0);
            ui.vertical(|ui| {
                // Header: Tháng / Năm navigation
                ui.horizontal(|ui| {
                    if ui.button("«").clicked() {
                        if view_date.0 == 1 {
                            view_date.0 = 12;
                            view_date.1 -= 1;
                        } else {
                            view_date.0 -= 1;
                        }
                    }

                    ui.label(format!("Tháng {} / {}", view_date.0, view_date.1));

                    if ui.button("»").clicked() {
                        if view_date.0 == 12 {
                            view_date.0 = 1;
                            view_date.1 += 1;
                        } else {
                            view_date.0 += 1;
                        }
                    }

                    ui.data_mut(|d| d.insert_temp(cal_id, view_date));
                });

                ui.separator();

                // Days grid
                let days_in_month = match view_date.0 {
                    1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
                    4 | 6 | 9 | 11 => 30,
                    2 => {
                        let y = view_date.1;
                        if (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0) { 29 } else { 28 }
                    }
                    _ => 30,
                };

                egui::Grid::new(format!("{}_days", id_source)).spacing([4.0, 4.0]).show(ui, |ui| {
                    for col in ["T2", "T3", "T4", "T5", "T6", "T7", "CN"] {
                        ui.label(egui::RichText::new(col).size(10.0).strong());
                    }
                    ui.end_row();

                    for day in 1..=days_in_month {
                        if ui.button(day.to_string()).clicked() {
                            *date_str = format!("{:02}/{:02}/{:04}", day, view_date.0, view_date.1);
                            changed = true;
                            ui.memory_mut(|m| m.close_popup());
                        }
                        if day % 7 == 0 {
                            ui.end_row();
                        }
                    }
                });
            });
        });
    });

    changed
}
