use chrono::{Datelike, NaiveDate, Utc};

/// Hitung selisih kalender (tahun, bulan, hari) dari tanggal `dd/mm/yyyy` sampai hari ini.
pub fn calculate_uptime(date_str: &str) -> String {
    let birth = match NaiveDate::parse_from_str(date_str, "%d/%m/%Y") {
        Ok(d) => d,
        Err(_) => return "-".to_string(), // format tanggal salah, fallback aman
    };

    let today = Utc::now().date_naive();

    if birth > today {
        return "-".to_string(); // tanggal di masa depan, nggak masuk akal dihitung
    }

    let mut years = today.year() - birth.year();
    let mut months = today.month() as i32 - birth.month() as i32;
    let mut days = today.day() as i32 - birth.day() as i32;

    if days < 0 {
        months -= 1;
        let (prev_year, prev_month) = if today.month() == 1 {
            (today.year() - 1, 12)
        } else {
            (today.year(), today.month() - 1)
        };
        days += days_in_month(prev_year, prev_month) as i32;
    }

    if months < 0 {
        years -= 1;
        months += 12;
    }

    format!(
        "{} year{}, {} month{}, {} day{}",
        years, if years != 1 { "s" } else { "" },
        months, if months != 1 { "s" } else { "" },
        days, if days != 1 { "s" } else { "" },
    )
}

/// Format angka gaya "power rating" RPG: 999 tetap "999", 1_000 -> "1.00k",
/// 390_447 -> "390.45k", 1_044_061 -> "1.04M", dst. Dibulatkan 2 angka desimal
/// (biar "child value"-nya tetap kelihatan), dan otomatis naik satu tingkat
/// unit kalau pembulatan menyentuh 1000.00 (contoh: 999_996 -> bukan
/// "1000.00k" tapi langsung "1.00M").
pub fn format_power_number(n: u64) -> String {
    const UNITS: [&str; 5] = ["", "k", "M", "B", "T"];

    let mut value = n as f64;
    let mut unit_idx = 0usize;

    while value >= 1000.0 && unit_idx < UNITS.len() - 1 {
        value /= 1000.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        return n.to_string();
    }

    let mut rounded = (value * 100.0).round() / 100.0;
    if rounded >= 1000.0 && unit_idx < UNITS.len() - 1 {
        rounded /= 1000.0;
        unit_idx += 1;
    }

    format!("{:.2}{}", rounded, UNITS[unit_idx])
}

/// Helper: jumlah hari dalam bulan tertentu (buat "pinjam" hari saat days < 0).
fn days_in_month(year: i32, month: u32) -> u32 {
    let next_month = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .unwrap();
    let this_month = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    (next_month - this_month).num_days() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_number_stays_plain_below_1000() {
        assert_eq!(format_power_number(999), "999");
    }

    #[test]
    fn power_number_rounds_up_a_unit() {
        assert_eq!(format_power_number(999_996), "1.00M");
    }
}
