#[cfg(feature = "afl")]
use afl::fuzz;
#[cfg(feature = "afl")]
use std::panic::AssertUnwindSafe;

#[cfg(feature = "simple-fuzzer")]
use fuzz_tests::fuzz;

use fuzz_tests::transaction::fuzz_tx::*;

// Fuzzer entry points
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
