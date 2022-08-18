use crate::types::*;
use crate::wasm::{WasmMeteringParams, WasmModule};

pub struct WasmInstrumenter {
    cache: HashMap<(Hash, Hash), Vec<u8>>,
}

impl WasmInstrumenter {
    pub fn new() -> Self {
        Self {
            // TODO: introduce a proper cache that supports replacement policy
            cache: HashMap::new(),
        }
    }

    pub fn instrument(&mut self, code: &[u8], wasm_metering_params: &WasmMeteringParams) -> &[u8] {
        let code_hash = hash(code);
        self.cache
            .entry((code_hash, wasm_metering_params.identifier()))
            .or_insert_with(|| {
                WasmModule::init(code)
                    .and_then(|m| {
                        m.inject_instruction_metering(wasm_metering_params.instruction_cost_rules())
                    })
                    .and_then(|m| m.inject_stack_metering(wasm_metering_params.max_stack_size()))
                    .and_then(|m| m.to_bytes())
                    .expect("Failed to instrument code")
                    .0
            })
    }
}
