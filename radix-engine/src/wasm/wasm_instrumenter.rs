use radix_engine_interface::crypto::hash;
use sbor::rust::cell::RefCell;
use sbor::rust::num::NonZeroUsize;
use sbor::rust::sync::Arc;

use crate::types::*;
use crate::wasm::{WasmMeteringConfig, WasmModule};

pub struct WasmInstrumenter {
    cache: RefCell<lru::LruCache<(Hash, Hash), Arc<Vec<u8>>>>,
}

#[derive(Debug, Clone)]
pub struct InstrumenterOptions {
    max_cache_size_bytes: usize,
}

impl Default for WasmInstrumenter {
    fn default() -> Self {
        Self::new(InstrumenterOptions {
            max_cache_size_bytes: 200 * 1024 * 1024,
        })
    }
}

pub struct InstrumentedCode {
    pub code: Arc<Vec<u8>>,
    pub code_hash: Hash,
}

impl WasmInstrumenter {
    pub fn new(options: InstrumenterOptions) -> Self {
        let cache = RefCell::new(lru::LruCache::new(
            NonZeroUsize::new(options.max_cache_size_bytes / (1024 * 1024)).unwrap(),
        ));

        Self { cache }
    }

    pub fn instrument(
        &self,
        code: &[u8],
        wasm_metering_config: &WasmMeteringConfig,
    ) -> InstrumentedCode {
        let code_hash = hash(code);
        let cache_key = (code_hash, *wasm_metering_config.identifier());

        {
            if let Some(cached) = self.cache.borrow_mut().get(&cache_key) {
                return InstrumentedCode {
                    code: cached.clone(),
                    code_hash,
                };
            }
        }

        let instrumented_ref = Arc::new(self.instrument_no_cache(code, wasm_metering_config));

        self.cache
            .borrow_mut()
            .put(cache_key, instrumented_ref.clone());

        InstrumentedCode {
            code: instrumented_ref,
            code_hash,
        }
    }

    pub fn instrument_no_cache(
        &self,
        code: &[u8],
        wasm_metering_config: &WasmMeteringConfig,
    ) -> Vec<u8> {
        WasmModule::init(code)
            .and_then(|m| {
                m.inject_instruction_metering(wasm_metering_config.instruction_cost_rules())
            })
            .and_then(|m| m.inject_stack_metering(wasm_metering_config.max_stack_size()))
            .and_then(|m| m.to_bytes())
            .expect("Failed to instrument WASM module")
            .0
    }
}
