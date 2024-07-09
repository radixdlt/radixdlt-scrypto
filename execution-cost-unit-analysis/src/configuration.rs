use radix_engine::vm::wasm::*;
use radix_engine::vm::*;

use crate::wasm_engines::cache::*;

use crate::wasm_engines::wasmi::engine::WasmiEngine;
use crate::wasm_engines::wasmi::module::WasmiModule;

use crate::wasm_engines::wasmer_v2::engine::WasmerV2Engine;
use crate::wasm_engines::wasmer_v2::module::WasmerV2Module;
use wasmer_compiler_cranelift::Cranelift;
use wasmer_compiler_singlepass::Singlepass;

/// The configuration that is used for the various test scenarios, this is, more or less, the
/// configuration under test.
pub struct Configuration<E> {
    pub wasm_engine: E,
    pub compilation_features: CompilationFeatures,
}

impl<E> Configuration<E>
where
    E: IntoDescriptor<Descriptor = (WasmRuntime, Cache, Compiler)>,
{
    pub fn configuration_descriptor(&self) -> ConfigurationDescriptor {
        let (runtime, cache, compiler) = E::descriptor();
        ConfigurationDescriptor {
            runtime,
            cache,
            compiler,
            decimal_arithmetic_in_engine: self.compilation_features.decimal_in_engine,
        }
    }
}

impl<E> Configuration<E>
where
    E: WasmEngine + Into<EngineDynWrapper<E>> + 'static,
{
    pub fn scrypto_vm(
        self,
    ) -> ScryptoVm<Box<dyn WasmEngine<WasmInstance = Box<dyn WasmInstance>>>> {
        ScryptoVm {
            wasm_engine: Box::new(Into::<EngineDynWrapper<E>>::into(self.wasm_engine))
                as Box<dyn WasmEngine<WasmInstance = Box<dyn WasmInstance>>>,
            wasm_validator_config: WasmValidatorConfigV1::new(),
        }
    }
}

pub struct EngineDynWrapper<E>(E);

impl<E> From<E> for EngineDynWrapper<E>
where
    E: WasmEngine,
{
    fn from(value: E) -> Self {
        Self(value)
    }
}

impl<E, I> WasmEngine for EngineDynWrapper<E>
where
    E: WasmEngine<WasmInstance = I>,
    I: WasmInstance + 'static,
{
    type WasmInstance = Box<dyn WasmInstance>;

    fn instantiate(
        &self,
        code_hash: radix_engine::blueprints::package::CodeHash,
        instrumented_code: &[u8],
    ) -> Self::WasmInstance {
        Box::new(self.0.instantiate(code_hash, instrumented_code)) as Box<dyn WasmInstance>
    }
}

/// Additional features or configurations related to the compilation of the Scrypto packages.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CompilationFeatures {
    pub decimal_in_engine: bool,
}

/// A complete description of the configuration that can be serialized to a text format and easily
/// understood by humans.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConfigurationDescriptor {
    pub runtime: WasmRuntime,
    pub cache: Cache,
    pub compiler: Compiler,
    pub decimal_arithmetic_in_engine: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WasmRuntime {
    Wasmi,
    WasmerV2,
    WasmerV4,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Cache {
    None,
    PersistentLru,
    PersistentMoka,
    PersistentWasmerFsCache,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Compiler {
    None,
    LLVM,
    SinglePass,
    Cranelift,
}

pub trait IntoDescriptor {
    type Descriptor;

    fn descriptor() -> Self::Descriptor;
}
