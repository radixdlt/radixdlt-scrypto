#![cfg_attr(feature = "libfuzzer-sys", no_main)]
use fuzz_tests::fuzz_template;
use radix_engine::vm::wasm::{PrepareError, WasmModule, WasmValidatorConfigV1};

fuzz_template!(|data: &[u8]| {
    let _ = fuzz_wasm(data);
});

fn fuzz_wasm(data: &[u8]) -> Result<WasmModule, PrepareError> {
    let instrumenter_config = WasmValidatorConfigV1::new();

    WasmModule::init(data)?
        .inject_instruction_metering(&instrumenter_config)?
        .inject_stack_metering(instrumenter_config.max_stack_size())
}
