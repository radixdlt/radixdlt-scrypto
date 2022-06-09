use sbor::rust::vec::Vec;

use crate::wasm::{WasmFeeTable, WasmModule};

// TODO: Add instrumented code cache since WASM engine is no longer responsible for this.
pub struct WasmInstrumenter {}

impl WasmInstrumenter {
    pub fn instrument_v1(code: &[u8]) -> Vec<u8> {
        let wasm_fee_table = WasmFeeTable::new(1, 100);
        let wasm_max_stack_size = 100;

        WasmModule::init(code)
            .and_then(|m| m.inject_instruction_metering(&wasm_fee_table))
            .and_then(|m| m.inject_stack_metering(wasm_max_stack_size))
            .and_then(|m| m.to_bytes())
            .expect("Failed to instrument code")
            .0
    }
}
