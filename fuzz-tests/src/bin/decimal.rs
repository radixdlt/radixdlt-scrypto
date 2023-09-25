#![cfg_attr(feature = "libfuzzer-sys", no_main)]

use arbitrary::Arbitrary;
#[cfg(feature = "libfuzzer-sys")]
use libfuzzer_sys::fuzz_target;

#[cfg(feature = "afl")]
use afl::fuzz;

#[cfg(feature = "simple-fuzzer")]
use fuzz_tests::fuzz;

use radix_engine_common::math::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Arbitrary, Serialize, Deserialize)]
struct OneDecimal(Decimal, Decimal, i64, u32, i32, RoundingMode);

fn fuzz_decimal(decimal: OneDecimal) {
    let _ = decimal.0.checked_sqrt();
    let _ = decimal.0.is_positive();
    let _ = decimal.0.is_negative();
    let _ = decimal.0.is_zero();
    let _ = decimal.0.checked_abs();
    let _ = decimal.0.checked_cbrt();
    let _ = decimal.0.checked_ceiling();
    let _ = decimal.0.checked_floor();
    let _ = decimal.0.checked_neg();
    if decimal.4 >= 0 && decimal.4 <= Decimal::SCALE as i32 {
        let _ = decimal.0.checked_round(decimal.4, decimal.5);
    }
    let _ = decimal.0.checked_add(decimal.1);
    let _ = decimal.0.checked_sub(decimal.1);
    let _ = decimal.0.checked_mul(decimal.1);
    let _ = decimal.0.checked_div(decimal.1);

    let _ = i8::try_from(decimal.0);
    let _ = i16::try_from(decimal.0);
    let _ = i32::try_from(decimal.0);
    let _ = i64::try_from(decimal.0);
    let _ = i128::try_from(decimal.0);
    let _ = isize::try_from(decimal.0);
    let _ = u8::try_from(decimal.0);
    let _ = u16::try_from(decimal.0);
    let _ = u32::try_from(decimal.0);
    let _ = u64::try_from(decimal.0);
    let _ = u128::try_from(decimal.0);
    let _ = usize::try_from(decimal.0);

    let string = decimal.0.to_string();
    assert_eq!(Decimal::try_from(string).unwrap(), decimal.0);

    // These two operations take too long to run in a fuzzer
    /*
    let _ = decimal.0.checked_powi(decimal.2);
    let _ = decimal.0.checked_nth_root(decimal.3);
     */
}

// Fuzzer entry points
#[cfg(feature = "libfuzzer-sys")]
fuzz_target!(|decimal: OneDecimal| {
    fuzz_decimal(decimal);
});

#[cfg(feature = "afl")]
fn main() {
    fuzz!(|decimal: OneDecimal| {
        fuzz_decimal(decimal);
    });
}

#[cfg(feature = "simple-fuzzer")]
fn main() {
    fuzz!(|decimal: OneDecimal| {
        fuzz_decimal(decimal);
    });
}

#[test]
fn test_decimal_generate_fuzz_input_data() {
    use bincode::serialize;
    use std::fs;

    let mut idx = 0;
    for d1 in [Decimal::MAX, Decimal::MIN, Decimal::ONE, -Decimal::ONE] {
        for d2 in [Decimal::MAX, Decimal::MIN, Decimal::ONE, -Decimal::ONE] {
            for decimal_places in [-20, -18, -10, -1, 0, 1, 10, 18, 20] {
                for mode in [
                    RoundingMode::ToPositiveInfinity,
                    RoundingMode::ToZero,
                    RoundingMode::ToNearestMidpointTowardZero,
                ] {
                    let d = OneDecimal(d1, d2, 1_i64, 1_u32, decimal_places, mode);
                    let serialized = serialize(&d).unwrap();
                    fs::write(format!("decimal_{:03?}.raw", idx), serialized)
                        .expect("Unable to write file");
                    idx += 1;
                }
            }
        }
    }
}
