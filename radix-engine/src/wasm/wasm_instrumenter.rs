use super::{CodeKey, MeteredCodeKey, WasmMeteringParams};
use crate::types::*;
use crate::wasm::{WasmMeteringConfig, WasmModule};
use sbor::rust::sync::Arc;

pub struct WasmInstrumenter {
    #[cfg(not(feature = "moka"))]
    cache: RefCell<lru::LruCache<MeteredCodeKey, Arc<Vec<u8>>>>,
    #[cfg(feature = "moka")]
    cache: moka::sync::Cache<MeteredCodeKey, Arc<Vec<u8>>>,
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
    pub metered_code_key: MeteredCodeKey,
    pub code: Arc<Vec<u8>>,
}

impl WasmInstrumenter {
    pub fn new(options: InstrumenterOptions) -> Self {
        #[cfg(not(feature = "moka"))]
        let cache = RefCell::new(lru::LruCache::new(
            NonZeroUsize::new(options.max_cache_size_bytes / (1024 * 1024)).unwrap(),
        ));
        #[cfg(feature = "moka")]
        let cache = moka::sync::Cache::builder()
            .weigher(|_key: &MeteredCodeKey, value: &Arc<Vec<u8>>| -> u32 {
                value.len().try_into().unwrap_or(u32::MAX)
            })
            .max_capacity(options.max_cache_size_bytes as u64)
            .build();

        Self { cache }
    }

    pub fn instrument(
        &self,
        code_key: CodeKey,
        code: &[u8],
        wasm_metering_config: WasmMeteringConfig,
    ) -> InstrumentedCode {
        let metered_code_key = (code_key, wasm_metering_config);

        #[cfg(not(feature = "moka"))]
        {
            if let Some(cached) = self.cache.borrow_mut().get(&metered_code_key) {
                return InstrumentedCode {
                    metered_code_key,
                    code: cached.clone(),
                };
            }
        }
        #[cfg(feature = "moka")]
        if let Some(cached) = self.cache.get(&metered_code_key) {
            return InstrumentedCode {
                metered_code_key,
                code: cached.clone(),
            };
        }

        let instrumented_ref =
            Arc::new(self.instrument_no_cache(code, wasm_metering_config.parameters()));

        #[cfg(not(feature = "moka"))]
        self.cache
            .borrow_mut()
            .put(metered_code_key, instrumented_ref.clone());
        #[cfg(feature = "moka")]
        self.cache
            .insert(metered_code_key, instrumented_ref.clone());

        InstrumentedCode {
            metered_code_key,
            code: instrumented_ref,
        }
    }

    pub fn instrument_no_cache(&self, code: &[u8], metering_params: WasmMeteringParams) -> Vec<u8> {
        WasmModule::init(code)
            .and_then(|m| m.inject_instruction_metering(metering_params.instruction_cost_rules()))
            .and_then(|m| m.inject_stack_metering(metering_params.max_stack_size()))
            .and_then(|m| m.to_bytes())
            .expect("Failed to instrument WASM module")
            .0
    }
}
