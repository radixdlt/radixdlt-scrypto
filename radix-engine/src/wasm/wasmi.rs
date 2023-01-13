use radix_engine_interface::data::IndexedScryptoValue;
use sbor::rust::sync::{Arc, Mutex};
//use wasmi::core::Value as WasmiValue;
use wasmi::core::Trap;
use wasmi::core::Value;
use wasmi::*;

use super::InstrumentedCode;
use super::MeteredCodeKey;
use crate::model::InvokeError;
use crate::types::*;
use crate::wasm::constants::*;
use crate::wasm::errors::*;
use crate::wasm::traits::*;

type HostState = WasmiInstanceEnv;

pub struct WasmiModule {
    module: Module,
    #[allow(dead_code)]
    code_size_bytes: usize,
}

#[derive(Clone)]
pub struct WasmiInstance {
    instance: Instance,
    store: Arc<Mutex<Store<HostState>>>,
    memory: Memory,
    runtime_ptr: Arc<Mutex<usize>>,
}

pub struct WasmiInstanceEnv {
    //instance: std::mem::MaybeUninit<WasmiInstance>,
    runtime_ptr: Arc<Mutex<usize>>,
}

impl WasmiInstanceEnv {
    pub fn new() -> Self {
        Self {
            runtime_ptr: Arc::new(Mutex::new(0)),
        }
    }
}

// native functions
fn wasmi_radix_engine(
    mut caller: Caller<'_, HostState>,
    input_ptr: i32,
) -> Result<i32, InvokeError<WasmError>> {
    let memory = match caller.get_export(EXPORT_MEMORY) {
        Some(Extern::Memory(memory)) => memory,
        _ => panic!("Failed to find memory export"),
    };

    let input = read_value(caller.as_context_mut(), memory, input_ptr as usize)
        .map_err(InvokeError::Error)
        .unwrap();

    let output = {
        let env = caller.data();
        let ptr = env
            .runtime_ptr
            .lock()
            .expect("Failed to lock WASM runtime pointer");
        let runtime: &mut Box<dyn WasmRuntime> = unsafe { &mut *(*ptr as *mut _) };

        runtime.main(input).unwrap()
    };
    let alloc_func = caller
        .get_export(EXPORT_SCRYPTO_ALLOC)
        .and_then(Extern::into_func)
        .ok_or_else(|| InvokeError::Error(WasmError::FunctionNotFound))
        .unwrap();

    send_value(caller.as_context_mut(), memory, alloc_func, &output)
}

fn consume_cost_units(
    caller: Caller<'_, HostState>,
    cost_unit: i32,
) -> Result<(), InvokeError<WasmError>> {
    let env = caller.data();

    let ptr = env
        .runtime_ptr
        .lock()
        .expect("Failed to lock WASM runtime pointer");
    let runtime: &mut Box<dyn WasmRuntime> = unsafe { &mut *(*ptr as *mut _) };
    runtime.consume_cost_units(cost_unit as u32)
}

impl WasmiModule {
    pub fn new(code: &[u8]) -> Self {
        let engine = Engine::default();
        let module = Module::new(&engine, code).expect("Failed to parse WASM module");

        let code_size_bytes = code.len();
        Self {
            module,
            code_size_bytes,
        }
    }

    pub fn engine(&self) -> &Engine {
        self.module.engine()
    }

    pub fn host_funcs_set(&self, store: &mut Store<HostState>) -> Result<InstancePre, Error> {
        let host_radix_engine = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>, input_ptr: i32| -> Result<i32, Trap> {
                wasmi_radix_engine(caller, input_ptr).map_err(|e| Trap::new(e.to_string()))
            },
        );

        let host_consume_cost_units = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>, cost_unit: i32| -> Result<(), Trap> {
                consume_cost_units(caller, cost_unit).map_err(|e| Trap::new(e.to_string()))
            },
        );

        let mut linker = <Linker<HostState>>::new();

        linker
            .define(
                MODULE_ENV_NAME,
                RADIX_ENGINE_FUNCTION_NAME,
                host_radix_engine,
            )
            .expect(stringify!(
                "Failed to define new linker item {}",
                RADIX_ENGINE_FUNCTION_NAME
            ));

        linker
            .define(
                MODULE_ENV_NAME,
                CONSUME_COST_UNITS_FUNCTION_NAME,
                host_consume_cost_units,
            )
            .expect(stringify!(
                "Failed to define new linker item {}",
                CONSUME_COST_UNITS_FUNCTION_NAME
            ));

        let pre_instance = match linker.instantiate(store.as_context_mut(), &self.module) {
            Ok(result) => result,
            Err(e) => {
                panic!("Failed to instantiate WASM module - {}", e.to_string());
            }
        };

        Ok(pre_instance)
    }

    fn instantiate(&self) -> WasmiInstance {
        let env = WasmiInstanceEnv::new();
        let runtime_ptr = env.runtime_ptr.clone();
        let engine = self.module.engine();
        let mut store = Store::new(engine, env);

        let pre_instance = self.host_funcs_set(&mut store).unwrap();
        //            expect("Failed to instantiate WASM module");

        let instance = pre_instance
            .start(store.as_context_mut())
            .expect("Failed to instantiate WASM module - start function exists");

        let memory = match instance.get_export(store.as_context_mut(), EXPORT_MEMORY) {
            Some(Extern::Memory(memory)) => memory,
            _ => panic!("Failed to find memory export"),
        };

        WasmiInstance {
            instance,
            store: Arc::new(Mutex::new(store)),
            memory,
            runtime_ptr,
        }
    }
}

fn send_value(
    mut store: impl AsContextMut,
    memory: Memory,
    alloc_func: Func,
    value: &[u8],
) -> Result<i32, InvokeError<WasmError>> {
    // TODO: fix this shitty arguments
    let n = [Value::I32(value.len() as i32)];
    let mut ret: [Value; 1] = [Value::I32(0)];

    let result = alloc_func.call(store.as_context_mut(), &n, &mut ret);

    match result {
        Ok(()) => {
            let ret: i32 = i32::try_from(ret[0]).unwrap();

            if memory
                .write(store.as_context_mut(), (ret + 4) as usize, value)
                .is_ok()
            {
                return Ok(ret);
            }

            return Err(InvokeError::Error(WasmError::MemoryAllocError));
        }
        Err(e) => Err(InvokeError::Error(WasmError::WasmError(e.to_string()))),
    }
}

fn read_value(
    mut store: impl AsContextMut,
    memory: Memory,
    ptr: usize,
) -> Result<IndexedScryptoValue, WasmError> {
    let mut buf = [0_u8; 4];

    let _result = memory
        .read(store.as_context_mut(), ptr, &mut buf)
        .map_err(|_| WasmError::MemoryAccessError);

    let len = u32::from_le_bytes(buf) as usize;

    let start = ptr.checked_add(4).ok_or(WasmError::MemoryAccessError)?;
    let end = start.checked_add(len).ok_or(WasmError::MemoryAccessError)?;

    let ctx = store.as_context_mut();
    let data = memory.data(&ctx);

    if end > data.len() {
        return Err(WasmError::MemoryAccessError);
    }
    IndexedScryptoValue::from_slice(&data[start..end]).map_err(WasmError::InvalidScryptoValue)
}

impl WasmiInstance {
    fn get_export_func(&mut self, name: &str) -> Result<Func, InvokeError<WasmError>> {
        let mut store = self.store.lock().unwrap();
        self.instance
            .get_export(store.as_context_mut(), name)
            .and_then(Extern::into_func)
            .ok_or_else(|| InvokeError::Error(WasmError::FunctionNotFound))
    }
}

impl WasmInstance for WasmiInstance {
    fn invoke_export<'r>(
        &mut self,
        func_name: &str,
        args: Vec<Vec<u8>>,
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

        // get_func() lock store as well, thus we call it before locking here
        let alloc_func = self.get_export_func(EXPORT_SCRYPTO_ALLOC).unwrap();
        let func = self.get_export_func(func_name).unwrap();

        let store = self.store.clone();
        let mut store = store.lock().unwrap();

        let mut pointers = Vec::new();
        for arg in args {
            let pointer = send_value(store.as_context_mut(), self.memory, alloc_func, &arg)?;
            pointers.push(Value::I32(pointer));
        }

        let mut ret = [Value::I32(0)];
        let result = func.call(store.as_context_mut(), &pointers[..], &mut ret);

        match result {
            Ok(()) => {
                let ret: i32 = i32::try_from(ret[0]).unwrap();

                return read_value(store.as_context_mut(), self.memory, ret as usize)
                    .map_err(InvokeError::Error);
            }
            Err(e) => Err(InvokeError::Error(WasmError::WasmError(e.to_string()))),
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

        let code = &instrumented_code.code.as_ref()[..];

        let new_module = Arc::new(WasmiModule::new(code));

        #[cfg(not(feature = "moka"))]
        self.modules_cache
            .borrow_mut()
            .put(*metered_code_key, new_module.clone());
        #[cfg(feature = "moka")]
        // FIXME: Temporary disabling cache
        // Wasmi::Engine is locked when it executes a function, which makes it impossible to reuse
        // cached module with locked Engine (case when invoked function tries to instantiate
        // already cached module).
        //        self.modules_cache
        //            .insert(*metered_code_key, new_module.clone());
        new_module.instantiate()
    }
}
