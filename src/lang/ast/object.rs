//! Object literals, and the structural comparison objects are matched by.
//! Comparison uses the runtime state to resolve object identities.

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

use indexmap::IndexMap;

use crate::error::Result;
use crate::lang::ast::Expr;
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::{ObjectData, ObjectId, Value};

pub struct Object<'a> {
    pub node: &'a Expr,
}

impl<'a> Object<'a> {
    pub fn new(node: &'a Expr) -> Self {
        Object { node }
    }

    pub fn resolve(&self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Value> {
        let Expr::ObjectLiteral(entries) = self.node else {
            unreachable!("Object only wraps Expr::ObjectLiteral")
        };

        runtime.evaluate_object_literal(entries, scope)
    }
}

impl Runtime {
    pub(crate) fn evaluate_object_literal(
        &mut self,
        entries: &[(String, Expr)],
        scope: &mut Scope,
    ) -> Result<Value> {
        let mut properties = IndexMap::new();

        for (key, value) in entries {
            properties.insert(key.clone(), self.evaluate(value, scope)?);
        }

        Ok(self.create_anonymous(properties))
    }

    /// Creates an object that belongs to no class, used for object literals and
    /// `Object()`. Such objects are skipped by `for ... of`.
    pub(crate) fn create_anonymous(&mut self, properties: IndexMap<String, Value>) -> Value {
        let id = ObjectId::anonymous();
        let mut data = ObjectData::new(None);
        data.properties = properties;

        self.insert_object(id.clone(), data);

        Value::Object(id)
    }

    /// Deep equality, which is what `assert` compares with. Objects match on
    /// their contents, and `null` matches a property that was never set.
    pub(crate) fn deep_equal(&self, left: &Value, right: &Value) -> bool {
        self.equal_within(left, right, 0)
    }

    fn equal_within(&self, left: &Value, right: &Value, depth: usize) -> bool {
        if depth > 64 {
            return false;
        }

        match (left, right) {
            (Value::Undefined | Value::Null, Value::Undefined | Value::Null) => true,

            (Value::Object(left), Value::Object(right)) => {
                if left == right {
                    return true;
                }

                let (Some(left), Some(right)) = (self.state.object(left), self.state.object(right))
                else {
                    return false;
                };

                let left_keys = defined_keys(left);
                let right_keys = defined_keys(right);

                if left_keys.len() != right_keys.len() {
                    return false;
                }

                left_keys.iter().all(|key| {
                    match (left.properties.get(*key), right.properties.get(*key)) {
                        (Some(left), Some(right)) => self.equal_within(left, right, depth + 1),
                        _ => false,
                    }
                })
            }

            (Value::List(left), Value::List(right)) => {
                left.len() == right.len()
                    && left
                        .iter()
                        .zip(right.iter())
                        .all(|(left, right)| self.equal_within(left, right, depth + 1))
            }

            (Value::Number(left), Value::Number(right)) => left == right,
            (Value::String(left), Value::String(right)) => left == right,
            (Value::Bool(left), Value::Bool(right)) => left == right,
            (Value::Date(left), Value::Date(right)) => left == right,
            (Value::Class(left), Value::Class(right)) => left == right,

            (Value::Date(millis), Value::Number(number))
            | (Value::Number(number), Value::Date(millis)) => *millis as f64 == *number,

            _ => false,
        }
    }

    pub(crate) fn distinct_values(&self, values: Vec<Value>) -> Vec<Value> {
        self.distinct_by(values, |kept, value| self.deep_equal(kept, value))
    }

    fn distinct_by(
        &self,
        values: Vec<Value>,
        mut matches: impl FnMut(&Value, &Value) -> bool,
    ) -> Vec<Value> {
        let mut kept: Vec<Value> = Vec::new();
        let mut buckets: HashMap<u64, Vec<usize>> = HashMap::new();
        let mut null_holders: HashMap<usize, Vec<(u64, usize)>> = HashMap::new();

        for value in values {
            let Some(key) = self.equality_key(&value) else {
                kept.push(value);
                continue;
            };

            let bucket = buckets.get(&key.hash).into_iter().flatten().copied();
            let loose = null_holders
                .get(&key.defined)
                .filter(|_| key.holds_undefined)
                .into_iter()
                .flatten()
                .filter(|&&(hash, _)| hash != key.hash)
                .map(|&(_, index)| index);

            if bucket
                .chain(loose)
                .any(|index| matches(&kept[index], &value))
            {
                continue;
            }

            let index = kept.len();
            buckets.entry(key.hash).or_default().push(index);
            if key.holds_null {
                null_holders
                    .entry(key.defined)
                    .or_default()
                    .push((key.hash, index));
            }
            kept.push(value);
        }

        kept
    }

    fn equality_key(&self, value: &Value) -> Option<EqualityKey> {
        let mut hasher = DefaultHasher::new();
        let mut defined = 0_usize;
        let mut holds_undefined = false;
        let mut holds_null = false;

        match value {
            Value::Number(number) if number.is_nan() => return None,
            Value::Function(_) | Value::Regex(_) => return None,
            Value::Object(id) => match self.state.object(id) {
                Some(object) => {
                    EqualityTag::Record.hash(&mut hasher);
                    let mut entries = 0_u64;
                    for (name, field) in &object.properties {
                        if field.is_undefined() {
                            holds_undefined = true;
                            continue;
                        }
                        holds_null |= field.is_null();
                        defined += 1;
                        let mut entry = DefaultHasher::new();
                        name.hash(&mut entry);
                        self.hash_member(field, 1, &mut entry);
                        entries = entries.wrapping_add(entry.finish());
                    }
                    defined.hash(&mut hasher);
                    entries.hash(&mut hasher);
                }
                None => {
                    EqualityTag::Missing.hash(&mut hasher);
                    id.as_str().hash(&mut hasher);
                }
            },
            other => self.hash_member(other, 0, &mut hasher),
        }

        Some(EqualityKey {
            hash: hasher.finish(),
            defined,
            holds_undefined,
            holds_null,
        })
    }

    fn hash_member(&self, value: &Value, depth: usize, hasher: &mut impl Hasher) {
        match value {
            Value::Undefined | Value::Null => EqualityTag::Nothing.hash(hasher),
            Value::Number(number) => {
                EqualityTag::Number.hash(hasher);
                number_bits(*number).hash(hasher);
            }
            Value::Date(millis) => {
                EqualityTag::Number.hash(hasher);
                number_bits(*millis as f64).hash(hasher);
            }
            Value::String(text) => {
                EqualityTag::Text.hash(hasher);
                text.hash(hasher);
            }
            Value::Bool(flag) => {
                EqualityTag::Flag.hash(hasher);
                flag.hash(hasher);
            }
            Value::Class(name) => {
                EqualityTag::Class.hash(hasher);
                name.hash(hasher);
            }
            Value::List(items) => {
                EqualityTag::List.hash(hasher);
                items.len().hash(hasher);
                if depth < HASHED_DEPTH {
                    for item in items {
                        self.hash_member(item, depth + 1, hasher);
                    }
                }
            }
            Value::Object(id) => match self.state.object(id) {
                Some(object) => {
                    EqualityTag::Record.hash(hasher);
                    defined_count(object).hash(hasher);
                }
                None => EqualityTag::Missing.hash(hasher),
            },
            Value::Function(_) | Value::Regex(_) => EqualityTag::Opaque.hash(hasher),
        }
    }
}

const HASHED_DEPTH: usize = 8;

struct EqualityKey {
    hash: u64,
    defined: usize,
    holds_undefined: bool,
    holds_null: bool,
}

#[derive(Clone, Copy, Hash)]
enum EqualityTag {
    Nothing,
    Number,
    Text,
    Flag,
    Class,
    List,
    Record,
    Missing,
    Opaque,
}

fn number_bits(number: f64) -> u64 {
    if number == 0.0 { 0 } else { number.to_bits() }
}

fn defined_count(object: &ObjectData) -> usize {
    object
        .properties
        .values()
        .filter(|value| !value.is_undefined())
        .count()
}

/// The property names an object actually holds a value for.
fn defined_keys(object: &ObjectData) -> Vec<&String> {
    object
        .properties
        .iter()
        .filter(|(_, value)| !value.is_undefined())
        .map(|(name, _)| name)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const ITEMS: u32 = 20_000;

    fn record(runtime: &mut Runtime, fields: Vec<(&str, Value)>) -> Value {
        let properties = fields
            .into_iter()
            .map(|(name, value)| (name.to_string(), value))
            .collect();
        runtime.create_anonymous(properties)
    }

    fn counted(runtime: &Runtime, values: Vec<Value>) -> (Vec<Value>, usize) {
        let mut comparisons = 0;
        let kept = runtime.distinct_by(values, |kept, value| {
            comparisons += 1;
            runtime.deep_equal(kept, value)
        });
        (kept, comparisons)
    }

    #[test]
    fn values_are_compared_only_with_kept_values_that_share_their_key() {
        let mut runtime = Runtime::new();
        let numbers: Vec<Value> = (0..ITEMS).map(Value::number).collect();
        let items = numbers.len();
        let repeated: Vec<Value> = (0..ITEMS).map(|item| Value::number(item % 10)).collect();
        let texts: Vec<Value> = (0..ITEMS)
            .map(|item| Value::string(item.to_string()))
            .collect();
        let pairs: Vec<Value> = (0..ITEMS)
            .map(|item| Value::List(vec![Value::number(item), Value::number(item % 7)]))
            .collect();
        let rows: Vec<Value> = (0..ITEMS)
            .map(|item| {
                let tags = Value::List(vec![Value::number(item % 3)]);
                let fields = vec![
                    ("k", Value::number(item)),
                    ("v", Value::Null),
                    ("tags", tags),
                ];
                record(&mut runtime, fields)
            })
            .collect();
        let gaps: Vec<Value> = (0..ITEMS)
            .map(|item| {
                let fields = vec![("k", Value::number(item)), ("gap", Value::Undefined)];
                record(&mut runtime, fields)
            })
            .collect();
        let helper = runtime.run("def helper(value) => value\nhelper").unwrap();
        let pattern = runtime.run("/a/").unwrap();
        let unmatchable = vec![
            Value::Number(f64::NAN),
            Value::Number(f64::NAN),
            helper.clone(),
            helper,
            pattern.clone(),
            pattern,
        ];

        for (label, values, kept, comparisons) in [
            ("numbers", numbers.clone(), items, 0),
            ("numbers modulo ten", repeated, 10, items - 10),
            ("strings", texts, items, 0),
            ("lists", pairs, items, 0),
            ("records holding null", rows.clone(), items, 0),
            ("records holding undefined", gaps.clone(), items, 0),
            (
                "both records",
                [&rows[..], &gaps[..]].concat(),
                2 * items,
                0,
            ),
            (
                "duplicated null holders",
                [&rows[..], &rows[..]].concat(),
                items,
                items,
            ),
            (
                "duplicated undefined holders",
                [&gaps[..], &gaps[..]].concat(),
                items,
                items,
            ),
            (
                "duplicated numbers",
                [&numbers[..], &numbers[..]].concat(),
                items,
                items,
            ),
            ("unmatchable values", unmatchable, 6, 0),
        ] {
            let (found, made) = counted(&runtime, values);
            assert_eq!((found.len(), made), (kept, comparisons), "{label}");
        }
    }

    #[test]
    fn undefined_holders_are_compared_once_with_null_holders_of_their_size() {
        let mut runtime = Runtime::new();
        let null_holder = record(
            &mut runtime,
            vec![("k", Value::Null), ("m", Value::number(1))],
        );
        let undefined_holder = record(
            &mut runtime,
            vec![
                ("k", Value::Undefined),
                ("m", Value::number(1)),
                ("z", Value::number(5)),
            ],
        );
        let smaller = record(&mut runtime, vec![("k", Value::Null)]);
        let first_inner = record(&mut runtime, vec![("x", Value::number(1))]);
        let second_inner = record(&mut runtime, vec![("x", Value::number(2))]);
        let shared_null_holder = record(&mut runtime, vec![("k", Value::Null), ("n", first_inner)]);
        let shared_undefined_holder = record(
            &mut runtime,
            vec![
                ("k", Value::Null),
                ("n", second_inner),
                ("q", Value::Undefined),
            ],
        );

        for (label, values, kept, comparisons) in [
            (
                "null holder first",
                vec![null_holder.clone(), undefined_holder.clone()],
                vec![null_holder.clone()],
                1,
            ),
            (
                "undefined holder first",
                vec![undefined_holder.clone(), null_holder.clone()],
                vec![undefined_holder.clone(), null_holder],
                0,
            ),
            (
                "null holder of another size",
                vec![smaller.clone(), undefined_holder.clone()],
                vec![smaller, undefined_holder],
                0,
            ),
            (
                "null holder sharing the hash",
                vec![shared_null_holder.clone(), shared_undefined_holder.clone()],
                vec![shared_null_holder, shared_undefined_holder],
                1,
            ),
        ] {
            assert_eq!(counted(&runtime, values), (kept, comparisons), "{label}");
        }
    }
}
