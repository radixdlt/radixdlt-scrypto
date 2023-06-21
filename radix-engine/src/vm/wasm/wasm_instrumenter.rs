use super::{CodeKey, MeteredCodeKey, PrepareError, WasmMeteringParams};
use crate::types::*;
use crate::vm::wasm::{WasmMeteringConfig, WasmModule};
use sbor::rust::sync::Arc;

pub const DEFAULT_CACHE_SIZE: usize = 1000;

pub struct WasmInstrumenter {
    // This flag disables cache in wasm_instrumenter/wasmi/wasmer to prevent non-determinism when fuzzing
    #[cfg(all(not(feature = "radix_engine_fuzzing"), not(feature = "moka")))]
    cache: RefCell<lru::LruCache<MeteredCodeKey, Arc<Vec<u8>>>>,
    #[cfg(all(not(feature = "radix_engine_fuzzing"), feature = "moka"))]
    cache: moka::sync::Cache<MeteredCodeKey, Arc<Vec<u8>>>,
    #[cfg(feature = "radix_engine_fuzzing")]
    #[allow(dead_code)]
    cache: usize,
}

#[derive(Debug, Clone)]
pub struct InstrumenterOptions {
    max_cache_size: usize,
}

impl Default for InstrumenterOptions {
    fn default() -> Self {
        InstrumenterOptions {
            max_cache_size: DEFAULT_CACHE_SIZE,
        }
    }
}

impl Default for WasmInstrumenter {
    fn default() -> Self {
        Self::new(InstrumenterOptions::default())
    }
}

pub struct InstrumentedCode {
    pub metered_code_key: MeteredCodeKey,
    pub code: Arc<Vec<u8>>,
}

impl WasmInstrumenter {
    pub fn new(options: InstrumenterOptions) -> Self {
        #[cfg(all(not(feature = "radix_engine_fuzzing"), not(feature = "moka")))]
        let cache = RefCell::new(lru::LruCache::new(
            NonZeroUsize::new(options.max_cache_size).unwrap(),
        ));
        #[cfg(all(not(feature = "radix_engine_fuzzing"), feature = "moka"))]
        let cache = moka::sync::Cache::builder()
            .weigher(|_key: &MeteredCodeKey, _value: &Arc<Vec<u8>>| -> u32 {
                // No sophisticated weighing mechanism, just keep a fixed size cache
                1u32
            })
            .max_capacity(options.max_cache_size as u64)
            .build();
        #[cfg(feature = "radix_engine_fuzzing")]
        let cache = options.max_cache_size;

        Self { cache }
    }

    pub fn instrument(
        &self,
        code_key: CodeKey,
        code: &[u8],
        wasm_metering_config: WasmMeteringConfig,
    ) -> Result<InstrumentedCode, PrepareError> {
        let metered_code_key = (code_key, wasm_metering_config);

        #[cfg(not(feature = "radix_engine_fuzzing"))]
        {
            #[cfg(not(feature = "moka"))]
            {
                if let Some(cached) = self.cache.borrow_mut().get(&metered_code_key) {
                    return Ok(InstrumentedCode {
                        metered_code_key,
                        code: cached.clone(),
                    });
                }
            }
            #[cfg(feature = "moka")]
            if let Some(cached) = self.cache.get(&metered_code_key) {
                return Ok(InstrumentedCode {
                    metered_code_key,
                    code: cached.clone(),
                });
            }
        }

        let instrumented_ref =
            Arc::new(self.instrument_no_cache(code, wasm_metering_config.parameters())?);

        #[cfg(not(feature = "radix_engine_fuzzing"))]
        {
            #[cfg(not(feature = "moka"))]
            self.cache
                .borrow_mut()
                .put(metered_code_key, instrumented_ref.clone());
            #[cfg(feature = "moka")]
            self.cache
                .insert(metered_code_key, instrumented_ref.clone());
        }

        Ok(InstrumentedCode {
            metered_code_key,
            code: instrumented_ref,
        })
    }

    pub fn instrument_no_cache(
        &self,
        code: &[u8],
        metering_params: WasmMeteringParams,
    ) -> Result<Vec<u8>, PrepareError> {
        WasmModule::init(code)
            .and_then(|m| m.inject_instruction_metering(metering_params.instruction_cost_rules()))
            .and_then(|m| m.inject_stack_metering(metering_params.max_stack_size()))
            .and_then(|m| m.to_bytes())
            .map(|m| m.0)
    }
}
