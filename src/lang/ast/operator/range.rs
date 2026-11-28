use crate::error::{Error, Result};
use crate::number;
use crate::value::Value;

const MAX_ITEMS: i64 = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum End {
    Excluded,
    Included,
}

fn bound(value: &Value) -> Option<i64> {
    match value {
        Value::Number(number) => number::safe_integer(*number),
        _ => None,
    }
}

pub(super) fn integers(start: &Value, end: &Value, last: End) -> Result<Value> {
    let (Some(start), Some(end)) = (bound(start), bound(end)) else {
        return Err(Error::type_error("Range bounds must be integers"));
    };
    let end = match last {
        End::Excluded => end,
        End::Included => end + 1,
    };
    if end - start > MAX_ITEMS {
        return Err(Error::type_error("Range is too large"));
    }
    Ok(Value::List(
        (start..end)
            .map(|item| Value::Number(number::from_i64(item)))
            .collect(),
    ))
}
