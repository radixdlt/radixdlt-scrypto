#![cfg_attr(feature = "libfuzzer-sys", no_main)]

#[cfg(feature = "libfuzzer-sys")]
use libfuzzer_sys::fuzz_target;

#[cfg(feature = "afl")]
use afl::fuzz;

#[cfg(feature = "simple-fuzzer")]
use fuzz_tests::utils::simple_fuzzer;

use radix_engine::vm::wasm::{PrepareError, WasmModule, WasmValidatorConfigV1};

fn fuzz_wasm(data: &[u8]) -> Result<WasmModule, PrepareError> {
    let instrumenter_config = WasmValidatorConfigV1::new();

    WasmModule::init(data)?
        .inject_instruction_metering(&instrumenter_config)?
        .inject_stack_metering(instrumenter_config.max_stack_size())
}

// Fuzzer entry points
#[cfg(feature = "libfuzzer-sys")]
fuzz_target!(|data: &[u8]| {
    let _ = fuzz_wasm(data);
});

#[cfg(feature = "afl")]
fn main() {
    fuzz!(|data: &[u8]| {
        let _ = fuzz_wasm(data);
    });
}

#[cfg(feature = "simple-fuzzer")]
fn main() {
    simple_fuzzer::fuzz(fuzz_wasm);
}
