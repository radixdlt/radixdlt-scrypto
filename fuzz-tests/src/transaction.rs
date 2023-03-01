#![no_main]

use libfuzzer_sys::fuzz_target;

mod fuzz;

fuzz_target!(|data: &[u8]| {
    fuzz::fuzz_transaction(data);
});
