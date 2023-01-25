use radix_engine_interface::api::wasm::Buffer;
use radix_engine_interface::api::wasm::BufferId;
use radix_engine_interface::api::wasm::Slice;
use sbor::rust::sync::Arc;
use wasmi::core::Value;
use wasmi::core::{HostError, Trap};
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
    // (Store, Instance) tuple are cached together, and never to be invoked
    // Every `WasmiModule` is going to clone the store and instance, so the state is not shared
    store: Store<HostState>,
    instance: Instance,
    #[allow(dead_code)]
    code_size_bytes: usize,
}

pub struct WasmiInstance {
    store: Store<HostState>,
    instance: Instance,
    memory: Memory,
}

#[derive(Clone)]
pub struct WasmiInstanceEnv {
    runtime_ptr: usize,
}

impl WasmiInstanceEnv {
    pub fn new() -> Self {
        Self { runtime_ptr: 0 }
    }
}

macro_rules! grab_runtime {
    ($caller: expr) => {{
        // FIXME
        //let ptr = $env.runtime_ptr.lock().expect("Runtime ptr unavailable");
        //let runtime: &mut Box<dyn WasmRuntime> = unsafe { &mut *(*ptr as *mut _) };
        let runtime: &mut Box<dyn WasmRuntime> =
            unsafe { &mut *($caller.data().runtime_ptr as *mut _) };

        let memory = match $caller.get_export(EXPORT_MEMORY) {
            Some(Extern::Memory(memory)) => memory,
            _ => panic!("Failed to find memory export"),
        };
        (memory, runtime)
    }};
}

// native functions start
fn consume_buffer(
    caller: Caller<'_, HostState>,
    buffer_id: BufferId,
    destination_ptr: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let result = runtime.consume_buffer(buffer_id);
    match result {
        Ok(slice) => {
            write_memory(caller, memory, destination_ptr, &slice)?;
            Ok(())
        }
        Err(e) => Err(e),
    }
}

fn invoke_method(
    mut caller: Caller<'_, HostState>,
    receiver_ptr: u32,
    receiver_len: u32,
    ident_ptr: u32,
    ident_len: u32,
    args_ptr: u32,
    args_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let receiver = read_memory(caller.as_context_mut(), memory, receiver_ptr, receiver_len)?;
    let ident = read_memory(caller.as_context_mut(), memory, ident_ptr, ident_len)?;
    let args = read_memory(caller.as_context_mut(), memory, args_ptr, args_len)?;

    runtime
        .invoke_method(receiver, ident, args)
        .map(|buffer| buffer.0)
}

fn invoke(
    mut caller: Caller<'_, HostState>,
    invocation_ptr: u32,
    invocation_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let invocation = read_memory(
        caller.as_context_mut(),
        memory,
        invocation_ptr,
        invocation_len,
    )?;

    runtime.invoke(invocation).map(|buffer| buffer.0)
}

fn create_node(
    mut caller: Caller<'_, HostState>,
    node_ptr: u32,
    node_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let node = read_memory(caller.as_context_mut(), memory, node_ptr, node_len)?;

    runtime.create_node(node).map(|buffer| buffer.0)
}

fn get_visible_nodes(caller: Caller<'_, HostState>) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (_memory, runtime) = grab_runtime!(caller);

    runtime.get_visible_nodes().map(|buffer| buffer.0)
}

fn drop_node(
    mut caller: Caller<'_, HostState>,
    node_id_ptr: u32,
    node_id_len: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let node_id = read_memory(caller.as_context_mut(), memory, node_id_ptr, node_id_len)?;

    runtime.drop_node(node_id)
}

fn lock_substate(
    mut caller: Caller<'_, HostState>,
    node_id_ptr: u32,
    node_id_len: u32,
    offset_ptr: u32,
    offset_len: u32,
    mutable: u32,
) -> Result<u32, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let node_id = read_memory(caller.as_context_mut(), memory, node_id_ptr, node_id_len)?;
    let offset = read_memory(caller.as_context_mut(), memory, offset_ptr, offset_len)?;

    runtime.lock_substate(node_id, offset, mutable != 0)
}

fn read_substate(
    caller: Caller<'_, HostState>,
    handle: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (_memory, runtime) = grab_runtime!(caller);

    runtime.read_substate(handle).map(|buffer| buffer.0)
}

fn write_substate(
    mut caller: Caller<'_, HostState>,
    handle: u32,
    data_ptr: u32,
    data_len: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let data = read_memory(caller.as_context_mut(), memory, data_ptr, data_len)?;

    runtime.write_substate(handle, data)
}

fn unlock_substate(
    caller: Caller<'_, HostState>,
    handle: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let (_memory, runtime) = grab_runtime!(caller);

    runtime.unlock_substate(handle)
}

fn get_actor(caller: Caller<'_, HostState>) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (_memory, runtime) = grab_runtime!(caller);

    runtime.get_actor().map(|buffer| buffer.0)
}

fn consume_cost_units(
    caller: Caller<'_, HostState>,
    cost_unit: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let (_memory, runtime) = grab_runtime!(caller);
    runtime.consume_cost_units(cost_unit)
}
// native functions ends

macro_rules! linker_define {
    ($linker: expr, $name: expr, $var: expr) => {
        $linker
            .define(MODULE_ENV_NAME, $name, $var)
            .expect(stringify!("Failed to define new linker item {}", $name));
    };
}

impl WasmiModule {
    pub fn new(code: &[u8]) -> Self {
        let engine = Engine::default();
        let module = Module::new(&engine, code).expect("Failed to parse WASM module");
        let mut store = Store::new(&engine, WasmiInstanceEnv::new());

        let instance = Self::host_funcs_set(&module, &mut store)
            .expect("Failed to instantiate WASM module - did you run WasmValidator?")
            .ensure_no_start(store.as_context_mut())
            .expect("Module has start function - did you run WasmValidator?");

        Self {
            store,
            instance,
            code_size_bytes: code.len(),
        }
    }

    pub fn host_funcs_set(
        module: &Module,
        store: &mut Store<HostState>,
    ) -> Result<InstancePre, Error> {
        let host_consume_buffer = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             buffer_id: BufferId,
             destination_ptr: u32|
             -> Result<(), Trap> {
                consume_buffer(caller, buffer_id, destination_ptr).map_err(|e| e.into())
            },
        );

        let host_invoke_method = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             receiver_ptr: u32,
             receiver_len: u32,
             ident_ptr: u32,
             ident_len: u32,
             args_ptr: u32,
             args_len: u32|
             -> Result<u64, Trap> {
                invoke_method(
                    caller,
                    receiver_ptr,
                    receiver_len,
                    ident_ptr,
                    ident_len,
                    args_ptr,
                    args_len,
                )
                .map_err(|e| e.into())
            },
        );

        let host_invoke = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             invocation_ptr: u32,
             invocation_len: u32|
             -> Result<u64, Trap> {
                invoke(caller, invocation_ptr, invocation_len).map_err(|e| e.into())
            },
        );

        let host_create_node = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>, node_ptr: u32, node_len: u32| -> Result<u64, Trap> {
                create_node(caller, node_ptr, node_len).map_err(|e| e.into())
            },
        );

        let host_get_visible_nodes = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>| -> Result<u64, Trap> {
                get_visible_nodes(caller).map_err(|e| e.into())
            },
        );

        let host_drop_node = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             node_id_ptr: u32,
             node_id_len: u32|
             -> Result<(), Trap> {
                drop_node(caller, node_id_ptr, node_id_len).map_err(|e| e.into())
            },
        );

        let host_lock_substate = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             node_id_ptr: u32,
             node_id_len: u32,
             offset_ptr: u32,
             offset_len: u32,
             mutable: u32|
             -> Result<u32, Trap> {
                lock_substate(
                    caller,
                    node_id_ptr,
                    node_id_len,
                    offset_ptr,
                    offset_len,
                    mutable,
                )
                .map_err(|e| e.into())
            },
        );

        let host_read_substate = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>, handle: u32| -> Result<u64, Trap> {
                read_substate(caller, handle).map_err(|e| e.into())
            },
        );

        let host_write_substate = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             handle: u32,
             data_ptr: u32,
             data_len: u32|
             -> Result<(), Trap> {
                write_substate(caller, handle, data_ptr, data_len).map_err(|e| e.into())
            },
        );

        let host_unlock_substate = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>, handle: u32| -> Result<(), Trap> {
                unlock_substate(caller, handle).map_err(|e| e.into())
            },
        );

        let host_get_actor = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>| -> Result<u64, Trap> {
                get_actor(caller).map_err(|e| e.into())
            },
        );

        let host_consume_cost_units = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>, cost_unit: u32| -> Result<(), Trap> {
                consume_cost_units(caller, cost_unit).map_err(|e| e.into())
            },
        );

        let mut linker = <Linker<HostState>>::new();
        linker_define!(linker, CONSUME_BUFFER_FUNCTION_NAME, host_consume_buffer);
        linker_define!(linker, INVOKE_METHOD_FUNCTION_NAME, host_invoke_method);
        linker_define!(linker, INVOKE_FUNCTION_NAME, host_invoke);
        linker_define!(linker, CREATE_NODE_FUNCTION_NAME, host_create_node);
        linker_define!(
            linker,
            GET_VISIBLE_NODES_FUNCTION_NAME,
            host_get_visible_nodes
        );
        linker_define!(linker, DROP_NODE_FUNCTION_NAME, host_drop_node);
        linker_define!(linker, LOCK_SUBSTATE_FUNCTION_NAME, host_lock_substate);
        linker_define!(linker, READ_SUBSTATE_FUNCTION_NAME, host_read_substate);
        linker_define!(linker, WRITE_SUBSTATE_FUNCTION_NAME, host_write_substate);
        linker_define!(linker, UNLOCK_SUBSTATE_FUNCTION_NAME, host_unlock_substate);
        linker_define!(linker, GET_ACTOR_FUNCTION_NAME, host_get_actor);
        linker_define!(
            linker,
            CONSUME_COST_UNITS_FUNCTION_NAME,
            host_consume_cost_units
        );

        let pre_instance = match linker.instantiate(store.as_context_mut(), &module) {
            Ok(result) => result,
            Err(e) => {
                panic!("Failed to instantiate WASM module - {}", e.to_string());
            }
        };

        Ok(pre_instance)
    }

    fn instantiate(&self) -> WasmiInstance {
        let instance = self.instance.clone();
        let mut store = self.store.clone();
        let memory = match instance.get_export(store.as_context_mut(), EXPORT_MEMORY) {
            Some(Extern::Memory(memory)) => memory,
            _ => panic!("Failed to find memory export"),
        };
        WasmiInstance {
            instance,
            store,
            memory,
        }
    }
}

fn read_memory(
    store: impl AsContextMut,
    memory: Memory,
    ptr: u32,
    len: u32,
) -> Result<Vec<u8>, InvokeError<WasmRuntimeError>> {
    let store_ctx = store.as_context();
    let data = memory.data(&store_ctx);
    let ptr = ptr as usize;
    let len = len as usize;

    if ptr > data.len() || ptr + len > data.len() {
        return Err(InvokeError::SelfError(WasmRuntimeError::MemoryAccessError));
    }
    Ok(data[ptr..ptr + len].to_vec())
}

fn write_memory(
    mut store: impl AsContextMut,
    memory: Memory,
    ptr: u32,
    data: &[u8],
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let mut store_ctx = store.as_context_mut();
    let mem_data = memory.data(&mut store_ctx);

    if ptr as usize > mem_data.len() || ptr as usize + data.len() > mem_data.len() {
        return Err(InvokeError::SelfError(WasmRuntimeError::MemoryAccessError));
    }

    memory
        .write(&mut store.as_context_mut(), ptr as usize, data)
        .or_else(|_| Err(InvokeError::SelfError(WasmRuntimeError::MemoryAccessError)))
}

fn read_slice(
    store: impl AsContextMut,
    memory: Memory,
    v: Slice,
) -> Result<Vec<u8>, InvokeError<WasmRuntimeError>> {
    let ptr = v.ptr();
    let len = v.len();

    read_memory(store, memory, ptr, len)
}

impl WasmiInstance {
    fn get_export_func(&mut self, name: &str) -> Result<Func, InvokeError<WasmRuntimeError>> {
        self.instance
            .get_export(self.store.as_context_mut(), name)
            .and_then(Extern::into_func)
            .ok_or_else(|| {
                InvokeError::SelfError(WasmRuntimeError::UnknownWasmFunction(name.to_string()))
            })
    }
}

impl HostError for InvokeError<WasmRuntimeError> {}

impl From<Error> for InvokeError<WasmRuntimeError> {
    fn from(err: Error) -> Self {
        let e_str = format!("{:?}", err);
        match err {
            Error::Trap(trap) => {
                let invoke_err = trap
                    .downcast_ref::<InvokeError<WasmRuntimeError>>()
                    .unwrap_or(&InvokeError::SelfError(WasmRuntimeError::InvalidExportReturn));
                invoke_err.clone()
            }
            _ => InvokeError::SelfError(WasmRuntimeError::InterpreterError(e_str)),
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
        {
            // set up runtime pointer
            //let mut guard = self.store
            //    data_mut().runtime_ptr.lock().expect("Runtime ptr unavailable");
            //*guard = runtime as *mut _ as usize;
            self.store.data_mut().runtime_ptr = runtime as *mut _ as usize;
        }

        let func = self.get_export_func(func_name).unwrap();
        let input: Vec<Value> = args
            .into_iter()
            .map(|buffer| Value::I64(buffer.as_i64()))
            .collect();
        let mut ret = [Value::I64(0)];

        let _result = func
            .call(self.store.as_context_mut(), &input, &mut ret)
            .map_err(|e| {
                let err: InvokeError<WasmRuntimeError> = e.into();
                err
            })?;

        match i64::try_from(ret[0]) {
            Ok(ret) => read_slice(
                self.store.as_context_mut(),
                self.memory,
                Slice::transmute_i64(ret),
            ),
            _ => Err(InvokeError::SelfError(WasmRuntimeError::InvalidExportReturn)),
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
            return cached_module.as_ref().instantiate();
        }

        let code = &instrumented_code.code.as_ref()[..];
        let module = WasmiModule::new(code);
        let instance = module.instantiate();

        #[cfg(not(feature = "moka"))]
        self.modules_cache
            .borrow_mut()
            .put(*metered_code_key, Arc::new(module));
        #[cfg(feature = "moka")]
        self.modules_cache
            .insert(*metered_code_key, Arc::new(module));

        instance
    }
}
