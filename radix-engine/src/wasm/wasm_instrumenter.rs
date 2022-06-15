use sbor::rust::vec::Vec;

use crate::wasm::{WasmMeteringParams, WasmModule};

// TODO: Add instrumented code cache since WASM engine is no longer responsible for this.
pub struct WasmInstrumenter {}

impl WasmInstrumenter {
    pub fn instrument(code: &[u8], wasm_metering_params: &WasmMeteringParams) -> Vec<u8> {
        WasmModule::init(code)
            .and_then(|m| {
                m.inject_instruction_metering(
                    wasm_metering_params.instruction_cost(),
                    wasm_metering_params.grow_memory_cost(),
                )
            })
            .and_then(|m| m.inject_stack_metering(wasm_metering_params.max_stack_size()))
            .and_then(|m| m.to_bytes())
            .expect("Failed to instrument code")
            .0
    }
}
