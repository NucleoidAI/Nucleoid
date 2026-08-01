use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, TimeZone, Timelike, Utc};

use crate::error::{Error, Result};
use crate::value::Value;

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

/// `Date(...)`.
pub fn construct(arguments: &[Value]) -> Result<Value> {
    match arguments.first() {
        None => Ok(Value::Date(now())),
        Some(Value::Number(millis)) => Ok(Value::Date(*millis as i64)),
        Some(Value::Date(millis)) => Ok(Value::Date(*millis)),
        Some(Value::String(text)) => match parse(text) {
            Some(millis) => Ok(Value::Date(millis)),
            None => Err(Error::type_error(format!("Invalid date '{text}'"))),
        },
        Some(other) => Err(Error::type_error(format!(
            "Cannot create a date from {}",
            other.type_name()
        ))),
    }
}

/// `Date.now()` and friends, called on the namespace rather than a date.
pub fn statics(name: &str, arguments: &[Value]) -> Result<Value> {
    match name {
        "now" => Ok(Value::Number(now() as f64)),
        "parse" => {
            let text = arguments
                .first()
                .cloned()
                .unwrap_or(Value::Undefined)
                .to_string();

            Ok(match parse(&text) {
                Some(millis) => Value::Number(millis as f64),
                None => Value::Number(f64::NAN),
            })
        }
        "UTC" => Ok(Value::Number(now() as f64)),
        _ => Err(Error::type_error(format!("Date.{name} is not a function"))),
    }
}

/// A method called on a date value.
pub fn method(millis: i64, name: &str, _arguments: &[Value]) -> Result<Value> {
    Ok(match name {
        "getTime" | "valueOf" => Value::Number(millis as f64),
        "getYear" => Value::Number(get_year(millis) as f64),
        "getFullYear" => Value::Number(get_full_year(millis) as f64),
        "getMonth" => Value::Number(get_month(millis) as f64),
        "getDate" => Value::Number(get_date(millis) as f64),
        "getDay" => Value::Number(get_day(millis) as f64),
        "getHours" => Value::Number(get_hours(millis) as f64),
        "getMinutes" => Value::Number(get_minutes(millis) as f64),
        "getSeconds" => Value::Number(get_seconds(millis) as f64),
        "toDateString" => Value::String(to_date_string(millis)),
        "toISOString" | "toJSON" => Value::String(to_iso_string(millis)),
        "toString" => Value::String(to_string(millis)),
        _ => {
            return Err(Error::type_error(format!("Date.{name} is not a function")));
        }
    })
}
