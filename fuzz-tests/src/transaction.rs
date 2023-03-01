#![cfg_attr(feature = "libfuzzer-sys", no_main)]

#[cfg(feature = "libfuzzer-sys")]
use libfuzzer_sys::fuzz_target;

#[cfg(feature = "afl")]
use afl::fuzz;


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
