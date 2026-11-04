use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, TimeZone, Timelike, Utc};

use crate::error::{Error, Result};
use crate::number;
use crate::value::Value;

/// Milliseconds since the Unix epoch, now.
#[must_use]
pub fn now() -> i64 {
    Utc::now().timestamp_millis()
}

/// Parses the date forms the language accepts: `YYYY-M-D`, an ISO timestamp, or
/// an RFC 2822 style string such as `04 Dec 1995 00:12:00 GMT`.
#[must_use]
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
#[must_use]
pub fn to_date_string(millis: i64) -> String {
    datetime(millis).format("%a %b %d %Y").to_string()
}

#[must_use]
pub fn to_string(millis: i64) -> String {
    datetime(millis)
        .format("%a %b %d %Y %H:%M:%S GMT+0000")
        .to_string()
}

#[must_use]
pub fn to_iso_string(millis: i64) -> String {
    datetime(millis)
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string()
}

/// The year minus 1900, as the language's `getYear` reports it.
#[must_use]
pub fn get_year(millis: i64) -> i64 {
    i64::from(datetime(millis).year()) - 1900
}

#[must_use]
pub fn get_full_year(millis: i64) -> i64 {
    i64::from(datetime(millis).year())
}

#[must_use]
pub fn get_month(millis: i64) -> i64 {
    i64::from(datetime(millis).month0())
}

#[must_use]
pub fn get_date(millis: i64) -> i64 {
    i64::from(datetime(millis).day())
}

#[must_use]
pub fn get_day(millis: i64) -> i64 {
    i64::from(datetime(millis).weekday().num_days_from_sunday())
}

#[must_use]
pub fn get_hours(millis: i64) -> i64 {
    i64::from(datetime(millis).hour())
}

#[must_use]
pub fn get_minutes(millis: i64) -> i64 {
    i64::from(datetime(millis).minute())
}

#[must_use]
pub fn get_seconds(millis: i64) -> i64 {
    i64::from(datetime(millis).second())
}

/// `Date(...)`.
pub fn construct(arguments: &[Value]) -> Result<Value> {
    match arguments.first() {
        None => Ok(Value::Date(now())),
        Some(Value::Number(millis)) => Ok(Value::Date(number::to_i64(*millis))),
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
        "now" | "UTC" => Ok(Value::Number(number::from_i64(now()))),
        "parse" => {
            let text = arguments
                .first()
                .cloned()
                .unwrap_or(Value::Undefined)
                .to_string();

            Ok(match parse(&text) {
                Some(millis) => Value::Number(number::from_i64(millis)),
                None => Value::Number(f64::NAN),
            })
        }
        _ => Err(Error::type_error(format!("Date.{name} is not a function"))),
    }
}

/// A method called on a date value.
pub fn method(millis: i64, name: &str, _arguments: &[Value]) -> Result<Value> {
    Ok(match name {
        "getTime" | "valueOf" => Value::Number(number::from_i64(millis)),
        "getYear" => Value::Number(number::from_i64(get_year(millis))),
        "getFullYear" => Value::Number(number::from_i64(get_full_year(millis))),
        "getMonth" => Value::Number(number::from_i64(get_month(millis))),
        "getDate" => Value::Number(number::from_i64(get_date(millis))),
        "getDay" => Value::Number(number::from_i64(get_day(millis))),
        "getHours" => Value::Number(number::from_i64(get_hours(millis))),
        "getMinutes" => Value::Number(number::from_i64(get_minutes(millis))),
        "getSeconds" => Value::Number(number::from_i64(get_seconds(millis))),
        "toDateString" => Value::String(to_date_string(millis)),
        "toISOString" | "toJSON" => Value::String(to_iso_string(millis)),
        "toString" => Value::String(to_string(millis)),
        _ => {
            return Err(Error::type_error(format!("Date.{name} is not a function")));
        }
    })
}
