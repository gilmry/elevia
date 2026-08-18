use chrono::{Datelike, NaiveDate, Utc};

/// Parses a "YYYY-MM" string into the first day of that month.
pub fn parse_month(value: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(&format!("{value}-01"), "%Y-%m-%d")
        .map_err(|_| format!("invalid month '{value}', expected format YYYY-MM"))
}

/// The first day of the current month, used to key "this month's submissions".
pub fn current_month_start() -> NaiveDate {
    Utc::now()
        .date_naive()
        .with_day(1)
        .expect("day 1 is always valid")
}

/// Formats a date as "YYYY-MM" (the day is ignored).
pub fn format_month(date: NaiveDate) -> String {
    date.format("%Y-%m").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_month() {
        assert_eq!(
            parse_month("2026-03").unwrap(),
            NaiveDate::from_ymd_opt(2026, 3, 1).unwrap()
        );
    }

    #[test]
    fn rejects_invalid_month() {
        assert!(parse_month("2026-13").is_err());
        assert!(parse_month("not-a-month").is_err());
    }

    #[test]
    fn formats_back_to_year_month() {
        assert_eq!(
            format_month(NaiveDate::from_ymd_opt(2026, 3, 15).unwrap()),
            "2026-03"
        );
    }
}
