use super::*;

fn bits(pair: (f64, f64)) -> (u64, u64) {
    (pair.0.to_bits(), pair.1.to_bits())
}

#[test]
fn hypot_constants_are_exact_powers_of_two() {
    for (constant, pattern, power) in [
        (HYPOT_DIRECT_MAX, 0x5C10_0000_0000_0000_u64, 450),
        (HYPOT_DIRECT_MIN, 0x23D0_0000_0000_0000, -450),
        (HYPOT_GROW, 0x6570_0000_0000_0000, 600),
        (HYPOT_SHRINK, 0x1A70_0000_0000_0000, -600),
    ] {
        assert_eq!(constant.to_bits(), pattern);
        assert_eq!(constant.to_bits(), 2.0_f64.powi(power).to_bits());
    }
    assert_eq!(
        (HYPOT_DIRECT_MAX * HYPOT_DIRECT_MIN).to_bits(),
        1.0_f64.to_bits()
    );
    assert_eq!((HYPOT_GROW * HYPOT_SHRINK).to_bits(), 1.0_f64.to_bits());
}

#[test]
fn hypot_scale_leaves_the_direct_range_unscaled_and_scales_beyond_it() {
    let unscaled = bits((1.0, 1.0));
    let shrink = bits((HYPOT_SHRINK, HYPOT_GROW));
    let grow = bits((HYPOT_GROW, HYPOT_SHRINK));
    for largest in [HYPOT_DIRECT_MIN, 1.0, HYPOT_DIRECT_MAX] {
        assert_eq!(bits(hypot_scale(largest)), unscaled, "{largest:e}");
    }
    let above = f64::from_bits(HYPOT_DIRECT_MAX.to_bits() + 1);
    let below = f64::from_bits(HYPOT_DIRECT_MIN.to_bits() - 1);
    for largest in [above, f64::MAX] {
        assert_eq!(bits(hypot_scale(largest)), shrink, "{largest:e}");
    }
    for largest in [below, f64::MIN_POSITIVE, f64::from_bits(1), 0.0] {
        assert_eq!(bits(hypot_scale(largest)), grow, "{largest:e}");
    }
}

#[test]
fn numeric_hypot_sums_squares_in_order_without_overflow_or_underflow() {
    let hypot = |numbers: &[f64]| numeric_hypot(numbers.iter().copied()).to_bits();
    assert_eq!(hypot(&[]), 0.0_f64.to_bits());
    assert_eq!(hypot(&[-0.0]), 0.0_f64.to_bits());
    assert_eq!(hypot(&[3.0, 4.0]), 5.0_f64.to_bits());
    assert_eq!(
        hypot(&[f64::NAN, f64::NEG_INFINITY]),
        f64::INFINITY.to_bits()
    );
    assert!(f64::from_bits(hypot(&[f64::NAN, 3.0])).is_nan());
    let scale = 2.0_f64.powi(700);
    assert_eq!(hypot(&[3.0 * scale, 4.0 * scale]), (5.0 * scale).to_bits());
    assert_eq!(hypot(&[3.0 / scale, 4.0 / scale]), (5.0 / scale).to_bits());
    let direct = 0.1_f64 * 0.1 + 0.2 * 0.2;
    assert_eq!(hypot(&[0.1, 0.2]), direct.sqrt().to_bits());
    let tiny = 2.0_f64.powi(-27);
    let one_first = [1.0, tiny, tiny, tiny, tiny, tiny, tiny, tiny, tiny];
    let one_last = [tiny, tiny, tiny, tiny, tiny, tiny, tiny, tiny, 1.0];
    assert_eq!(hypot(&one_first), 1.0_f64.to_bits());
    assert_eq!(hypot(&one_last), (1.0 + f64::EPSILON).to_bits());
}
