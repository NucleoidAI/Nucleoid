use crate::error::{Error, Result};
use crate::number;
use crate::value::Value;

/// `String.<name>(...)`
pub fn string(name: &str, arguments: &[Value]) -> Result<Value> {
    match name {
        "fromCharCode" => {
            let mut output = String::new();

            for argument in arguments {
                let code = number::to_i64(argument.to_number());
                if let Some(character) = u32::try_from(code).ok().and_then(char::from_u32) {
                    output.push(character);
                }
            }

            Ok(Value::String(output))
        }
        _ => Err(Error::type_error(format!(
            "String.{name} is not a function"
        ))),
    }
}

/// String instance methods.
pub fn string_method(receiver: &str, name: &str, arguments: &[Value]) -> Result<Value> {
    let first = arguments.first().unwrap_or(&Value::Undefined);

    if let Some(value) = string_query(receiver, name, first) {
        return Ok(value);
    }

    string_transform(receiver, name, first, arguments)?
        .ok_or_else(|| Error::type_error(format!("{receiver}.{name} is not a function")))
}

fn string_query(receiver: &str, name: &str, first: &Value) -> Option<Value> {
    Some(match name {
        "charAt" => {
            let index = number::to_usize(first.to_number());
            Value::String(
                receiver
                    .chars()
                    .nth(index)
                    .map(|c| c.to_string())
                    .unwrap_or_default(),
            )
        }
        "charCodeAt" => {
            let index = number::to_usize(first.to_number());
            match receiver.chars().nth(index) {
                Some(character) => Value::Number(f64::from(u32::from(character))),
                None => Value::Number(f64::NAN),
            }
        }
        "indexOf" | "find" => {
            let needle = first.to_string();
            match receiver.find(&needle) {
                Some(index) => Value::Number(number::from_usize(
                    receiver
                        .char_indices()
                        .take_while(|&(offset, _)| offset < index)
                        .count(),
                )),
                None => Value::Number(-1.0),
            }
        }
        "includes" | "contains" => Value::Bool(receiver.contains(&first.to_string())),
        "startsWith" => Value::Bool(receiver.starts_with(&first.to_string())),
        "endsWith" => Value::Bool(receiver.ends_with(&first.to_string())),
        _ => return None,
    })
}

fn string_transform(
    receiver: &str,
    name: &str,
    first: &Value,
    arguments: &[Value],
) -> Result<Option<Value>> {
    Ok(Some(match name {
        "lower" | "toLowerCase" => Value::String(receiver.to_lowercase()),
        "upper" | "toUpperCase" => Value::String(receiver.to_uppercase()),
        "trim" | "strip" => Value::String(receiver.trim().to_string()),
        "split" => {
            let separator = first.to_string();
            let parts: Vec<Value> = if separator.is_empty() {
                receiver
                    .chars()
                    .map(|character| Value::String(character.to_string()))
                    .collect()
            } else {
                receiver
                    .split(separator.as_str())
                    .map(|part| Value::String(part.to_string()))
                    .collect()
            };
            Value::List(parts)
        }
        "replace" => {
            let from = first.to_string();
            let to = arguments.get(1).unwrap_or(&Value::Undefined).to_string();
            Value::String(receiver.replacen(from.as_str(), to.as_str(), 1))
        }
        "replaceAll" => {
            let from = first.to_string();
            let to = arguments.get(1).unwrap_or(&Value::Undefined).to_string();
            Value::String(string_replace_all(receiver, &from, &to)?)
        }
        "repeat" => Value::String(string_repeat(
            receiver,
            number::to_usize(first.to_number().max(0.0)),
        )?),
        "substring" | "slice" | "substr" => {
            let end = arguments
                .get(1)
                .filter(|value| !value.is_nullish())
                .map(Value::to_number);
            Value::String(string_slice(receiver, first.to_number(), end))
        }
        "toString" => Value::String(receiver.to_string()),
        _ => return Ok(None),
    }))
}

pub(super) fn string_replace_all(
    receiver: &str,
    search: &str,
    replacement: &str,
) -> Result<String> {
    // str's literal matcher is non-overlapping; an empty pattern visits only
    // Unicode scalar boundaries, including the start and end exactly once.
    let matches = receiver.match_indices(search).count();
    let mut output =
        string_replacement_buffer(receiver.len(), search.len(), replacement.len(), matches)?;
    let mut pieces = receiver.split(search);
    if let Some(piece) = pieces.next() {
        output.push_str(piece);
    }
    for piece in pieces {
        output.push_str(replacement);
        output.push_str(piece);
    }
    Ok(output)
}

/// Check the complete byte size before writing, so pushes cannot reallocate.
/// Keeping sizing and reservation together also permits safe boundary tests
/// without constructing enormous input strings or requesting huge allocations.
pub(super) fn string_replacement_buffer(
    receiver_len: usize,
    search_len: usize,
    replacement_len: usize,
    matches: usize,
) -> Result<String> {
    let too_large = || Error::type_error("Replaced string is too large");
    let removed = matches.checked_mul(search_len).ok_or_else(too_large)?;
    let inserted = matches.checked_mul(replacement_len).ok_or_else(too_large)?;
    let capacity = receiver_len
        .checked_sub(removed)
        .and_then(|remaining| remaining.checked_add(inserted))
        .ok_or_else(too_large)?;
    let mut output = String::new();
    output
        .try_reserve_exact(capacity)
        .map_err(|_| too_large())?;
    Ok(output)
}

fn string_repeat(receiver: &str, count: usize) -> Result<String> {
    if receiver.is_empty() || count == 0 {
        return Ok(String::new());
    }

    let capacity = receiver
        .len()
        .checked_mul(count)
        .ok_or_else(|| Error::type_error("Repeated string is too large"))?;
    let mut output = String::new();
    output
        .try_reserve_exact(capacity)
        .map_err(|_| Error::type_error("Repeated string is too large"))?;

    for _ in 0..count {
        output.push_str(receiver);
    }

    Ok(output)
}

/// Slices by Unicode scalar indices without allocating a character buffer.
pub(crate) fn string_slice(receiver: &str, start: f64, end: Option<f64>) -> String {
    let length = receiver.chars().count();
    let start = slice_index(start, length);
    let end = end.map_or(length, |end| slice_index(end, length));

    receiver
        .chars()
        .skip(start)
        .take(end.saturating_sub(start))
        .collect()
}

/// Resolves a possibly negative index against a length, the way slicing does.
#[must_use]
pub fn slice_index(index: f64, length: usize) -> usize {
    if index < 0.0 {
        let from_end = number::from_usize(length) + index;
        number::to_usize(from_end.max(0.0))
    } else {
        number::to_usize(index).min(length)
    }
}
