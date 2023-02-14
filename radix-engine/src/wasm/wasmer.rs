use super::InstrumentedCode;
use super::MeteredCodeKey;
use crate::model::InvokeError;
use crate::types::*;
use crate::wasm::constants::*;
use crate::wasm::errors::*;
use crate::wasm::traits::*;
use radix_engine_interface::api::wasm::*;
use sbor::rust::sync::{Arc, Mutex};
use wasmer::{
    imports, Function, HostEnvInitError, Instance, LazyInit, Module, RuntimeError, Store,
    Universal, Val, WasmerEnv,
};
use wasmer_compiler_singlepass::Singlepass;

// IMPORTANT:
// The below integration of Wasmer is not yet checked rigorously enough for production use
// TODO: Address the below issues before considering production use.

/// A `WasmerModule` defines a parsed WASM module, which is a template which can be instantiated.
///
/// Unlike `WasmerInstance`, this is correctly `Send + Sync` - which is good, because this is the
/// thing which is cached in the ScryptoInterpreter caches.
pub struct WasmerModule {
    module: Module,
    #[allow(dead_code)]
    code_size_bytes: usize,
}

/// WARNING - this type should not actually be Send + Sync - it should really store a raw pointer,
/// not a raw pointer masked as a usize.
///
/// For information on why the pointer is masked, see the docs for `WasmerInstanceEnv`
pub struct WasmerInstance {
    instance: Instance,

    /// This field stores a (masked) runtime pointer to a `Box<dyn WasmRuntime>` which is shared
    /// by the instance and each WasmerInstanceEnv (every function that requires `env`).
    ///
    /// On every call into the WASM (ie every call to `invoke_export`), a `&'a mut System API` is
    /// wrapped in a temporary `RadixEngineWasmRuntime<'a>` and boxed, and a pointer to the freshly
    /// created `Box<dyn WasmRuntime>` is written behind the Mutex into this field.
    ///
    /// This same Mutex (via Arc cloning) is shared into each `WasmerInstanceEnv`, and so
    /// when the WASM makes calls back into env, it can read the pointer to the current
    /// WasmRuntime, and use that to call into the `&mut System API`.
    ///
    /// For information on why the pointer is masked, see the docs for `WasmerInstanceEnv`
    runtime_ptr: Arc<Mutex<usize>>,
}

/// The WasmerInstanceEnv implements WasmerEnv - and this needs to be `Send + Sync` for
/// Wasmer to work (see `Function::new_native_with_env`).
///
/// This is likely because Wasmer wants to be forward-compatible with multi-threaded WASM,
/// or that it uses multiple threads internally.
///
/// Currently, the SystemAPI is not Sync (and so should not be accessed by multiple threads)
/// we believe our use of Wasmer does not allow it to call us from multiple threads -
/// but we need to double-check this.
///
/// In any case, we temporarily work around this incompatibility by masking the pointer as a usize.
///
/// There are still a number of changes we should consider to improve things:
/// * `WasmerInstanceEnv` shouldn't contain an Instance - just a memory reference - see
///    the docs on the `WasmerEnv` trait
/// * If we instantiate the module just before we call into it, we could potentially pass an actual
///   `Arc<Mutex<T>>` for `a', T: WasmRuntime<'a>` (wrapping a `&'a mut SystemAPI`) into the WasmerInstanceEnv
///   on *module instantiation*. In this case, it doesn't need to be on a WasmerInstance at all
/// * Else at the very least, change this to be a pointer type, and manually implement Sync/Send
#[derive(Clone)]
pub struct WasmerInstanceEnv {
    instance: LazyInit<Instance>,
    /// See notes on `WasmerInstance.runtime_ptr`
    runtime_ptr: Arc<Mutex<usize>>,
}

pub struct WasmerEngine {
    store: Store,
    #[cfg(not(feature = "moka"))]
    modules_cache: RefCell<lru::LruCache<MeteredCodeKey, Arc<WasmerModule>>>,
    #[cfg(feature = "moka")]
    modules_cache: moka::sync::Cache<MeteredCodeKey, Arc<WasmerModule>>,
}

pub fn read_memory(instance: &Instance, ptr: u32, len: u32) -> Result<Vec<u8>, WasmRuntimeError> {
    let ptr = ptr as usize;
    let len = len as usize;

    let memory = instance
        .exports
        .get_memory(EXPORT_MEMORY)
        .map_err(|_| WasmRuntimeError::MemoryAccessError)?;
    let memory_slice = unsafe { memory.data_unchecked() };
    let memory_size = memory_slice.len();
    if ptr > memory_size || ptr + len > memory_size {
        return Err(WasmRuntimeError::MemoryAccessError);
    }

    Ok(memory_slice[ptr..ptr + len].to_vec())
}

pub fn write_memory(instance: &Instance, ptr: u32, data: &[u8]) -> Result<(), WasmRuntimeError> {
    let ptr = ptr as usize;
    let len = data.len();

    let memory = instance
        .exports
        .get_memory(EXPORT_MEMORY)
        .map_err(|_| WasmRuntimeError::MemoryAccessError)?;
    let memory_slice = unsafe { memory.data_unchecked_mut() };
    let memory_size = memory_slice.len();
    if ptr > memory_size || ptr + len > memory_size {
        return Err(WasmRuntimeError::MemoryAccessError);
    }

    memory_slice[ptr..ptr + data.len()].copy_from_slice(data);
    Ok(())
}

pub fn read_slice(instance: &Instance, v: Slice) -> Result<Vec<u8>, WasmRuntimeError> {
    let ptr = v.ptr();
    let len = v.len();

    read_memory(instance, ptr, len)
}

impl WasmerEnv for WasmerInstanceEnv {
    fn init_with_instance(&mut self, instance: &Instance) -> Result<(), HostEnvInitError> {
        self.instance.initialize(instance.clone());
        Ok(())
    }
}

macro_rules! grab_runtime {
    ($env: expr) => {{
        let instance = unsafe { $env.instance.get_unchecked() };
        let ptr = $env.runtime_ptr.lock().expect("Runtime ptr unavailable");
        let runtime: &mut Box<dyn WasmRuntime> = unsafe { &mut *(*ptr as *mut _) };
        (instance, runtime)
    }};
}

impl From<WasmRuntimeError> for RuntimeError {
    fn from(error: WasmRuntimeError) -> Self {
        RuntimeError::user(Box::new(error))
    }
}

impl WasmerModule {
    fn instantiate(&self) -> WasmerInstance {
        // native functions starts
        pub fn consume_buffer(
            env: &WasmerInstanceEnv,
            buffer_id: BufferId,
            destination_ptr: u32,
        ) -> Result<(), RuntimeError> {
            let (instance, runtime) = grab_runtime!(env);

            let slice = runtime
                .consume_buffer(buffer_id)
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            write_memory(&instance, destination_ptr, &slice)?;

            Ok(())
        }

        pub fn invoke_method(
            env: &WasmerInstanceEnv,
            receiver_ptr: u32,
            receiver_len: u32,
            ident_ptr: u32,
            ident_len: u32,
            args_ptr: u32,
            args_len: u32,
        ) -> Result<u64, RuntimeError> {
            let (instance, runtime) = grab_runtime!(env);

            let receiver = read_memory(&instance, receiver_ptr, receiver_len)?;
            let ident = read_memory(&instance, ident_ptr, ident_len)?;
            let args = read_memory(&instance, args_ptr, args_len)?;

            let buffer = runtime
                .invoke_method(receiver, ident, args)
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            Ok(buffer.0)
        }

        pub fn invoke(
            env: &WasmerInstanceEnv,
            invocation_ptr: u32,
            invocation_len: u32,
        ) -> Result<u64, RuntimeError> {
            let (instance, runtime) = grab_runtime!(env);

            let invocation = read_memory(&instance, invocation_ptr, invocation_len)?;

            let buffer = runtime
                .invoke(invocation)
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            Ok(buffer.0)
        }

        pub fn create_node(
            env: &WasmerInstanceEnv,
            node_ptr: u32,
            node_len: u32,
        ) -> Result<u64, RuntimeError> {
            let (instance, runtime) = grab_runtime!(env);

            let node = read_memory(&instance, node_ptr, node_len)?;

            let buffer = runtime
                .create_node(node)
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            Ok(buffer.0)
        }

        pub fn get_visible_nodes(env: &WasmerInstanceEnv) -> Result<u64, RuntimeError> {
            let (_instance, runtime) = grab_runtime!(env);

            let buffer = runtime
                .get_visible_nodes()
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            Ok(buffer.0)
        }

        pub fn drop_node(
            env: &WasmerInstanceEnv,
            node_id_ptr: u32,
            node_id_len: u32,
        ) -> Result<(), RuntimeError> {
            let (instance, runtime) = grab_runtime!(env);

            let node_id = read_memory(&instance, node_id_ptr, node_id_len)?;

            runtime
                .drop_node(node_id)
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            Ok(())
        }

        pub fn lock_substate(
            env: &WasmerInstanceEnv,
            node_id_ptr: u32,
            node_id_len: u32,
            offset_ptr: u32,
            offset_len: u32,
            mutable: u32,
        ) -> Result<u32, RuntimeError> {
            let (instance, runtime) = grab_runtime!(env);

            let node_id = read_memory(&instance, node_id_ptr, node_id_len)?;
            let offset = read_memory(&instance, offset_ptr, offset_len)?;

            let handle = runtime
                .lock_substate(node_id, offset, mutable != 0)
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            Ok(handle)
        }

        pub fn read_substate(env: &WasmerInstanceEnv, handle: u32) -> Result<u64, RuntimeError> {
            let (_instance, runtime) = grab_runtime!(env);

            let buffer = runtime
                .read_substate(handle)
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            Ok(buffer.0)
        }

        pub fn write_substate(
            env: &WasmerInstanceEnv,
            handle: u32,
            data_ptr: u32,
            data_len: u32,
        ) -> Result<(), RuntimeError> {
            let (instance, runtime) = grab_runtime!(env);

            let data = read_memory(&instance, data_ptr, data_len)?;

            runtime
                .write_substate(handle, data)
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            Ok(())
        }

        pub fn unlock_substate(env: &WasmerInstanceEnv, handle: u32) -> Result<(), RuntimeError> {
            let (_instance, runtime) = grab_runtime!(env);

            runtime
                .unlock_substate(handle)
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            Ok(())
        }

        pub fn get_actor(env: &WasmerInstanceEnv) -> Result<u64, RuntimeError> {
            let (_instance, runtime) = grab_runtime!(env);

            let buffer = runtime
                .get_actor()
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            Ok(buffer.0)
        }

        fn consume_cost_units(env: &WasmerInstanceEnv, cost_unit: u32) -> Result<(), RuntimeError> {
            let (_instance, runtime) = grab_runtime!(env);
            runtime
                .consume_cost_units(cost_unit)
                .map_err(|e| RuntimeError::user(Box::new(e)))
        }
        // native functions ends

        // env
        let env = WasmerInstanceEnv {
            instance: LazyInit::new(),
            runtime_ptr: Arc::new(Mutex::new(0)),
        };

        // imports
        let import_object = imports! {
            MODULE_ENV_NAME => {
                CONSUME_BUFFER_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), consume_buffer),
                INVOKE_METHOD_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), invoke_method),
                INVOKE_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), invoke),
                CREATE_NODE_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), create_node),
                GET_VISIBLE_NODES_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), get_visible_nodes),
                DROP_NODE_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), drop_node),
                LOCK_SUBSTATE_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), lock_substate),
                READ_SUBSTATE_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), read_substate),
                WRITE_SUBSTATE_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), write_substate),
                UNLOCK_SUBSTATE_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), unlock_substate),
                GET_ACTOR_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), get_actor),
                CONSUME_COST_UNITS_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), consume_cost_units),
            }
        };

        // instantiate
        let instance =
            Instance::new(&self.module, &import_object).expect("Failed to instantiate WASM module");

        WasmerInstance {
            instance,
            runtime_ptr: env.runtime_ptr,
        }
    }
}

impl From<RuntimeError> for InvokeError<WasmRuntimeError> {
    fn from(error: RuntimeError) -> Self {
        let e_str = format!("{:?}", error);
        match error.downcast::<InvokeError<WasmRuntimeError>>() {
            Ok(e) => e,
            _ => InvokeError::SelfError(WasmRuntimeError::InterpreterError(e_str)),
        }
    }
}

impl WasmInstance for WasmerInstance {
    fn invoke_export<'r>(
        &mut self,
        func_name: &str,
        args: Vec<Buffer>,
        runtime: &mut Box<dyn WasmRuntime + 'r>,
    ) -> Result<Vec<u8>, InvokeError<WasmRuntimeError>> {
        {
            // set up runtime pointer
            let mut guard = self.runtime_ptr.lock().expect("Runtime ptr unavailable");
            *guard = runtime as *mut _ as usize;
        }

        let input: Vec<Val> = args
            .into_iter()
            .map(|buffer| Val::I64(buffer.as_i64()))
            .collect();
        let return_data = self
            .instance
            .exports
            .get_function(func_name)
            .map_err(|_| {
                InvokeError::SelfError(WasmRuntimeError::UnknownWasmFunction(func_name.to_string()))
            })?
            .call(&input)
            .map_err(|e| {
                let err: InvokeError<WasmRuntimeError> = e.into();
                err
            })?;

        if let Some(v) = return_data.as_ref().get(0).and_then(|x| x.i64()) {
            read_slice(&self.instance, Slice::transmute_i64(v)).map_err(InvokeError::SelfError)
        } else {
            Err(InvokeError::SelfError(
                WasmRuntimeError::InvalidExportReturn,
            ))
        }
    }
}

#[derive(Debug, Clone)]
pub struct EngineOptions {
    max_cache_size_bytes: usize,
}

impl Default for WasmerEngine {
    fn default() -> Self {
        Self::new(EngineOptions {
            max_cache_size_bytes: 200 * 1024 * 1024,
        })
    }
}

impl WasmerEngine {
    pub fn new(options: EngineOptions) -> Self {
        let compiler = Singlepass::new();
        #[cfg(not(feature = "moka"))]
        let modules_cache = RefCell::new(lru::LruCache::new(
            NonZeroUsize::new(options.max_cache_size_bytes / (1024 * 1024)).unwrap(),
        ));
        #[cfg(feature = "moka")]
        let modules_cache = moka::sync::Cache::builder()
            .weigher(
                |_metered_code_key: &MeteredCodeKey, value: &Arc<WasmerModule>| -> u32 {
                    // Approximate the module entry size by the code size
                    value.code_size_bytes.try_into().unwrap_or(u32::MAX)
                },
            )
            .max_capacity(options.max_cache_size_bytes as u64)
            .build();
        Self {
            store: Store::new(&Universal::new(compiler).engine()),
            modules_cache,
        }
    }
}

impl WasmEngine for WasmerEngine {
    type WasmInstance = WasmerInstance;

    fn instantiate(&self, instrumented_code: &InstrumentedCode) -> WasmerInstance {
        let metered_code_key = &instrumented_code.metered_code_key;
        #[cfg(not(feature = "moka"))]
        {
            if let Some(cached_module) = self.modules_cache.borrow_mut().get(key) {
                return cached_module.instantiate();
            }
        }
        #[cfg(feature = "moka")]
        if let Some(cached_module) = self.modules_cache.get(metered_code_key) {
            return cached_module.instantiate();
        }

        let code = instrumented_code.code.as_ref();

        let new_module = Arc::new(WasmerModule {
            module: Module::new(&self.store, code).expect("Failed to parse WASM module"),
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
