pub fn is_valid_date(d: u32, m: u32, y: u32) -> bool {
    if y < 1900 || y > 2100 || m < 1 || m > 12 || d < 1 {
        return false;
    }
    let max_days = match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0) { 29 } else { 28 }
        }
        _ => return false,
    };
    d <= max_days
}

pub fn parse_dmy(s: &str) -> Option<(u32, u32, u32)> {
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() == 3 {
        if let (Ok(d), Ok(m), Ok(y)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>(), parts[2].parse::<u32>()) {
            if is_valid_date(d, m, y) {
                return Some((d, m, y));
            }
        }
    }
    None
}

// TÍNH TUỔI TỪ NGÀY SINH (DD/MM/YYYY)
pub fn calculate_age(ngay_sinh: &str) -> u32 {
    use chrono::Datelike;
    if let Some((_, _, y)) = parse_dmy(ngay_sinh) {
        let now = chrono::Local::now();
        let cur_year = now.year() as u32;
        if cur_year >= y {
            return cur_year - y;
        }
    }
    30 // Mặc định 30 nếu chưa nhập ngày sinh
}
