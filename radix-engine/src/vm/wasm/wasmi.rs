use crate::errors::InvokeError;
use crate::internal_prelude::*;
#[cfg(feature = "coverage")]
use crate::utils::save_coverage_data;
use crate::vm::wasm::constants::*;
use crate::vm::wasm::errors::*;
use crate::vm::wasm::traits::*;
use crate::vm::wasm::WasmEngine;
use radix_engine_interface::api::actor_api::EventFlags;
use radix_engine_interface::blueprints::package::CodeHash;
use sbor::rust::mem::MaybeUninit;
#[cfg(not(feature = "fuzzing"))]
use sbor::rust::sync::Arc;
use wasmi::core::HostError;
use wasmi::errors::InstantiationError;
use wasmi::*;

type HostState = WasmiInstanceEnv;

/// A `WasmiModule` defines a compiled WASM module
pub struct WasmiModule {
    module: Module,
    #[allow(dead_code)]
    code_size_bytes: usize,
}

/// A `WasmiModule` defines
/// - an instantiated WASM module
/// - a Store , which keeps user data
/// - a Memory - linear memory reference to the Store
pub struct WasmiInstance {
    store: Store<HostState>,
    instance: Instance,
    memory: Memory,
}

/// This is to construct a `Store<WasmiInstanceEnv>`
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
        runtime
    }};
}
macro_rules! grab_memory {
    ($caller: expr) => {{
        match $caller.get_export(EXPORT_MEMORY) {
            Some(Extern::Memory(memory)) => memory,
            _ => panic!("Failed to find memory export"),
        }
    }};
}

// native functions start
fn consume_buffer(
    caller: Caller<'_, HostState>,
    buffer_id: BufferId,
    destination_ptr: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);
    let memory = grab_memory!(caller);

    let result = runtime.buffer_consume(buffer_id);
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
    ident_ptr: u32,
    ident_len: u32,
    args_ptr: u32,
    args_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);
    let memory = grab_memory!(caller);

    let receiver = read_memory(caller.as_context_mut(), memory, receiver_ptr, receiver_len)?;
    let ident = read_memory(caller.as_context_mut(), memory, ident_ptr, ident_len)?;
    let args = read_memory(caller.as_context_mut(), memory, args_ptr, args_len)?;

    runtime
        .object_call(receiver, ident, args)
        .map(|buffer| buffer.0)
}

fn call_direct_method(
    mut caller: Caller<'_, HostState>,
    receiver_ptr: u32,
    receiver_len: u32,
    ident_ptr: u32,
    ident_len: u32,
    args_ptr: u32,
    args_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);
    let memory = grab_memory!(caller);

    let receiver = read_memory(caller.as_context_mut(), memory, receiver_ptr, receiver_len)?;
    let ident = read_memory(caller.as_context_mut(), memory, ident_ptr, ident_len)?;
    let args = read_memory(caller.as_context_mut(), memory, args_ptr, args_len)?;

    runtime
        .object_call_direct(receiver, ident, args)
        .map(|buffer| buffer.0)
}

fn call_module_method(
    mut caller: Caller<'_, HostState>,
    receiver_ptr: u32,
    receiver_len: u32,
    module_id: u32,
    ident_ptr: u32,
    ident_len: u32,
    args_ptr: u32,
    args_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);
    let memory = grab_memory!(caller);

    let receiver = read_memory(caller.as_context_mut(), memory, receiver_ptr, receiver_len)?;
    let ident = read_memory(caller.as_context_mut(), memory, ident_ptr, ident_len)?;
    let args = read_memory(caller.as_context_mut(), memory, args_ptr, args_len)?;

    runtime
        .object_call_module(receiver, module_id, ident, args)
        .map(|buffer| buffer.0)
}

fn call_function(
    mut caller: Caller<'_, HostState>,
    package_address_ptr: u32,
    package_address_len: u32,
    blueprint_name_ptr: u32,
    blueprint_name_len: u32,
    ident_ptr: u32,
    ident_len: u32,
    args_ptr: u32,
    args_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);
    let memory = grab_memory!(caller);

    let package_address = read_memory(
        caller.as_context_mut(),
        memory,
        package_address_ptr,
        package_address_len,
    )?;
    let blueprint_name = read_memory(
        caller.as_context_mut(),
        memory,
        blueprint_name_ptr,
        blueprint_name_len,
    )?;
    let ident = read_memory(caller.as_context_mut(), memory, ident_ptr, ident_len)?;
    let args = read_memory(caller.as_context_mut(), memory, args_ptr, args_len)?;

    runtime
        .blueprint_call(package_address, blueprint_name, ident, args)
        .map(|buffer| buffer.0)
}

fn new_object(
    mut caller: Caller<'_, HostState>,
    blueprint_name_ptr: u32,
    blueprint_name_len: u32,
    object_states_ptr: u32,
    object_states_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);
    let memory = grab_memory!(caller);

    runtime
        .object_new(
            read_memory(
                caller.as_context_mut(),
                memory,
                blueprint_name_ptr,
                blueprint_name_len,
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
    let runtime = grab_runtime!(caller);
    let memory = grab_memory!(caller);

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
    package_address_ptr: u32,
    package_address_len: u32,
    blueprint_name_ptr: u32,
    blueprint_name_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);
    let memory = grab_memory!(caller);

    runtime
        .address_allocate(
            read_memory(
                caller.as_context_mut(),
                memory,
                package_address_ptr,
                package_address_len,
            )?,
            read_memory(
                caller.as_context_mut(),
                memory,
                blueprint_name_ptr,
                blueprint_name_len,
            )?,
        )
        .map(|buffer| buffer.0)
}

fn get_reservation_address(
    mut caller: Caller<'_, HostState>,
    node_id_ptr: u32,
    node_id_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);
    let memory = grab_memory!(caller);

    runtime
        .address_get_reservation_address(read_memory(
            caller.as_context_mut(),
            memory,
            node_id_ptr,
            node_id_len,
        )?)
        .map(|buffer| buffer.0)
}

fn execution_cost_unit_limit(
    caller: Caller<'_, HostState>,
) -> Result<u32, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);

    runtime.costing_get_execution_cost_unit_limit()
}

fn execution_cost_unit_price(
    caller: Caller<'_, HostState>,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);

    runtime
        .costing_get_execution_cost_unit_price()
        .map(|buffer| buffer.0)
}

fn finalization_cost_unit_limit(
    caller: Caller<'_, HostState>,
) -> Result<u32, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);

    runtime.costing_get_finalization_cost_unit_limit()
}

fn finalization_cost_unit_price(
    caller: Caller<'_, HostState>,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);

    runtime
        .costing_get_finalization_cost_unit_price()
        .map(|buffer| buffer.0)
}

fn usd_price(caller: Caller<'_, HostState>) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);

    runtime.costing_get_usd_price().map(|buffer| buffer.0)
}

fn tip_percentage(caller: Caller<'_, HostState>) -> Result<u32, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);

    runtime.costing_get_tip_percentage()
}

fn fee_balance(caller: Caller<'_, HostState>) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);

    runtime.costing_get_fee_balance().map(|buffer| buffer.0)
}

fn globalize_object(
    mut caller: Caller<'_, HostState>,
    obj_id_ptr: u32,
    obj_id_len: u32,
    modules_ptr: u32,
    modules_len: u32,
    address_ptr: u32,
    address_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);
    let memory = grab_memory!(caller);

    runtime
        .globalize_object(
            read_memory(caller.as_context_mut(), memory, obj_id_ptr, obj_id_len)?,
            read_memory(caller.as_context_mut(), memory, modules_ptr, modules_len)?,
            read_memory(caller.as_context_mut(), memory, address_ptr, address_len)?,
        )
        .map(|buffer| buffer.0)
}

fn instance_of(
    mut caller: Caller<'_, HostState>,
    component_id_ptr: u32,
    component_id_len: u32,
    package_address_ptr: u32,
    package_address_len: u32,
    blueprint_name_ptr: u32,
    blueprint_name_len: u32,
) -> Result<u32, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);
    let memory = grab_memory!(caller);

    runtime.instance_of(
        read_memory(
            caller.as_context_mut(),
            memory,
            component_id_ptr,
            component_id_len,
        )?,
        read_memory(
            caller.as_context_mut(),
            memory,
            package_address_ptr,
            package_address_len,
        )?,
        read_memory(
            caller.as_context_mut(),
            memory,
            blueprint_name_ptr,
            blueprint_name_len,
        )?,
    )
}

fn blueprint_id(
    mut caller: Caller<'_, HostState>,
    component_id_ptr: u32,
    component_id_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);
    let memory = grab_memory!(caller);

    runtime
        .blueprint_id(read_memory(
            caller.as_context_mut(),
            memory,
            component_id_ptr,
            component_id_len,
        )?)
        .map(|buffer| buffer.0)
}

fn get_outer_object(
    mut caller: Caller<'_, HostState>,
    component_id_ptr: u32,
    component_id_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);
    let memory = grab_memory!(caller);

    runtime
        .get_outer_object(read_memory(
            caller.as_context_mut(),
            memory,
            component_id_ptr,
            component_id_len,
        )?)
        .map(|buffer| buffer.0)
}

fn lock_key_value_store_entry(
    mut caller: Caller<'_, HostState>,
    node_id_ptr: u32,
    node_id_len: u32,
    offset_ptr: u32,
    offset_len: u32,
    flags: u32,
) -> Result<u32, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);
    let memory = grab_memory!(caller);

    let node_id = read_memory(caller.as_context_mut(), memory, node_id_ptr, node_id_len)?;
    let substate_key = read_memory(caller.as_context_mut(), memory, offset_ptr, offset_len)?;

    runtime.key_value_store_open_entry(node_id, substate_key, flags)
}

fn key_value_entry_get(
    caller: Caller<'_, HostState>,
    handle: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);

    runtime.key_value_entry_get(handle).map(|buffer| buffer.0)
}

fn key_value_entry_set(
    mut caller: Caller<'_, HostState>,
    handle: u32,
    buffer_ptr: u32,
    buffer_len: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);
    let memory = grab_memory!(caller);
    let data = read_memory(caller.as_context_mut(), memory, buffer_ptr, buffer_len)?;
    runtime.key_value_entry_set(handle, data)
}

fn key_value_entry_remove(
    caller: Caller<'_, HostState>,
    handle: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);

    runtime
        .key_value_entry_remove(handle)
        .map(|buffer| buffer.0)
}

fn unlock_key_value_entry(
    caller: Caller<'_, HostState>,
    handle: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);

    runtime.key_value_entry_close(handle)
}

fn key_value_store_remove(
    mut caller: Caller<'_, HostState>,
    node_id_ptr: u32,
    node_id_len: u32,
    key_ptr: u32,
    key_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);
    let memory = grab_memory!(caller);
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
    let runtime = grab_runtime!(caller);

    runtime.actor_open_field(object_handle, field as u8, flags)
}

fn field_lock_read(
    caller: Caller<'_, HostState>,
    handle: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);

    runtime.field_entry_read(handle).map(|buffer| buffer.0)
}

fn field_lock_write(
    mut caller: Caller<'_, HostState>,
    handle: u32,
    data_ptr: u32,
    data_len: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);
    let memory = grab_memory!(caller);

    let data = read_memory(caller.as_context_mut(), memory, data_ptr, data_len)?;

    runtime.field_entry_write(handle, data)
}

fn field_lock_release(
    caller: Caller<'_, HostState>,
    handle: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);

    runtime.field_entry_close(handle)
}

fn actor_get_node_id(
    caller: Caller<'_, HostState>,
    handle: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);

    runtime.actor_get_node_id(handle).map(|buffer| buffer.0)
}

fn get_package_address(
    caller: Caller<'_, HostState>,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);

    runtime.actor_get_package_address().map(|buffer| buffer.0)
}

fn get_blueprint_name(caller: Caller<'_, HostState>) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);

    runtime.actor_get_blueprint_name().map(|buffer| buffer.0)
}

#[inline]
fn consume_wasm_execution_units(
    caller: Caller<'_, HostState>,
    n: u64,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let runtime: &mut Box<dyn WasmRuntime> =
        unsafe { &mut *caller.data().runtime_ptr.assume_init() };

    // TODO: wasm-instrument uses u64 for cost units. We need to decide if we want to move from u32
    // to u64 as well.
    runtime.consume_wasm_execution_units(n as u32)
}

fn emit_event(
    mut caller: Caller<'_, HostState>,
    event_name_ptr: u32,
    event_name_len: u32,
    event_data_ptr: u32,
    event_data_len: u32,
    flags: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);
    let memory = grab_memory!(caller);

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
    let event_flags = EventFlags::from_bits(flags).ok_or(InvokeError::SelfError(
        WasmRuntimeError::InvalidEventFlags(flags),
    ))?;

    runtime.actor_emit_event(event_name, event_data, event_flags)
}

fn get_transaction_hash(
    caller: Caller<'_, HostState>,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);

    runtime.sys_get_transaction_hash().map(|buffer| buffer.0)
}

fn generate_ruid(caller: Caller<'_, HostState>) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);

    runtime.sys_generate_ruid().map(|buffer| buffer.0)
}

fn emit_log(
    mut caller: Caller<'_, HostState>,
    level_ptr: u32,
    level_len: u32,
    message_ptr: u32,
    message_len: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);
    let memory = grab_memory!(caller);

    let level = read_memory(caller.as_context_mut(), memory, level_ptr, level_len)?;
    let message = read_memory(caller.as_context_mut(), memory, message_ptr, message_len)?;

    runtime.sys_log(level, message)
}

fn bech32_encode_address(
    mut caller: Caller<'_, HostState>,
    address_ptr: u32,
    address_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);
    let memory = grab_memory!(caller);

    let address = read_memory(caller.as_context_mut(), memory, address_ptr, address_len)?;

    runtime
        .sys_bech32_encode_address(address)
        .map(|buffer| buffer.0)
}

fn panic(
    mut caller: Caller<'_, HostState>,
    message_ptr: u32,
    message_len: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);
    let memory = grab_memory!(caller);

    let message = read_memory(caller.as_context_mut(), memory, message_ptr, message_len)?;

    runtime.sys_panic(message)
}

fn bls12381_v1_verify(
    mut caller: Caller<'_, HostState>,
    message_ptr: u32,
    message_len: u32,
    public_key_ptr: u32,
    public_key_len: u32,
    signature_ptr: u32,
    signature_len: u32,
) -> Result<u32, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);
    let memory = grab_memory!(caller);

    let message = read_memory(caller.as_context_mut(), memory, message_ptr, message_len)?;
    let public_key = read_memory(
        caller.as_context_mut(),
        memory,
        public_key_ptr,
        public_key_len,
    )?;
    let signature = read_memory(
        caller.as_context_mut(),
        memory,
        signature_ptr,
        signature_len,
    )?;

    runtime.crypto_utils_bls12381_v1_verify(message, public_key, signature)
}

fn bls12381_v1_aggregate_verify(
    mut caller: Caller<'_, HostState>,
    pub_keys_and_msgs_ptr: u32,
    pub_keys_and_msgs_len: u32,
    signature_ptr: u32,
    signature_len: u32,
) -> Result<u32, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);
    let memory = grab_memory!(caller);

    let pub_keys_and_msgs = read_memory(
        caller.as_context_mut(),
        memory,
        pub_keys_and_msgs_ptr,
        pub_keys_and_msgs_len,
    )?;
    let signature = read_memory(
        caller.as_context_mut(),
        memory,
        signature_ptr,
        signature_len,
    )?;

    runtime.crypto_utils_bls12381_v1_aggregate_verify(pub_keys_and_msgs, signature)
}

fn bls12381_v1_fast_aggregate_verify(
    mut caller: Caller<'_, HostState>,
    message_ptr: u32,
    message_len: u32,
    public_keys_ptr: u32,
    public_keys_len: u32,
    signature_ptr: u32,
    signature_len: u32,
) -> Result<u32, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);
    let memory = grab_memory!(caller);

    let message = read_memory(caller.as_context_mut(), memory, message_ptr, message_len)?;
    let public_keys = read_memory(
        caller.as_context_mut(),
        memory,
        public_keys_ptr,
        public_keys_len,
    )?;
    let signature = read_memory(
        caller.as_context_mut(),
        memory,
        signature_ptr,
        signature_len,
    )?;

    runtime.crypto_utils_bls12381_v1_fast_aggregate_verify(message, public_keys, signature)
}

fn bls12381_g2_signature_aggregate(
    mut caller: Caller<'_, HostState>,
    signatures_ptr: u32,
    signatures_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);
    let memory = grab_memory!(caller);

    let signatures = read_memory(
        caller.as_context_mut(),
        memory,
        signatures_ptr,
        signatures_len,
    )?;

    runtime
        .crypto_utils_bls12381_g2_signature_aggregate(signatures)
        .map(|buffer| buffer.0)
}

fn keccak256_hash(
    mut caller: Caller<'_, HostState>,
    data_ptr: u32,
    data_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);
    let memory = grab_memory!(caller);

    let data = read_memory(caller.as_context_mut(), memory, data_ptr, data_len)?;

    runtime
        .crypto_utils_keccak256_hash(data)
        .map(|buffer| buffer.0)
}

fn blake2b_256_hash(
    mut caller: Caller<'_, HostState>,
    data_ptr: u32,
    data_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);
    let memory = grab_memory!(caller);

    let data = read_memory(caller.as_context_mut(), memory, data_ptr, data_len)?;

    runtime
        .crypto_utils_blake2b_256_hash(data)
        .map(|buffer| buffer.0)
}

fn ed25519_verify(
    mut caller: Caller<'_, HostState>,
    message_ptr: u32,
    message_len: u32,
    public_key_ptr: u32,
    public_key_len: u32,
    signature_ptr: u32,
    signature_len: u32,
) -> Result<u32, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);
    let memory = grab_memory!(caller);

    let message = read_memory(caller.as_context_mut(), memory, message_ptr, message_len)?;
    let public_key = read_memory(
        caller.as_context_mut(),
        memory,
        public_key_ptr,
        public_key_len,
    )?;
    let signature = read_memory(
        caller.as_context_mut(),
        memory,
        signature_ptr,
        signature_len,
    )?;

    runtime.crypto_utils_ed25519_verify(message, public_key, signature)
}

fn secp256k1_ecdsa_verify(
    mut caller: Caller<'_, HostState>,
    message_ptr: u32,
    message_len: u32,
    public_key_ptr: u32,
    public_key_len: u32,
    signature_ptr: u32,
    signature_len: u32,
) -> Result<u32, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);
    let memory = grab_memory!(caller);

    let message = read_memory(caller.as_context_mut(), memory, message_ptr, message_len)?;
    let public_key = read_memory(
        caller.as_context_mut(),
        memory,
        public_key_ptr,
        public_key_len,
    )?;
    let signature = read_memory(
        caller.as_context_mut(),
        memory,
        signature_ptr,
        signature_len,
    )?;

    runtime.crypto_utils_secp256k1_ecdsa_verify(message, public_key, signature)
}

fn secp256k1_ecdsa_verify_and_key_recover(
    mut caller: Caller<'_, HostState>,
    message_ptr: u32,
    message_len: u32,
    signature_ptr: u32,
    signature_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);
    let memory = grab_memory!(caller);

    let message = read_memory(caller.as_context_mut(), memory, message_ptr, message_len)?;
    let signature = read_memory(
        caller.as_context_mut(),
        memory,
        signature_ptr,
        signature_len,
    )?;

    runtime
        .crypto_utils_secp256k1_ecdsa_verify_and_key_recover(message, signature)
        .map(|buffer| buffer.0)
}

fn secp256k1_ecdsa_verify_and_key_recover_uncompressed(
    mut caller: Caller<'_, HostState>,
    message_ptr: u32,
    message_len: u32,
    signature_ptr: u32,
    signature_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let runtime = grab_runtime!(caller);
    let memory = grab_memory!(caller);

    let message = read_memory(caller.as_context_mut(), memory, message_ptr, message_len)?;
    let signature = read_memory(
        caller.as_context_mut(),
        memory,
        signature_ptr,
        signature_len,
    )?;

    runtime
        .crypto_utils_secp256k1_ecdsa_verify_and_key_recover_uncompressed(message, signature)
        .map(|buffer| buffer.0)
}

#[cfg(feature = "radix_engine_tests")]
fn test_host_read_memory(
    mut caller: Caller<'_, HostState>,
    memory_offs: u32,
    data_len: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    // - attempt to read data of given length data starting from given memory offset memory_ptr
    let memory = grab_memory!(caller);

    read_memory(caller.as_context_mut(), memory, memory_offs, data_len)?;

    Ok(())
}

#[cfg(feature = "radix_engine_tests")]
fn test_host_write_memory(
    mut caller: Caller<'_, HostState>,
    memory_ptr: u32,
    data_len: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    // - generate some random data of of given length data_len
    // - attempt to write this data into given memory offset memory_ptr
    let memory = grab_memory!(caller);

    let data = vec![0u8; data_len as usize];
    write_memory(caller.as_context_mut(), memory, memory_ptr, &data)?;

    Ok(())
}

#[cfg(feature = "radix_engine_tests")]
fn test_host_check_memory_is_clean(
    caller: Caller<'_, HostState>,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    // - attempt to read data of given length data starting from given memory offset memory_ptr
    let memory = grab_memory!(caller);
    let store_ctx = caller.as_context();

    let data = memory.data(&store_ctx);
    let clean = !data.iter().any(|&x| x != 0x0);

    Ok(clean as u64)
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
    CompilationError(Error),
    PreInstantiationError(Error),
    InstantiationError(InstantiationError),
}

impl WasmiModule {
    pub fn new(code: &[u8]) -> Result<Self, WasmiInstantiationError> {
        let mut config = wasmi::Config::default();

        // In order to speed compilation we deliberately
        // - use LazyTranslation compilation mode
        // - compiling without WASM validation (Module::new_checked())
        // (for more details see: https://github.com/wasmi-labs/wasmi/releases/tag/v0.32.0)
        //
        // It is assumed that WASM code passed here is already WASM validated,
        // so above combination should be fine.
        config.compilation_mode(wasmi::CompilationMode::LazyTranslation);
        let engine = Engine::new(&config);

        let module = unsafe {
            Module::new_unchecked(&engine, code)
                .map_err(WasmiInstantiationError::CompilationError)?
        };

        Ok(Self {
            module,
            code_size_bytes: code.len(),
        })
    }

    fn host_funcs_set(module: &Module, store: &mut Store<HostState>) -> Result<InstancePre, Error> {
        let host_consume_buffer = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             buffer_id: BufferId,
             destination_ptr: u32|
             -> Result<(), Error> {
                consume_buffer(caller, buffer_id, destination_ptr).map_err(|e| Error::host(e))
            },
        );

        let host_call_method = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             receiver_ptr: u32,
             receiver_len: u32,
             ident_ptr: u32,
             ident_len: u32,
             args_ptr: u32,
             args_len: u32|
             -> Result<u64, Error> {
                call_method(
                    caller,
                    receiver_ptr,
                    receiver_len,
                    ident_ptr,
                    ident_len,
                    args_ptr,
                    args_len,
                )
                .map_err(|e| Error::host(e))
            },
        );

        let host_call_module_method = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             receiver_ptr: u32,
             receiver_len: u32,
             module_id: u32,
             ident_ptr: u32,
             ident_len: u32,
             args_ptr: u32,
             args_len: u32|
             -> Result<u64, Error> {
                call_module_method(
                    caller,
                    receiver_ptr,
                    receiver_len,
                    module_id,
                    ident_ptr,
                    ident_len,
                    args_ptr,
                    args_len,
                )
                .map_err(|e| Error::host(e))
            },
        );

        let host_call_direct_method = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             receiver_ptr: u32,
             receiver_len: u32,
             ident_ptr: u32,
             ident_len: u32,
             args_ptr: u32,
             args_len: u32|
             -> Result<u64, Error> {
                call_direct_method(
                    caller,
                    receiver_ptr,
                    receiver_len,
                    ident_ptr,
                    ident_len,
                    args_ptr,
                    args_len,
                )
                .map_err(|e| Error::host(e))
            },
        );

        let host_blueprint_call = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             package_address_ptr: u32,
             package_address_len: u32,
             blueprint_name_ptr: u32,
             blueprint_name_len: u32,
             ident_ptr: u32,
             ident_len: u32,
             args_ptr: u32,
             args_len: u32|
             -> Result<u64, Error> {
                call_function(
                    caller,
                    package_address_ptr,
                    package_address_len,
                    blueprint_name_ptr,
                    blueprint_name_len,
                    ident_ptr,
                    ident_len,
                    args_ptr,
                    args_len,
                )
                .map_err(|e| Error::host(e))
            },
        );

        let host_new_component = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             blueprint_name_ptr: u32,
             blueprint_name_len: u32,
             object_states_ptr: u32,
             object_states_len: u32|
             -> Result<u64, Error> {
                new_object(
                    caller,
                    blueprint_name_ptr,
                    blueprint_name_len,
                    object_states_ptr,
                    object_states_len,
                )
                .map_err(|e| Error::host(e))
            },
        );

        let host_new_key_value_store = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             schema_ptr: u32,
             schema_len: u32|
             -> Result<u64, Error> {
                new_key_value_store(caller, schema_ptr, schema_len).map_err(|e| Error::host(e))
            },
        );

        let host_allocate_global_address = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             package_address_ptr: u32,
             package_address_len: u32,
             blueprint_name_ptr: u32,
             blueprint_name_len: u32|
             -> Result<u64, Error> {
                allocate_global_address(
                    caller,
                    package_address_ptr,
                    package_address_len,
                    blueprint_name_ptr,
                    blueprint_name_len,
                )
                .map_err(|e| Error::host(e))
            },
        );

        let host_get_reservation_address = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             node_id_ptr: u32,
             node_id_len: u32|
             -> Result<u64, Error> {
                get_reservation_address(caller, node_id_ptr, node_id_len)
                    .map_err(|e| Error::host(e))
            },
        );

        let host_execution_cost_unit_limit = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>| -> Result<u32, Error> {
                execution_cost_unit_limit(caller).map_err(|e| Error::host(e))
            },
        );

        let host_execution_cost_unit_price = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>| -> Result<u64, Error> {
                execution_cost_unit_price(caller).map_err(|e| Error::host(e))
            },
        );

        let host_finalization_cost_unit_limit = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>| -> Result<u32, Error> {
                finalization_cost_unit_limit(caller).map_err(|e| Error::host(e))
            },
        );

        let host_finalization_cost_unit_price = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>| -> Result<u64, Error> {
                finalization_cost_unit_price(caller).map_err(|e| Error::host(e))
            },
        );

        let host_usd_price = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>| -> Result<u64, Error> {
                usd_price(caller).map_err(|e| Error::host(e))
            },
        );

        let host_tip_percentage = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>| -> Result<u32, Error> {
                tip_percentage(caller).map_err(|e| Error::host(e))
            },
        );

        let host_fee_balance = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>| -> Result<u64, Error> {
                fee_balance(caller).map_err(|e| Error::host(e))
            },
        );

        let host_globalize_object = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             obj_ptr: u32,
             obj_len: u32,
             modules_ptr: u32,
             modules_len: u32,
             address_ptr: u32,
             address_len: u32|
             -> Result<u64, Error> {
                globalize_object(
                    caller,
                    obj_ptr,
                    obj_len,
                    modules_ptr,
                    modules_len,
                    address_ptr,
                    address_len,
                )
                .map_err(|e| Error::host(e))
            },
        );

        let host_instance_of = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             object_id_ptr: u32,
             object_id_len: u32,
             package_address_ptr: u32,
             package_address_len: u32,
             blueprint_name_ptr: u32,
             blueprint_name_len: u32|
             -> Result<u32, Error> {
                instance_of(
                    caller,
                    object_id_ptr,
                    object_id_len,
                    package_address_ptr,
                    package_address_len,
                    blueprint_name_ptr,
                    blueprint_name_len,
                )
                .map_err(|e| Error::host(e))
            },
        );

        let host_get_blueprint_id = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             object_id_ptr: u32,
             object_id_len: u32|
             -> Result<u64, Error> {
                blueprint_id(caller, object_id_ptr, object_id_len).map_err(|e| Error::host(e))
            },
        );

        let host_get_outer_object = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             object_id_ptr: u32,
             object_id_len: u32|
             -> Result<u64, Error> {
                get_outer_object(caller, object_id_ptr, object_id_len).map_err(|e| Error::host(e))
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
             -> Result<u32, Error> {
                lock_key_value_store_entry(
                    caller,
                    node_id_ptr,
                    node_id_len,
                    offset_ptr,
                    offset_len,
                    mutable,
                )
                .map_err(|e| Error::host(e))
            },
        );

        let host_key_value_entry_get = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>, handle: u32| -> Result<u64, Error> {
                key_value_entry_get(caller, handle).map_err(|e| Error::host(e))
            },
        );

        let host_key_value_entry_set = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             handle: u32,
             buffer_ptr: u32,
             buffer_len: u32|
             -> Result<(), Error> {
                key_value_entry_set(caller, handle, buffer_ptr, buffer_len)
                    .map_err(|e| Error::host(e))
            },
        );

        let host_key_value_entry_remove = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>, handle: u32| -> Result<u64, Error> {
                key_value_entry_remove(caller, handle).map_err(|e| Error::host(e))
            },
        );

        let host_unlock_key_value_entry = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>, handle: u32| -> Result<(), Error> {
                unlock_key_value_entry(caller, handle).map_err(|e| Error::host(e))
            },
        );

        let host_key_value_store_remove = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             node_id_ptr: u32,
             node_id_len: u32,
             key_ptr: u32,
             key_len: u32|
             -> Result<u64, Error> {
                key_value_store_remove(caller, node_id_ptr, node_id_len, key_ptr, key_len)
                    .map_err(|e| Error::host(e))
            },
        );

        let host_lock_field = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             object_handle: u32,
             field: u32,
             lock_flags: u32|
             -> Result<u32, Error> {
                lock_field(caller, object_handle, field, lock_flags).map_err(|e| Error::host(e))
            },
        );

        let host_field_lock_read = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>, handle: u32| -> Result<u64, Error> {
                field_lock_read(caller, handle).map_err(|e| Error::host(e))
            },
        );

        let host_field_lock_write = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             handle: u32,
             data_ptr: u32,
             data_len: u32|
             -> Result<(), Error> {
                field_lock_write(caller, handle, data_ptr, data_len).map_err(|e| Error::host(e))
            },
        );

        let host_field_lock_release = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>, handle: u32| -> Result<(), Error> {
                field_lock_release(caller, handle).map_err(|e| Error::host(e))
            },
        );

        let host_actor_get_node_id = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>, handle: u32| -> Result<u64, Error> {
                actor_get_node_id(caller, handle).map_err(|e| Error::host(e))
            },
        );

        let host_get_package_address = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>| -> Result<u64, Error> {
                get_package_address(caller).map_err(|e| Error::host(e))
            },
        );

        let host_get_blueprint_name = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>| -> Result<u64, Error> {
                get_blueprint_name(caller).map_err(|e| Error::host(e))
            },
        );

        let host_consume_wasm_execution_units = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>, n: u64| -> Result<(), Error> {
                consume_wasm_execution_units(caller, n).map_err(|e| Error::host(e))
            },
        );

        let host_emit_event = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             event_name_ptr: u32,
             event_name_len: u32,
             event_data_ptr: u32,
             event_data_len: u32,
             flags: u32|
             -> Result<(), Error> {
                emit_event(
                    caller,
                    event_name_ptr,
                    event_name_len,
                    event_data_ptr,
                    event_data_len,
                    flags,
                )
                .map_err(|e| Error::host(e))
            },
        );

        let host_emit_log = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             level_ptr: u32,
             level_len: u32,
             message_ptr: u32,
             message_len: u32|
             -> Result<(), Error> {
                emit_log(caller, level_ptr, level_len, message_ptr, message_len)
                    .map_err(|e| Error::host(e))
            },
        );

        let host_panic = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             message_ptr: u32,
             message_len: u32|
             -> Result<(), Error> {
                panic(caller, message_ptr, message_len).map_err(|e| Error::host(e))
            },
        );

        let host_bech32_encode_address = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             address_ptr: u32,
             address_len: u32|
             -> Result<u64, Error> {
                bech32_encode_address(caller, address_ptr, address_len).map_err(|e| Error::host(e))
            },
        );

        let host_get_transaction_hash = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>| -> Result<u64, Error> {
                get_transaction_hash(caller).map_err(|e| Error::host(e))
            },
        );

        let host_generate_ruid = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>| -> Result<u64, Error> {
                generate_ruid(caller).map_err(|e| Error::host(e))
            },
        );

        let host_bls12381_v1_verify = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             message_ptr: u32,
             message_len: u32,
             public_key_ptr: u32,
             public_key_len: u32,
             signature_ptr: u32,
             signature_len: u32|
             -> Result<u32, Error> {
                bls12381_v1_verify(
                    caller,
                    message_ptr,
                    message_len,
                    public_key_ptr,
                    public_key_len,
                    signature_ptr,
                    signature_len,
                )
                .map_err(|e| Error::host(e))
            },
        );

        let host_bls12381_v1_aggregate_verify = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             pub_keys_and_msgs_ptr: u32,
             pub_keys_and_msgs_len: u32,
             signature_ptr: u32,
             signature_len: u32|
             -> Result<u32, Error> {
                bls12381_v1_aggregate_verify(
                    caller,
                    pub_keys_and_msgs_ptr,
                    pub_keys_and_msgs_len,
                    signature_ptr,
                    signature_len,
                )
                .map_err(|e| Error::host(e))
            },
        );

        let host_bls12381_v1_fast_aggregate_verify = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             message_ptr: u32,
             message_len: u32,
             public_keys_ptr: u32,
             public_keys_len: u32,
             signature_ptr: u32,
             signature_len: u32|
             -> Result<u32, Error> {
                bls12381_v1_fast_aggregate_verify(
                    caller,
                    message_ptr,
                    message_len,
                    public_keys_ptr,
                    public_keys_len,
                    signature_ptr,
                    signature_len,
                )
                .map_err(|e| Error::host(e))
            },
        );

        let host_bls12381_g2_signature_aggregate = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             signatures_ptr: u32,
             signatures_len: u32|
             -> Result<u64, Error> {
                bls12381_g2_signature_aggregate(caller, signatures_ptr, signatures_len)
                    .map_err(|e| Error::host(e))
            },
        );

        let host_keccak256_hash = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>, data_ptr: u32, data_len: u32| -> Result<u64, Error> {
                keccak256_hash(caller, data_ptr, data_len).map_err(|e| Error::host(e))
            },
        );

        let host_blake2b_256_hash = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>, data_ptr: u32, data_len: u32| -> Result<u64, Error> {
                blake2b_256_hash(caller, data_ptr, data_len).map_err(|e| Error::host(e))
            },
        );

        let host_ed25519_verify = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             message_ptr: u32,
             message_len: u32,
             public_key_ptr: u32,
             public_key_len: u32,
             signature_ptr: u32,
             signature_len: u32|
             -> Result<u32, Error> {
                ed25519_verify(
                    caller,
                    message_ptr,
                    message_len,
                    public_key_ptr,
                    public_key_len,
                    signature_ptr,
                    signature_len,
                )
                .map_err(|e| Error::host(e))
            },
        );
        let host_secp2561k1_ecdsa_verify = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             message_ptr: u32,
             message_len: u32,
             public_key_ptr: u32,
             public_key_len: u32,
             signature_ptr: u32,
             signature_len: u32|
             -> Result<u32, Error> {
                secp256k1_ecdsa_verify(
                    caller,
                    message_ptr,
                    message_len,
                    public_key_ptr,
                    public_key_len,
                    signature_ptr,
                    signature_len,
                )
                .map_err(|e| Error::host(e))
            },
        );
        let host_secp2561k1_ecdsa_verify_and_key_recover = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             message_ptr: u32,
             message_len: u32,
             signature_ptr: u32,
             signature_len: u32|
             -> Result<u64, Error> {
                secp256k1_ecdsa_verify_and_key_recover(
                    caller,
                    message_ptr,
                    message_len,
                    signature_ptr,
                    signature_len,
                )
                .map_err(|e| Error::host(e))
            },
        );
        let host_secp2561k1_ecdsa_verify_and_key_recover_uncompressed = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             message_ptr: u32,
             message_len: u32,
             signature_ptr: u32,
             signature_len: u32|
             -> Result<u64, Error> {
                secp256k1_ecdsa_verify_and_key_recover_uncompressed(
                    caller,
                    message_ptr,
                    message_len,
                    signature_ptr,
                    signature_len,
                )
                .map_err(|e| Error::host(e))
            },
        );

        let mut linker = <Linker<HostState>>::new(module.engine());

        linker_define!(linker, BUFFER_CONSUME_FUNCTION_NAME, host_consume_buffer);
        linker_define!(linker, OBJECT_CALL_FUNCTION_NAME, host_call_method);
        linker_define!(
            linker,
            OBJECT_CALL_MODULE_FUNCTION_NAME,
            host_call_module_method
        );
        linker_define!(
            linker,
            OBJECT_CALL_DIRECT_FUNCTION_NAME,
            host_call_direct_method
        );
        linker_define!(linker, BLUEPRINT_CALL_FUNCTION_NAME, host_blueprint_call);
        linker_define!(linker, OBJECT_NEW_FUNCTION_NAME, host_new_component);

        linker_define!(
            linker,
            ADDRESS_ALLOCATE_FUNCTION_NAME,
            host_allocate_global_address
        );
        linker_define!(
            linker,
            ADDRESS_GET_RESERVATION_ADDRESS_FUNCTION_NAME,
            host_get_reservation_address
        );
        linker_define!(
            linker,
            COSTING_GET_EXECUTION_COST_UNIT_LIMIT_FUNCTION_NAME,
            host_execution_cost_unit_limit
        );
        linker_define!(
            linker,
            COSTING_GET_EXECUTION_COST_UNIT_PRICE_FUNCTION_NAME,
            host_execution_cost_unit_price
        );
        linker_define!(
            linker,
            COSTING_GET_FINALIZATION_COST_UNIT_LIMIT_FUNCTION_NAME,
            host_finalization_cost_unit_limit
        );
        linker_define!(
            linker,
            COSTING_GET_FINALIZATION_COST_UNIT_PRICE_FUNCTION_NAME,
            host_finalization_cost_unit_price
        );
        linker_define!(linker, COSTING_GET_USD_PRICE_FUNCTION_NAME, host_usd_price);
        linker_define!(
            linker,
            COSTING_GET_TIP_PERCENTAGE_FUNCTION_NAME,
            host_tip_percentage
        );
        linker_define!(
            linker,
            COSTING_GET_FEE_BALANCE_FUNCTION_NAME,
            host_fee_balance
        );
        linker_define!(
            linker,
            OBJECT_GLOBALIZE_FUNCTION_NAME,
            host_globalize_object
        );
        linker_define!(linker, OBJECT_INSTANCE_OF_FUNCTION_NAME, host_instance_of);
        linker_define!(
            linker,
            OBJECT_GET_BLUEPRINT_ID_FUNCTION_NAME,
            host_get_blueprint_id
        );
        linker_define!(
            linker,
            OBJECT_GET_OUTER_OBJECT_FUNCTION_NAME,
            host_get_outer_object
        );
        linker_define!(linker, ACTOR_OPEN_FIELD_FUNCTION_NAME, host_lock_field);

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
            KEY_VALUE_ENTRY_READ_FUNCTION_NAME,
            host_key_value_entry_get
        );
        linker_define!(
            linker,
            KEY_VALUE_ENTRY_WRITE_FUNCTION_NAME,
            host_key_value_entry_set
        );
        linker_define!(
            linker,
            KEY_VALUE_ENTRY_REMOVE_FUNCTION_NAME,
            host_key_value_entry_remove
        );
        linker_define!(
            linker,
            KEY_VALUE_ENTRY_CLOSE_FUNCTION_NAME,
            host_unlock_key_value_entry
        );
        linker_define!(
            linker,
            KEY_VALUE_STORE_REMOVE_ENTRY_FUNCTION_NAME,
            host_key_value_store_remove
        );

        linker_define!(linker, FIELD_ENTRY_READ_FUNCTION_NAME, host_field_lock_read);
        linker_define!(
            linker,
            FIELD_ENTRY_WRITE_FUNCTION_NAME,
            host_field_lock_write
        );
        linker_define!(
            linker,
            FIELD_ENTRY_CLOSE_FUNCTION_NAME,
            host_field_lock_release
        );
        linker_define!(
            linker,
            ACTOR_GET_OBJECT_ID_FUNCTION_NAME,
            host_actor_get_node_id
        );
        linker_define!(
            linker,
            ACTOR_GET_PACKAGE_ADDRESS_FUNCTION_NAME,
            host_get_package_address
        );
        linker_define!(
            linker,
            ACTOR_GET_BLUEPRINT_NAME_FUNCTION_NAME,
            host_get_blueprint_name
        );
        linker_define!(
            linker,
            COSTING_CONSUME_WASM_EXECUTION_UNITS_FUNCTION_NAME,
            host_consume_wasm_execution_units
        );
        linker_define!(linker, ACTOR_EMIT_EVENT_FUNCTION_NAME, host_emit_event);
        linker_define!(linker, SYS_LOG_FUNCTION_NAME, host_emit_log);
        linker_define!(linker, SYS_PANIC_FUNCTION_NAME, host_panic);
        linker_define!(
            linker,
            SYS_GET_TRANSACTION_HASH_FUNCTION_NAME,
            host_get_transaction_hash
        );
        linker_define!(
            linker,
            SYS_BECH32_ENCODE_ADDRESS_FUNCTION_NAME,
            host_bech32_encode_address
        );
        linker_define!(linker, SYS_GENERATE_RUID_FUNCTION_NAME, host_generate_ruid);
        linker_define!(
            linker,
            CRYPTO_UTILS_BLS12381_V1_VERIFY_FUNCTION_NAME,
            host_bls12381_v1_verify
        );
        linker_define!(
            linker,
            CRYPTO_UTILS_BLS12381_V1_AGGREGATE_VERIFY_FUNCTION_NAME,
            host_bls12381_v1_aggregate_verify
        );
        linker_define!(
            linker,
            CRYPTO_UTILS_BLS12381_V1_FAST_AGGREGATE_VERIFY_FUNCTION_NAME,
            host_bls12381_v1_fast_aggregate_verify
        );
        linker_define!(
            linker,
            CRYPTO_UTILS_BLS12381_G2_SIGNATURE_AGGREGATE_FUNCTION_NAME,
            host_bls12381_g2_signature_aggregate
        );
        linker_define!(
            linker,
            CRYPTO_UTILS_KECCAK256_HASH_FUNCTION_NAME,
            host_keccak256_hash
        );
        linker_define!(
            linker,
            CRYPTO_UTILS_BLAKE2B_256_HASH_FUNCTION_NAME,
            host_blake2b_256_hash
        );
        linker_define!(
            linker,
            CRYPTO_UTILS_ED25519_VERIFY_FUNCTION_NAME,
            host_ed25519_verify
        );
        linker_define!(
            linker,
            CRYPTO_UTILS_SECP256K1_ECDSA_VERIFY_FUNCTION_NAME,
            host_secp2561k1_ecdsa_verify
        );
        linker_define!(
            linker,
            CRYPTO_UTILS_SECP256K1_ECDSA_VERIFY_AND_KEY_RECOVER_FUNCTION_NAME,
            host_secp2561k1_ecdsa_verify_and_key_recover
        );
        linker_define!(
            linker,
            CRYPTO_UTILS_SECP256K1_ECDSA_VERIFY_AND_KEY_RECOVER_UNCOMPRESSED_FUNCTION_NAME,
            host_secp2561k1_ecdsa_verify_and_key_recover_uncompressed
        );

        #[cfg(feature = "radix_engine_tests")]
        {
            let host_read_memory = Func::wrap(
                store.as_context_mut(),
                |caller: Caller<'_, HostState>,
                 memory_offs: u32,
                 data_len: u32|
                 -> Result<(), Error> {
                    test_host_read_memory(caller, memory_offs, data_len).map_err(|e| Error::host(e))
                },
            );
            let host_write_memory = Func::wrap(
                store.as_context_mut(),
                |caller: Caller<'_, HostState>,
                 memory_offs: u32,
                 data_len: u32|
                 -> Result<(), Error> {
                    test_host_write_memory(caller, memory_offs, data_len)
                        .map_err(|e| Error::host(e))
                },
            );
            let host_check_memory_is_clean = Func::wrap(
                store.as_context_mut(),
                |caller: Caller<'_, HostState>| -> Result<u64, Error> {
                    test_host_check_memory_is_clean(caller).map_err(|e| Error::host(e))
                },
            );
            linker_define!(linker, "test_host_read_memory", host_read_memory);
            linker_define!(linker, "test_host_write_memory", host_write_memory);
            linker_define!(
                linker,
                "test_host_check_memory_is_clean",
                host_check_memory_is_clean
            );
        }

        linker.instantiate(store.as_context_mut(), &module)
    }

    pub fn instantiate(&self) -> Result<WasmiInstance, WasmiInstantiationError> {
        let mut store = Store::new(self.module.engine(), WasmiInstanceEnv::new());

        let instance = Self::host_funcs_set(&self.module, &mut store)
            .map_err(WasmiInstantiationError::PreInstantiationError)?
            .ensure_no_start(store.as_context_mut())
            .map_err(WasmiInstantiationError::InstantiationError)?;

        let memory = match instance.get_export(store.as_context_mut(), EXPORT_MEMORY) {
            Some(Extern::Memory(memory)) => memory,
            _ => panic!("Failed to find memory export"),
        };

        Ok(WasmiInstance {
            instance,
            store,
            memory,
        })
    }

    fn instantiate_unchecked(&self) -> WasmiInstance {
        self.instantiate().expect("Failed to instantiate")
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
        if let Some(invoke_err) = err.downcast::<InvokeError<WasmRuntimeError>>() {
            invoke_err.clone()
        } else {
            InvokeError::SelfError(WasmRuntimeError::ExecutionError(e_str))
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
        let input: Vec<Val> = args
            .into_iter()
            .map(|buffer| Val::I64(buffer.as_i64()))
            .collect();
        let mut ret = [Val::I64(0)];

        let call_result = func
            .call(self.store.as_context_mut(), &input, &mut ret)
            .map_err(|e| {
                let err: InvokeError<WasmRuntimeError> = e.into();
                err
            });

        let result = match call_result {
            Ok(_) => match ret[0] {
                Val::I64(ret) => read_slice(
                    self.store.as_context_mut(),
                    self.memory,
                    Slice::transmute_i64(ret),
                ),
                _ => Err(InvokeError::SelfError(WasmRuntimeError::InvalidWasmPointer)),
            },
            Err(err) => Err(err),
        };

        #[cfg(feature = "coverage")]
        if let Ok(dump_coverage) = self.get_export_func("dump_coverage") {
            if let Ok(blueprint_buffer) = runtime.actor_get_blueprint_name() {
                let blueprint_name =
                    String::from_utf8(runtime.buffer_consume(blueprint_buffer.id()).unwrap())
                        .unwrap();

                let mut ret = [Val::I64(0)];
                dump_coverage
                    .call(self.store.as_context_mut(), &[], &mut ret)
                    .unwrap();
                let coverage_data = match ret[0] {
                    Val::I64(ret) => read_slice(
                        self.store.as_context_mut(),
                        self.memory,
                        Slice::transmute_i64(ret),
                    ),
                    _ => Err(InvokeError::SelfError(WasmRuntimeError::InvalidWasmPointer)),
                }
                .unwrap();
                save_coverage_data(&blueprint_name, &coverage_data);
            }
        }

        result
    }
}

#[derive(Debug, Clone)]
pub struct WasmiEngineOptions {
    max_cache_size: usize,
}

pub struct WasmiEngine {
    // This flag disables cache in wasm_instrumenter/wasmi to prevent non-determinism when fuzzing
    #[cfg(all(not(feature = "fuzzing"), not(feature = "moka")))]
    modules_cache: RefCell<lru::LruCache<CodeHash, Arc<WasmiModule>>>,
    #[cfg(all(not(feature = "fuzzing"), feature = "moka"))]
    modules_cache: moka::sync::Cache<CodeHash, Arc<WasmiModule>>,
    #[cfg(feature = "fuzzing")]
    #[allow(dead_code)]
    modules_cache: usize,
}

impl Default for WasmiEngine {
    fn default() -> Self {
        Self::new(WasmiEngineOptions {
            max_cache_size: WASM_ENGINE_CACHE_SIZE,
        })
    }
}

impl WasmiEngine {
    pub fn new(options: WasmiEngineOptions) -> Self {
        #[cfg(all(not(feature = "fuzzing"), not(feature = "moka")))]
        let modules_cache = RefCell::new(lru::LruCache::new(
            sbor::rust::num::NonZeroUsize::new(options.max_cache_size).unwrap(),
        ));
        #[cfg(all(not(feature = "fuzzing"), feature = "moka"))]
        let modules_cache = moka::sync::Cache::builder()
            .weigher(|_key: &CodeHash, _value: &Arc<WasmiModule>| -> u32 {
                // No sophisticated weighing mechanism, just keep a fixed size cache
                1u32
            })
            .max_capacity(options.max_cache_size as u64)
            .build();
        #[cfg(feature = "fuzzing")]
        let modules_cache = options.max_cache_size;

        Self { modules_cache }
    }
}

impl WasmEngine for WasmiEngine {
    type WasmInstance = WasmiInstance;

    #[allow(unused_variables)]
    fn instantiate(&self, code_hash: CodeHash, instrumented_code: &[u8]) -> WasmiInstance {
        #[cfg(not(feature = "fuzzing"))]
        {
            #[cfg(not(feature = "moka"))]
            {
                if let Some(cached_module) = self.modules_cache.borrow_mut().get(&code_hash) {
                    return cached_module.instantiate_unchecked();
                }
            }
            #[cfg(feature = "moka")]
            if let Some(cached_module) = self.modules_cache.get(&code_hash) {
                return cached_module.as_ref().instantiate_unchecked();
            }
        }

        let module = WasmiModule::new(instrumented_code).expect("Failed to compile module");
        let instance = module.instantiate_unchecked();

        #[cfg(not(feature = "fuzzing"))]
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
#[cfg(test)]
mod tests {
    use super::*;
    use wabt::{wat2wasm, wat2wasm_with_features, ErrorKind, Features};
    use wasmi::Global;

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
        module: &Module,
        mut store: StoreContextMut<WasmiInstanceEnv>,
        func_name: &str,
        global_name: &str,
        global_value: &Global,
        step: i32,
    ) {
        let mut linker = <Linker<HostState>>::new(module.engine());
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

        let input = [Val::I32(step)];
        let mut ret = [Val::I32(0)];

        let _ = func.call(store.as_context_mut(), &input, &mut ret);
    }

    #[test]
    fn test_wasm_non_mvp_mutable_globals_execute_code() {
        // wat2wasm has "mutable-globals" enabled by default
        let code = wat2wasm(MODULE_MUTABLE_GLOBALS).unwrap();

        let wasmi_module = WasmiModule::new(&code).unwrap();
        let module = wasmi_module.module;

        let mut store = Store::new(&module.engine(), WasmiInstanceEnv::new());

        // Value of this Global shall be updated by the below WASM module calls
        let global_value = Global::new(store.as_context_mut(), Val::I32(100), Mutability::Var);

        run_module_with_mutable_global(
            &module,
            store.as_context_mut(),
            "increase_global_value",
            "global_mutable_value",
            &global_value,
            1000,
        );
        let updated_value = global_value.get(store.as_context());
        let val = match updated_value {
            Val::I32(val) => val,
            _ => panic!("Unexpected return value type"),
        };
        assert_eq!(val, 1100);

        run_module_with_mutable_global(
            &module,
            store.as_context_mut(),
            "increase_global_value",
            "global_mutable_value",
            &global_value,
            10000,
        );
        let updated_value = global_value.get(store.as_context());
        let val = match updated_value {
            Val::I32(val) => val,
            _ => panic!("Unexpected return value type"),
        };
        assert_eq!(val, 11100);
    }
}
