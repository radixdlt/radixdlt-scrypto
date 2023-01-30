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

type HostState<'r, 'a> = WasmiInstanceEnv<'r, 'a>;

/// A `WasmiModule` defines a parsed WASM module "template" Instance (with imports already defined) and Store,
/// which keeps user data.
/// "Template" (Store, Instance) tuple are cached together, and never to be invoked.
/// Upon instantiation Instance and Store are cloned, so the state is not shared between instances.
/// It is correctly `Send + Sync` - which is good, because this is the thing which is cached in the
/// ScryptoInterpreter caches.
pub struct WasmiModule {
    template_store: Store<WasmiInstanceEnv<'static, 'static>>,
    template_instance: Instance,
    #[allow(dead_code)]
    code_size_bytes: usize,
}

pub struct WasmiInstance<'r, 'a: 'r> {
    store: Store<WasmiInstanceEnv<'r, 'a>>,
    instance: Instance,
    memory: Memory,
}

/// WasmiInstanceEnv stores masked runtime pointer to a `Box<dyn WasmRuntime>`.
/// We mask it for below reasons:
/// - Store must be `Send + Sync`
///   - According to the `wasmi` documentation
///     At the moment `&'a mut System API` which is wrapped into `RadixEngineWasmRuntime` and boxed
///     is not `Sync`
///     (the same reason as in alternative `wasmer` implementation)
///   - Store also shall be `Send + Sync` because of `WasmiModule`, which is cached.
/// - WasmiInstanceEnv must be clonable (it is a user data for the Store, which is cloned when
///   WasmiModule is instantiated)
pub struct WasmiInstanceEnv<'r, 'a: 'r> {
    runtime: Option<&'r mut Box<dyn WasmRuntime + 'a>>,
}

unsafe impl Send for WasmiInstanceEnv<'static, 'static> {}
unsafe impl Sync for WasmiInstanceEnv<'static, 'static> {}

impl Clone for WasmiInstanceEnv<'static, 'static> {
    fn clone(&self) -> Self {
        Self { runtime: None }
    }
}

impl WasmiInstanceEnv<'static, 'static> {
    pub fn new() -> Self {
        Self { runtime: None }
    }
}

pub fn clone_store_with_new_runtime_reference<'r, 'a: 'r>(
    template_store: Store<WasmiInstanceEnv<'static, 'static>>,
    runtime: &'r mut Box<dyn WasmRuntime + 'a>,
) -> Store<WasmiInstanceEnv<'r, 'a>> {
    let mut new_store: Store<WasmiInstanceEnv<'r, 'a>> = unsafe {
        sbor::rust::mem::transmute::<
            Store<WasmiInstanceEnv<'static, 'static>>,
            Store<WasmiInstanceEnv<'r, 'a>>,
        >(template_store)
    };
    new_store.data_mut().runtime = Some(runtime);
    new_store
}

pub fn get_memory(caller: &Caller<'_, HostState>) -> Memory {
    match caller.get_export(EXPORT_MEMORY) {
        Some(Extern::Memory(memory)) => memory,
        _ => panic!("Failed to find memory export"),
    }
}

pub fn get_runtime<'a: 'o, 'b: 'o, 'r: 'o, 's: 'o, 'o>(
    caller: &'b mut Caller<'s, HostState<'r, 'a>>,
) -> &'o mut Box<dyn WasmRuntime + 'a> {
    caller.data_mut().runtime.as_deref_mut().unwrap()
}

// native functions start
fn consume_buffer(
    mut caller: Caller<'_, HostState>,
    buffer_id: BufferId,
    destination_ptr: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let result = get_runtime(&mut caller).consume_buffer(buffer_id);

    let memory = get_memory(&caller);
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
    let memory = get_memory(&caller);

    let receiver = read_memory(caller.as_context_mut(), memory, receiver_ptr, receiver_len)?;
    let ident = read_memory(caller.as_context_mut(), memory, ident_ptr, ident_len)?;
    let args = read_memory(caller.as_context_mut(), memory, args_ptr, args_len)?;

    get_runtime(&mut caller)
        .invoke_method(receiver, ident, args)
        .map(|buffer| buffer.0)
}

fn invoke(
    mut caller: Caller<'_, HostState>,
    invocation_ptr: u32,
    invocation_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let memory = get_memory(&caller);

    let invocation = read_memory(
        caller.as_context_mut(),
        memory,
        invocation_ptr,
        invocation_len,
    )?;

    get_runtime(&mut caller).invoke(invocation).map(|buffer| buffer.0)
}

fn create_node(
    mut caller: Caller<'_, HostState>,
    node_ptr: u32,
    node_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let memory = get_memory(&caller);

    let node = read_memory(caller.as_context_mut(), memory, node_ptr, node_len)?;

    get_runtime(&mut caller).create_node(node).map(|buffer| buffer.0)
}

fn get_visible_nodes(
    mut caller: Caller<'_, HostState>,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    get_runtime(&mut caller).get_visible_nodes().map(|buffer| buffer.0)
}

fn drop_node(
    mut caller: Caller<'_, HostState>,
    node_id_ptr: u32,
    node_id_len: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let memory = get_memory(&caller);

    let node_id = read_memory(caller.as_context_mut(), memory, node_id_ptr, node_id_len)?;

    get_runtime(&mut caller).drop_node(node_id)
}

fn lock_substate(
    mut caller: Caller<'_, HostState>,
    node_id_ptr: u32,
    node_id_len: u32,
    offset_ptr: u32,
    offset_len: u32,
    mutable: u32,
) -> Result<u32, InvokeError<WasmRuntimeError>> {
    let memory = get_memory(&caller);

    let node_id = read_memory(caller.as_context_mut(), memory, node_id_ptr, node_id_len)?;
    let offset = read_memory(caller.as_context_mut(), memory, offset_ptr, offset_len)?;

    get_runtime(&mut caller).lock_substate(node_id, offset, mutable != 0)
}

fn read_substate(
    mut caller: Caller<'_, HostState>,
    handle: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    get_runtime(&mut caller).read_substate(handle).map(|buffer| buffer.0)
}

fn write_substate(
    mut caller: Caller<'_, HostState>,
    handle: u32,
    data_ptr: u32,
    data_len: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let memory = get_memory(&caller);

    let data = read_memory(caller.as_context_mut(), memory, data_ptr, data_len)?;

    get_runtime(&mut caller).write_substate(handle, data)
}

fn unlock_substate(
    mut caller: Caller<'_, HostState>,
    handle: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    get_runtime(&mut caller).unlock_substate(handle)
}

fn get_actor(mut caller: Caller<'_, HostState>) -> Result<u64, InvokeError<WasmRuntimeError>> {
    get_runtime(&mut caller).get_actor().map(|buffer| buffer.0)
}

fn consume_cost_units(
    mut caller: Caller<'_, HostState>,
    cost_unit: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    get_runtime(&mut caller).consume_cost_units(cost_unit)
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
    pub fn new(code: &[u8]) -> Result<Self, PrepareError> {
        let engine = Engine::default();
        let module = Module::new(&engine, code).expect("Failed to parse WASM module");
        let mut store = Store::new(&engine, WasmiInstanceEnv::new());

        let instance = Self::host_funcs_set(&module, &mut store)
            .map_err(|_| PrepareError::NotInstantiatable)?
            .ensure_no_start(store.as_context_mut())
            .map_err(|_| PrepareError::NotInstantiatable)?;

        Ok(Self {
            template_store: store,
            template_instance: instance,
            code_size_bytes: code.len(),
        })
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

        linker.instantiate(store.as_context_mut(), &module)
    }

    fn instantiate_template_instance(&self) -> WasmiInstance<'static, 'static> {
        let instance = self.template_instance.clone();
        let mut blank_store = self.template_store.clone();
        let memory = match instance.get_export(blank_store.as_context_mut(), EXPORT_MEMORY) {
            Some(Extern::Memory(memory)) => memory,
            _ => panic!("Failed to find memory export"),
        };

        WasmiInstance {
            instance,
            store: blank_store,
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

impl<'r, 'a: 'r> WasmiInstance<'r, 'a> {
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
                    .unwrap_or(&InvokeError::SelfError(
                        WasmRuntimeError::InvalidExportReturn,
                    ));
                invoke_err.clone()
            }
            _ => InvokeError::SelfError(WasmRuntimeError::InterpreterError(e_str)),
        }
    }
}

impl<'r, 'a: 'r> WasmInstance for WasmiInstance<'r, 'a> {
    fn invoke_export(
        &mut self,
        func_name: &str,
        args: Vec<Buffer>,
    ) -> Result<Vec<u8>, InvokeError<WasmRuntimeError>> {
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
            _ => Err(InvokeError::SelfError(
                WasmRuntimeError::InvalidExportReturn,
            )),
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

impl TemplateWasmInstance for WasmiInstance<'static, 'static> {
    type WasmInstance<'r, 'a: 'r> = WasmiInstance<'r, 'a>;

    fn install_runtime<'r, 'a: 'r>(
        self,
        runtime: &'r mut Box<dyn WasmRuntime + 'a>,
    ) -> Self::WasmInstance<'r, 'a> {
        WasmiInstance {
            store: clone_store_with_new_runtime_reference(self.store, runtime),
            instance: self.instance,
            memory: self.memory,
        }
    }
}

impl WasmEngine for WasmiEngine {
    type TemplateWasmInstance = WasmiInstance<'static, 'static>;

    fn instantiate_template_instance(
        &self,
        instrumented_code: &InstrumentedCode,
    ) -> Self::TemplateWasmInstance {
        let metered_code_key = &instrumented_code.metered_code_key;

        #[cfg(not(feature = "moka"))]
        {
            if let Some(cached_module) = self.modules_cache.borrow_mut().get(metered_code_key) {
                return cached_module.instantiate_template_instance();
            }
        }
        #[cfg(feature = "moka")]
        if let Some(cached_module) = self.modules_cache.get(metered_code_key) {
            return cached_module.as_ref().instantiate_template_instance();
        }

        let code = &instrumented_code.code.as_ref()[..];
        let module = WasmiModule::new(code).unwrap();
        let instance = module.instantiate_template_instance();

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
