use super::instance::*;
use super::module::*;
use crate::configuration::*;
use crate::wasm_engines::traits::*;
use radix_common::constants::*;
use radix_engine::vm::wasm::WasmEngine;
use radix_engine_interface::blueprints::package::CodeHash;
use sbor::rust::marker::*;
use std::num::NonZero;
use wasmer::*;

#[derive(Debug, Clone)]
pub struct WasmerV2EngineOptions {
    max_cache_size: usize,
}

pub struct WasmerV2Engine<M, C> {
    store: Store,
    /// The cache of the modules.
    modules_cache: M,
    /// The compiler to use for the WasmerV2Engine - this is phantom data since we don't need to store
    /// it here at all.
    compiler: PhantomData<C>,
}

impl<M, C> Default for WasmerV2Engine<M, C>
where
    M: ModuleCache<WasmerV2Module>,
    C: Into<Box<dyn CompilerConfig>> + Default,
{
    fn default() -> Self {
        Self::new(
            WasmerV2EngineOptions {
                max_cache_size: WASM_ENGINE_CACHE_SIZE,
            },
            Default::default(),
        )
    }
}

impl<M, C> IntoDescriptor for WasmerV2Engine<M, C>
where
    M: IntoDescriptor<Descriptor = Cache>,
    C: IntoDescriptor<Descriptor = Compiler>,
{
    type Descriptor = (WasmRuntime, Cache, Compiler);

    fn descriptor() -> Self::Descriptor {
        (WasmRuntime::WasmerV2, M::descriptor(), C::descriptor())
    }
}

impl<M, C> WasmerV2Engine<M, C>
where
    M: ModuleCache<WasmerV2Module>,
    C: Into<Box<dyn CompilerConfig>>,
{
    pub fn new(options: WasmerV2EngineOptions, compiler: C) -> Self {
        // TODO: Use of unsafe here is not really needed and can be replaced.
        let modules_cache = M::new(CacheSize::Entries(unsafe {
            NonZero::new_unchecked(options.max_cache_size)
        }));

        Self {
            store: Store::new(&Universal::new(compiler).engine()),
            modules_cache,
            compiler: Default::default(),
        }
    }
}

impl<M, C> WasmEngine for WasmerV2Engine<M, C>
where
    M: ModuleCache<WasmerV2Module>,
    C: Into<Box<dyn CompilerConfig>>,
{
    type WasmInstance = WasmerV2Instance;

    fn instantiate(&self, code_hash: CodeHash, code: &[u8]) -> WasmerV2Instance {
        match self
            .modules_cache
            .load(&code_hash, |module| module.instantiate())
        {
            Some(instance) => instance,
            None => {
                let module = WasmerV2Module {
                    module: Module::new(&self.store, code).expect("Failed to parse WASM module"),
                };
                let instance = module.instantiate();
                self.modules_cache.store(code_hash, module);
                instance
            }
        }
    }
}

impl IntoDescriptor for wasmer_compiler_singlepass::Singlepass {
    type Descriptor = Compiler;

    fn descriptor() -> Self::Descriptor {
        Compiler::SinglePass
    }
}

impl IntoDescriptor for wasmer_compiler_cranelift::Cranelift {
    type Descriptor = Compiler;

    fn descriptor() -> Self::Descriptor {
        Compiler::Cranelift
    }
}
