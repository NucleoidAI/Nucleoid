//! Object literals, and the structural comparison objects are matched by.
//! Comparison uses the runtime state to resolve object identities.

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

use indexmap::IndexMap;

use crate::error::{Error, Result};
use crate::graph::{NodeKey, ShapeKey};
use crate::lang::ast::{Expr, ObjectEntry};
use crate::number;
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::{ObjectData, ObjectId, Value};

#[derive(Debug)]
pub struct Object<'a> {
    pub node: &'a Expr,
}

impl<'a> Object<'a> {
    #[must_use]
    pub fn new(node: &'a Expr) -> Self {
        Object { node }
    }

    pub fn resolve(&self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Value> {
        match self.node {
            Expr::ObjectLiteral(entries) => runtime.evaluate_object_literal(entries, scope),
            Expr::SpreadObject(entries) => runtime.evaluate_spread_object(entries, scope),
            other => Err(Error::type_error(format!("{other} is not an object"))),
        }
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

    fn evaluate_spread_object(
        &mut self,
        entries: &[ObjectEntry],
        scope: &mut Scope,
    ) -> Result<Value> {
        let keyed: HashMap<&str, usize> = entries
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| match entry {
                ObjectEntry::Property(key, _) => Some((key.as_str(), index)),
                ObjectEntry::Spread(_) => None,
            })
            .collect();
        let mut properties = IndexMap::new();

        for (index, entry) in entries.iter().enumerate() {
            match entry {
                ObjectEntry::Property(key, value) => {
                    properties.insert(key.clone(), self.evaluate(value, scope)?);
                }
                ObjectEntry::Spread(source) => {
                    let value = self.evaluate(source, scope)?;
                    let replaced = |key: &str| keyed.get(key).is_some_and(|&last| last > index);
                    self.spread_entries(source, &value, &mut properties, replaced)?;
                }
            }
        }

        Ok(self.create_anonymous(properties))
    }

    fn spread_entries(
        &mut self,
        source: &Expr,
        value: &Value,
        properties: &mut IndexMap<String, Value>,
        replaced: impl Fn(&str) -> bool,
    ) -> Result<()> {
        let id = match value {
            Value::Object(id) => id,
            Value::Null | Value::Undefined => return Ok(()),
            Value::Bool(_)
            | Value::Number(_)
            | Value::String(_)
            | Value::List(_)
            | Value::Class(_)
            | Value::Date(_)
            | Value::Regex(_)
            | Value::Function(_) => {
                return Err(Error::type_error(format!("{source} is not an object")));
            }
        };

        self.track_shape(&ShapeKey::Entries(id.clone()));

        let entries: Vec<(String, Value)> = self
            .state
            .object(id)
            .map(|object| {
                object
                    .properties
                    .iter()
                    .map(|(key, entry)| (key.clone(), entry.clone()))
                    .collect()
            })
            .unwrap_or_default();

        for (key, entry) in entries {
            let slot = NodeKey::property(id, &key);
            if self.declares(&slot) {
                continue;
            }
            if !replaced(&key) {
                self.track(&slot);
            }
            if !entry.is_undefined() {
                properties.insert(key, entry);
            }
        }

        Ok(())
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
    fn distinct_by(
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
    fn tally_by(
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
struct Tally {
    kept: Vec<Value>,
    /// For each kept value, itself and every value counted as its repeat.
    counts: Vec<usize>,
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

    fn tallied(runtime: &Runtime, values: Vec<Value>) -> (Vec<Value>, Vec<usize>, usize) {
        let mut comparisons = 0;
        let tally = runtime.tally_by(values, |kept, value| {
            comparisons += 1;
            runtime.deep_equal(kept, value)
        });
        (tally.kept, tally.counts, comparisons)
    }

    #[test]
    fn the_mode_is_the_earliest_of_the_highest_counts() {
        let runtime = Runtime::new();
        assert_eq!(runtime.mode_value(Vec::new()), Value::Undefined);
        for (values, mode) in [
            (vec![3, 4, 4, 3], 3),
            (vec![3, 4, 4], 4),
            (vec![1, 2, 3], 1),
            (vec![5, 1, 1, 5, 5], 5),
            (vec![2, 9, 9, 2, 9], 9),
        ] {
            let values: Vec<Value> = values.into_iter().map(Value::number).collect();
            assert_eq!(
                runtime.mode_value(values.clone()),
                Value::number(mode),
                "{values:?}"
            );
        }
        let signed = vec![Value::Number(-0.0), Value::number(0), Value::number(1)];
        match runtime.mode_value(signed) {
            Value::Number(zero) => assert!(zero == 0.0 && zero.is_sign_negative()),
            other => panic!("the first zero is kept, not {other:?}"),
        }
    }

    #[test]
    fn repeats_count_once_for_the_earliest_kept_value_they_match() {
        let mut runtime = Runtime::new();
        let null_holder = record(
            &mut runtime,
            vec![("k", Value::Null), ("m", Value::number(1))],
        );
        let plain = record(
            &mut runtime,
            vec![("m", Value::number(1)), ("z", Value::number(5))],
        );
        let other_holder = record(
            &mut runtime,
            vec![("l", Value::Null), ("m", Value::number(1))],
        );
        let gap = record(
            &mut runtime,
            vec![
                ("k", Value::Undefined),
                ("m", Value::number(1)),
                ("z", Value::number(5)),
            ],
        );
        let gaps = record(
            &mut runtime,
            vec![
                ("k", Value::Undefined),
                ("l", Value::Undefined),
                ("m", Value::number(1)),
                ("z", Value::number(5)),
            ],
        );

        // `gap` shares the hash of `plain` and is compared with the null
        // holders of its size too, so it is a candidate repeat of both
        // records before it; the earlier one counts it, after one comparison.
        for (label, values, kept, counts, comparisons) in [
            (
                "null holder before the plain record",
                vec![null_holder.clone(), plain.clone(), gap.clone()],
                vec![null_holder.clone(), plain.clone()],
                vec![2, 1],
                1,
            ),
            (
                "plain record before the null holder",
                vec![plain.clone(), null_holder.clone(), gap.clone()],
                vec![plain.clone(), null_holder.clone()],
                vec![2, 1],
                1,
            ),
            (
                "two null holders",
                vec![null_holder.clone(), other_holder.clone(), gaps.clone()],
                vec![null_holder.clone(), other_holder.clone()],
                vec![2, 1],
                1,
            ),
            (
                "two null holders reversed",
                vec![other_holder.clone(), null_holder.clone(), gaps.clone()],
                vec![other_holder.clone(), null_holder.clone()],
                vec![2, 1],
                1,
            ),
            (
                "undefined holder first",
                vec![gap.clone(), null_holder.clone()],
                vec![gap.clone(), null_holder.clone()],
                vec![1, 1],
                0,
            ),
        ] {
            assert_eq!(
                tallied(&runtime, values.clone()),
                (kept.clone(), counts, comparisons),
                "{label}"
            );
            assert_eq!(runtime.mode_value(values), kept[0], "{label}");
        }
    }

    #[test]
    fn a_tally_compares_each_repeat_once_with_the_value_it_repeats() {
        let mut runtime = Runtime::new();
        let mut residues: Vec<Value> = (0..ITEMS).map(|item| Value::number(item % 10)).collect();
        residues.push(Value::number(7));
        let (kept, counts, comparisons) = tallied(&runtime, residues.clone());
        assert_eq!(kept.len(), 10);
        assert_eq!(counts[7], 2_001);
        assert_eq!(comparisons, ITEMS as usize - 10 + 1);
        assert_eq!(runtime.mode_value(residues), Value::number(7));

        let distinct: Vec<Value> = (0..ITEMS).map(Value::number).collect();
        let (kept, counts, comparisons) = tallied(&runtime, distinct);
        assert_eq!((kept.len(), comparisons), (ITEMS as usize, 0));
        assert!(counts.iter().all(|&count| count == 1));

        let rows: Vec<Value> = (0..ITEMS)
            .map(|item| {
                let fields = vec![("k", Value::number(item % 100)), ("v", Value::Null)];
                record(&mut runtime, fields)
            })
            .collect();
        let (kept, counts, comparisons) = tallied(&runtime, rows.clone());
        assert_eq!(kept.len(), 100);
        assert_eq!(comparisons, ITEMS as usize - 100);
        assert!(counts.iter().all(|&count| count == ITEMS as usize / 100));
        assert_eq!(runtime.mode_value(rows.clone()), rows[0]);

        let unmatchable = vec![
            Value::Number(f64::NAN),
            Value::Number(f64::NAN),
            Value::number(1),
        ];
        let (kept, counts, comparisons) = tallied(&runtime, unmatchable);
        assert_eq!((kept.len(), counts, comparisons), (3, vec![1, 1, 1], 0));
    }

    #[test]
    fn distinct_tries_the_hash_bucket_first_and_a_tally_tries_receiver_order() {
        const HOLDERS: u32 = 100;
        const REPEATS: usize = 100;
        let holders = HOLDERS as usize;
        let mut runtime = Runtime::new();
        // `null` holders of two defined properties: `repeat` holds an
        // undefined property, so it is compared with them, but matches none.
        let mut values: Vec<Value> = (0..HOLDERS)
            .map(|item| {
                let fields = vec![("k", Value::Null), ("x", Value::number(item))];
                record(&mut runtime, fields)
            })
            .collect();
        // The record that shares the hash of `repeat` and matches it.
        let plain = record(
            &mut runtime,
            vec![("m", Value::number(1)), ("z", Value::number(5))],
        );
        values.push(plain);
        let repeat = record(
            &mut runtime,
            vec![
                ("q", Value::Undefined),
                ("m", Value::number(1)),
                ("z", Value::number(5)),
            ],
        );
        values.extend(std::iter::repeat_n(repeat, REPEATS));
        let kept = &values[..=holders];

        // Any match drops a value, so `distinct` compares each repeat only
        // with the record of its hash.
        let (found, comparisons) = counted(&runtime, values.clone());
        assert_eq!(found, kept);
        assert_eq!(comparisons, REPEATS);

        // A tally credits the earliest match, so it also compares each repeat
        // with every `null` holder kept before the record it repeats.
        let (found, counts, comparisons) = tallied(&runtime, values.clone());
        assert_eq!(found, kept);
        assert_eq!(counts[holders], REPEATS + 1);
        assert_eq!(comparisons, REPEATS * (holders + 1));
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
