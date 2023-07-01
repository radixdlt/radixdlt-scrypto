use sbor::rust::mem::transmute;
use sbor::rust::mem::MaybeUninit;
#[cfg(not(feature = "radix_engine_fuzzing"))]
use sbor::rust::sync::Arc;
use wasmi::core::Value;
use wasmi::core::{HostError, Trap};
use wasmi::errors::InstantiationError;
use wasmi::*;

use crate::errors::InvokeError;
use crate::types::*;
use crate::vm::wasm::constants::*;
use crate::vm::wasm::errors::*;
use crate::vm::wasm::traits::*;
use crate::vm::wasm::WasmEngine;

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

fn actor_call_module_method(
    mut caller: Caller<'_, HostState>,
    object_handle: u32,
    module_id: u32,
    ident_ptr: u32,
    ident_len: u32,
    args_ptr: u32,
    args_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let ident = read_memory(caller.as_context_mut(), memory, ident_ptr, ident_len)?;
    let args = read_memory(caller.as_context_mut(), memory, args_ptr, args_len)?;

    runtime
        .actor_call_module_method(object_handle, module_id, ident, args)
        .map(|buffer| buffer.0)
}

fn call_method(
    mut caller: Caller<'_, HostState>,
    receiver_ptr: u32,
    receiver_len: u32,
    direct_access: u32,
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

    runtime
        .call_method(receiver, direct_access, module_id, ident, args)
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
        .key_value_store_new(read_memory(
            caller.as_context_mut(),
            memory,
            schema_id_ptr,
            schema_id_len,
        )?)
        .map(|buffer| buffer.0)
}

fn allocate_global_address(
    mut caller: Caller<'_, HostState>,
    blueprint_id_ptr: u32,
    blueprint_id_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    runtime
        .allocate_global_address(read_memory(
            caller.as_context_mut(),
            memory,
            blueprint_id_ptr,
            blueprint_id_len,
        )?)
        .map(|buffer| buffer.0)
}

fn cost_unit_limit(caller: Caller<'_, HostState>) -> Result<u32, InvokeError<WasmRuntimeError>> {
    let (_memory, runtime) = grab_runtime!(caller);

    runtime.cost_unit_limit()
}

fn cost_unit_price(caller: Caller<'_, HostState>) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (_memory, runtime) = grab_runtime!(caller);

    runtime.cost_unit_price().map(|buffer| buffer.0)
}

fn tip_percentage(caller: Caller<'_, HostState>) -> Result<u32, InvokeError<WasmRuntimeError>> {
    let (_memory, runtime) = grab_runtime!(caller);

    runtime.tip_percentage()
}

fn fee_balance(caller: Caller<'_, HostState>) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (_memory, runtime) = grab_runtime!(caller);

    runtime.fee_balance().map(|buffer| buffer.0)
}

fn globalize_object(
    mut caller: Caller<'_, HostState>,
    modules_ptr: u32,
    modules_len: u32,
    address_ptr: u32,
    address_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    runtime
        .globalize_object(
            read_memory(caller.as_context_mut(), memory, modules_ptr, modules_len)?,
            read_memory(caller.as_context_mut(), memory, address_ptr, address_len)?,
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

fn lock_key_value_store_entry(
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

    runtime.key_value_store_open_entry(node_id, substate_key, flags)
}

fn key_value_entry_get(
    caller: Caller<'_, HostState>,
    handle: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (_memory, runtime) = grab_runtime!(caller);
    runtime.key_value_entry_get(handle).map(|buffer| buffer.0)
}

fn key_value_entry_set(
    mut caller: Caller<'_, HostState>,
    handle: u32,
    buffer_ptr: u32,
    buffer_len: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);
    let data = read_memory(caller.as_context_mut(), memory, buffer_ptr, buffer_len)?;
    runtime.key_value_entry_set(handle, data)
}

fn unlock_key_value_entry(
    caller: Caller<'_, HostState>,
    handle: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let (_memory, runtime) = grab_runtime!(caller);
    runtime.key_value_entry_release(handle)
}

fn key_value_entry_remove(
    mut caller: Caller<'_, HostState>,
    node_id_ptr: u32,
    node_id_len: u32,
    key_ptr: u32,
    key_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);
    let node_id = read_memory(caller.as_context_mut(), memory, node_id_ptr, node_id_len)?;
    let key = read_memory(caller.as_context_mut(), memory, key_ptr, key_len)?;

    runtime
        .key_value_store_remove_entry(node_id, key)
        .map(|buffer| buffer.0)
}

fn lock_field(
    caller: Caller<'_, HostState>,
    object_handle: u32,
    field: u32,
    flags: u32,
) -> Result<u32, InvokeError<WasmRuntimeError>> {
    let (_memory, runtime) = grab_runtime!(caller);
    runtime.actor_open_field(object_handle, field as u8, flags)
}

fn field_lock_read(
    caller: Caller<'_, HostState>,
    handle: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (_memory, runtime) = grab_runtime!(caller);

    runtime.field_lock_read(handle).map(|buffer| buffer.0)
}

fn field_lock_write(
    mut caller: Caller<'_, HostState>,
    handle: u32,
    data_ptr: u32,
    data_len: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let data = read_memory(caller.as_context_mut(), memory, data_ptr, data_len)?;

    runtime.field_lock_write(handle, data)
}

fn field_lock_release(
    caller: Caller<'_, HostState>,
    handle: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let (_memory, runtime) = grab_runtime!(caller);

    runtime.field_lock_release(handle)
}

fn get_node_id(caller: Caller<'_, HostState>) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (_memory, runtime) = grab_runtime!(caller);

    runtime.get_node_id().map(|buffer| buffer.0)
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

fn consume_wasm_execution_units(
    caller: Caller<'_, HostState>,
    n: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let (_memory, runtime) = grab_runtime!(caller);
    runtime.consume_wasm_execution_units(n)
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

fn generate_ruid(caller: Caller<'_, HostState>) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (_, runtime) = grab_runtime!(caller);

    runtime.generate_ruid().map(|buffer| buffer.0)
}

fn emit_log(
    mut caller: Caller<'_, HostState>,
    level_ptr: u32,
    level_len: u32,
    message_ptr: u32,
    message_len: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let level = read_memory(caller.as_context_mut(), memory, level_ptr, level_len)?;
    let message = read_memory(caller.as_context_mut(), memory, message_ptr, message_len)?;

    runtime.emit_log(level, message)
}

fn panic(
    mut caller: Caller<'_, HostState>,
    message_ptr: u32,
    message_len: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let message = read_memory(caller.as_context_mut(), memory, message_ptr, message_len)?;

    runtime.panic(message)
}
// native functions ends

macro_rules! linker_define {
    ($linker: expr, $name: expr, $var: expr) => {
        $linker
            .define(MODULE_ENV_NAME, $name, $var)
            .expect(stringify!("Failed to define new linker item {}", $name));
    };
}

#[derive(Debug)]
pub enum WasmiInstantiationError {
    ValidationError(Error),
    PreInstantiationError(Error),
    InstantiationError(InstantiationError),
}

impl WasmiModule {
    pub fn new(code: &[u8]) -> Result<Self, WasmiInstantiationError> {
        let engine = Engine::default();
        let mut store = Store::new(&engine, WasmiInstanceEnv::new());

        let module =
            Module::new(&engine, code).map_err(WasmiInstantiationError::ValidationError)?;

        let instance = Self::host_funcs_set(&module, &mut store)
            .map_err(WasmiInstantiationError::PreInstantiationError)?
            .ensure_no_start(store.as_context_mut())
            .map_err(WasmiInstantiationError::InstantiationError)?;

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

        let host_actor_call_module_method = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             object_handle: u32,
             module_id: u32,
             ident_ptr: u32,
             ident_len: u32,
             args_ptr: u32,
             args_len: u32|
             -> Result<u64, Trap> {
                actor_call_module_method(
                    caller,
                    object_handle,
                    module_id,
                    ident_ptr,
                    ident_len,
                    args_ptr,
                    args_len,
                )
                .map_err(|e| e.into())
            },
        );

        let host_call_method = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             receiver_ptr: u32,
             receiver_len: u32,
             direct_access: u32,
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
                    direct_access,
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

        let host_allocate_global_address = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             blueprint_id_ptr: u32,
             blueprint_id_len: u32|
             -> Result<u64, Trap> {
                allocate_global_address(caller, blueprint_id_ptr, blueprint_id_len)
                    .map_err(|e| e.into())
            },
        );

        let host_cost_unit_limit = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>| -> Result<u32, Trap> {
                cost_unit_limit(caller).map_err(|e| e.into())
            },
        );

        let host_cost_unit_price = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>| -> Result<u64, Trap> {
                cost_unit_price(caller).map_err(|e| e.into())
            },
        );

        let host_tip_percentage = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>| -> Result<u32, Trap> {
                tip_percentage(caller).map_err(|e| e.into())
            },
        );

        let host_fee_balance = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>| -> Result<u64, Trap> {
                fee_balance(caller).map_err(|e| e.into())
            },
        );

        let host_globalize_object = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             modules_ptr: u32,
             modules_len: u32,
             address_ptr: u32,
             address_len: u32|
             -> Result<u64, Trap> {
                globalize_object(caller, modules_ptr, modules_len, address_ptr, address_len)
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

        let host_lock_key_value_store_entry = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             node_id_ptr: u32,
             node_id_len: u32,
             offset_ptr: u32,
             offset_len: u32,
             mutable: u32|
             -> Result<u32, Trap> {
                lock_key_value_store_entry(
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

        let host_key_value_entry_get = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>, handle: u32| -> Result<u64, Trap> {
                key_value_entry_get(caller, handle).map_err(|e| e.into())
            },
        );

        let host_key_value_entry_set = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             handle: u32,
             buffer_ptr: u32,
             buffer_len: u32|
             -> Result<(), Trap> {
                key_value_entry_set(caller, handle, buffer_ptr, buffer_len).map_err(|e| e.into())
            },
        );

        let host_unlock_key_value_entry = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>, handle: u32| -> Result<(), Trap> {
                unlock_key_value_entry(caller, handle).map_err(|e| e.into())
            },
        );

        let host_key_value_entry_remove = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             node_id_ptr: u32,
             node_id_len: u32,
             key_ptr: u32,
             key_len: u32|
             -> Result<u64, Trap> {
                key_value_entry_remove(caller, node_id_ptr, node_id_len, key_ptr, key_len)
                    .map_err(|e| e.into())
            },
        );

        let host_lock_field = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             object_handle: u32,
             field: u32,
             lock_flags: u32|
             -> Result<u32, Trap> {
                lock_field(caller, object_handle, field, lock_flags).map_err(|e| e.into())
            },
        );

        let host_field_lock_read = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>, handle: u32| -> Result<u64, Trap> {
                field_lock_read(caller, handle).map_err(|e| e.into())
            },
        );

        let host_field_lock_write = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             handle: u32,
             data_ptr: u32,
             data_len: u32|
             -> Result<(), Trap> {
                field_lock_write(caller, handle, data_ptr, data_len).map_err(|e| e.into())
            },
        );

        let host_field_lock_release = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>, handle: u32| -> Result<(), Trap> {
                field_lock_release(caller, handle).map_err(|e| e.into())
            },
        );

        let host_get_node_id = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>| -> Result<u64, Trap> {
                get_node_id(caller).map_err(|e| e.into())
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

        let host_consume_wasm_execution_units = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>, n: u32| -> Result<(), Trap> {
                consume_wasm_execution_units(caller, n).map_err(|e| e.into())
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

        let host_emit_log = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             level_ptr: u32,
             level_len: u32,
             message_ptr: u32,
             message_len: u32|
             -> Result<(), Trap> {
                emit_log(caller, level_ptr, level_len, message_ptr, message_len)
                    .map_err(|e| e.into())
            },
        );

        let host_panic = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             message_ptr: u32,
             message_len: u32|
             -> Result<(), Trap> {
                panic(caller, message_ptr, message_len).map_err(|e| e.into())
            },
        );

        let host_get_transaction_hash = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>| -> Result<u64, Trap> {
                get_transaction_hash(caller).map_err(|e| e.into())
            },
        );

        let host_generate_ruid = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>| -> Result<u64, Trap> {
                generate_ruid(caller).map_err(|e| e.into())
            },
        );

        let mut linker = <Linker<HostState>>::new();

        linker_define!(linker, CONSUME_BUFFER_FUNCTION_NAME, host_consume_buffer);
        linker_define!(linker, CALL_METHOD_FUNCTION_NAME, host_call_method);
        linker_define!(linker, CALL_FUNCTION_FUNCTION_NAME, host_call_function);
        linker_define!(linker, NEW_OBJECT_FUNCTION_NAME, host_new_component);

        linker_define!(
            linker,
            ALLOCATE_GLOBAL_ADDRESS_FUNCTION_NAME,
            host_allocate_global_address
        );
        linker_define!(linker, COST_UNIT_LIMIT_FUNCTION_NAME, host_cost_unit_limit);
        linker_define!(linker, COST_UNIT_PRICE_FUNCTION_NAME, host_cost_unit_price);
        linker_define!(linker, TIP_PERCENTAGE_FUNCTION_NAME, host_tip_percentage);
        linker_define!(linker, FEE_BALANCE_FUNCTION_NAME, host_fee_balance);
        linker_define!(linker, GLOBALIZE_FUNCTION_NAME, host_globalize_object);
        linker_define!(linker, GET_OBJECT_INFO_FUNCTION_NAME, host_get_object_info);
        linker_define!(linker, DROP_OBJECT_FUNCTION_NAME, host_drop_node);
        linker_define!(linker, ACTOR_OPEN_FIELD_FUNCTION_NAME, host_lock_field);
        linker_define!(
            linker,
            ACTOR_CALL_MODULE_METHOD_FUNCTION_NAME,
            host_actor_call_module_method
        );

        linker_define!(
            linker,
            KEY_VALUE_STORE_NEW_FUNCTION_NAME,
            host_new_key_value_store
        );
        linker_define!(
            linker,
            KEY_VALUE_STORE_OPEN_ENTRY_FUNCTION_NAME,
            host_lock_key_value_store_entry
        );
        linker_define!(
            linker,
            KEY_VALUE_ENTRY_GET_FUNCTION_NAME,
            host_key_value_entry_get
        );
        linker_define!(
            linker,
            KEY_VALUE_ENTRY_SET_FUNCTION_NAME,
            host_key_value_entry_set
        );
        linker_define!(
            linker,
            KEY_VALUE_ENTRY_RELEASE_FUNCTION_NAME,
            host_unlock_key_value_entry
        );
        linker_define!(
            linker,
            KEY_VALUE_STORE_REMOVE_ENTRY_FUNCTION_NAME,
            host_key_value_entry_remove
        );

        linker_define!(linker, FIELD_LOCK_READ_FUNCTION_NAME, host_field_lock_read);
        linker_define!(
            linker,
            FIELD_LOCK_WRITE_FUNCTION_NAME,
            host_field_lock_write
        );
        linker_define!(
            linker,
            FIELD_LOCK_RELEASE_FUNCTION_NAME,
            host_field_lock_release
        );
        linker_define!(linker, GET_NODE_ID_FUNCTION_NAME, host_get_node_id);
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
            CONSUME_WASM_EXECUTION_UNITS_FUNCTION_NAME,
            host_consume_wasm_execution_units
        );
        linker_define!(linker, EMIT_EVENT_FUNCTION_NAME, host_emit_event);
        linker_define!(linker, EMIT_LOG_FUNCTION_NAME, host_emit_log);
        linker_define!(linker, PANIC_FUNCTION_NAME, host_panic);
        linker_define!(
            linker,
            GET_TRANSACTION_HASH_FUNCTION_NAME,
            host_get_transaction_hash
        );
        linker_define!(linker, GENERATE_RUID_FUNCTION_NAME, host_generate_ruid);

        let global_value = Global::new(store.as_context_mut(), Value::I32(-1), Mutability::Var);
        linker_define!(linker, "test_global_mutable_value", global_value);

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
                InvokeError::SelfError(WasmRuntimeError::UnknownExport(name.to_string()))
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
                    InvokeError::SelfError(WasmRuntimeError::ExecutionError(e_str))
                }
            }
            _ => InvokeError::SelfError(WasmRuntimeError::ExecutionError(e_str)),
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
            // Using triple casting is to workaround this error message:
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
pub struct WasmiEngineOptions {
    max_cache_size: usize,
}

pub struct WasmiEngine {
    // This flag disables cache in wasm_instrumenter/wasmi/wasmer to prevent non-determinism when fuzzing
    #[cfg(all(not(feature = "radix_engine_fuzzing"), not(feature = "moka")))]
    modules_cache: RefCell<lru::LruCache<Hash, Arc<WasmiModule>>>,
    #[cfg(all(not(feature = "radix_engine_fuzzing"), feature = "moka"))]
    modules_cache: moka::sync::Cache<Hash, Arc<WasmiModule>>,
    #[cfg(feature = "radix_engine_fuzzing")]
    #[allow(dead_code)]
    modules_cache: usize,
}

impl Default for WasmiEngine {
    fn default() -> Self {
        Self::new(WasmiEngineOptions {
            max_cache_size: DEFAULT_WASM_ENGINE_CACHE_SIZE,
        })
    }
}

impl WasmiEngine {
    pub fn new(options: WasmiEngineOptions) -> Self {
        #[cfg(all(not(feature = "radix_engine_fuzzing"), not(feature = "moka")))]
        let modules_cache = RefCell::new(lru::LruCache::new(
            NonZeroUsize::new(options.max_cache_size).unwrap(),
        ));
        #[cfg(all(not(feature = "radix_engine_fuzzing"), feature = "moka"))]
        let modules_cache = moka::sync::Cache::builder()
            .weigher(|_key: &Hash, _value: &Arc<WasmiModule>| -> u32 {
                // No sophisticated weighing mechanism, just keep a fixed size cache
                1u32
            })
            .max_capacity(options.max_cache_size as u64)
            .build();
        #[cfg(feature = "radix_engine_fuzzing")]
        let modules_cache = options.max_cache_size;

        Self { modules_cache }
    }
}

impl WasmEngine for WasmiEngine {
    type WasmInstance = WasmiInstance;

    #[allow(unused_variables)]
    fn instantiate(&self, code_hash: Hash, instrumented_code: &[u8]) -> WasmiInstance {
        #[cfg(not(feature = "radix_engine_fuzzing"))]
        {
            #[cfg(not(feature = "moka"))]
            {
                if let Some(cached_module) = self.modules_cache.borrow_mut().get(&code_hash) {
                    return cached_module.instantiate();
                }
            }
            #[cfg(feature = "moka")]
            if let Some(cached_module) = self.modules_cache.get(&code_hash) {
                return cached_module.as_ref().instantiate();
            }
        }

        let module = WasmiModule::new(instrumented_code).expect("Failed to instantiate module");
        let instance = module.instantiate();

        #[cfg(not(feature = "radix_engine_fuzzing"))]
        {
            #[cfg(not(feature = "moka"))]
            self.modules_cache
                .borrow_mut()
                .put(code_hash, Arc::new(module));
            #[cfg(feature = "moka")]
            self.modules_cache.insert(code_hash, Arc::new(module));
        }

        instance
    }
}

// Below tests verify WASM "mutable-global" feature, which allows importing/exporting mutable globals.
// more details:
// - https://github.com/WebAssembly/mutable-global/blob/master/proposals/mutable-global/Overview.md

// NOTE!
//  We test only WASM code, because Rust currently does not use the WASM "global" construct for globals
//  (it places them into the linear memory instead).
//  more details:
//  - https://github.com/rust-lang/rust/issues/60825
//  - https://github.com/rust-lang/rust/issues/65987
#[cfg(not(feature = "wasmer"))]
#[cfg(test)]
mod tests {
    use super::*;
    use wabt::{wat2wasm, wat2wasm_with_features, ErrorKind, Features};

    static MODULE_MUTABLE_GLOBALS: &str = r#"
            (module
                ;; below line is invalid if feature 'Import/Export mutable globals' is disabled
                ;; see: https://github.com/WebAssembly/mutable-global/blob/master/proposals/mutable-global/Overview.md
                (global $g (import "env" "global_mutable_value") (mut i32))

                ;; Simple function that always returns `0`
                (func $increase_global_value (param $step i32) (result i32)

                    (global.set $g
                        (i32.add
                            (global.get $g)
                            (local.get $step)))

                    (i32.const 0)
                )
                (memory $0 1)
                (export "memory" (memory $0))
                (export "increase_global_value" (func $increase_global_value))
            )
        "#;

    // This test is not wasmi-specific, but decided to put it here along with next one
    #[test]
    fn test_wasm_non_mvp_mutable_globals_build_with_feature_disabled() {
        let mut features = Features::new();
        features.disable_mutable_globals();

        assert!(
            match wat2wasm_with_features(MODULE_MUTABLE_GLOBALS, features) {
                Err(err) => {
                    match err.kind() {
                        ErrorKind::Validate(msg) => {
                            msg.contains("mutable globals cannot be imported")
                        }
                        _ => false,
                    }
                }
                Ok(_) => false,
            }
        )
    }
    pub fn run_module_with_mutable_global(
        engine: &Engine,
        mut store: StoreContextMut<WasmiInstanceEnv>,
        code: &[u8],
        func_name: &str,
        global_name: &str,
        global_value: &Global,
        step: i32,
    ) {
        let module = Module::new(&engine, code).unwrap();

        let mut linker = <Linker<HostState>>::new();
        linker_define!(linker, global_name, *global_value);

        let instance = linker
            .instantiate(store.as_context_mut(), &module)
            .unwrap()
            .ensure_no_start(store.as_context_mut())
            .unwrap();

        let func = instance
            .get_export(store.as_context_mut(), func_name)
            .and_then(Extern::into_func)
            .unwrap();

        let input = [Value::I32(step)];
        let mut ret = [Value::I32(0)];

        let _ = func.call(store.as_context_mut(), &input, &mut ret);
    }

    #[test]
    fn test_wasm_non_mvp_mutable_globals_execute_code() {
        // wat2wasm has "mutable-globals" enabled by default
        let code = wat2wasm(MODULE_MUTABLE_GLOBALS).unwrap();

        let engine = Engine::default();
        let mut store = Store::new(&engine, WasmiInstanceEnv::new());

        // Value of this Global shall be updated by the below WASM module calls
        let global_value = Global::new(store.as_context_mut(), Value::I32(100), Mutability::Var);

        run_module_with_mutable_global(
            &engine,
            store.as_context_mut(),
            &code,
            "increase_global_value",
            "global_mutable_value",
            &global_value,
            1000,
        );
        let updated_value = global_value.get(store.as_context());
        let val = i32::try_from(updated_value).unwrap();
        assert_eq!(val, 1100);

        run_module_with_mutable_global(
            &engine,
            store.as_context_mut(),
            &code,
            "increase_global_value",
            "global_mutable_value",
            &global_value,
            10000,
        );
        let updated_value = global_value.get(store.as_context());
        let val = i32::try_from(updated_value).unwrap();
        assert_eq!(val, 11100);
    }
}
