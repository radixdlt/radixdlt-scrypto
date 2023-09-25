#![cfg_attr(feature = "libfuzzer-sys", no_main)]

#[cfg(feature = "libfuzzer-sys")]
use libfuzzer_sys::fuzz_target;

use radix_engine_common::math::Decimal;

// Fuzzer entry points
#[cfg(feature = "libfuzzer-sys")]
fuzz_target!(|data: String| {
    let _ = Decimal::try_from(data);
});
