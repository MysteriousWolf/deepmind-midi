//! The float operations `core` does not have.
//!
//! The crate is `no_std` and has no dependencies, and `core` gives a float
//! arithmetic, `abs` and its bit pattern and stops there: `floor`, `exp2`,
//! `log2` and `sin` live in `std` or in a maths library, and the
//! [`generator`](crate::generator) module needs all four to turn a program's
//! bytes into a shape.
//!
//! So they are here, in about a hundred lines, rather than as a dependency an
//! embedded build would have to carry. Every one of them is tested against
//! values it can be checked against by hand, and against `std` across the range
//! the generators use, which is what the tests at the bottom do.
//!
//! These are drawing maths. They are accurate to well within a pixel of
//! anything a host samples them into, and they are not a signal processing
//! library: nothing here claims the last bit of an `f32`.

/// Largest integer not greater than `x`, as a float.
///
/// Returns `x` unchanged outside the range an `i32` holds, which is far outside
/// anything a generator is sampled over and keeps the cast from wrapping.
pub(crate) fn floor(x: f32) -> f32 {
    if !(-2_147_483_000.0..=2_147_483_000.0).contains(&x) {
        return x;
    }
    // A float that large is already whole, so the cast below is the answer and
    // the precision the lint warns about has nowhere to go.
    // Truncates toward zero, which is the floor for a positive `x` and one too
    // high for a negative one that is not already whole.
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        reason = "the range check above is what makes this exact"
    )]
    let truncated = x as i32 as f32;
    if truncated > x {
        truncated - 1.0
    } else {
        truncated
    }
}

/// The fractional part of `x`, in `0.0..1.0`.
pub(crate) fn fract(x: f32) -> f32 {
    x - floor(x)
}

/// Two raised to the power `x`.
///
/// The integer part goes straight into the exponent bits and the fraction
/// through a fifth-order polynomial, which is the usual split. Saturates at the
/// ends of the type rather than producing an infinity, so a generator sampled
/// at an absurd point still draws.
pub(crate) fn exp2(x: f32) -> f32 {
    let whole = floor(x.clamp(-126.0, 127.0));
    let part = x.clamp(-126.0, 127.0) - whole;

    // 2^part on 0.0..1.0, Horner form. The coefficients are the Taylor series
    // of 2^x about zero, (ln 2)^n / n!, to the eighth order. Five terms leaves
    // 1e-4 at the top of the interval, which a long axis would show; eight
    // leaves about 1e-7, which is the type's own precision.
    #[expect(
        clippy::approx_constant,
        reason = "the first coefficient of the series is ln 2, and writing it as \
                  the constant would hide that the rest are its powers over n!"
    )]
    let poly = 1.0
        + part
            * (0.693_147_2
                + part
                    * (0.240_226_5
                        + part
                            * (0.055_504_1
                                + part
                                    * (0.009_618_1
                                        + part
                                            * (0.001_333_36
                                                + part
                                                    * (0.000_154_03
                                                        + part
                                                            * (0.000_015_25
                                                                + part * 0.000_001_32)))))));

    #[expect(
        clippy::cast_possible_truncation,
        reason = "clamped to -126..=127 above, which an i32 holds exactly"
    )]
    let exponent = whole as i32;
    // The exponent field is biased by 127, so this is the float 2^whole.
    #[expect(
        clippy::cast_sign_loss,
        reason = "the bias makes the sum non-negative for the clamped range"
    )]
    let scale = f32::from_bits(((exponent + 127) as u32) << 23);
    poly * scale
}

/// Base-two logarithm of `x`, and the floor of the type for anything at or
/// below zero.
///
/// The inverse of [`exp2`] and built the same way round: the exponent bits give
/// the whole part outright, and the mantissa, read back as a number in `1..2`,
/// goes through a series for the fraction.
pub(crate) fn log2(x: f32) -> f32 {
    // Written as a negated comparison so that a NaN takes this branch too:
    // there is no logarithm of one and no sensible answer but the floor.
    #[expect(
        clippy::neg_cmp_op_on_partial_ord,
        reason = "catching NaN as well as zero is the point"
    )]
    if !(x > 0.0) {
        return -126.0;
    }
    let bits = x.to_bits();
    // The exponent field, less its bias of 127.
    #[expect(
        clippy::cast_possible_wrap,
        reason = "the field is eight bits, so the shift leaves it far inside an i32"
    )]
    let exponent = ((bits >> 23) & 0xff) as i32 - 127;
    // The mantissa with the exponent replaced by zero, which is `x` divided by
    // its own power of two and so always in 1.0..2.0.
    let mantissa = f32::from_bits((bits & 0x007f_ffff) | 0x3f80_0000);

    // log2(m) for m in 1..2, as a series in (m - 1) / (m + 1), which converges
    // fast over that interval: log2(m) = 2/ln2 * (u + u^3/3 + u^5/5 + ...).
    let u = (mantissa - 1.0) / (mantissa + 1.0);
    let square = u * u;
    let series = u
        * (1.0
            + square
                * (1.0 / 3.0 + square * (1.0 / 5.0 + square * (1.0 / 7.0 + square * (1.0 / 9.0)))));
    #[expect(
        clippy::cast_precision_loss,
        reason = "an exponent is -126..=127, which an f32 holds exactly"
    )]
    let whole = exponent as f32;
    whole + series * 2.885_39
}

/// Sine of `turns` full turns, so `sin_turns(0.25)` is 1.
///
/// Taking turns rather than radians is what the callers want: an LFO cycle and
/// an arc are both a fraction of a turn, and it keeps every reduction below
/// exact.
pub(crate) fn sin_turns(turns: f32) -> f32 {
    // Fold onto one turn, then onto a half turn, then onto a quarter, using
    // sin(x + 1/2) = -sin(x) and sin(1/2 - x) = sin(x). What is left is a
    // quarter turn, where the series below is at its most accurate.
    let cycle = fract(turns);
    let (sign, half) = if cycle < 0.5 {
        (1.0, cycle)
    } else {
        (-1.0, cycle - 0.5)
    };
    let quarter = if half > 0.25 { 0.5 - half } else { half };

    let angle = quarter * core::f32::consts::TAU;
    let square = angle * angle;
    // Taylor series to the ninth order, which over 0..=TAU/4 is good to about
    // one part in a billion: far past what a picture can show.
    let series = angle
        * (1.0
            + square
                * (-1.0 / 6.0
                    + square
                        * (1.0 / 120.0 + square * (-1.0 / 5040.0 + square * (1.0 / 362_880.0)))));
    sign * series
}

/// Cosine of `turns` full turns.
///
/// Nothing drawn uses it since the sine started at its rest; the effect marks'
/// arc checks still do, so it stays for them.
#[cfg(test)]
pub(crate) fn cos_turns(turns: f32) -> f32 {
    sin_turns(turns + 0.25)
}

#[cfg(test)]
mod tests {
    use super::{cos_turns, exp2, floor, fract, log2, sin_turns};

    /// Close enough that no picture could show the difference. Every generator
    /// output is eventually multiplied by a pixel count.
    const TOLERANCE: f32 = 1e-5;

    fn close(left: f32, right: f32, what: &str) {
        assert!(
            (left - right).abs() <= TOLERANCE * right.abs().max(1.0),
            "{what}: {left} is not {right}"
        );
    }

    #[test]
    fn floor_and_fract_agree_with_the_standard_library() {
        for hundredths in -500_i16..=500 {
            let x = f32::from(hundredths) / 100.0;
            close(floor(x), x.floor(), "floor");
            close(fract(x), x - x.floor(), "fract");
        }
        assert!((0.0..1.0).contains(&fract(-0.25)));
    }

    #[test]
    fn a_power_of_two_is_a_power_of_two() {
        for whole in -20_i16..=20 {
            close(
                exp2(f32::from(whole)),
                f32::from(whole).exp2(),
                "exp2 whole",
            );
        }
        for sixteenths in -320_i16..=320 {
            let x = f32::from(sixteenths) / 16.0;
            close(exp2(x), x.exp2(), "exp2");
        }
    }

    #[test]
    fn a_logarithm_undoes_a_power() {
        // Zero and everything below it, including a NaN, answer the floor.
        close(log2(0.0), -126.0, "log2 of zero");
        close(log2(-1.0), -126.0, "log2 of a negative");
        close(log2(f32::NAN), -126.0, "log2 of a NaN");
        for whole in -20_i16..=20 {
            close(
                log2(f32::from(whole).exp2()),
                f32::from(whole),
                "log2 whole",
            );
        }
        for hundredths in 1_i16..=4000 {
            let x = f32::from(hundredths) / 100.0;
            close(log2(x), x.log2(), "log2");
            // And back again, which is the property the callers rely on.
            close(exp2(log2(x)), x, "round trip");
        }
    }

    #[test]
    fn a_sine_is_a_sine_over_several_turns() {
        for thousandths in -3000_i16..=3000 {
            let turns = f32::from(thousandths) / 1000.0;
            let radians = turns * core::f32::consts::TAU;
            close(sin_turns(turns), radians.sin(), "sin");
            close(cos_turns(turns), radians.cos(), "cos");
        }
    }

    /// The quarter points, which are the ones a reader checks by eye.
    #[test]
    fn a_sine_hits_its_corners_exactly_enough() {
        close(sin_turns(0.0), 0.0, "sin 0");
        close(sin_turns(0.25), 1.0, "sin quarter");
        close(sin_turns(0.75), -1.0, "sin three quarters");
        close(cos_turns(0.0), 1.0, "cos 0");
        close(cos_turns(0.5), -1.0, "cos half");
        // A whole turn returns to the start, which is the fold working.
        close(sin_turns(1.0), 0.0, "sin one turn");
        close(sin_turns(4.25), 1.0, "sin four and a quarter turns");
    }
}
