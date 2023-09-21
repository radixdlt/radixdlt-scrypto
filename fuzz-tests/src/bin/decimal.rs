#![cfg_attr(feature = "libfuzzer-sys", no_main)]

use arbitrary::Arbitrary;
#[cfg(feature = "libfuzzer-sys")]
use libfuzzer_sys::fuzz_target;

use radix_engine_common::math::*;

#[derive(Debug, Arbitrary)]
struct OneDecimal(Decimal, Decimal, i64, u32, i32, RoundingMode);

// Fuzzer entry points
#[cfg(feature = "libfuzzer-sys")]
fuzz_target!(|decimal: OneDecimal| {
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

    let string = decimal.0.to_string();
    assert_eq!(Decimal::try_from(string).unwrap(), decimal.0);

    // These two operations take too long to run in a fuzzer
    /*
    let _ = decimal.0.checked_powi(decimal.2);
    let _ = decimal.0.checked_nth_root(decimal.3);
     */
});