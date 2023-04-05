use sbor::rust::mem::transmute;
use sbor::rust::mem::MaybeUninit;
use sbor::rust::sync::Arc;
use wasmi::core::Value;
use wasmi::core::{HostError, Trap};
use wasmi::*;

use super::InstrumentedCode;
use super::MeteredCodeKey;
use crate::errors::InvokeError;
use crate::types::*;
use crate::wasm::constants::*;
use crate::wasm::errors::*;
use crate::wasm::traits::*;

type FakeHostState = FakeWasmiInstanceEnv;
type HostState = WasmiInstanceEnv;

/// A `WasmiModule` defines a parsed WASM module "template" Instance (with imports already defined)
/// and Store, which keeps user data.
/// "Template" (Store, Instance) tuple are cached together, and never to be invoked.
/// Upon instantiation Instance and Store are cloned, so the state is not shared between instances.
/// It is safe to clone an `Instance` and a `Store`, since they don't use pointers, but `Arena`
/// allocator. `Instance` is owned by `Store`, it is basically some offset within `Store`'s vector
/// of `Instance`s. So after clone we receive the same `Store`, where we are able to set different
/// data, more specifically a `runtime_ptr`.
/// Also, it is correctly `Send + Sync` (under the assumption that the data in the Store is set to
/// a valid value upon invocation , because this is the thing which is cached in the
/// ScryptoInterpreter caches.
pub struct WasmiModule {
    template_store: Store<FakeHostState>,
    template_instance: Instance,
    #[allow(dead_code)]
    code_size_bytes: usize,
}

pub struct WasmiInstance {
    store: Store<HostState>,
    instance: Instance,
    memory: Memory,
}

/// This is to construct a stub `Store<FakeWasmiInstanceEnv>`, which is a part of
/// `WasmiModule` struct and serves as a placeholder for the real `Store<WasmiInstanceEnv>`.
/// The real store is set (prior being transumted) when the `WasmiModule` is being instantiated.
/// In fact the only difference between a stub and real Store is the `Send + Sync` manually
/// implemented for the former one, which is required by `WasmiModule` cache (for `std`
/// configuration) but shall not be implemented for the latter one to prevent sharing it between
/// the threads since pointer might point to volatile data.
#[derive(Clone)]
pub struct FakeWasmiInstanceEnv {
    #[allow(dead_code)]
    runtime_ptr: MaybeUninit<*mut Box<dyn WasmRuntime>>,
}

impl FakeWasmiInstanceEnv {
    pub fn new() -> Self {
        Self {
            runtime_ptr: MaybeUninit::uninit(),
        }
    }
}

unsafe impl Send for FakeWasmiInstanceEnv {}
unsafe impl Sync for FakeWasmiInstanceEnv {}

/// This is to construct a real `Store<WasmiInstanceEnv>
pub struct WasmiInstanceEnv {
    runtime_ptr: MaybeUninit<*mut Box<dyn WasmRuntime>>,
}

impl WasmiInstanceEnv {
    pub fn new() -> Self {
        Self {
            runtime_ptr: MaybeUninit::uninit(),
        }
    }
}

macro_rules! grab_runtime {
    ($caller: expr) => {{
        let runtime: &mut Box<dyn WasmRuntime> =
            unsafe { &mut *$caller.data().runtime_ptr.assume_init() };
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

fn call_method(
    mut caller: Caller<'_, HostState>,
    receiver_ptr: u32,
    receiver_len: u32,
    module_id: u32,
    ident_ptr: u32,
    ident_len: u32,
    args_ptr: u32,
    args_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let receiver = read_memory(caller.as_context_mut(), memory, receiver_ptr, receiver_len)?;
    let ident = read_memory(caller.as_context_mut(), memory, ident_ptr, ident_len)?;
    let args = read_memory(caller.as_context_mut(), memory, args_ptr, args_len)?;

    // Get current memory consumption and update it in transaction limit kernel module
    // for current call frame through runtime call.
    let mem = memory
        .current_pages(caller.as_context())
        .to_bytes()
        .ok_or(InvokeError::SelfError(WasmRuntimeError::MemoryAccessError))?;
    runtime.update_wasm_memory_usage(mem)?;

    runtime
        .call_method(receiver, module_id, ident, args)
        .map(|buffer| buffer.0)
}

fn call_function(
    mut caller: Caller<'_, HostState>,
    package_address_ptr: u32,
    package_address_len: u32,
    blueprint_ident_ptr: u32,
    blueprint_ident_len: u32,
    ident_ptr: u32,
    ident_len: u32,
    args_ptr: u32,
    args_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let package_address = read_memory(
        caller.as_context_mut(),
        memory,
        package_address_ptr,
        package_address_len,
    )?;
    let blueprint_ident = read_memory(
        caller.as_context_mut(),
        memory,
        blueprint_ident_ptr,
        blueprint_ident_len,
    )?;
    let ident = read_memory(caller.as_context_mut(), memory, ident_ptr, ident_len)?;
    let args = read_memory(caller.as_context_mut(), memory, args_ptr, args_len)?;

    // Get current memory consumption and update it in transaction limit kernel module
    // for current call frame through runtime call.
    let mem = memory
        .current_pages(caller.as_context())
        .to_bytes()
        .ok_or(InvokeError::SelfError(WasmRuntimeError::MemoryAccessError))?;
    runtime.update_wasm_memory_usage(mem)?;

    runtime
        .call_function(package_address, blueprint_ident, ident, args)
        .map(|buffer| buffer.0)
}

fn new_object(
    mut caller: Caller<'_, HostState>,
    blueprint_ident_ptr: u32,
    blueprint_ident_len: u32,
    object_states_ptr: u32,
    object_states_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    runtime
        .new_object(
            read_memory(
                caller.as_context_mut(),
                memory,
                blueprint_ident_ptr,
                blueprint_ident_len,
            )?,
            read_memory(
                caller.as_context_mut(),
                memory,
                object_states_ptr,
                object_states_len,
            )?,
        )
        .map(|buffer| buffer.0)
}

fn new_key_value_store(
    mut caller: Caller<'_, HostState>,
    schema_id_ptr: u32,
    schema_id_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    runtime
        .new_key_value_store(read_memory(
            caller.as_context_mut(),
            memory,
            schema_id_ptr,
            schema_id_len,
        )?)
        .map(|buffer| buffer.0)
}

fn globalize_object(
    mut caller: Caller<'_, HostState>,
    component_id_ptr: u32,
    component_id_len: u32,
    access_rules_ptr: u32,
    access_rules_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    runtime
        .globalize_object(
            read_memory(
                caller.as_context_mut(),
                memory,
                component_id_ptr,
                component_id_len,
            )?,
            read_memory(
                caller.as_context_mut(),
                memory,
                access_rules_ptr,
                access_rules_len,
            )?,
        )
        .map(|buffer| buffer.0)
}

fn get_object_info(
    mut caller: Caller<'_, HostState>,
    component_id_ptr: u32,
    component_id_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    runtime
        .get_object_info(read_memory(
            caller.as_context_mut(),
            memory,
            component_id_ptr,
            component_id_len,
        )?)
        .map(|buffer| buffer.0)
}

fn drop_object(
    mut caller: Caller<'_, HostState>,
    node_id_ptr: u32,
    node_id_len: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let node_id = read_memory(caller.as_context_mut(), memory, node_id_ptr, node_id_len)?;

    runtime.drop_object(node_id)
}

fn lock_substate(
    mut caller: Caller<'_, HostState>,
    node_id_ptr: u32,
    node_id_len: u32,
    offset_ptr: u32,
    offset_len: u32,
    flags: u32,
) -> Result<u32, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let node_id = read_memory(caller.as_context_mut(), memory, node_id_ptr, node_id_len)?;
    let substate_key = read_memory(caller.as_context_mut(), memory, offset_ptr, offset_len)?;

    runtime.lock_substate(node_id, substate_key, flags)
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

fn drop_lock(
    caller: Caller<'_, HostState>,
    handle: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let (_memory, runtime) = grab_runtime!(caller);

    runtime.drop_lock(handle)
}

fn get_global_address(caller: Caller<'_, HostState>) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (_memory, runtime) = grab_runtime!(caller);

    runtime.get_global_address().map(|buffer| buffer.0)
}

fn get_actor(caller: Caller<'_, HostState>) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (_memory, runtime) = grab_runtime!(caller);

    runtime.get_blueprint().map(|buffer| buffer.0)
}

fn get_auth_zone(caller: Caller<'_, HostState>) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (_memory, runtime) = grab_runtime!(caller);

    runtime.get_auth_zone().map(|buffer| buffer.0)
}

fn assert_access_rule(
    mut caller: Caller<'_, HostState>,
    data_ptr: u32,
    data_len: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let data = read_memory(caller.as_context_mut(), memory, data_ptr, data_len)?;

    runtime.assert_access_rule(data)
}

fn consume_cost_units(
    caller: Caller<'_, HostState>,
    cost_unit: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let (_memory, runtime) = grab_runtime!(caller);
    runtime.consume_cost_units(cost_unit)
}

fn emit_event(
    mut caller: Caller<'_, HostState>,
    event_name_ptr: u32,
    event_name_len: u32,
    event_data_ptr: u32,
    event_data_len: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let event_name = read_memory(
        caller.as_context_mut(),
        memory,
        event_name_ptr,
        event_name_len,
    )?;
    let event_data = read_memory(
        caller.as_context_mut(),
        memory,
        event_data_ptr,
        event_data_len,
    )?;

    runtime.emit_event(event_name, event_data)
}

fn get_transaction_hash(
    caller: Caller<'_, HostState>,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (_, runtime) = grab_runtime!(caller);

    runtime.get_transaction_hash().map(|buffer| buffer.0)
}

fn generate_uuid(caller: Caller<'_, HostState>) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (_, runtime) = grab_runtime!(caller);

    runtime.generate_uuid().map(|buffer| buffer.0)
}

fn log_message(
    mut caller: Caller<'_, HostState>,
    level_ptr: u32,
    level_len: u32,
    message_ptr: u32,
    message_len: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let level = read_memory(caller.as_context_mut(), memory, level_ptr, level_len)?;
    let message = read_memory(caller.as_context_mut(), memory, message_ptr, message_len)?;

    runtime.log_message(level, message)
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
        let module = Module::new(&engine, code).expect("WASM undecodable, prepare step missed?");
        let mut store = Store::new(&engine, WasmiInstanceEnv::new());

        let instance = Self::host_funcs_set(&module, &mut store)
            .map_err(|_| PrepareError::NotInstantiatable)?
            .ensure_no_start(store.as_context_mut())
            .expect("WASM contains start function, prepare step missed?");

        Ok(Self {
            template_store: unsafe { transmute(store) },
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

        let host_call_method = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             receiver_ptr: u32,
             receiver_len: u32,
             module_id: u32,
             ident_ptr: u32,
             ident_len: u32,
             args_ptr: u32,
             args_len: u32|
             -> Result<u64, Trap> {
                call_method(
                    caller,
                    receiver_ptr,
                    receiver_len,
                    module_id,
                    ident_ptr,
                    ident_len,
                    args_ptr,
                    args_len,
                )
                .map_err(|e| e.into())
            },
        );

        let host_call_function = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             package_address_ptr: u32,
             package_address_len: u32,
             blueprint_ident_ptr: u32,
             blueprint_ident_len: u32,
             ident_ptr: u32,
             ident_len: u32,
             args_ptr: u32,
             args_len: u32|
             -> Result<u64, Trap> {
                call_function(
                    caller,
                    package_address_ptr,
                    package_address_len,
                    blueprint_ident_ptr,
                    blueprint_ident_len,
                    ident_ptr,
                    ident_len,
                    args_ptr,
                    args_len,
                )
                .map_err(|e| e.into())
            },
        );

        let host_new_component = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             blueprint_ident_ptr: u32,
             blueprint_ident_len: u32,
             object_states_ptr: u32,
             object_states_len: u32|
             -> Result<u64, Trap> {
                new_object(
                    caller,
                    blueprint_ident_ptr,
                    blueprint_ident_len,
                    object_states_ptr,
                    object_states_len,
                )
                .map_err(|e| e.into())
            },
        );

        let host_new_key_value_store = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             schema_ptr: u32,
             schema_len: u32|
             -> Result<u64, Trap> {
                new_key_value_store(caller, schema_ptr, schema_len).map_err(|e| e.into())
            },
        );

        let host_globalize_object = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             component_id_ptr: u32,
             component_id_len: u32,
             access_rules_ptr: u32,
             access_rules_len: u32|
             -> Result<u64, Trap> {
                globalize_object(
                    caller,
                    component_id_ptr,
                    component_id_len,
                    access_rules_ptr,
                    access_rules_len,
                )
                .map_err(|e| e.into())
            },
        );

        let host_get_object_info = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             object_id_ptr: u32,
             object_id_len: u32|
             -> Result<u64, Trap> {
                get_object_info(caller, object_id_ptr, object_id_len).map_err(|e| e.into())
            },
        );

        let host_drop_node = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             node_id_ptr: u32,
             node_id_len: u32|
             -> Result<(), Trap> {
                drop_object(caller, node_id_ptr, node_id_len).map_err(|e| e.into())
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

        let host_drop_lock = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>, handle: u32| -> Result<(), Trap> {
                drop_lock(caller, handle).map_err(|e| e.into())
            },
        );

        let host_get_global_address = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>| -> Result<u64, Trap> {
                get_global_address(caller).map_err(|e| e.into())
            },
        );

        let host_get_blueprint = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>| -> Result<u64, Trap> {
                get_actor(caller).map_err(|e| e.into())
            },
        );

        let host_get_auth_zone = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>| -> Result<u64, Trap> {
                get_auth_zone(caller).map_err(|e| e.into())
            },
        );

        let host_assert_access_rule = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>, data_ptr: u32, data_len: u32| -> Result<(), Trap> {
                assert_access_rule(caller, data_ptr, data_len).map_err(|e| e.into())
            },
        );

        let host_consume_cost_units = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>, cost_unit: u32| -> Result<(), Trap> {
                consume_cost_units(caller, cost_unit).map_err(|e| e.into())
            },
        );

        let host_emit_event = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             event_name_ptr: u32,
             event_name_len: u32,
             event_data_ptr: u32,
             event_data_len: u32|
             -> Result<(), Trap> {
                emit_event(
                    caller,
                    event_name_ptr,
                    event_name_len,
                    event_data_ptr,
                    event_data_len,
                )
                .map_err(|e| e.into())
            },
        );

        let host_log = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             level_ptr: u32,
             level_len: u32,
             message_ptr: u32,
             message_len: u32|
             -> Result<(), Trap> {
                log_message(caller, level_ptr, level_len, message_ptr, message_len)
                    .map_err(|e| e.into())
            },
        );

        let host_get_transaction_hash = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>| -> Result<u64, Trap> {
                get_transaction_hash(caller).map_err(|e| e.into())
            },
        );

        let host_generate_uuid = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>| -> Result<u64, Trap> {
                generate_uuid(caller).map_err(|e| e.into())
            },
        );

        let mut linker = <Linker<HostState>>::new();
        linker_define!(linker, CONSUME_BUFFER_FUNCTION_NAME, host_consume_buffer);
        linker_define!(linker, CALL_METHOD_FUNCTION_NAME, host_call_method);
        linker_define!(linker, CALL_FUNCTION_FUNCTION_NAME, host_call_function);
        linker_define!(linker, NEW_OBJECT_FUNCTION_NAME, host_new_component);
        linker_define!(
            linker,
            NEW_KEY_VALUE_STORE_FUNCTION_NAME,
            host_new_key_value_store
        );
        linker_define!(
            linker,
            GLOBALIZE_OBJECT_FUNCTION_NAME,
            host_globalize_object
        );
        linker_define!(linker, GET_OBJECT_INFO_FUNCTION_NAME, host_get_object_info);
        linker_define!(linker, DROP_OBJECT_FUNCTION_NAME, host_drop_node);
        linker_define!(linker, LOCK_SUBSTATE_FUNCTION_NAME, host_lock_substate);
        linker_define!(linker, READ_SUBSTATE_FUNCTION_NAME, host_read_substate);
        linker_define!(linker, WRITE_SUBSTATE_FUNCTION_NAME, host_write_substate);
        linker_define!(linker, DROP_LOCK_FUNCTION_NAME, host_drop_lock);
        linker_define!(
            linker,
            GET_GLOBAL_ADDRESS_FUNCTION_NAME,
            host_get_global_address
        );
        linker_define!(linker, GET_BLUEPRINT_FUNCTION_NAME, host_get_blueprint);
        linker_define!(linker, GET_AUTH_ZONE_FUNCTION_NAME, host_get_auth_zone);
        linker_define!(
            linker,
            ASSERT_ACCESS_RULE_FUNCTION_NAME,
            host_assert_access_rule
        );
        linker_define!(
            linker,
            CONSUME_COST_UNITS_FUNCTION_NAME,
            host_consume_cost_units
        );
        linker_define!(linker, EMIT_EVENT_FUNCTION_NAME, host_emit_event);
        linker_define!(linker, LOG_FUNCTION_NAME, host_log);
        linker_define!(
            linker,
            GET_TRANSACTION_HASH_FUNCTION_NAME,
            host_get_transaction_hash
        );
        linker_define!(linker, GENERATE_UUID_FUNCTION_NAME, host_generate_uuid);

        linker.instantiate(store.as_context_mut(), &module)
    }

    fn instantiate(&self) -> WasmiInstance {
        let instance = self.template_instance.clone();
        let mut store = self.template_store.clone();
        let memory = match instance.get_export(store.as_context_mut(), EXPORT_MEMORY) {
            Some(Extern::Memory(memory)) => memory,
            _ => panic!("Failed to find memory export"),
        };

        WasmiInstance {
            instance,
            store: unsafe { transmute(store) },
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
                if let Some(invoke_err) = trap.downcast_ref::<InvokeError<WasmRuntimeError>>() {
                    invoke_err.clone()
                } else {
                    InvokeError::SelfError(WasmRuntimeError::Trap(format!("{:?}", trap)))
                }
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
            // FIXME: Triple casting to workaround this error message:
            // error[E0521]: borrowed data escapes outside of associated function
            //  `runtime` escapes the associated function body here argument requires that `'r` must outlive `'static`
            self.store
                .data_mut()
                .runtime_ptr
                .write(runtime as *mut _ as usize as *mut _);
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
            _ => Err(InvokeError::SelfError(WasmRuntimeError::InvalidWasmPointer)),
        }
    }

    fn consumed_memory(&self) -> Result<usize, InvokeError<WasmRuntimeError>> {
        self.memory
            .current_pages(self.store.as_context())
            .to_bytes()
            .ok_or(InvokeError::SelfError(WasmRuntimeError::MemoryAccessError))
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
        let module = WasmiModule::new(code).unwrap();
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
