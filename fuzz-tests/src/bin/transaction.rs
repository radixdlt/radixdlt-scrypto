#![cfg_attr(feature = "libfuzzer-sys", no_main)]

#[cfg(feature = "libfuzzer-sys")]
use libfuzzer_sys::fuzz_target;
#[cfg(feature = "libfuzzer-sys")]
use once_cell::sync::Lazy;

#[cfg(feature = "afl")]
use afl::fuzz;
#[cfg(feature = "afl")]
use std::panic::AssertUnwindSafe;

#[cfg(feature = "simple-fuzzer")]
use fuzz_tests::fuzz;

#[cfg(feature = "ziggy")]
use ziggy::fuzz;
#[cfg(feature = "ziggy")]
use std::panic::AssertUnwindSafe;

use fuzz_tests::transaction::fuzz_tx::*;

// Fuzzer entry points
#[cfg(feature = "libfuzzer-sys")]
fuzz_target!(|data: &[u8]| {
    unsafe {
        static mut FUZZER: Lazy<TxFuzzer> = Lazy::new(|| {
            fuzz_tx_init_statics();
            TxFuzzer::new()
        });

        FUZZER.reset_runner();
        FUZZER.fuzz_tx_manifest(data);
    }
});

#[cfg(feature = "afl")]
fn main() {
    fuzz_tx_init_statics();

    // fuzz! uses `catch_unwind` and it requires RefUnwindSafe trait, which is not auto-implemented by
    // Fuzzer members (TestRunner mainly). `AssertUnwindSafe` annotates the variable is indeed
    // unwind safe
    let mut fuzzer = AssertUnwindSafe(TxFuzzer::new());

    fuzz!(|data: &[u8]| {
        fuzzer.reset_runner();
        fuzzer.fuzz_tx_manifest(data);
    });
}

#[cfg(feature = "simple-fuzzer")]
fn main() {
    fuzz_tx_init_statics();

    let mut fuzzer = TxFuzzer::new();

    fuzz!(|data: &[u8]| {
        fuzzer.reset_runner();
        fuzzer.fuzz_tx_manifest(data);
    });
}

#[cfg(feature = "ziggy")]
fn main() {
    fuzz_tx_init_statics();

    let mut fuzzer = AssertUnwindSafe(TxFuzzer::new());

    fuzz!(|data: &[u8]| {
        fuzzer.reset_runner();
        fuzzer.fuzz_tx_manifest(data);
    });
}
