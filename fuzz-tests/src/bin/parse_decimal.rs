#![cfg_attr(feature = "libfuzzer-sys", no_main)]

#[cfg(feature = "libfuzzer-sys")]
use libfuzzer_sys::fuzz_target;

#[cfg(feature = "afl")]
use afl::fuzz;

#[cfg(feature = "simple-fuzzer")]
use fuzz_tests::fuzz;

use radix_engine_common::math::Decimal;

// Fuzzer entry points
#[cfg(feature = "libfuzzer-sys")]
fuzz_target!(|data: String| {
    let _ = Decimal::try_from(data);
});

#[cfg(feature = "afl")]
fn main() {
    fuzz!(|data: String| {
        let _ = Decimal::try_from(data);
    });
}

#[cfg(feature = "simple-fuzzer")]
fn main() {
    fuzz!(|data: String| {
        let _ = Decimal::try_from(data);
    });
}

#[test]
fn test_parse_decimal_generate_fuzz_input_data() {
    use std::fs;

    for (idx, s) in [
        "1.0",
        "-1.0",
        "1",
        "a",
        "",
        &Decimal::MAX.to_string(),
        "0.000000000000000001",
        "0.0000000000000000001",
    ]
    .into_iter()
    .enumerate()
    {
        fs::write(format!("parse_decimal_{:03?}.raw", idx), s).expect("Unable to write file");
    }
}
