use crate::model::InvokeError;
use radix_engine_interface::data::IndexedScryptoValue;
use sbor::rust::sync::{Arc, Mutex};
use wasmer::{
    imports, Function, HostEnvInitError, Instance, LazyInit, Module, RuntimeError, Store,
    Universal, Val, WasmerEnv,
};
use wasmer_compiler_singlepass::Singlepass;

use crate::types::*;
use crate::wasm::constants::*;
use crate::wasm::errors::*;
use crate::wasm::traits::*;

use super::InstrumentedCode;

pub struct WasmerModule {
    module: Module,
    #[allow(dead_code)]
    code_size_bytes: usize,
}

// IMPORTANT:
// The below implementation is not yet checked rigorously enough for production use
// TODO: Address the below issues before considering production use.

pub struct WasmerInstance {
    instance: Instance,

    /// This field stores a (masked) runtime pointer to `Box<dyn WasmRuntime>` which is shared
    /// by the instance and each WasmerInstanceEnv (every function that requires `env`).
    ///
    /// On every call into the WASM (ie every call to `invoke_export`), a &mut System API is
    /// temporarily wrapped in the `Box<dyn WasmRuntime>` and this pointer is updated.
    ///
    /// Because same pointer (via Arc cloning), it shared in each `WasmerInstanceEnv`,
    /// and so they can use it to call into the current `WasmRuntime` / `&mut System API`.
    ///
    /// For information on why the pointer is masked, see the docs for `WasmerInstanceEnv`
    runtime_ptr: Arc<Mutex<usize>>,
}

// We explicitly mark WasmerInstance as not Sync/Send because of its masked raw pointer
unsafe impl !Sync for WasmerInstance {}
unsafe impl !Send for WasmerInstance {}

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
    modules_cache: RefCell<lru::LruCache<Hash, Arc<WasmerModule>>>,
    #[cfg(feature = "moka")]
    modules_cache: moka::sync::Cache<Hash, Arc<WasmerModule>>,
}

pub fn send_value(instance: &Instance, value: &[u8]) -> Result<usize, InvokeError<WasmError>> {
    let n = value.len();

    let result = instance
        .exports
        .get_function(EXPORT_SCRYPTO_ALLOC)
        .expect("ScryptoAlloc not found")
        .call(&[Val::I32(n as i32)])
        .map_err(|e| {
            let error: InvokeError<WasmError> = e.into();
            error
        })?;

    if let Some(wasmer::Value::I32(ptr)) = result.as_ref().get(0) {
        let ptr = *ptr as usize;
        let memory = instance
            .exports
            .get_memory(EXPORT_MEMORY)
            .map_err(|_| InvokeError::Error(WasmError::MemoryAllocError))?;
        let size = memory.size().bytes().0;
        if size > ptr && size - ptr >= n {
            unsafe {
                let dest = memory.data_ptr().add(ptr + 4);
                ptr::copy(value.as_ptr(), dest, n);
            }
            return Ok(ptr);
        }
    }

    Err(InvokeError::Error(WasmError::MemoryAllocError))
}

pub fn read_value(instance: &Instance, ptr: usize) -> Result<IndexedScryptoValue, WasmError> {
    let memory = instance
        .exports
        .get_memory(EXPORT_MEMORY)
        .map_err(|_| WasmError::MemoryAccessError)?;
    let size = memory.size().bytes().0;
    if size > ptr && size - ptr >= 4 {
        // read len
        let mut temp = [0u8; 4];
        unsafe {
            let from = memory.data_ptr().add(ptr);
            ptr::copy(from, temp.as_mut_ptr(), 4);
        }
        let n = u32::from_le_bytes(temp) as usize;

        // read value
        if size - ptr - 4 >= (n as usize) {
            // TODO: avoid copying
            let mut temp = Vec::with_capacity(n);
            unsafe {
                let from = memory.data_ptr().add(ptr).add(4);
                ptr::copy(from, temp.as_mut_ptr(), n);
                temp.set_len(n);
            }

            return IndexedScryptoValue::from_slice(&temp).map_err(WasmError::InvalidScryptoValue);
        }
    }

    Err(WasmError::MemoryAccessError)
}

impl WasmerEnv for WasmerInstanceEnv {
    fn init_with_instance(&mut self, instance: &Instance) -> Result<(), HostEnvInitError> {
        self.instance.initialize(instance.clone());
        Ok(())
    }
}

impl WasmerModule {
    fn instantiate(&self) -> WasmerInstance {
        // native functions
        fn radix_engine(env: &WasmerInstanceEnv, input_ptr: i32) -> Result<i32, RuntimeError> {
            let instance = unsafe { env.instance.get_unchecked() };
            let input = read_value(&instance, input_ptr as usize)
                .map_err(|e| RuntimeError::user(Box::new(e)))?;

            let output = {
                let ptr = env
                    .runtime_ptr
                    .lock()
                    .expect("Failed to lock WASM runtime pointer");
                let runtime: &mut Box<dyn WasmRuntime> = unsafe { &mut *(*ptr as *mut _) };
                runtime
                    .main(input)
                    .map_err(|e| RuntimeError::user(Box::new(e)))?
            };

            send_value(&instance, &output)
                .map(|ptr| ptr as i32)
                .map_err(|e| RuntimeError::user(Box::new(e)))
        }

        fn consume_cost_units(env: &WasmerInstanceEnv, cost_unit: i32) -> Result<(), RuntimeError> {
            let ptr = env
                .runtime_ptr
                .lock()
                .expect("Failed to lock WASM runtime pointer");
            let runtime: &mut Box<dyn WasmRuntime> = unsafe { &mut *(*ptr as *mut _) };
            runtime
                .consume_cost_units(cost_unit as u32)
                .map_err(|e| RuntimeError::user(Box::new(e)))
        }

        // env
        let env = WasmerInstanceEnv {
            instance: LazyInit::new(),
            runtime_ptr: Arc::new(Mutex::new(0)),
        };

        // imports
        let import_object = imports! {
            MODULE_ENV_NAME => {
                RADIX_ENGINE_FUNCTION_NAME => Function::new_native_with_env(self.module.store(), env.clone(), radix_engine),
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

impl From<RuntimeError> for InvokeError<WasmError> {
    fn from(error: RuntimeError) -> Self {
        let e_str = format!("{:?}", error);
        match error.downcast::<InvokeError<WasmError>>() {
            Ok(e) => e,
            _ => InvokeError::Error(WasmError::WasmError(e_str)),
        }
    }
}

impl WasmInstance for WasmerInstance {
    fn invoke_export<'r>(
        &mut self,
        func_name: &str,
        args: &IndexedScryptoValue,
        runtime: &mut Box<dyn WasmRuntime + 'r>,
    ) -> Result<IndexedScryptoValue, InvokeError<WasmError>> {
        {
            // set up runtime pointer
            let mut guard = self
                .runtime_ptr
                .lock()
                .expect("Failed to lock WASM runtime pointer");
            *guard = runtime as *mut _ as usize;
        }

        let pointer = send_value(&self.instance, &args.raw)?;
        let result = self
            .instance
            .exports
            .get_function(func_name)
            .map_err(|_| InvokeError::Error(WasmError::FunctionNotFound))?
            .call(&[Val::I32(pointer as i32)]);

        match result {
            Ok(return_data) => {
                let ptr = return_data
                    .as_ref()
                    .get(0)
                    .ok_or(InvokeError::Error(WasmError::MissingReturnData))?
                    .i32()
                    .ok_or(InvokeError::Error(WasmError::InvalidReturnData))?;
                read_value(&self.instance, ptr as usize).map_err(InvokeError::Error)
            }
            Err(e) => Err(e.into()),
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
            .weigher(|_key: &Hash, value: &Arc<WasmerModule>| -> u32 {
                // Approximate the module entry size by the code size
                value.code_size_bytes.try_into().unwrap_or(u32::MAX)
            })
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
        let code_hash = &instrumented_code.code_hash;
        #[cfg(not(feature = "moka"))]
        {
            if let Some(cached_module) = self.modules_cache.borrow_mut().get(code_hash) {
                return cached_module.instantiate();
            }
        }
        #[cfg(feature = "moka")]
        if let Some(cached_module) = self.modules_cache.get(code_hash) {
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
            .put(*code_hash, new_module.clone());
        #[cfg(feature = "moka")]
        self.modules_cache.insert(*code_hash, new_module.clone());

        new_module.instantiate()
    }
}
