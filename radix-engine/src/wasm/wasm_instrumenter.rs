use sbor::rust::collections::HashMap;
use sbor::rust::vec::Vec;
use scrypto::crypto::{hash, Hash};

use crate::wasm::{WasmMeteringParams, WasmModule};

// TODO: Add instrumented code cache since WASM engine is no longer responsible for this.
pub struct WasmInstrumenter {
    cache: HashMap<(Hash, u8), Vec<u8>>,
}

impl WasmInstrumenter {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn instrument(
        &mut self,
        code: &[u8],
        wasm_metering_params: &WasmMeteringParams,
    ) -> Vec<u8> {
        let code_hash = hash(code);
        self.cache
            .entry((code_hash, wasm_metering_params.identifier()))
            .or_insert_with(|| {
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
            })
            .clone()
    }
}
