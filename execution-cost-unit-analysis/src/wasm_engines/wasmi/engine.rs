use super::instance::*;
use super::module::*;
use crate::configuration::*;
use crate::wasm_engines::cache::*;
use crate::wasm_engines::traits::*;
use radix_common::constants::*;
use radix_engine::vm::wasm::WasmEngine;
use radix_engine_interface::blueprints::package::CodeHash;
use std::num::NonZero;

#[derive(Debug, Clone)]
pub struct WasmiEngineOptions {
    max_cache_size: usize,
}

pub struct WasmiEngine<C> {
    /// The cache to use for the modules
    modules_cache: C,
}

impl<C> Default for WasmiEngine<C>
where
    C: ModuleCache<WasmiModule>,
{
    fn default() -> Self {
        Self::new(WasmiEngineOptions {
            max_cache_size: WASM_ENGINE_CACHE_SIZE,
        })
    }
}

impl<C> IntoDescriptor for WasmiEngine<C>
where
    C: IntoDescriptor<Descriptor = Cache>,
{
    type Descriptor = (WasmRuntime, Cache, Compiler);

    fn descriptor() -> Self::Descriptor {
        (WasmRuntime::Wasmi, C::descriptor(), Compiler::None)
    }
}

impl<C> WasmiEngine<C>
where
    C: ModuleCache<super::module::WasmiModule>,
{
    pub fn new(options: WasmiEngineOptions) -> Self {
        // TODO: Use of unsafe here is not really needed and can be replaced.
        let modules_cache = C::new(CacheSize::Entries(unsafe {
            NonZero::new_unchecked(options.max_cache_size)
        }));

        Self { modules_cache }
    }
}

impl<C> WasmEngine for WasmiEngine<C>
where
    C: ModuleCache<super::module::WasmiModule>,
{
    type WasmInstance = WasmiInstance;

    #[allow(unused_variables)]
    fn instantiate(&self, code_hash: CodeHash, instrumented_code: &[u8]) -> WasmiInstance {
        match self
            .modules_cache
            .load(&code_hash, |module| module.instantiate())
        {
            Some(instance) => instance,
            None => {
                let module =
                    WasmiModule::new(instrumented_code).expect("Failed to instantiate module");
                let instance = module.instantiate();
                self.modules_cache.store(code_hash, module);
                instance
            }
        }
    }
}
