#[expect(clippy::cast_precision_loss)]
pub(crate) fn from_usize(value: usize) -> f64 {
    value as f64
}

#[expect(clippy::cast_precision_loss)]
pub(crate) fn from_i64(value: i64) -> f64 {
    value as f64
}

#[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub(crate) fn to_usize(number: f64) -> usize {
    number as usize
}

#[expect(clippy::cast_possible_truncation)]
pub(crate) fn to_i64(number: f64) -> i64 {
    number as i64
}

pub(crate) fn non_negative_integer(number: f64) -> Option<usize> {
    (number >= 0.0 && equal(number.fract(), 0.0)).then(|| to_usize(number))
}

#[expect(clippy::float_cmp)]
pub(crate) fn equal(left: f64, right: f64) -> bool {
    left == right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversions_keep_the_saturating_cast_semantics() {
        assert_eq!(to_usize(f64::NAN), 0);
        assert_eq!(to_usize(-3.5), 0);
        assert_eq!(to_usize(2.9), 2);
        assert_eq!(to_usize(f64::INFINITY), usize::MAX);
        assert_eq!(to_i64(f64::NAN), 0);
        assert_eq!(to_i64(-2.9), -2);
        assert_eq!(to_i64(f64::NEG_INFINITY), i64::MIN);
        assert_eq!(to_i64(f64::INFINITY), i64::MAX);
    }

    #[test]
    fn widening_conversions_are_exact_below_two_to_the_fifty_third() {
        assert!(equal(
            from_usize(9_007_199_254_740_991),
            9_007_199_254_740_991.0
        ));
        assert!(equal(from_i64(-8_640_000_000_000_000), -8.64e15));
    }

    #[test]
    fn only_finite_non_negative_integers_are_counts() {
        assert_eq!(non_negative_integer(0.0), Some(0));
        assert_eq!(non_negative_integer(-0.0), Some(0));
        assert_eq!(non_negative_integer(3.0), Some(3));
        assert_eq!(non_negative_integer(1e300), Some(usize::MAX));
        assert_eq!(non_negative_integer(-1.0), None);
        assert_eq!(non_negative_integer(0.5), None);
        assert_eq!(non_negative_integer(f64::NAN), None);
        assert_eq!(non_negative_integer(f64::INFINITY), None);
        assert_eq!(non_negative_integer(f64::NEG_INFINITY), None);
    }

    #[test]
    fn equality_is_ieee_equality() {
        assert!(equal(0.0, -0.0));
        assert!(!equal(f64::NAN, f64::NAN));
        assert!(equal(f64::INFINITY, f64::INFINITY));
    }
}
