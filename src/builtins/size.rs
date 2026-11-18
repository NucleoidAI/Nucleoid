use crate::number;
use crate::value::Value;

#[must_use]
pub(crate) fn size_query(name: &str, length: impl FnOnce() -> usize) -> Option<Value> {
    Some(match name {
        "length" | "size" | "len" => Value::Number(number::from_usize(length())),
        "isEmpty" | "is_empty" => Value::Bool(length() == 0),
        "isNotEmpty" => Value::Bool(length() > 0),
        _ => return None,
    })
}
