use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

use crate::number;
use crate::runtime::Runtime;
use crate::value::{ObjectData, Value};

const HASHED_DEPTH: usize = 8;

/// The positions of kept values, filed under the keys of the values that can
/// match them.
#[derive(Default)]
struct EqualityIndex {
    /// Kept positions by structural hash.
    buckets: HashMap<u64, Vec<usize>>,
    /// The hash and position of each kept object holding a `null` property,
    /// by its number of defined properties.
    null_holders: HashMap<usize, Vec<(u64, usize)>>,
}

impl EqualityIndex {
    /// The kept positions a value with `key` can match, as two ascending
    /// sequences: those sharing its hash and, for an object holding an
    /// undefined-valued property, the `null` holders of its size outside that
    /// hash, which deep equality lets it match.
    fn candidates(
        &self,
        key: &EqualityKey,
    ) -> (impl Iterator<Item = usize>, impl Iterator<Item = usize>) {
        let hash = key.hash;
        let bucket = self.buckets.get(&hash).into_iter().flatten().copied();
        let loose = self
            .null_holders
            .get(&key.defined)
            .filter(|_| key.holds_undefined)
            .into_iter()
            .flatten()
            .filter(move |&&(other, _)| other != hash)
            .map(|&(_, position)| position);
        (bucket, loose)
    }

    /// Files the value kept at `position` under its `key`.
    fn insert(&mut self, key: &EqualityKey, position: usize) {
        self.buckets.entry(key.hash).or_default().push(position);
        if key.holds_null {
            self.null_holders
                .entry(key.defined)
                .or_default()
                .push((key.hash, position));
        }
    }
}

/// The values `distinct` keeps, in receiver order, each with how many values
/// it stands for.
#[derive(Default)]
pub(super) struct Tally {
    pub(super) kept: Vec<Value>,
    /// For each kept value, itself and every value counted as its repeat.
    pub(super) counts: Vec<usize>,
}

impl Tally {
    fn keep(&mut self, value: Value) {
        self.kept.push(value);
        self.counts.push(1);
        debug_assert_eq!(self.kept.len(), self.counts.len());
    }

    fn repeat(&mut self, position: usize) {
        debug_assert!(position < self.counts.len());
        if let Some(count) = self.counts.get_mut(position) {
            *count += 1;
        }
    }

    /// The kept value with the highest count, the earliest on a tie.
    fn into_mode(self) -> Value {
        let Tally { mut kept, counts } = self;

        // `max_by_key` returns the last of equal maxima, so scanning in
        // reverse returns the earliest.
        counts
            .iter()
            .enumerate()
            .rev()
            .max_by_key(|&(_, count)| *count)
            .map_or(Value::Undefined, |(index, _)| kept.swap_remove(index))
    }
}

/// Merges two ascending sequences of kept positions into one ascending
/// sequence.
fn ascending(
    left: impl Iterator<Item = usize>,
    right: impl Iterator<Item = usize>,
) -> impl Iterator<Item = usize> {
    let mut left = left.peekable();
    let mut right = right.peekable();

    std::iter::from_fn(move || match (left.peek(), right.peek()) {
        (Some(&first), Some(&second)) if second < first => right.next(),
        (Some(_), _) => left.next(),
        (None, _) => right.next(),
    })
}

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

impl Runtime {
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

            (Value::Number(left), Value::Number(right)) => number::equal(*left, *right),
            (Value::String(left), Value::String(right))
            | (Value::Class(left), Value::Class(right)) => left == right,
            (Value::Bool(left), Value::Bool(right)) => left == right,
            (Value::Date(left), Value::Date(right)) => left == right,

            (Value::Date(millis), Value::Number(numeric))
            | (Value::Number(numeric), Value::Date(millis)) => {
                number::equal(number::from_i64(*millis), *numeric)
            }

            _ => false,
        }
    }

    pub(crate) fn distinct_values(&self, values: Vec<Value>) -> Vec<Value> {
        self.distinct_by(values, |kept, value| self.deep_equal(kept, value))
    }

    /// The most frequent of `values`, counted against the values `distinct`
    /// keeps: the kept value with the highest count, the earliest on a tie,
    /// or undefined when there are no values.
    pub(crate) fn mode_value(&self, values: Vec<Value>) -> Value {
        self.tally_by(values, |kept, value| self.deep_equal(kept, value))
            .into_mode()
    }

    /// Keeps a value when `matches(kept, value)` holds for no value kept
    /// before it.
    pub(super) fn distinct_by(
        &self,
        values: Vec<Value>,
        mut matches: impl FnMut(&Value, &Value) -> bool,
    ) -> Vec<Value> {
        let mut kept: Vec<Value> = Vec::new();
        let mut index = EqualityIndex::default();

        for value in values {
            let Some(key) = self.equality_key(&value) else {
                kept.push(value);
                continue;
            };

            // Any match drops the value, so the values sharing its hash,
            // which match unless their hashes collide, are tried first.
            let (bucket, loose) = index.candidates(&key);
            if bucket
                .chain(loose)
                .filter_map(|position| kept.get(position))
                .any(|candidate| matches(candidate, &value))
            {
                continue;
            }

            index.insert(&key, kept.len());
            kept.push(value);
        }

        kept
    }

    /// Keeps the values `distinct_by` keeps, and counts every other value as
    /// a repeat of the earliest kept value it matches.
    pub(super) fn tally_by(
        &self,
        values: Vec<Value>,
        mut matches: impl FnMut(&Value, &Value) -> bool,
    ) -> Tally {
        let mut tally = Tally::default();
        let mut index = EqualityIndex::default();

        for value in values {
            let Some(key) = self.equality_key(&value) else {
                tally.keep(value);
                continue;
            };

            // Both candidate lists ascend, so the merged search meets kept
            // values in receiver order and stops at the earliest match.
            let (bucket, loose) = index.candidates(&key);
            if let Some(position) = ascending(bucket, loose).find(|&position| {
                tally
                    .kept
                    .get(position)
                    .is_some_and(|candidate| matches(candidate, &value))
            }) {
                tally.repeat(position);
                continue;
            }

            index.insert(&key, tally.kept.len());
            tally.keep(value);
        }

        tally
    }

    fn equality_key(&self, value: &Value) -> Option<EqualityKey> {
        let mut hasher = DefaultHasher::new();
        let mut defined = 0_usize;
        let mut holds_undefined = false;
        let mut holds_null = false;

        match value {
            Value::Number(number) if number.is_nan() => return None,
            Value::Function(_) | Value::Regex(_) => return None,
            Value::Object(id) => {
                if let Some(object) = self.state.object(id) {
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
                } else {
                    EqualityTag::Missing.hash(&mut hasher);
                    id.as_str().hash(&mut hasher);
                }
            }
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
                number_bits(number::from_i64(*millis)).hash(hasher);
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
