use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, TimeZone, Timelike, Utc};

/// Milliseconds since the Unix epoch, now.
pub fn now() -> i64 {
    Utc::now().timestamp_millis()
}

/// Parses the date forms the language accepts: `YYYY-M-D`, an ISO timestamp, or
/// an RFC 2822 style string such as `04 Dec 1995 00:12:00 GMT`.
pub fn parse(input: &str) -> Option<i64> {
    let trimmed = input.trim();

    if let Ok(date) = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        return date
            .and_hms_opt(0, 0, 0)
            .map(|datetime| Utc.from_utc_datetime(&datetime).timestamp_millis());
    }

    if let Ok(datetime) = DateTime::parse_from_rfc3339(trimmed) {
        return Some(datetime.timestamp_millis());
    }

    if let Ok(datetime) = DateTime::parse_from_rfc2822(trimmed) {
        return Some(datetime.timestamp_millis());
    }

    for format in [
        "%d %b %Y %H:%M:%S GMT",
        "%d %b %Y %H:%M:%S",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S",
        "%b %d %Y",
        "%m/%d/%Y",
    ] {
        if let Ok(datetime) = NaiveDateTime::parse_from_str(trimmed, format) {
            return Some(Utc.from_utc_datetime(&datetime).timestamp_millis());
        }

        if let Ok(date) = NaiveDate::parse_from_str(trimmed, format) {
            if let Some(datetime) = date.and_hms_opt(0, 0, 0) {
                return Some(Utc.from_utc_datetime(&datetime).timestamp_millis());
            }
        }
    }

    None
}

fn datetime(millis: i64) -> DateTime<Utc> {
    Utc.timestamp_millis_opt(millis)
        .single()
        .unwrap_or_default()
}

/// `Wed Jan 01 2020`
pub fn to_date_string(millis: i64) -> String {
    datetime(millis).format("%a %b %d %Y").to_string()
}

pub fn to_string(millis: i64) -> String {
    datetime(millis)
        .format("%a %b %d %Y %H:%M:%S GMT+0000")
        .to_string()
}

pub fn to_iso_string(millis: i64) -> String {
    datetime(millis)
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string()
}

/// The year minus 1900, as the language's `getYear` reports it.
pub fn get_year(millis: i64) -> i64 {
    datetime(millis).year() as i64 - 1900
}

pub fn get_full_year(millis: i64) -> i64 {
    datetime(millis).year() as i64
}

pub fn get_month(millis: i64) -> i64 {
    datetime(millis).month0() as i64
}

pub fn get_date(millis: i64) -> i64 {
    datetime(millis).day() as i64
}

pub fn get_day(millis: i64) -> i64 {
    datetime(millis).weekday().num_days_from_sunday() as i64
}

pub fn get_hours(millis: i64) -> i64 {
    datetime(millis).hour() as i64
}

pub fn get_minutes(millis: i64) -> i64 {
    datetime(millis).minute() as i64
}

pub fn get_seconds(millis: i64) -> i64 {
    datetime(millis).second() as i64
}
