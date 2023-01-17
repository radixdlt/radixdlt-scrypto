use radix_engine_interface::api::wasm::VecLeak;
use radix_engine_interface::data::IndexedScryptoValue;
use sbor::rust::sync::Arc;
use wasmi::*;

use super::InstrumentedCode;
use super::MeteredCodeKey;
use crate::model::InvokeError;
use crate::types::*;
use crate::wasm::constants::*;
use crate::wasm::errors::*;
use crate::wasm::traits::*;

pub struct WasmiModule {
    module: Module,
    #[allow(dead_code)]
    code_size_bytes: usize,
}

pub struct WasmiInstance {
    module_ref: ModuleRef,
    memory_ref: MemoryRef,
}

pub struct WasmiExternals<'a, 'b, 'r> {
    instance: &'a WasmiInstance,
    runtime: &'b mut Box<dyn WasmRuntime + 'r>,
}

pub struct WasmiEnvModule {}

impl ModuleImportResolver for WasmiEnvModule {
    fn resolve_func(
        &self,
        field_name: &str,
        signature: &wasmi::Signature,
    ) -> Result<FuncRef, Error> {
        match field_name {
            GET_BUFFER_FUNCTION_NAME => Ok(FuncInstance::alloc_host(
                signature.clone(),
                GET_BUFFER_FUNCTION_ID,
            )),
            INVOKE_METHOD_FUNCTION_NAME => Ok(FuncInstance::alloc_host(
                signature.clone(),
                INVOKE_METHOD_FUNCTION_ID,
            )),
            INVOKE_FUNCTION_NAME => Ok(FuncInstance::alloc_host(
                signature.clone(),
                INVOKE_FUNCTION_ID,
            )),
            CREATE_NODE_FUNCTION_NAME => Ok(FuncInstance::alloc_host(
                signature.clone(),
                CREATE_NODE_FUNCTION_ID,
            )),
            GET_VISIBLE_NODES_FUNCTION_NAME => Ok(FuncInstance::alloc_host(
                signature.clone(),
                GET_VISIBLE_NODES_FUNCTION_ID,
            )),
            DROP_NODE_FUNCTION_NAME => Ok(FuncInstance::alloc_host(
                signature.clone(),
                DROP_NODE_FUNCTION_ID,
            )),
            LOCK_SUBSTATE_FUNCTION_NAME => Ok(FuncInstance::alloc_host(
                signature.clone(),
                LOCK_SUBSTATE_FUNCTION_ID,
            )),
            READ_SUBSTATE_FUNCTION_NAME => Ok(FuncInstance::alloc_host(
                signature.clone(),
                READ_SUBSTATE_FUNCTION_ID,
            )),
            WRITE_SUBSTATE_FUNCTION_NAME => Ok(FuncInstance::alloc_host(
                signature.clone(),
                WRITE_SUBSTATE_FUNCTION_ID,
            )),
            UNLOCK_SUBSTATE_FUNCTION_NAME => Ok(FuncInstance::alloc_host(
                signature.clone(),
                UNLOCK_SUBSTATE_FUNCTION_ID,
            )),
            GET_ACTOR_FUNCTION_NAME => Ok(FuncInstance::alloc_host(
                signature.clone(),
                GET_ACTOR_FUNCTION_ID,
            )),
            CONSUME_COST_UNITS_FUNCTION_NAME => Ok(FuncInstance::alloc_host(
                signature.clone(),
                CONSUME_COST_UNITS_FUNCTION_ID,
            )),
            _ => Err(Error::Instantiation(format!(
                "Function {} not found",
                field_name
            ))),
        }
    }
}

impl From<Error> for InvokeError<WasmShimError> {
    fn from(error: Error) -> Self {
        let e_str = format!("{:?}", error);
        match error.into_host_error() {
            // Pass-through invoke errors
            Some(host_error) => *host_error
                .downcast::<InvokeError<WasmShimError>>()
                .expect("Failed to downcast error into InvokeError<WasmShimError>"),
            None => InvokeError::Error(WasmShimError::InterpreterError(e_str)),
        }
    }
}

impl WasmiModule {
    fn instantiate(&self) -> WasmiInstance {
        // link with env module
        let module_ref = ModuleInstance::new(
            &self.module,
            &ImportsBuilder::new().with_resolver(MODULE_ENV_NAME, &WasmiEnvModule {}),
        )
        .expect("Failed to instantiate WASM module")
        .assert_no_start();

        // find memory ref
        let memory_ref = match module_ref.export_by_name(EXPORT_MEMORY) {
            Some(ExternVal::Memory(memory)) => memory,
            _ => panic!("Failed to find memory export"),
        };

        WasmiInstance {
            module_ref,
            memory_ref,
        }
    }
}

impl<'a, 'b, 'r> WasmiExternals<'a, 'b, 'r> {
    pub fn read_memory(&self, ptr: usize, len: usize) -> Result<Vec<u8>, WasmShimError> {
        let direct = self.instance.memory_ref.direct_access();
        let buffer = direct.as_ref();
        if ptr > buffer.len() || ptr + len > buffer.len() {
            return Err(WasmShimError::MemoryAccessError);
        }
        Ok(buffer[ptr..ptr + len].to_vec())
    }

    pub fn read_vec(&self, v: VecLeak) -> Result<Vec<u8>, WasmShimError> {
        let ptr = vec_ptr!(v);
        let len = vec_len!(v);

        self.read_memory(ptr, len)
    }
}

impl<'a, 'b, 'r> Externals for WasmiExternals<'a, 'b, 'r> {
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        match index {
            RADIX_ENGINE_FUNCTION_ID => {
                let id = args.nth_checked::<u8>(0)?;
                let input_ptr = args.nth_checked::<u32>(1)? as usize;
                let input = self.read_value(input_ptr)?;
                let output = self.runtime.main(id, input)?;
                self.send_value(output.as_slice())
                    .map(Option::Some)
                    .map_err(|e| e.into())
            }
            CONSUME_COST_UNITS_FUNCTION_ID => {
                let n: u32 = args.nth_checked(0)?;
                self.runtime
                    .consume_cost_units(n)
                    .map(|_| Option::None)
                    .map_err(|e| e.into())
            }
            _ => Err(WasmShimError::UnknownFunctionIndex(index).into()),
        }
    }
}

impl WasmInstance for WasmiInstance {
    fn invoke_export<'r>(
        &mut self,
        func_name: &str,
        args: Vec<Vec<u8>>,
        runtime: &mut Box<dyn WasmRuntime + 'r>,
    ) -> Result<IndexedScryptoValue, InvokeError<WasmShimError>> {
        let mut externals = WasmiExternals {
            instance: self,
            runtime,
        };

        let mut pointers = Vec::new();
        for arg in args {
            let pointer = externals.send_value(&arg)?;
            pointers.push(pointer);
        }
        let result = self
            .module_ref
            .clone()
            .invoke_export(func_name, &pointers, &mut externals);

        let rtn = result
            .map_err(|e| {
                let err: InvokeError<WasmShimError> = e.into();
                err
            })?
            .ok_or(InvokeError::Error(WasmShimError::InvalidReturn))?;
        match rtn {
            RuntimeValue::I32(ptr) => externals
                .read_value(ptr as usize)
                .map_err(InvokeError::Error),
            _ => Err(InvokeError::Error(WasmShimError::InvalidReturn)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EngineOptions {
    max_cache_size_bytes: usize,
}

pub struct WasmiEngine {
    #[cfg(not(feature = "moka"))]
    modules_cache: RefCell<lru::LruCache<MeteredCodeKey, Arc<WasmiModule>>>,
    #[cfg(feature = "moka")]
    modules_cache: moka::sync::Cache<MeteredCodeKey, Arc<WasmiModule>>,
}

impl Default for WasmiEngine {
    fn default() -> Self {
        Self::new(EngineOptions {
            max_cache_size_bytes: 200 * 1024 * 1024,
        })
    }
}

impl WasmiEngine {
    pub fn new(options: EngineOptions) -> Self {
        #[cfg(not(feature = "moka"))]
        let modules_cache = RefCell::new(lru::LruCache::new(
            NonZeroUsize::new(options.max_cache_size_bytes / (1024 * 1024)).unwrap(),
        ));
        #[cfg(feature = "moka")]
        let modules_cache = moka::sync::Cache::builder()
            .weigher(|_key: &MeteredCodeKey, value: &Arc<WasmiModule>| -> u32 {
                // Approximate the module entry size by the code size
                value.code_size_bytes.try_into().unwrap_or(u32::MAX)
            })
            .max_capacity(options.max_cache_size_bytes as u64)
            .build();
        Self { modules_cache }
    }
}

impl WasmEngine for WasmiEngine {
    type WasmInstance = WasmiInstance;

    fn instantiate(&self, instrumented_code: &InstrumentedCode) -> WasmiInstance {
        let metered_code_key = &instrumented_code.metered_code_key;

        #[cfg(not(feature = "moka"))]
        {
            if let Some(cached_module) = self.modules_cache.borrow_mut().get(metered_code_key) {
                return cached_module.instantiate();
            }
        }
        #[cfg(feature = "moka")]
        if let Some(cached_module) = self.modules_cache.get(metered_code_key) {
            return cached_module.instantiate();
        }

        let code = instrumented_code.code.as_ref();

        let new_module = Arc::new(WasmiModule {
            module: Module::from_buffer(code).expect("Failed to parse WASM module"),
            code_size_bytes: code.len(),
        });

        #[cfg(not(feature = "moka"))]
        self.modules_cache
            .borrow_mut()
            .put(*metered_code_key, new_module.clone());
        #[cfg(feature = "moka")]
        self.modules_cache
            .insert(*metered_code_key, new_module.clone());

        new_module.instantiate()
    }
}
