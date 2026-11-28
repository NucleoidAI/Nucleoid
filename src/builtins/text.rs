use super::size::size_query;
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
        "includes" | "contains" => Value::Bool(string_contains(receiver, first)),
        "startsWith" | "startswith" | "starts_with" => {
            Value::Bool(receiver.starts_with(&first.to_string()))
        }
        "endsWith" | "endswith" | "ends_with" => {
            Value::Bool(receiver.ends_with(&first.to_string()))
        }
        _ => return size_query(name, || receiver.chars().count()),
    })
}

fn string_transform(
    receiver: &str,
    name: &str,
    first: &Value,
    arguments: &[Value],
) -> Result<Option<Value>> {
    Ok(Some(match name {
        "lower" | "toLowerCase" | "lowercase" | "to_lowercase" | "lowercased" => {
            Value::String(receiver.to_lowercase())
        }
        "upper" | "toUpperCase" | "uppercase" | "to_uppercase" | "uppercased" => {
            Value::String(receiver.to_uppercase())
        }
        "trim" | "strip" | "Trim" => Value::String(receiver.trim().to_string()),
        "reverse" | "reversed" => Value::String(receiver.chars().rev().collect()),
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
            number::repetitions(first.to_number()),
        )?),
        "padStart" | "rjust" | "PadLeft" => {
            Value::String(string_pad(receiver, first, arguments.get(1), Side::Start)?)
        }
        "padEnd" | "ljust" | "PadRight" => {
            Value::String(string_pad(receiver, first, arguments.get(1), Side::End)?)
        }
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
    let capacity = matches
        .checked_mul(search_len)
        .zip(matches.checked_mul(replacement_len))
        .and_then(|(removed, inserted)| receiver_len.checked_sub(removed)?.checked_add(inserted));
    string_buffer(capacity, "Replaced string is too large")
}

fn string_repeat(receiver: &str, count: usize) -> Result<String> {
    if receiver.is_empty() || count == 0 {
        return Ok(String::new());
    }

    let mut output = string_buffer(
        receiver.len().checked_mul(count),
        "Repeated string is too large",
    )?;

    for _ in 0..count {
        output.push_str(receiver);
    }

    Ok(output)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    Start,
    End,
}

fn string_pad(receiver: &str, length: &Value, fill: Option<&Value>, side: Side) -> Result<String> {
    let missing =
        number::to_usize(length.to_number().max(0.0)).saturating_sub(receiver.chars().count());
    if missing == 0 {
        return Ok(receiver.to_string());
    }

    let fill = match fill {
        None | Some(Value::Undefined) => String::from(" "),
        Some(value) => value.to_string(),
    };
    let fill_length = fill.chars().count();
    if fill_length == 0 {
        return Ok(receiver.to_string());
    }

    let whole = missing / fill_length;
    let rest = missing % fill_length;
    let mut output = string_padding_buffer(receiver.len(), &fill, whole, rest)?;
    match side {
        Side::Start => {
            push_padding(&mut output, &fill, whole, rest);
            output.push_str(receiver);
        }
        Side::End => {
            output.push_str(receiver);
            push_padding(&mut output, &fill, whole, rest);
        }
    }

    Ok(output)
}

fn push_padding(output: &mut String, fill: &str, whole: usize, rest: usize) {
    for _ in 0..whole {
        output.push_str(fill);
    }
    output.extend(fill.chars().take(rest));
}

pub(super) fn string_padding_buffer(
    receiver_len: usize,
    fill: &str,
    whole: usize,
    rest: usize,
) -> Result<String> {
    let cut = fill
        .char_indices()
        .nth(rest)
        .map_or(fill.len(), |(offset, _)| offset);
    let capacity = whole
        .checked_mul(fill.len())
        .and_then(|bytes| bytes.checked_add(cut))
        .and_then(|bytes| bytes.checked_add(receiver_len));
    string_buffer(capacity, "Padded string is too large")
}

fn string_buffer(capacity: Option<usize>, message: &'static str) -> Result<String> {
    let too_large = || Error::type_error(message);
    let mut output = String::new();
    output
        .try_reserve_exact(capacity.ok_or_else(too_large)?)
        .map_err(|_| too_large())?;
    Ok(output)
}

pub(crate) fn string_contains(receiver: &str, needle: &Value) -> bool {
    receiver.contains(&needle.to_string())
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
