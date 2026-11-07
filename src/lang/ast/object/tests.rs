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
