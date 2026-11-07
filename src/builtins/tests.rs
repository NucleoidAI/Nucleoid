use std::sync::Arc;

use regex::Regex;

use super::text::{string_replace_all, string_replacement_buffer};
use super::*;
use crate::lang::ast::operator::binary;
use crate::lang::ast::{BinaryOp, Expr, Function, FunctionBody};
use crate::number;
use crate::{ErrorKind, ObjectId};

#[test]
fn replacement_size_arithmetic_fails_before_any_allocation() {
    for (receiver, search, replacement, matches) in [
        (0, usize::MAX, 0, 2),
        (0, 0, usize::MAX, 2),
        (usize::MAX, 0, 1, 1),
        (0, 1, 0, 1),
    ] {
        let error = string_replacement_buffer(receiver, search, replacement, matches).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Type);
        assert_eq!(error.message(), "Replaced string is too large");
        assert_eq!(error.position(), None);
    }
}

#[test]
fn replacement_reservation_rejects_impossible_capacities_without_allocating() {
    // Vec<u8> rejects capacities above isize::MAX before consulting the
    // allocator; these exercise reservation errors, not dangerous OOMs.
    for length in [isize::MAX as usize + 1, usize::MAX] {
        let error = string_replacement_buffer(length, 0, 0, 0).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Type);
        assert_eq!(error.message(), "Replaced string is too large");
    }
    assert_eq!(string_replacement_buffer(0, 0, 0, usize::MAX).unwrap(), "");
}

#[test]
fn replacement_buffer_accounts_for_utf8_and_shrinking_matches_exactly() {
    for (receiver, search, replacement, expected) in [
        ("é😀", "", "|", "|é|😀|"),
        ("é😀é", "é", "🦀", "🦀😀🦀"),
        ("aaaaa", "aa", "", "a"),
        ("a", "a", "aaaa", "aaaa"),
        ("", "", "é", "é"),
        ("abc", "z", "", "abc"),
    ] {
        let output = string_replace_all(receiver, search, replacement).unwrap();
        assert_eq!(output, expected);
        assert!(output.capacity() >= output.len());
    }
}

fn values() -> Vec<Value> {
    vec![
        Value::Undefined,
        Value::Null,
        Value::Bool(false),
        Value::Bool(true),
        Value::Number(-0.0),
        Value::Number(0.0),
        Value::Number(-3.5),
        Value::Number(7.25),
        Value::Number(f64::MIN_POSITIVE),
        Value::Number(f64::MAX),
        Value::Number(f64::NEG_INFINITY),
        Value::Number(f64::INFINITY),
        Value::Number(f64::NAN),
        Value::String(String::new()),
        Value::String("  4.25  ".to_string()),
        Value::String("invalid".to_string()),
        Value::List(vec![]),
        Value::List(vec![Value::String("4".to_string())]),
        Value::List(vec![Value::Undefined]),
        Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
        Value::Object(ObjectId::named("probe")),
        Value::Class("Probe".to_string()),
        Value::Date(12),
        Value::Regex(Arc::new(Regex::new("probe").unwrap())),
        Value::Function(Arc::new(Function {
            name: None,
            parameters: vec![],
            body: FunctionBody::Expression(Box::new(Expr::Null)),
        })),
    ]
}

fn ordered_composition(value: &Value, lower: &Value, upper: &Value) -> Value {
    let bounded = math("max", &[value.clone(), lower.clone()]).unwrap();
    math("min", &[bounded, upper.clone()]).unwrap()
}

fn assert_same(actual: &Value, expected: &Value) {
    assert!(actual.same_value(expected), "{actual:?} != {expected:?}");
    if let (Value::Number(actual), Value::Number(expected)) = (actual, expected) {
        assert_eq!(actual.to_bits(), expected.to_bits());
    }
}

fn ordered_interpolation(start: &Value, end: &Value, weight: &Value) -> Value {
    if [start, end, weight].into_iter().any(Value::is_undefined) {
        return Value::Undefined;
    }
    let [start, end, weight] = [start, end, weight].map(|value| Value::Number(value.to_number()));
    let delta = binary(BinaryOp::Subtract, &end, &start).unwrap();
    let scaled = binary(BinaryOp::Multiply, &delta, &weight).unwrap();
    binary(BinaryOp::Add, &start, &scaled).unwrap()
}

fn ordered_smoothing(lower: &Value, upper: &Value, value: &Value) -> Value {
    if [lower, upper, value].into_iter().any(Value::is_undefined) {
        return Value::Undefined;
    }
    let [lower, upper, value] = [lower, upper, value].map(|value| Value::Number(value.to_number()));
    let numerator = binary(BinaryOp::Subtract, &value, &lower).unwrap();
    let denominator = binary(BinaryOp::Subtract, &upper, &lower).unwrap();
    let normalize = binary(BinaryOp::Divide, &numerator, &denominator).unwrap();
    let t = ordered_composition(&normalize, &Value::number(0), &Value::number(1));
    let square = binary(BinaryOp::Multiply, &t, &t).unwrap();
    let scaled = binary(BinaryOp::Multiply, &Value::number(2), &t).unwrap();
    let falloff = binary(BinaryOp::Subtract, &Value::number(3), &scaled).unwrap();
    binary(BinaryOp::Multiply, &square, &falloff).unwrap()
}

fn assert_smoothing_same(actual: &Value, expected: &Value) {
    if matches!(expected, Value::Number(number) if number.is_nan()) {
        assert!(matches!(actual, Value::Number(number) if number.is_nan()));
    } else {
        assert_same(actual, expected);
    }
}

fn ordered_quintic_smoothing(lower: &Value, upper: &Value, value: &Value) -> Value {
    if [lower, upper, value].into_iter().any(Value::is_undefined) {
        return Value::Undefined;
    }
    let [lower, upper, value] = [lower, upper, value].map(|value| Value::Number(value.to_number()));
    let numerator = binary(BinaryOp::Subtract, &value, &lower).unwrap();
    let denominator = binary(BinaryOp::Subtract, &upper, &lower).unwrap();
    let normalize = binary(BinaryOp::Divide, &numerator, &denominator).unwrap();
    let t = ordered_composition(&normalize, &Value::number(0), &Value::number(1));
    let square = binary(BinaryOp::Multiply, &t, &t).unwrap();
    let cube = binary(BinaryOp::Multiply, &square, &t).unwrap();
    let scaled = binary(BinaryOp::Multiply, &Value::number(6), &t).unwrap();
    let offset = binary(BinaryOp::Subtract, &scaled, &Value::number(15)).unwrap();
    let weighted = binary(BinaryOp::Multiply, &t, &offset).unwrap();
    let polynomial = binary(BinaryOp::Add, &weighted, &Value::number(10)).unwrap();
    binary(BinaryOp::Multiply, &cube, &polynomial).unwrap()
}

fn ordered_inverse_interpolation(start: &Value, end: &Value, value: &Value) -> Value {
    let numerator = binary(BinaryOp::Subtract, value, start).unwrap();
    let denominator = binary(BinaryOp::Subtract, end, start).unwrap();
    binary(BinaryOp::Divide, &numerator, &denominator).unwrap()
}

// Independent accepted binary operators, not Math.remap or its dispatch.
fn ordered_remapping(arguments: &[Value]) -> Value {
    if arguments.iter().any(Value::is_undefined) {
        return Value::Undefined;
    }
    let [start, end, target_start, target_end, value] = std::array::from_fn::<_, 5, _>(|i| {
        Value::Number(arguments.get(i).map_or(f64::NAN, Value::to_number))
    });
    let numerator = binary(BinaryOp::Subtract, &value, &start).unwrap();
    let denominator = binary(BinaryOp::Subtract, &end, &start).unwrap();
    let weight = binary(BinaryOp::Divide, &numerator, &denominator).unwrap();
    let delta = binary(BinaryOp::Subtract, &target_end, &target_start).unwrap();
    let scaled = binary(BinaryOp::Multiply, &delta, &weight).unwrap();
    binary(BinaryOp::Add, &target_start, &scaled).unwrap()
}

#[test]
fn remap_all_twenty_five_values_in_each_slot_match_binary_controls_at_multiple_anchors() {
    let values = values();
    assert_eq!(values.len(), 25);
    for anchor in [
        [0.0, 8.0, 10.0, 30.0, 2.0],
        [10.0, 2.0, 30.0, 10.0, 8.0],
        [-1.0, 1.0, -2.0, 2.0, 0.25],
    ] {
        for position in 0..5 {
            for value in &values {
                let mut arguments = anchor.map(Value::Number);
                arguments[position] = value.clone();
                assert_smoothing_same(
                    &math("remap", &arguments).unwrap(),
                    &ordered_remapping(&arguments),
                );
            }
        }
    }
}

#[test]
fn remap_bounded_seven_to_the_fifth_edge_grid_matches_ordered_binary_bits() {
    let numbers = [
        -f64::MAX,
        -0.0,
        0.0,
        f64::from_bits(1),
        1.0,
        f64::INFINITY,
        f64::NAN,
    ];
    let mut checked = 0;
    for start in numbers {
        for end in numbers {
            for target_start in numbers {
                for target_end in numbers {
                    for value in numbers {
                        let arguments =
                            [start, end, target_start, target_end, value].map(Value::Number);
                        assert_smoothing_same(
                            &math("remap", &arguments).unwrap(),
                            &ordered_remapping(&arguments),
                        );
                        checked += 1;
                    }
                }
            }
        }
    }
    assert_eq!(checked, 16_807);
}

#[test]
fn remap_four_thousand_ninety_six_raw_bit_five_tuples_match_binary_controls() {
    let mut bits = 0x39fa_1c70_8e25_d49b_u64;
    for _ in 0..4_096 {
        let arguments = std::array::from_fn::<_, 5, _>(|_| {
            bits = bits
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            Value::Number(f64::from_bits(bits))
        });
        assert_smoothing_same(
            &math("remap", &arguments).unwrap(),
            &ordered_remapping(&arguments),
        );
    }
}

#[test]
fn remap_overflow_subnormal_cancellation_signed_zero_and_no_fuse_controls() {
    for (arguments, expected) in [
        ([0.0, 8.0, 10.0, 30.0, 2.0], 15.0),
        ([0.0, 8.0, 10.0, 30.0, 16.0], 50.0),
        ([0.0, 8.0, 10.0, 30.0, -8.0], -10.0),
        ([10.0, 2.0, 30.0, 10.0, 8.0], 25.0),
        ([2.0, 2.0, 10.0, 30.0, 2.0], f64::NAN),
        ([2.0, 2.0, 10.0, 30.0, 3.0], f64::INFINITY),
        ([2.0, 2.0, 10.0, 30.0, 1.0], f64::NEG_INFINITY),
        ([-f64::MAX, f64::MAX, 10.0, 30.0, 0.0], 10.0),
        ([0.0, 1.0, 4.0, 4.0, f64::INFINITY], f64::NAN),
        ([0.0, 8.0, -f64::MAX, f64::MAX, 2.0], f64::INFINITY),
        ([0.0, 8.0, -f64::MAX, f64::MAX, 0.0], f64::NAN),
        ([0.0, 8.0, -f64::MAX, f64::MAX, 8.0], f64::INFINITY),
        ([0.0, 1.0, 0.0, f64::from_bits(2), 0.5], f64::from_bits(1)),
        ([0.0, f64::MAX, 0.0, 1.0, f64::from_bits(1)], 0.0),
        ([0.0, 1.0, -0.0, -0.0, -0.0], -0.0),
        ([0.0, 1.0, 0.0, -0.0, -0.0], 0.0),
        ([1e16, 1e16 + 2.0, -1e16, 1e16, 1e16], -1e16),
    ] {
        let arguments = arguments.map(Value::Number);
        let actual = math("remap", &arguments).unwrap();
        assert_smoothing_same(&actual, &Value::Number(expected));
        assert_smoothing_same(&actual, &ordered_remapping(&arguments));
    }
    // A fused multiply-add retains a nonzero residue; separate rounding
    // must cancel to zero instead.
    let target_start = -1.0;
    let target_end = f64::EPSILON;
    let weight = 1.0 - f64::EPSILON;
    let delta = target_end - target_start;
    let scaled = delta * weight;
    let separate = target_start + scaled;
    assert_ne!(
        delta.mul_add(weight, target_start).to_bits(),
        separate.to_bits()
    );
    let arguments = [0.0, 1.0, target_start, target_end, weight].map(Value::Number);
    assert_smoothing_same(
        &math("remap", &arguments).unwrap(),
        &Value::Number(separate),
    );
}

#[test]
fn remap_omissions_extras_and_undefined_in_every_position_preserve_the_shared_gate() {
    let arguments = [0.0, 8.0, 10.0, 30.0, 2.0].map(Value::Number);
    for length in 0..5 {
        assert_smoothing_same(
            &math("remap", &arguments[..length]).unwrap(),
            &Value::Number(f64::NAN),
        );
    }
    for extra in values() {
        let mut supplied = arguments.to_vec();
        supplied.push(extra);
        supplied.push(Value::Null);
        assert_smoothing_same(
            &math("remap", &supplied).unwrap(),
            &ordered_remapping(&supplied),
        );
    }
    for position in 0..8 {
        let mut supplied = arguments.to_vec();
        supplied.extend([Value::number(99), Value::Null, Value::number(1)]);
        supplied[position] = Value::Undefined;
        assert_same(&math("remap", &supplied).unwrap(), &Value::Undefined);
    }
    assert_same(
        &math("remap", &[Value::Undefined]).unwrap(),
        &Value::Undefined,
    );
    assert!(math_constant("remap").is_none());
}

#[test]
fn inverse_lerp_matches_independent_binary_controls_for_every_value_kind_and_position() {
    let values = values();
    for start in &values {
        for end in &values {
            for value in &values {
                assert_smoothing_same(
                    &math("inverseLerp", &[start.clone(), end.clone(), value.clone()]).unwrap(),
                    &ordered_inverse_interpolation(start, end, value),
                );
            }
        }
    }
}

#[test]
fn inverse_lerp_edge_grid_keeps_nonfinite_overflow_subnormal_and_zero_composition_bits() {
    let one = 1.0_f64.to_bits();
    let half = 0.5_f64.to_bits();
    let numbers = [
        -f64::MAX,
        -1e16,
        -1.0,
        -f64::MIN_POSITIVE,
        -f64::from_bits(1),
        -0.0,
        0.0,
        f64::from_bits(1),
        f64::MIN_POSITIVE,
        f64::EPSILON,
        0.1,
        0.125,
        0.25,
        f64::from_bits(half - 1),
        0.5,
        f64::from_bits(half + 1),
        0.75,
        f64::from_bits(one - 1),
        1.0,
        f64::from_bits(one + 1),
        1e16,
        f64::MAX,
        f64::NEG_INFINITY,
        f64::INFINITY,
        f64::NAN,
        f64::from_bits(0x7ff8_0000_0000_0042),
        f64::from_bits(0xfff8_0000_0000_0042),
    ];
    for start in numbers {
        for end in numbers {
            for value in numbers {
                let arguments = [start, end, value].map(Value::Number);
                assert_smoothing_same(
                    &math("inverseLerp", &arguments).unwrap(),
                    &ordered_inverse_interpolation(&arguments[0], &arguments[1], &arguments[2]),
                );
            }
        }
    }
    for (arguments, expected) in [
        ([0.0, 8.0, 2.0], 0.25),
        ([0.0, 8.0, -8.0], -1.0),
        ([0.0, 8.0, 16.0], 2.0),
        ([10.0, 2.0, 8.0], 0.25),
        ([2.0, 2.0, 2.0], f64::NAN),
        ([2.0, 2.0, 3.0], f64::INFINITY),
        ([2.0, 2.0, 1.0], f64::NEG_INFINITY),
        ([-f64::MAX, f64::MAX, 0.0], 0.0),
        ([-f64::MAX, f64::MAX, f64::MAX], f64::NAN),
        ([1e16, 1e16 + 2.0, 1e16], 0.0),
    ] {
        assert_smoothing_same(
            &math("inverseLerp", &arguments.map(Value::Number)).unwrap(),
            &Value::Number(expected),
        );
    }
}

#[test]
fn inverse_lerp_full_bit_triples_match_ordered_subtraction_then_division() {
    let mut bits = 0x38fa_1c70_8e25_d49b_u64;
    for _ in 0..4_096 {
        let arguments = std::array::from_fn::<_, 3, _>(|_| {
            bits = bits
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            Value::Number(f64::from_bits(bits))
        });
        assert_smoothing_same(
            &math("inverseLerp", &arguments).unwrap(),
            &ordered_inverse_interpolation(&arguments[0], &arguments[1], &arguments[2]),
        );
    }
}

#[test]
fn inverse_lerp_missing_inputs_and_unconverted_extras_keep_the_supplied_undefined_gate() {
    let arguments = [Value::number(0), Value::number(8), Value::number(2)];
    for length in 0..3 {
        assert_smoothing_same(
            &math("inverseLerp", &arguments[..length]).unwrap(),
            &Value::Number(f64::NAN),
        );
    }
    for extra in values() {
        let mut supplied = arguments.to_vec();
        let expected = if extra.is_undefined() {
            Value::Undefined
        } else {
            Value::number(0.25)
        };
        supplied.push(extra);
        assert_smoothing_same(&math("inverseLerp", &supplied).unwrap(), &expected);
        supplied.push(Value::number(99));
        assert_smoothing_same(&math("inverseLerp", &supplied).unwrap(), &expected);
    }
    for position in 0..6 {
        let mut supplied = vec![Value::number(0.5); 6];
        supplied[position] = Value::Undefined;
        assert_same(&math("inverseLerp", &supplied).unwrap(), &Value::Undefined);
    }
    assert_same(
        &math("inverseLerp", &[Value::Undefined]).unwrap(),
        &Value::Undefined,
    );
    assert!(math_constant("inverseLerp").is_none());
}

#[test]
fn fract_matches_independent_floor_subtraction_for_every_value_kind() {
    for value in values() {
        let floor = math("floor", std::slice::from_ref(&value)).unwrap();
        let expected = binary(BinaryOp::Subtract, &value, &floor).unwrap();
        assert_smoothing_same(
            &math("fract", std::slice::from_ref(&value)).unwrap(),
            &expected,
        );
    }
}

#[test]
fn fract_preserves_negative_rounding_subnormal_nonfinite_and_zero_bits() {
    let one = 1.0_f64.to_bits();
    let half = 0.5_f64.to_bits();
    for number in [
        -f64::MAX,
        -9_007_199_254_740_991.0,
        -4_503_599_627_370_495.5,
        -3.0,
        -2.75,
        -1.25,
        -1.0,
        -f64::from_bits(one - 1),
        -0.5,
        -1e-20,
        -f64::MIN_POSITIVE,
        -f64::from_bits(1),
        -0.0,
        0.0,
        f64::from_bits(1),
        f64::MIN_POSITIVE,
        f64::EPSILON,
        0.1,
        f64::from_bits(half - 1),
        0.5,
        f64::from_bits(half + 1),
        f64::from_bits(one - 1),
        1.0,
        f64::from_bits(one + 1),
        1.25,
        4_503_599_627_370_495.5,
        9_007_199_254_740_991.0,
        f64::MAX,
        f64::NEG_INFINITY,
        f64::INFINITY,
        f64::NAN,
        f64::from_bits(0x7ff8_0000_0000_0042),
        f64::from_bits(0xfff8_0000_0000_0042),
    ] {
        let value = Value::Number(number);
        let floor = math("floor", std::slice::from_ref(&value)).unwrap();
        let expected = binary(BinaryOp::Subtract, &value, &floor).unwrap();
        assert_smoothing_same(&math("fract", &[value]).unwrap(), &expected);
    }
    for (input, expected) in [(-1.25, 0.75), (-1e-20, 1.0), (2.75, 0.75)] {
        assert_same(
            &math("fract", &[Value::Number(input)]).unwrap(),
            &Value::Number(expected),
        );
    }
}

#[test]
fn fract_full_bit_samples_match_the_accepted_arithmetic_control() {
    let mut bits = 0x36fa_1c70_8e25_d49b_u64;
    for _ in 0..4_096 {
        bits = bits
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let value = Value::Number(f64::from_bits(bits));
        let floor = math("floor", std::slice::from_ref(&value)).unwrap();
        let expected = binary(BinaryOp::Subtract, &value, &floor).unwrap();
        assert_smoothing_same(&math("fract", &[value]).unwrap(), &expected);
    }
}

#[test]
fn fract_omission_and_all_extra_values_keep_the_existing_undefined_gate() {
    assert_smoothing_same(&math("fract", &[]).unwrap(), &Value::Number(f64::NAN));
    let values = values();
    for value in &values {
        let floor = math("floor", std::slice::from_ref(value)).unwrap();
        let fraction = binary(BinaryOp::Subtract, value, &floor).unwrap();
        for extra in &values {
            let expected = if value.is_undefined() || extra.is_undefined() {
                &Value::Undefined
            } else {
                &fraction
            };
            let supplied = [value.clone(), extra.clone(), Value::number(99)];
            assert_smoothing_same(&math("fract", &supplied[..2]).unwrap(), expected);
            assert_smoothing_same(&math("fract", &supplied).unwrap(), expected);
        }
    }
    for position in 0..5 {
        let mut supplied = vec![Value::number(0.375); 5];
        supplied[position] = Value::Undefined;
        assert_same(&math("fract", &supplied).unwrap(), &Value::Undefined);
    }
    assert!(math_constant("fract").is_none());
}

#[test]
fn exp2_numeric_bits_match_accepted_pow_including_extremes_and_underflow() {
    let exponents = [
        -f64::MAX,
        f64::NEG_INFINITY,
        -1075.5,
        -1075.0,
        -1074.5,
        -1074.0,
        -1022.0,
        -3.5,
        -1.0,
        -f64::MIN_POSITIVE,
        -0.0,
        0.0,
        f64::from_bits(1),
        f64::MIN_POSITIVE,
        f64::EPSILON,
        0.5,
        1.0,
        7.25,
        1023.0,
        f64::from_bits(1024.0_f64.to_bits() - 1),
        1024.0,
        f64::MAX,
        f64::INFINITY,
        f64::NAN,
        f64::from_bits(0xfff8_0000_0000_0042),
        f64::from_bits(0x7ff0_0000_0000_0001),
    ];
    let mut bits = 0x56e2_3471_6fca_903d_u64;
    let samples = (0..4096).flat_map(|_| {
        bits = bits
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        // Full-bit inputs plus fractional exponents around the finite range.
        [
            f64::from_bits(bits),
            number::from_i64(i64::from_ne_bytes(bits.to_ne_bytes())) / number::from_i64(i64::MAX)
                * 1076.0,
        ]
    });
    for exponent in exponents.into_iter().chain(samples) {
        let input = Value::Number(exponent);
        assert_smoothing_same(
            &math("exp2", std::slice::from_ref(&input)).unwrap(),
            &math("pow", &[Value::number(2), input]).unwrap(),
        );
    }
    for (input, expected) in [
        (-2.0, 0.25),
        (-0.0, 1.0),
        (0.0, 1.0),
        (4.0, 16.0),
        (-1074.0, f64::from_bits(1)),
        (-1075.0, 0.0),
        (f64::NEG_INFINITY, 0.0),
        (f64::INFINITY, f64::INFINITY),
        (1024.0, f64::INFINITY),
    ] {
        assert_same(
            &math("exp2", &[Value::Number(input)]).unwrap(),
            &Value::Number(expected),
        );
    }
}

#[test]
fn exp2_value_coercions_omission_and_extras_keep_the_accepted_pow_gate() {
    assert_smoothing_same(
        &math("exp2", &[]).unwrap(),
        &math("pow", &[Value::number(2)]).unwrap(),
    );
    let values = values();
    for input in &values {
        assert_smoothing_same(
            &math("exp2", std::slice::from_ref(input)).unwrap(),
            &math("pow", &[Value::number(2), input.clone()]).unwrap(),
        );
        for extra in &values {
            let arguments = [input.clone(), extra.clone(), Value::Null];
            let accepted = [Value::number(2), input.clone(), extra.clone(), Value::Null];
            assert_smoothing_same(
                &math("exp2", &arguments).unwrap(),
                &math("pow", &accepted).unwrap(),
            );
        }
    }
    for position in 0..4 {
        let mut arguments = [Value::number(2), Value::Null, Value::Null, Value::Null];
        arguments[position] = Value::Undefined;
        assert_same(&math("exp2", &arguments).unwrap(), &Value::Undefined);
    }
    assert!(math_constant("exp2").is_none());
}

#[test]
fn power_operator_matches_accepted_pow_bits_for_every_value_pair() {
    let mut numbers = vec![
        -f64::MAX,
        f64::NEG_INFINITY,
        -8.0,
        -2.0,
        -1.0,
        -0.5,
        -f64::MIN_POSITIVE,
        -0.0,
        0.0,
        f64::from_bits(1),
        f64::MIN_POSITIVE,
        1.0 / 3.0,
        0.5,
        1.0,
        2.0,
        3.0,
        1023.0,
        1024.0,
        f64::MAX,
        f64::INFINITY,
        f64::NAN,
    ];
    let mut bits = 0x2545_f491_4f6c_dd1d_u64;
    for _ in 0..128 {
        bits = bits
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        numbers.push(f64::from_bits(bits));
        numbers.push(
            number::from_i64(i64::from_ne_bytes(bits.to_ne_bytes())) / number::from_i64(i64::MAX)
                * 64.0,
        );
    }
    let operands: Vec<Value> = values()
        .into_iter()
        .chain(numbers.into_iter().map(Value::Number))
        .collect();
    for base in &operands {
        for exponent in &operands {
            assert_same(
                &binary(BinaryOp::Power, base, exponent).unwrap(),
                &math("pow", &[base.clone(), exponent.clone()]).unwrap(),
            );
        }
    }
    assert_eq!(BinaryOp::Power.as_str(), "**");
}

// Independent accepted binary operations: divide PI before multiplying.
// In particular, multiplying MAX by PI first would overflow incorrectly.
fn ordered_radians(value: &Value) -> Value {
    let factor = binary(
        BinaryOp::Divide,
        &Value::Number(std::f64::consts::PI),
        &Value::number(180),
    )
    .unwrap();
    binary(BinaryOp::Multiply, value, &factor).unwrap()
}

#[test]
fn to_radians_every_value_kind_matches_independent_divide_then_multiply() {
    let values = values();
    assert_eq!(values.len(), 25);
    for value in values {
        assert_smoothing_same(
            &math("toRadians", std::slice::from_ref(&value)).unwrap(),
            &ordered_radians(&value),
        );
    }
}

#[test]
fn to_radians_keeps_finite_max_subnormal_rounding_nonfinite_and_zero_bits() {
    let one = 1.0_f64.to_bits();
    for number in [
        -f64::MAX,
        -9_007_199_254_740_991.0,
        -360.0,
        -180.0,
        -90.0,
        -1.0,
        -f64::MIN_POSITIVE,
        -f64::from_bits(0x000f_ffff_ffff_ffff),
        -f64::from_bits(58),
        -f64::from_bits(29),
        -f64::from_bits(28),
        -f64::from_bits(1),
        -0.0,
        0.0,
        f64::from_bits(1),
        f64::from_bits(28),
        f64::from_bits(29),
        f64::from_bits(57),
        f64::from_bits(58),
        f64::from_bits(0x000f_ffff_ffff_ffff),
        f64::MIN_POSITIVE,
        f64::EPSILON,
        f64::from_bits(one - 1),
        1.0,
        f64::from_bits(one + 1),
        45.0,
        90.0,
        180.0,
        360.0,
        9_007_199_254_740_991.0,
        f64::MAX,
        f64::NEG_INFINITY,
        f64::INFINITY,
        f64::NAN,
        f64::from_bits(0x7ff8_0000_0000_0042),
        f64::from_bits(0xfff8_0000_0000_0042),
        f64::from_bits(0x7ff0_0000_0000_0001),
    ] {
        let value = Value::Number(number);
        assert_smoothing_same(
            &math("toRadians", std::slice::from_ref(&value)).unwrap(),
            &ordered_radians(&value),
        );
    }
    for (input, expected) in [
        (180.0, std::f64::consts::PI),
        (-180.0, -std::f64::consts::PI),
        (0.0, 0.0),
        (-0.0, -0.0),
        (f64::from_bits(28), 0.0),
        (-f64::from_bits(28), -0.0),
        (f64::from_bits(29), f64::from_bits(1)),
        (-f64::from_bits(29), -f64::from_bits(1)),
    ] {
        assert_same(
            &math("toRadians", &[Value::Number(input)]).unwrap(),
            &Value::Number(expected),
        );
    }
    for input in [-f64::MAX, f64::MAX] {
        let Value::Number(result) = math("toRadians", &[Value::Number(input)]).unwrap() else {
            panic!("numeric input must produce a number");
        };
        assert!(result.is_finite());
        assert!(((input * std::f64::consts::PI) / 180.0).is_infinite());
    }
}

#[test]
fn to_radians_deterministic_full_bit_samples_match_accepted_binary_controls() {
    let mut bits = 0x50fa_1c70_8e25_d49b_u64;
    for _ in 0..4_096 {
        bits = bits
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let value = Value::Number(f64::from_bits(bits));
        assert_smoothing_same(
            &math("toRadians", std::slice::from_ref(&value)).unwrap(),
            &ordered_radians(&value),
        );
    }
}

#[test]
fn to_radians_omission_and_all_extras_keep_the_supplied_undefined_gate() {
    assert_smoothing_same(&math("toRadians", &[]).unwrap(), &Value::Number(f64::NAN));
    let values = values();
    for value in &values {
        let radians = ordered_radians(value);
        for extra in &values {
            let expected = if value.is_undefined() || extra.is_undefined() {
                &Value::Undefined
            } else {
                &radians
            };
            let supplied = [value.clone(), extra.clone(), Value::number(99)];
            assert_smoothing_same(&math("toRadians", &supplied[..2]).unwrap(), expected);
            assert_smoothing_same(&math("toRadians", &supplied).unwrap(), expected);
        }
    }
    for position in 0..5 {
        let mut supplied = vec![Value::number(180); 5];
        supplied[position] = Value::Undefined;
        assert_same(&math("toRadians", &supplied).unwrap(), &Value::Undefined);
    }
    assert!(math_constant("toRadians").is_none());
}

#[test]
fn smootherstep_matches_independent_ordered_binary_controls_for_every_value_kind() {
    let values = values();
    for lower in &values {
        for upper in &values {
            for value in &values {
                assert_smoothing_same(
                    &math(
                        "smootherstep",
                        &[lower.clone(), upper.clone(), value.clone()],
                    )
                    .unwrap(),
                    &ordered_quintic_smoothing(lower, upper, value),
                );
            }
        }
    }
}

#[test]
fn smootherstep_keeps_nonfinite_overflow_subnormal_boundary_and_zero_composition_bits() {
    let one = 1.0_f64.to_bits();
    let half = 0.5_f64.to_bits();
    let numbers = [
        -f64::MAX,
        -1e16,
        -1.0,
        -f64::MIN_POSITIVE,
        -f64::from_bits(1),
        -0.0,
        0.0,
        f64::from_bits(1),
        f64::MIN_POSITIVE,
        f64::EPSILON,
        0.1,
        0.125,
        0.25,
        f64::from_bits(half - 1),
        0.5,
        f64::from_bits(half + 1),
        0.75,
        0.9,
        f64::from_bits(one - 3),
        f64::from_bits(one - 1),
        1.0,
        f64::from_bits(one + 1),
        1e16,
        f64::MAX,
        f64::NEG_INFINITY,
        f64::INFINITY,
        f64::NAN,
        f64::from_bits(0x7ff8_0000_0000_0042),
        f64::from_bits(0xfff8_0000_0000_0042),
    ];
    for lower in numbers {
        for upper in numbers {
            for value in numbers {
                let arguments = [lower, upper, value].map(Value::Number);
                assert_smoothing_same(
                    &math("smootherstep", &arguments).unwrap(),
                    &ordered_quintic_smoothing(&arguments[0], &arguments[1], &arguments[2]),
                );
            }
        }
    }
}

#[test]
fn smootherstep_retains_quintic_rounding_overshoot_and_unvalidated_bound_order() {
    for (arguments, expected) in [
        ([0.0, 8.0, 1.0], 0.016_052_246_093_75),
        ([0.0, 8.0, 2.0], 0.103_515_625),
        ([0.0, 8.0, 3.0], 0.275_207_519_531_25),
        ([0.0, 8.0, 4.0], 0.5),
        ([0.0, 8.0, 6.0], 0.896_484_375),
        ([0.0, 8.0, -2.0], 0.0),
        ([0.0, 8.0, 10.0], 1.0),
        ([8.0, 0.0, 2.0], 0.896_484_375),
        ([2.0, 2.0, 1.0], 0.0),
        ([2.0, 2.0, 3.0], 1.0),
        ([2.0, 2.0, 2.0], f64::NAN),
        ([-f64::MAX, f64::MAX, 0.0], 0.0),
        ([-f64::MAX, f64::MAX, f64::MAX], f64::NAN),
        ([0.0, 1.0, 0.1], f64::from_bits(0x3f81_87e7_c06e_19bb)),
        (
            [0.0, 1.0, f64::from_bits(1.0_f64.to_bits() - 1)],
            f64::from_bits(0x3ff0_0000_0000_0006),
        ),
    ] {
        assert_smoothing_same(
            &math("smootherstep", &arguments.map(Value::Number)).unwrap(),
            &Value::Number(expected),
        );
    }
}

#[test]
fn smootherstep_omissions_are_nan_and_unconverted_extras_keep_the_undefined_gate() {
    let arguments = [Value::number(0), Value::number(8), Value::number(2)];
    for length in 0..3 {
        assert_smoothing_same(
            &math("smootherstep", &arguments[..length]).unwrap(),
            &Value::Number(f64::NAN),
        );
    }
    for extra in values() {
        let mut supplied = arguments.to_vec();
        let expected = if extra.is_undefined() {
            Value::Undefined
        } else {
            Value::number(0.103_515_625)
        };
        supplied.push(extra);
        assert_smoothing_same(&math("smootherstep", &supplied).unwrap(), &expected);
        supplied.push(Value::number(99));
        assert_smoothing_same(&math("smootherstep", &supplied).unwrap(), &expected);
    }
    for position in 0..6 {
        let mut supplied = vec![Value::number(0.5); 6];
        supplied[position] = Value::Undefined;
        assert_same(&math("smootherstep", &supplied).unwrap(), &Value::Undefined);
    }
    assert_same(
        &math("smootherstep", &[Value::Undefined]).unwrap(),
        &Value::Undefined,
    );
    assert!(math_constant("smootherstep").is_none());
}

#[test]
fn smoothstep_matches_ordered_normalization_extrema_and_cubic_for_every_value_kind() {
    let values = values();
    for lower in &values {
        for upper in &values {
            for value in &values {
                assert_smoothing_same(
                    &math("smoothstep", &[lower.clone(), upper.clone(), value.clone()]).unwrap(),
                    &ordered_smoothing(lower, upper, value),
                );
            }
        }
    }
}

#[test]
fn smoothstep_preserves_rounding_overflow_nonfinite_subnormal_and_zero_composition_bits() {
    let one = 1.0_f64.to_bits();
    let half = 0.5_f64.to_bits();
    let numbers = [
        -f64::MAX,
        -1e16,
        -1.0,
        -f64::MIN_POSITIVE,
        -f64::from_bits(1),
        -0.0,
        0.0,
        f64::from_bits(1),
        f64::MIN_POSITIVE,
        f64::EPSILON,
        f64::from_bits(half - 1),
        0.5,
        f64::from_bits(half + 1),
        f64::from_bits(one - 3),
        f64::from_bits(one - 1),
        1.0,
        f64::from_bits(one + 1),
        1e16,
        f64::MAX,
        f64::NEG_INFINITY,
        f64::INFINITY,
        f64::NAN,
        f64::from_bits(0x7ff8_0000_0000_0042),
        f64::from_bits(0xfff8_0000_0000_0042),
    ];
    for lower in numbers {
        for upper in numbers {
            for value in numbers {
                let arguments = [lower, upper, value].map(Value::Number);
                assert_smoothing_same(
                    &math("smoothstep", &arguments).unwrap(),
                    &ordered_smoothing(&arguments[0], &arguments[1], &arguments[2]),
                );
            }
        }
    }
}

#[test]
fn smoothstep_uses_lower_upper_value_order_without_equal_or_reversed_bound_validation() {
    for (arguments, expected) in [
        ([0.0, 8.0, 2.0], 0.15625),
        ([0.0, 8.0, 6.0], 0.84375),
        ([0.0, 8.0, -2.0], 0.0),
        ([0.0, 8.0, 10.0], 1.0),
        ([8.0, 0.0, 2.0], 0.84375),
        ([2.0, 2.0, 1.0], 0.0),
        ([2.0, 2.0, 3.0], 1.0),
        ([2.0, 2.0, 2.0], f64::NAN),
        ([-f64::MAX, f64::MAX, 0.0], 0.0),
        ([-f64::MAX, f64::MAX, f64::MAX], f64::NAN),
    ] {
        assert_smoothing_same(
            &math("smoothstep", &arguments.map(Value::Number)).unwrap(),
            &Value::Number(expected),
        );
    }
}

#[test]
fn smoothstep_omissions_are_nan_and_every_supplied_extra_retains_the_undefined_gate() {
    let arguments = [Value::number(0), Value::number(8), Value::number(2)];
    for length in 0..3 {
        assert_smoothing_same(
            &math("smoothstep", &arguments[..length]).unwrap(),
            &Value::Number(f64::NAN),
        );
    }
    for extra in values() {
        let mut supplied = arguments.to_vec();
        let expected = if extra.is_undefined() {
            Value::Undefined
        } else {
            Value::number(0.15625)
        };
        supplied.push(extra);
        assert_smoothing_same(&math("smoothstep", &supplied).unwrap(), &expected);
        supplied.push(Value::number(99));
        assert_smoothing_same(&math("smoothstep", &supplied).unwrap(), &expected);
    }
    for position in 0..6 {
        let mut supplied = vec![Value::number(0.5); 6];
        supplied[position] = Value::Undefined;
        assert_same(&math("smoothstep", &supplied).unwrap(), &Value::Undefined);
    }
    assert_same(
        &math("smoothstep", &[Value::Undefined]).unwrap(),
        &Value::Undefined,
    );
    assert!(math_constant("smoothstep").is_none());
}

#[test]
fn lerp_matches_ordered_binary_arithmetic_for_every_value_kind_without_panicking() {
    let values = values();
    for start in &values {
        for end in &values {
            for weight in &values {
                assert_same(
                    &math("lerp", &[start.clone(), end.clone(), weight.clone()]).unwrap(),
                    &ordered_interpolation(start, end, weight),
                );
            }
        }
    }
}

#[test]
fn lerp_preserves_ordered_rounding_overflow_cancellation_nonfinite_and_zero_bits() {
    let numbers = [
        -f64::MAX,
        -1e16,
        -1.0,
        -f64::MIN_POSITIVE,
        -f64::from_bits(1),
        -0.0,
        0.0,
        f64::from_bits(1),
        f64::MIN_POSITIVE,
        f64::EPSILON,
        1.0,
        1.000_000_000_000_000_2,
        1e16,
        f64::MAX,
        f64::NEG_INFINITY,
        f64::INFINITY,
        f64::NAN,
    ];
    for start in numbers {
        for end in numbers {
            for weight in numbers {
                let arguments = [start, end, weight].map(Value::Number);
                assert_same(
                    &math("lerp", &arguments).unwrap(),
                    &ordered_interpolation(&arguments[0], &arguments[1], &arguments[2]),
                );
            }
        }
    }
}

#[test]
fn lerp_keeps_finite_fractional_extrapolated_equal_and_reversed_results() {
    for (arguments, expected) in [
        ([0.0, 10.0, 0.25], 2.5),
        ([2.0, 10.0, 0.25], 4.0),
        ([2.0, 18.0, 0.5], 10.0),
        ([2.0, 18.0, -0.5], -6.0),
        ([2.0, 18.0, 1.5], 26.0),
        ([4.0, 4.0, 2.0], 4.0),
        ([10.0, 2.0, 0.25], 8.0),
    ] {
        assert_same(
            &math("lerp", &arguments.map(Value::Number)).unwrap(),
            &Value::Number(expected),
        );
    }
}

#[test]
fn lerp_omissions_are_nan_and_all_supplied_extras_remain_in_the_undefined_gate() {
    let arguments = [Value::number(2), Value::number(10), Value::number(0.5)];
    for length in 0..3 {
        assert_same(
            &math("lerp", &arguments[..length]).unwrap(),
            &Value::Number(f64::NAN),
        );
    }
    for extra in values() {
        let mut supplied = arguments.to_vec();
        let expected = if extra.is_undefined() {
            Value::Undefined
        } else {
            Value::number(6)
        };
        supplied.push(extra);
        assert_same(&math("lerp", &supplied).unwrap(), &expected);
        supplied.push(Value::number(99));
        assert_same(&math("lerp", &supplied).unwrap(), &expected);
    }
    for position in 0..5 {
        let mut supplied = vec![Value::number(2); 5];
        supplied[position] = Value::Undefined;
        assert_same(&math("lerp", &supplied).unwrap(), &Value::Undefined);
    }
    assert!(math_constant("lerp").is_none());
}

#[test]
fn clamp_matches_ordered_composition_for_every_value_kind_without_panicking() {
    let values = values();
    for value in &values {
        for lower in &values {
            for upper in &values {
                let actual = math("clamp", &[value.clone(), lower.clone(), upper.clone()]).unwrap();
                let expected = ordered_composition(value, lower, upper);
                assert_same(&actual, &expected);
            }
        }
    }
}

#[test]
fn clamp_handles_finite_fractional_coerced_equal_and_reversed_inputs() {
    for (arguments, expected) in [
        (
            [Value::number(12), Value::number(0), Value::number(10)],
            10.0,
        ),
        (
            [Value::number(-3), Value::number(0), Value::number(10)],
            0.0,
        ),
        (
            [
                Value::number(1.25),
                Value::number(-0.5),
                Value::number(0.75),
            ],
            0.75,
        ),
        ([Value::number(3), Value::number(4), Value::number(4)], 4.0),
        ([Value::number(5), Value::number(10), Value::number(0)], 0.0),
        (
            [Value::string(" 7 "), Value::number(0), Value::number(5)],
            5.0,
        ),
        (
            [Value::number(3), Value::string(" 4 "), Value::number(5)],
            4.0,
        ),
        (
            [Value::number(7), Value::number(0), Value::string("5")],
            5.0,
        ),
        ([Value::Null, Value::number(-1), Value::number(1)], 0.0),
        (
            [Value::Bool(true), Value::Bool(false), Value::number(2)],
            1.0,
        ),
        (
            [Value::List(vec![]), Value::number(1), Value::number(3)],
            1.0,
        ),
        (
            [
                Value::List(vec![Value::string("4")]),
                Value::number(0),
                Value::number(3),
            ],
            3.0,
        ),
        ([Value::Date(12), Value::number(0), Value::Date(10)], 10.0),
    ] {
        assert_same(
            &math("clamp", &arguments).unwrap(),
            &Value::number(expected),
        );
    }
}

#[test]
fn clamp_signed_zero_bits_match_both_ordered_extremum_folds() {
    let numbers = [-0.0, 0.0, -1.0, 1.0, f64::NEG_INFINITY, f64::INFINITY];
    for value in numbers {
        for lower in numbers {
            for upper in numbers {
                let [value, lower, upper] = [value, lower, upper].map(Value::Number);
                let expected = ordered_composition(&value, &lower, &upper);
                assert_same(&math("clamp", &[value, lower, upper]).unwrap(), &expected);
            }
        }
    }
}

#[test]
fn clamp_omissions_are_nan_and_extras_are_ignored_but_included_in_undefined_gate() {
    let arguments = [Value::number(3), Value::number(0), Value::number(5)];
    for length in 0..3 {
        let actual = math("clamp", &arguments[..length]).unwrap();
        assert_same(&actual, &Value::Number(f64::NAN));
    }
    for extra in values() {
        let mut supplied = arguments.to_vec();
        supplied.push(extra.clone());
        let expected = if extra.is_undefined() {
            Value::Undefined
        } else {
            Value::number(3)
        };
        assert_same(&math("clamp", &supplied).unwrap(), &expected);
        supplied.push(Value::number(99));
        assert_same(&math("clamp", &supplied).unwrap(), &expected);
    }
    for position in 0..5 {
        let mut supplied = vec![Value::number(3); 5];
        supplied[position] = Value::Undefined;
        assert_same(&math("clamp", &supplied).unwrap(), &Value::Undefined);
    }
}

#[test]
fn saturate_matches_unit_clamp_and_ordered_folds_for_every_value_kind() {
    let lower = Value::number(0);
    let upper = Value::number(1);
    for value in values() {
        let actual = math("saturate", std::slice::from_ref(&value)).unwrap();
        let ordered = ordered_composition(&value, &lower, &upper);
        let clamped = math("clamp", &[value, lower.clone(), upper.clone()]).unwrap();
        assert_same(&actual, &ordered);
        assert_same(&actual, &clamped);
    }
}

#[test]
fn saturate_preserves_boundary_subnormal_nonfinite_and_zero_bits_from_accepted_folds() {
    let one = 1.0_f64.to_bits();
    for number in [
        -f64::MAX,
        -1.0,
        -f64::MIN_POSITIVE,
        -f64::from_bits(1),
        -0.0,
        0.0,
        f64::from_bits(1),
        f64::MIN_POSITIVE,
        0.375,
        f64::from_bits(one - 1),
        1.0,
        f64::from_bits(one + 1),
        f64::MAX,
        f64::NEG_INFINITY,
        f64::INFINITY,
        f64::NAN,
        f64::from_bits(0x7ff8_0000_0000_0042),
        f64::from_bits(0xfff8_0000_0000_0042),
    ] {
        let value = Value::Number(number);
        let expected = ordered_composition(&value, &Value::number(0), &Value::number(1));
        assert_same(
            &math("saturate", std::slice::from_ref(&value)).unwrap(),
            &expected,
        );
        assert_same(
            &math("clamp", &[value, Value::number(0), Value::number(1)]).unwrap(),
            &expected,
        );
    }
}

#[test]
fn saturate_omission_is_nan_and_every_supplied_extra_retains_the_undefined_gate() {
    assert_same(
        &math("saturate", &[]).unwrap(),
        &math("clamp", &[]).unwrap(),
    );
    assert_same(&math("saturate", &[]).unwrap(), &Value::Number(f64::NAN));
    let values = values();
    for value in &values {
        for extra in &values {
            let expected = if value.is_undefined() || extra.is_undefined() {
                Value::Undefined
            } else {
                ordered_composition(value, &Value::number(0), &Value::number(1))
            };
            let supplied = [value.clone(), extra.clone(), Value::number(99)];
            assert_same(&math("saturate", &supplied[..2]).unwrap(), &expected);
            assert_same(&math("saturate", &supplied).unwrap(), &expected);
            assert_same(
                &math(
                    "clamp",
                    &[
                        value.clone(),
                        Value::number(0),
                        Value::number(1),
                        extra.clone(),
                        Value::number(99),
                    ],
                )
                .unwrap(),
                &expected,
            );
        }
    }
    for position in 0..5 {
        let mut supplied = vec![Value::number(0.375); 5];
        supplied[position] = Value::Undefined;
        assert_same(&math("saturate", &supplied).unwrap(), &Value::Undefined);
    }
    assert!(math_constant("saturate").is_none());
}

#[test]
fn existing_extrema_keep_variadic_seeds_coercion_nan_and_zero_rules() {
    let values = values();
    for (name, initial) in [("max", f64::NEG_INFINITY), ("min", f64::INFINITY)] {
        assert_same(&math(name, &[]).unwrap(), &Value::Number(initial));
        for left in &values {
            for right in &values {
                let left_number = left.to_number();
                let right_number = right.to_number();
                let expected = if left.is_undefined() || right.is_undefined() {
                    Value::Undefined
                } else if left_number.is_nan() || right_number.is_nan() {
                    Value::Number(f64::NAN)
                } else if name == "max" {
                    Value::Number(left_number.max(right_number))
                } else {
                    Value::Number(left_number.min(right_number))
                };
                assert_same(
                    &math(name, &[left.clone(), right.clone()]).unwrap(),
                    &expected,
                );
            }
        }
        for position in 0..4 {
            let mut arguments = vec![Value::number(1); 4];
            arguments[position] = Value::Number(f64::NAN);
            assert_same(&math(name, &arguments).unwrap(), &Value::Number(f64::NAN));
        }
    }
}

#[test]
fn existing_math_dispatch_rounding_constants_and_errors_are_unchanged() {
    for (name, input, expected) in [
        ("abs", -2.0, 2.0),
        ("ceil", 1.25, 2.0),
        ("floor", 1.25, 1.0),
        ("round", 0.499_999_999_999_999_94, 0.0),
        ("round", 0.5, 1.0),
        ("round", -0.5, -0.0),
        ("trunc", -0.25, -0.0),
        ("sqrt", 4.0, 2.0),
        ("cbrt", 8.0, 2.0),
        ("sign", -0.0, -0.0),
        ("exp", 0.0, 1.0),
        ("log", 1.0, 0.0),
        ("log2", 1.0, 0.0),
        ("log10", 1.0, 0.0),
        ("sin", 0.0, 0.0),
        ("cos", 0.0, 1.0),
        ("tan", 0.0, 0.0),
        ("atan", 0.0, 0.0),
        ("asin", 0.0, 0.0),
        ("acos", 1.0, 0.0),
    ] {
        assert_same(
            &math(name, &[Value::number(input)]).unwrap(),
            &Value::number(expected),
        );
        assert_same(
            &math(name, &[Value::number(input), Value::Number(f64::NAN)]).unwrap(),
            &Value::number(expected),
        );
        assert_same(&math(name, &[]).unwrap(), &Value::Number(f64::NAN));
    }
    assert_same(
        &math("pow", &[Value::number(2), Value::number(3)]).unwrap(),
        &Value::number(8),
    );
    assert_same(
        &math("atan2", &[Value::number(0), Value::number(1)]).unwrap(),
        &Value::number(0),
    );
    for name in ["pow", "atan2"] {
        assert_same(
            &math(name, &[Value::number(2)]).unwrap(),
            &Value::Number(f64::NAN),
        );
    }
    for (name, expected) in [("PI", std::f64::consts::PI), ("E", std::f64::consts::E)] {
        assert_same(&math(name, &[]).unwrap(), &Value::Number(expected));
        assert_same(&math_constant(name).unwrap(), &Value::Number(expected));
    }
    assert!(math_constant("clamp").is_none());
    assert!(math_constant("max").is_none());
    for (name, message) in [
        ("random", "Math.random is not supported"),
        ("unknown", "Math.unknown is not a function"),
    ] {
        let error = math(name, &[]).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Type);
        assert_eq!(error.message(), message);
        assert_same(&math(name, &[Value::Undefined]).unwrap(), &Value::Undefined);
    }
}
