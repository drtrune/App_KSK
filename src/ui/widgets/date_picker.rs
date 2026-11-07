use eframe::egui;
use chrono::Datelike;
use crate::domain::dates::{is_valid_date, parse_dmy};

pub struct DatePicker;

impl DatePicker {
    pub fn show(ui: &mut egui::Ui, id_source: &str, date_str: &mut String) -> bool {
        date_input_picker(ui, id_source, date_str)
    }
}

/// Tự động chèn dấu '/' thông minh khi người dùng gõ ngày tháng (dd/mm/yyyy).
/// - Khi gõ đủ 2 số ngày -> tự chèn '/' (thành "dd/")
/// - Nếu người dùng tự bấm phím '/' -> không chèn thêm dấu '/' thừa (không bị "dd//")
/// - Khi gõ đủ 2 số tháng -> tự chèn '/' (thành "dd/mm/")
/// - Khi xóa lùi (Backspace) -> không can thiệp để xóa tự nhiên
pub fn format_date_input_auto_slash(old_val: &str, new_val: &str) -> String {
    // Nếu người dùng đang xóa lùi (độ dài giảm), giữ nguyên thao tác xóa
    if new_val.len() < old_val.len() {
        return new_val.to_string();
    }

    // Lọc ký tự: chỉ chấp nhận số và dấu '/'
    let clean: String = new_val.chars().filter(|c| c.is_ascii_digit() || *c == '/').collect();

    // Loại bỏ các dấu '/' liên tiếp (người dùng bấm nút '/' khi đã có '/' thì không chèn thêm)
    let mut no_double_slash = String::new();
    let mut prev_is_slash = false;
    for c in clean.chars() {
        if c == '/' {
            if !prev_is_slash && !no_double_slash.is_empty() {
                no_double_slash.push(c);
                prev_is_slash = true;
            }
        } else {
            no_double_slash.push(c);
            prev_is_slash = false;
        }
    }

    let parts: Vec<&str> = no_double_slash.split('/').collect();

    if parts.len() == 1 {
        let day_digits = parts[0];
        if day_digits.len() == 2 && old_val.len() < 2 {
            return format!("{}/", day_digits);
        } else if day_digits.len() > 2 {
            let day = &day_digits[..2];
            let rest = &day_digits[2..];
            if rest.len() >= 2 {
                let month = &rest[..2];
                let year = &rest[2..rest.len().min(6)];
                if year.is_empty() {
                    return format!("{}/{}/", day, month);
                } else {
                    return format!("{}/{}/{}", day, month, year);
                }
            } else {
                return format!("{}/{}", day, rest);
            }
        }
        return no_double_slash;
    } else if parts.len() == 2 {
        let day = parts[0];
        let month_digits = parts[1];
        if month_digits.len() == 2 && !old_val.ends_with('/') {
            return format!("{}/{}/", day, month_digits);
        } else if month_digits.len() > 2 {
            let month = &month_digits[..2];
            let year = &month_digits[2..month_digits.len().min(6)];
            if year.is_empty() {
                return format!("{}/{}/", day, month);
            } else {
                return format!("{}/{}/{}", day, month, year);
            }
        }
        return no_double_slash;
    } else if parts.len() >= 3 {
        let day = parts[0];
        let month = parts[1];
        let year: String = parts[2..].join("").chars().take(4).collect();
        return format!("{}/{}/{}", day, month, year);
    }

    no_double_slash
}

pub fn date_input_picker(ui: &mut egui::Ui, id_source: &str, date_str: &mut String) -> bool {
    let mut changed = false;

    // Lưu và so sánh với giá trị trước đó để nhận diện Backspace
    let old_val_id = ui.make_persistent_id(format!("{}_old_text", id_source));
    let old_val: String = ui.data(|d| d.get_temp(old_val_id).unwrap_or_else(|| date_str.clone()));

    let mut is_error = false;
    if !date_str.is_empty() {
        if let Some((d, m, y)) = parse_dmy(date_str) {
            if !is_valid_date(d, m, y) {
                is_error = true;
            }
        } else if date_str.len() >= 10 {
            is_error = true;
        }
    }

    let cal_id = ui.make_persistent_id(format!("{}_cal_state", id_source));
    let mut view_date: (u32, u32) = ui.data_mut(|d| {
        *d.get_temp_mut_or_insert_with(cal_id, || {
            if let Some((_, m, y)) = parse_dmy(date_str) {
                (m, y)
            } else {
                let now = chrono::Local::now();
                (now.month(), now.year() as u32)
            }
        })
    });

    ui.horizontal(|ui| {
        let mut text_edit = egui::TextEdit::singleline(date_str).desired_width(115.0).hint_text("DD/MM/YYYY");
        if is_error {
            text_edit = text_edit.text_color(egui::Color32::from_rgb(220, 38, 38));
        }

        let resp = ui.add(text_edit);
        if resp.changed() {
            let formatted = format_date_input_auto_slash(&old_val, date_str);
            *date_str = formatted;
            ui.data_mut(|d| d.insert_temp(old_val_id, date_str.clone()));

            // Nếu người dùng vừa nhập xong ngày tháng năm hợp lệ, cập nhật luôn view_date của lịch
            if let Some((_, m, y)) = parse_dmy(date_str) {
                view_date = (m, y);
                ui.data_mut(|d| d.insert_temp(cal_id, view_date));
            }
            changed = true;
        } else {
            ui.data_mut(|d| d.insert_temp(old_val_id, date_str.clone()));
        }

        // Warning hint
        if is_error {
            ui.label(egui::RichText::new("⚠").size(12.0).color(egui::Color32::from_rgb(220, 38, 38)))
                .on_hover_text("Định dạng ngày không hợp lệ. Vui lòng nhập ngày dạng DD/MM/YYYY");
        }

        // Date picker popup button
        let popup_id = ui.make_persistent_id(format!("{}_popup", id_source));
        let btn_resp = ui.small_button("📅");
        if btn_resp.clicked() {
            if let Some((_, m, y)) = parse_dmy(date_str) {
                view_date = (m, y);
                ui.data_mut(|d| d.insert_temp(cal_id, view_date));
            }
            ui.memory_mut(|m| m.toggle_popup(popup_id));
        }

        egui::popup_below_widget(ui, popup_id, &btn_resp, |ui| {
            ui.set_min_width(240.0);
            ui.vertical(|ui| {
                // Header: Lùi / Tiến Năm và Tháng nhanh chóng
                ui.horizontal(|ui| {
                    if ui.button("««").on_hover_text("Lùi 1 năm").clicked() {
                        if view_date.1 > 1920 { view_date.1 -= 1; }
                    }
                    if ui.button("«").on_hover_text("Lùi 1 tháng").clicked() {
                        if view_date.0 == 1 {
                            view_date.0 = 12;
                            view_date.1 -= 1;
                        } else {
                            view_date.0 -= 1;
                        }
                    }

                    // Chọn tháng nhanh bằng ComboBox
                    egui::ComboBox::from_id_source(format!("{}_combo_m", id_source))
                        .width(70.0)
                        .selected_text(format!("Tháng {}", view_date.0))
                        .show_ui(ui, |ui| {
                            for m in 1..=12 {
                                ui.selectable_value(&mut view_date.0, m, format!("Tháng {}", m));
                            }
                        });

                    // Chọn năm nhanh bằng DragValue
                    ui.add(egui::DragValue::new(&mut view_date.1).clamp_range(1920..=2050).speed(1));

                    if ui.button("»").on_hover_text("Tiến 1 tháng").clicked() {
                        if view_date.0 == 12 {
                            view_date.0 = 1;
                            view_date.1 += 1;
                        } else {
                            view_date.0 += 1;
                        }
                    }
                    if ui.button("»»").on_hover_text("Tiến 1 năm").clicked() {
                        if view_date.1 < 2050 { view_date.1 += 1; }
                    }

                    ui.data_mut(|d| d.insert_temp(cal_id, view_date));
                });

                // Nút chọn Hôm nay
                ui.horizontal(|ui| {
                    let now = chrono::Local::now();
                    if ui.small_button("⚡ Hôm nay").clicked() {
                        *date_str = format!("{:02}/{:02}/{:04}", now.day(), now.month(), now.year());
                        view_date = (now.month(), now.year() as u32);
                        ui.data_mut(|d| d.insert_temp(cal_id, view_date));
                        changed = true;
                        ui.memory_mut(|m| m.close_popup());
                    }
                });

                ui.separator();

                // Days grid: Tính chính xác thứ trong tuần của ngày 1
                let days_in_month = match view_date.0 {
                    1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
                    4 | 6 | 9 | 11 => 30,
                    2 => {
                        let y = view_date.1;
                        if (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0) { 29 } else { 28 }
                    }
                    _ => 30,
                };

                let first_weekday = chrono::NaiveDate::from_ymd_opt(view_date.1 as i32, view_date.0, 1)
                    .map(|d| d.weekday().num_days_from_monday())
                    .unwrap_or(0);

                let now = chrono::Local::now();

                egui::Grid::new(format!("{}_days", id_source)).spacing([2.0, 2.0]).show(ui, |ui| {
                    for col in ["T2", "T3", "T4", "T5", "T6", "T7", "CN"] {
                        ui.label(egui::RichText::new(col).size(10.5).strong());
                    }
                    ui.end_row();

                    // Thụt lề các ô trống trước ngày 1
                    for _ in 0..first_weekday {
                        ui.label("");
                    }

                    for day in 1..=days_in_month {
                        let is_today = day == now.day() && view_date.0 == now.month() && view_date.1 == now.year() as u32;
                        let btn_text = if is_today {
                            egui::RichText::new(day.to_string()).strong().color(egui::Color32::from_rgb(37, 99, 235))
                        } else {
                            egui::RichText::new(day.to_string())
                        };

                        if ui.add_sized([26.0, 22.0], egui::Button::new(btn_text)).clicked() {
                            *date_str = format!("{:02}/{:02}/{:04}", day, view_date.0, view_date.1);
                            changed = true;
                            ui.memory_mut(|m| m.close_popup());
                        }

                        if (first_weekday + day as u32) % 7 == 0 {
                            ui.end_row();
                        }
                    }
                });
            });
        });
    });

    changed
}
