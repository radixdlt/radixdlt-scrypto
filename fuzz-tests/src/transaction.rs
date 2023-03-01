#![cfg_attr(feature = "libfuzzer-sys", no_main)]

#[cfg(feature = "libfuzzer-sys")]
use libfuzzer_sys::fuzz_target;

#[cfg(feature = "afl")]
use afl::fuzz;

#[cfg(feature = "simple-fuzzer")]
mod simple_fuzzer;

mod fuzz;

#[cfg(feature = "libfuzzer-sys")]
fuzz_target!(|data: &[u8]| {
    fuzz::fuzz_transaction(data);
});

#[cfg(feature = "afl")]
fn main() {
    fuzz!(|data: &[u8]| {
        fuzz::fuzz_transaction(data);
    });
}

#[cfg(feature = "simple-fuzzer")]
fn main() {
    simple_fuzzer::fuzz(|data: &[u8]| {
        fuzz::fuzz_transaction(data);
    });
}
