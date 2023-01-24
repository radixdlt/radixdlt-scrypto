use radix_engine_interface::api::wasm::Buffer;
use radix_engine_interface::api::wasm::Slice;
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
            CONSUME_BUFFER_FUNCTION_NAME => Ok(FuncInstance::alloc_host(
                signature.clone(),
                CONSUME_BUFFER_FUNCTION_ID,
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

impl From<Error> for InvokeError<WasmRuntimeError> {
    fn from(error: Error) -> Self {
        let e_str = format!("{:?}", error);
        match error.into_host_error() {
            // Pass-through invoke errors
            Some(host_error) => *host_error
                .downcast::<InvokeError<WasmRuntimeError>>()
                .expect("Failed to downcast error into InvokeError<WasmRuntimeError>"),
            None => InvokeError::SelfError(WasmRuntimeError::InterpreterError(e_str)),
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
    pub fn read_memory(&self, ptr: u32, len: u32) -> Result<Vec<u8>, WasmRuntimeError> {
        let ptr = ptr as usize;
        let len = len as usize;

        let memory = self.instance.memory_ref.direct_access();
        let memory_slice = memory.as_ref();
        let memory_size = memory_slice.len();
        if ptr > memory_size || ptr + len > memory_size {
            return Err(WasmRuntimeError::MemoryAccessError);
        }

        Ok(memory_slice[ptr..ptr + len].to_vec())
    }

    pub fn write_memory(&self, ptr: u32, data: &[u8]) -> Result<(), WasmRuntimeError> {
        let ptr = ptr as usize;
        let len = data.len();

        let mut memory = self.instance.memory_ref.direct_access_mut();
        let memory_slice = memory.as_mut();
        let memory_size = memory_slice.len();
        if ptr > memory_size || ptr + len > memory_size {
            return Err(WasmRuntimeError::MemoryAccessError);
        }

        memory_slice[ptr..ptr + len].copy_from_slice(data);
        Ok(())
    }

    pub fn read_slice(&self, v: Slice) -> Result<Vec<u8>, WasmRuntimeError> {
        let ptr = v.ptr();
        let len = v.len();

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
            CONSUME_BUFFER_FUNCTION_ID => {
                let buffer_id = args.nth_checked::<u32>(0)?;
                let destination = args.nth_checked::<u32>(1)?;

                let slice = self.runtime.consume_buffer(buffer_id)?;
                self.write_memory(destination, &slice)?;

                Ok(None)
            }
            INVOKE_METHOD_FUNCTION_ID => {
                let receiver_ptr = args.nth_checked::<u32>(0)?;
                let receiver_len = args.nth_checked::<u32>(1)?;
                let ident_ptr = args.nth_checked::<u32>(2)?;
                let ident_len = args.nth_checked::<u32>(3)?;
                let args_ptr = args.nth_checked::<u32>(4)?;
                let args_len = args.nth_checked::<u32>(5)?;

                let buffer = self.runtime.invoke_method(
                    self.read_memory(receiver_ptr, receiver_len)?,
                    self.read_memory(ident_ptr, ident_len)?,
                    self.read_memory(args_ptr, args_len)?,
                )?;

                Ok(Some(RuntimeValue::I64(buffer.as_i64())))
            }
            INVOKE_FUNCTION_ID => {
                let invocation_ptr = args.nth_checked::<u32>(0)?;
                let invocation_len = args.nth_checked::<u32>(1)?;

                let buffer = self
                    .runtime
                    .invoke(self.read_memory(invocation_ptr, invocation_len)?)?;

                Ok(Some(RuntimeValue::I64(buffer.as_i64())))
            }
            CREATE_NODE_FUNCTION_ID => {
                let node_ptr = args.nth_checked::<u32>(0)?;
                let node_len = args.nth_checked::<u32>(1)?;

                let buffer = self
                    .runtime
                    .create_node(self.read_memory(node_ptr, node_len)?)?;

                Ok(Some(RuntimeValue::I64(buffer.as_i64())))
            }
            GET_VISIBLE_NODES_FUNCTION_ID => {
                let buffer = self.runtime.get_visible_nodes()?;

                Ok(Some(RuntimeValue::I64(buffer.as_i64())))
            }
            DROP_NODE_FUNCTION_ID => {
                let node_id_ptr = args.nth_checked::<u32>(0)?;
                let node_id_len = args.nth_checked::<u32>(1)?;

                self.runtime
                    .drop_node(self.read_memory(node_id_ptr, node_id_len)?)?;

                Ok(None)
            }
            LOCK_SUBSTATE_FUNCTION_ID => {
                let node_id_ptr = args.nth_checked::<u32>(0)?;
                let node_id_len = args.nth_checked::<u32>(1)?;
                let offset_ptr = args.nth_checked::<u32>(2)?;
                let offset_len = args.nth_checked::<u32>(3)?;
                let mutable = args.nth_checked::<u32>(4)? != 0;

                let handle = self.runtime.lock_substate(
                    self.read_memory(node_id_ptr, node_id_len)?,
                    self.read_memory(offset_ptr, offset_len)?,
                    mutable,
                )?;

                Ok(Some(RuntimeValue::I32(handle as i32)))
            }
            READ_SUBSTATE_FUNCTION_ID => {
                let handle = args.nth_checked::<u32>(0)?;

                let buffer = self.runtime.read_substate(handle)?;

                Ok(Some(RuntimeValue::I64(buffer.as_i64())))
            }
            WRITE_SUBSTATE_FUNCTION_ID => {
                let handle = args.nth_checked::<u32>(0)?;
                let data_ptr = args.nth_checked::<u32>(1)?;
                let data_len = args.nth_checked::<u32>(2)?;

                self.runtime
                    .write_substate(handle, self.read_memory(data_ptr, data_len)?)?;

                Ok(None)
            }
            UNLOCK_SUBSTATE_FUNCTION_ID => {
                let handle = args.nth_checked::<u32>(0)?;

                self.runtime.unlock_substate(handle)?;

                Ok(None)
            }
            GET_ACTOR_FUNCTION_ID => {
                let buffer = self.runtime.get_actor()?;

                Ok(Some(RuntimeValue::I64(buffer.as_i64())))
            }
            CONSUME_COST_UNITS_FUNCTION_ID => {
                let n: u32 = args.nth_checked(0)?;
                self.runtime
                    .consume_cost_units(n)
                    .map(|_| Option::None)
                    .map_err(|e| e.into())
            }
            _ => Err(WasmRuntimeError::UnknownHostFunction(index).into()),
        }
    }
}

impl WasmInstance for WasmiInstance {
    fn invoke_export<'r>(
        &mut self,
        func_name: &str,
        args: Vec<Buffer>,
        runtime: &mut Box<dyn WasmRuntime + 'r>,
    ) -> Result<Vec<u8>, InvokeError<WasmRuntimeError>> {
        let mut externals = WasmiExternals {
            instance: self,
            runtime,
        };

        let args: Vec<RuntimeValue> = args
            .into_iter()
            .map(|buffer| RuntimeValue::I64(buffer.as_i64()))
            .collect();

        let return_data = self
            .module_ref
            .clone()
            .invoke_export(func_name, &args, &mut externals)
            .map_err(|e| {
                let err: InvokeError<WasmRuntimeError> = e.into();
                err
            })?;

        if let Some(RuntimeValue::I64(v)) = return_data {
            externals.read_slice(Slice::transmute_i64(v))
        } else {
            Err(WasmRuntimeError::InvalidExportReturn)
        }
        .map_err(InvokeError::SelfError)
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
