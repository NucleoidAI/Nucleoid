use crate::error::{Error, Result};
use crate::value::Value;

/// `Math.<name>(...)`
pub fn math(name: &str, arguments: &[Value]) -> Result<Value> {
    if arguments.iter().any(Value::is_undefined) {
        return Ok(Value::Undefined);
    }

    let first = numeric_argument(arguments, 0);

    let result = match name {
        "max" | "min" => numeric_extremum(arguments.iter().map(Value::to_number), name == "max"),
        "random" => return Err(Error::type_error("Math.random is not supported")),
        "PI" => std::f64::consts::PI,
        "E" => std::f64::consts::E,
        _ => match math_unary(name, first).or_else(|| math_interpolation(name, first, arguments)) {
            Some(result) => result,
            None => return Err(Error::type_error(format!("Math.{name} is not a function"))),
        },
    };

    Ok(Value::Number(result))
}

fn numeric_argument(arguments: &[Value], index: usize) -> f64 {
    arguments.get(index).map_or(f64::NAN, Value::to_number)
}

fn math_unary(name: &str, first: f64) -> Option<f64> {
    Some(match name {
        "abs" => first.abs(),
        "ceil" => first.ceil(),
        "floor" => first.floor(),
        "fract" => {
            let floor = first.floor();
            first - floor
        }
        "toRadians" => {
            let factor = std::f64::consts::PI / 180.0;
            first * factor
        }
        "round" => {
            // Compare the fraction directly: host `round` can lose precision
            // just below 0.5. Truncation also preserves negative zero.
            let integer = first.trunc();
            let fraction = first - integer;
            if fraction >= 0.5 {
                integer + 1.0
            } else if fraction < -0.5 {
                integer - 1.0
            } else {
                integer
            }
        }
        "trunc" => first.trunc(),
        "sqrt" => first.sqrt(),
        "cbrt" => first.cbrt(),
        "sign" => {
            if first > 0.0 {
                1.0
            } else if first < 0.0 {
                -1.0
            } else {
                first
            }
        }
        "exp" => first.exp(),
        "exp2" => 2.0_f64.powf(first),
        "log" => first.ln(),
        "log2" => first.log2(),
        "log10" => first.log10(),
        "sin" => first.sin(),
        "cos" => first.cos(),
        "tan" => first.tan(),
        "cot" => 1.0 / first.tan(),
        "atan" => first.atan(),
        "asin" => first.asin(),
        "acos" => first.acos(),
        _ => return None,
    })
}

fn math_interpolation(name: &str, first: f64, arguments: &[Value]) -> Option<f64> {
    let argument = |index| numeric_argument(arguments, index);

    Some(match name {
        "pow" => first.powf(argument(1)),
        "atan2" => first.atan2(argument(1)),
        "clamp" => numeric_clamp(first, argument(1), argument(2)),
        "saturate" => numeric_clamp(first, 0.0, 1.0),
        "smoothstep" => smoothstep(first, argument(1), argument(2)),
        "smootherstep" => smootherstep(first, argument(1), argument(2)),
        "lerp" => lerp(first, argument(1), argument(2)),
        "inverseLerp" => inverse_lerp(first, argument(1), argument(2)),
        "remap" => remap(first, argument(1), argument(2), argument(3), argument(4)),
        _ => return None,
    })
}

fn smoothstep(first: f64, upper: f64, value: f64) -> f64 {
    // `first` is the lower bound, not clamp's value-first argument.
    // Keep normalization and the cubic as separate ordered operations.
    let numerator = value - first;
    let denominator = upper - first;
    let normalize = numerator / denominator;
    let t = numeric_clamp(normalize, 0.0, 1.0);
    let square = t * t;
    let scaled = 2.0 * t;
    let falloff = 3.0 - scaled;
    square * falloff
}

fn smootherstep(first: f64, upper: f64, value: f64) -> f64 {
    // Match the accepted normalization and quintic expression's order,
    // including its rounding; do not clamp the polynomial's result.
    let numerator = value - first;
    let denominator = upper - first;
    let normalize = numerator / denominator;
    let t = numeric_clamp(normalize, 0.0, 1.0);
    let square = t * t;
    let cube = square * t;
    let scaled = 6.0 * t;
    let offset = scaled - 15.0;
    let weighted = t * offset;
    let polynomial = weighted + 10.0;
    cube * polynomial
}

fn lerp(first: f64, end: f64, weight: f64) -> f64 {
    // Preserve ordinary subtraction, multiplication, then addition.
    // No endpoint shortcut, clamping, reordering or fused operation.
    let delta = end - first;
    let scaled = delta * weight;
    first + scaled
}

fn inverse_lerp(first: f64, end: f64, value: f64) -> f64 {
    let numerator = value - first;
    let denominator = end - first;
    numerator / denominator
}

fn remap(first: f64, end: f64, target_start: f64, target_end: f64, value: f64) -> f64 {
    // Keep inverse interpolation and interpolation's separate rounding.
    // No clamping, rescaling, endpoint shortcuts or fused operation.
    let numerator = value - first;
    let denominator = end - first;
    let weight = numerator / denominator;
    let delta = target_end - target_start;
    let scaled = delta * weight;
    target_start + scaled
}

/// Ordered min(max(...), ...), not `f64::clamp`: retain the accepted folds for
/// NaN, reversed bounds and signed zeros, without allocating runtime values.
fn numeric_clamp(value: f64, lower: f64, upper: f64) -> f64 {
    let bounded = numeric_extremum([value, lower], true);
    numeric_extremum([bounded, upper], false)
}

/// The existing Math min/max fold, shared without changing its NaN or zero rules.
fn numeric_extremum(numbers: impl IntoIterator<Item = f64>, maximum: bool) -> f64 {
    let initial = if maximum {
        f64::NEG_INFINITY
    } else {
        f64::INFINITY
    };

    numbers.into_iter().fold(initial, |extreme, number| {
        if extreme.is_nan() || number.is_nan() {
            f64::NAN
        } else if maximum {
            extreme.max(number)
        } else {
            extreme.min(number)
        }
    })
}

/// `Math.<name>` read as a value rather than called.
#[must_use]
pub fn math_constant(name: &str) -> Option<Value> {
    Some(match name {
        "PI" => Value::Number(std::f64::consts::PI),
        "E" => Value::Number(std::f64::consts::E),
        "LN2" => Value::Number(std::f64::consts::LN_2),
        "LN10" => Value::Number(std::f64::consts::LN_10),
        "SQRT2" => Value::Number(std::f64::consts::SQRT_2),
        _ => return None,
    })
}
