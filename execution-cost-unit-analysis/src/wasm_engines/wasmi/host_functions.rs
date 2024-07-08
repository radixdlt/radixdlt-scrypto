use super::env::*;
use radix_common::prelude::*;
use radix_engine::errors::InvokeError;
use radix_engine::vm::wasm::*;
use radix_engine_interface::api::actor_api::EventFlags;
use radix_wasmi::*;

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

pub(super) fn consume_buffer(
    caller: Caller<'_, HostState>,
    buffer_id: BufferId,
    destination_ptr: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let result = runtime.buffer_consume(buffer_id);
    match result {
        Ok(slice) => {
            write_memory(caller, memory, destination_ptr, &slice)?;
            Ok(())
        }
        Err(e) => Err(e),
    }
}

pub(super) fn call_method(
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
        .object_call(receiver, ident, args)
        .map(|buffer| buffer.0)
}

pub(super) fn call_direct_method(
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
        .object_call_direct(receiver, ident, args)
        .map(|buffer| buffer.0)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn call_module_method(
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

    runtime
        .object_call_module(receiver, module_id, ident, args)
        .map(|buffer| buffer.0)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn call_function(
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
    let (memory, runtime) = grab_runtime!(caller);

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

pub(super) fn new_object(
    mut caller: Caller<'_, HostState>,
    blueprint_name_ptr: u32,
    blueprint_name_len: u32,
    object_states_ptr: u32,
    object_states_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

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

pub(super) fn new_key_value_store(
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

pub(super) fn allocate_global_address(
    mut caller: Caller<'_, HostState>,
    package_address_ptr: u32,
    package_address_len: u32,
    blueprint_name_ptr: u32,
    blueprint_name_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

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

pub(super) fn get_reservation_address(
    mut caller: Caller<'_, HostState>,
    node_id_ptr: u32,
    node_id_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    runtime
        .address_get_reservation_address(read_memory(
            caller.as_context_mut(),
            memory,
            node_id_ptr,
            node_id_len,
        )?)
        .map(|buffer| buffer.0)
}

pub(super) fn execution_cost_unit_limit(
    caller: Caller<'_, HostState>,
) -> Result<u32, InvokeError<WasmRuntimeError>> {
    let (_memory, runtime) = grab_runtime!(caller);

    runtime.costing_get_execution_cost_unit_limit()
}

pub(super) fn execution_cost_unit_price(
    caller: Caller<'_, HostState>,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (_memory, runtime) = grab_runtime!(caller);

    runtime
        .costing_get_execution_cost_unit_price()
        .map(|buffer| buffer.0)
}

pub(super) fn finalization_cost_unit_limit(
    caller: Caller<'_, HostState>,
) -> Result<u32, InvokeError<WasmRuntimeError>> {
    let (_memory, runtime) = grab_runtime!(caller);

    runtime.costing_get_finalization_cost_unit_limit()
}

pub(super) fn finalization_cost_unit_price(
    caller: Caller<'_, HostState>,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (_memory, runtime) = grab_runtime!(caller);

    runtime
        .costing_get_finalization_cost_unit_price()
        .map(|buffer| buffer.0)
}

pub(super) fn usd_price(
    caller: Caller<'_, HostState>,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (_memory, runtime) = grab_runtime!(caller);

    runtime.costing_get_usd_price().map(|buffer| buffer.0)
}

pub(super) fn tip_percentage(
    caller: Caller<'_, HostState>,
) -> Result<u32, InvokeError<WasmRuntimeError>> {
    let (_memory, runtime) = grab_runtime!(caller);

    runtime.costing_get_tip_percentage()
}

pub(super) fn fee_balance(
    caller: Caller<'_, HostState>,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (_memory, runtime) = grab_runtime!(caller);

    runtime.costing_get_fee_balance().map(|buffer| buffer.0)
}

pub(super) fn globalize_object(
    mut caller: Caller<'_, HostState>,
    obj_id_ptr: u32,
    obj_id_len: u32,
    modules_ptr: u32,
    modules_len: u32,
    address_ptr: u32,
    address_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    runtime
        .globalize_object(
            read_memory(caller.as_context_mut(), memory, obj_id_ptr, obj_id_len)?,
            read_memory(caller.as_context_mut(), memory, modules_ptr, modules_len)?,
            read_memory(caller.as_context_mut(), memory, address_ptr, address_len)?,
        )
        .map(|buffer| buffer.0)
}

pub(super) fn instance_of(
    mut caller: Caller<'_, HostState>,
    component_id_ptr: u32,
    component_id_len: u32,
    package_address_ptr: u32,
    package_address_len: u32,
    blueprint_name_ptr: u32,
    blueprint_name_len: u32,
) -> Result<u32, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

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

pub(super) fn blueprint_id(
    mut caller: Caller<'_, HostState>,
    component_id_ptr: u32,
    component_id_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    runtime
        .blueprint_id(read_memory(
            caller.as_context_mut(),
            memory,
            component_id_ptr,
            component_id_len,
        )?)
        .map(|buffer| buffer.0)
}

pub(super) fn get_outer_object(
    mut caller: Caller<'_, HostState>,
    component_id_ptr: u32,
    component_id_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    runtime
        .get_outer_object(read_memory(
            caller.as_context_mut(),
            memory,
            component_id_ptr,
            component_id_len,
        )?)
        .map(|buffer| buffer.0)
}

pub(super) fn lock_key_value_store_entry(
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

pub(super) fn key_value_entry_get(
    caller: Caller<'_, HostState>,
    handle: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (_memory, runtime) = grab_runtime!(caller);
    runtime.key_value_entry_get(handle).map(|buffer| buffer.0)
}

pub(super) fn key_value_entry_set(
    mut caller: Caller<'_, HostState>,
    handle: u32,
    buffer_ptr: u32,
    buffer_len: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);
    let data = read_memory(caller.as_context_mut(), memory, buffer_ptr, buffer_len)?;
    runtime.key_value_entry_set(handle, data)
}

pub(super) fn key_value_entry_remove(
    caller: Caller<'_, HostState>,
    handle: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (_memory, runtime) = grab_runtime!(caller);
    runtime
        .key_value_entry_remove(handle)
        .map(|buffer| buffer.0)
}

pub(super) fn unlock_key_value_entry(
    caller: Caller<'_, HostState>,
    handle: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let (_memory, runtime) = grab_runtime!(caller);
    runtime.key_value_entry_close(handle)
}

pub(super) fn key_value_store_remove(
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

pub(super) fn lock_field(
    caller: Caller<'_, HostState>,
    object_handle: u32,
    field: u32,
    flags: u32,
) -> Result<u32, InvokeError<WasmRuntimeError>> {
    let (_memory, runtime) = grab_runtime!(caller);
    runtime.actor_open_field(object_handle, field as u8, flags)
}

pub(super) fn field_lock_read(
    caller: Caller<'_, HostState>,
    handle: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (_memory, runtime) = grab_runtime!(caller);

    runtime.field_entry_read(handle).map(|buffer| buffer.0)
}

pub(super) fn field_lock_write(
    mut caller: Caller<'_, HostState>,
    handle: u32,
    data_ptr: u32,
    data_len: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let data = read_memory(caller.as_context_mut(), memory, data_ptr, data_len)?;

    runtime.field_entry_write(handle, data)
}

pub(super) fn field_lock_release(
    caller: Caller<'_, HostState>,
    handle: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let (_memory, runtime) = grab_runtime!(caller);

    runtime.field_entry_close(handle)
}

pub(super) fn actor_get_node_id(
    caller: Caller<'_, HostState>,
    handle: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (_memory, runtime) = grab_runtime!(caller);

    runtime.actor_get_node_id(handle).map(|buffer| buffer.0)
}

pub(super) fn get_package_address(
    caller: Caller<'_, HostState>,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (_memory, runtime) = grab_runtime!(caller);

    runtime.actor_get_package_address().map(|buffer| buffer.0)
}

pub(super) fn get_blueprint_name(
    caller: Caller<'_, HostState>,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (_memory, runtime) = grab_runtime!(caller);

    runtime.actor_get_blueprint_name().map(|buffer| buffer.0)
}

pub(super) fn consume_wasm_execution_units(
    caller: Caller<'_, HostState>,
    n: u64,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let (_memory, runtime) = grab_runtime!(caller);

    // TODO: wasm-instrument uses u64 for cost units. We need to decide if we want to move from u32
    // to u64 as well.
    runtime.consume_wasm_execution_units(n as u32)
}

pub(super) fn emit_event(
    mut caller: Caller<'_, HostState>,
    event_name_ptr: u32,
    event_name_len: u32,
    event_data_ptr: u32,
    event_data_len: u32,
    flags: u32,
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
    let event_flags = EventFlags::from_bits(flags).ok_or(InvokeError::SelfError(
        WasmRuntimeError::InvalidEventFlags(flags),
    ))?;

    runtime.actor_emit_event(event_name, event_data, event_flags)
}

pub(super) fn get_transaction_hash(
    caller: Caller<'_, HostState>,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (_, runtime) = grab_runtime!(caller);

    runtime.sys_get_transaction_hash().map(|buffer| buffer.0)
}

pub(super) fn generate_ruid(
    caller: Caller<'_, HostState>,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (_, runtime) = grab_runtime!(caller);

    runtime.sys_generate_ruid().map(|buffer| buffer.0)
}

pub(super) fn emit_log(
    mut caller: Caller<'_, HostState>,
    level_ptr: u32,
    level_len: u32,
    message_ptr: u32,
    message_len: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let level = read_memory(caller.as_context_mut(), memory, level_ptr, level_len)?;
    let message = read_memory(caller.as_context_mut(), memory, message_ptr, message_len)?;

    runtime.sys_log(level, message)
}

pub(super) fn bech32_encode_address(
    mut caller: Caller<'_, HostState>,
    address_ptr: u32,
    address_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let address = read_memory(caller.as_context_mut(), memory, address_ptr, address_len)?;

    runtime
        .sys_bech32_encode_address(address)
        .map(|buffer| buffer.0)
}

pub(super) fn panic(
    mut caller: Caller<'_, HostState>,
    message_ptr: u32,
    message_len: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let message = read_memory(caller.as_context_mut(), memory, message_ptr, message_len)?;

    runtime.sys_panic(message)
}

pub(super) fn bls12381_v1_verify(
    mut caller: Caller<'_, HostState>,
    message_ptr: u32,
    message_len: u32,
    public_key_ptr: u32,
    public_key_len: u32,
    signature_ptr: u32,
    signature_len: u32,
) -> Result<u32, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

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

pub(super) fn bls12381_v1_aggregate_verify(
    mut caller: Caller<'_, HostState>,
    pub_keys_and_msgs_ptr: u32,
    pub_keys_and_msgs_len: u32,
    signature_ptr: u32,
    signature_len: u32,
) -> Result<u32, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

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

pub(super) fn bls12381_v1_fast_aggregate_verify(
    mut caller: Caller<'_, HostState>,
    message_ptr: u32,
    message_len: u32,
    public_keys_ptr: u32,
    public_keys_len: u32,
    signature_ptr: u32,
    signature_len: u32,
) -> Result<u32, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

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

pub(super) fn bls12381_g2_signature_aggregate(
    mut caller: Caller<'_, HostState>,
    signatures_ptr: u32,
    signatures_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

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

pub(super) fn keccak256_hash(
    mut caller: Caller<'_, HostState>,
    data_ptr: u32,
    data_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let data = read_memory(caller.as_context_mut(), memory, data_ptr, data_len)?;

    runtime
        .crypto_utils_keccak256_hash(data)
        .map(|buffer| buffer.0)
}

pub(super) fn decimal_checked_add(
    mut caller: Caller<'_, HostState>,
    num1_ptr: u32,
    num1_len: u32,
    num2_ptr: u32,
    num2_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let num1 = read_memory(caller.as_context_mut(), memory, num1_ptr, num1_len)?;
    let num2 = read_memory(caller.as_context_mut(), memory, num2_ptr, num2_len)?;

    runtime
        .decimal_checked_add(num1, num2)
        .map(|buffer| buffer.0)
}

pub(super) fn decimal_checked_sub(
    mut caller: Caller<'_, HostState>,
    num1_ptr: u32,
    num1_len: u32,
    num2_ptr: u32,
    num2_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let num1 = read_memory(caller.as_context_mut(), memory, num1_ptr, num1_len)?;
    let num2 = read_memory(caller.as_context_mut(), memory, num2_ptr, num2_len)?;

    runtime
        .decimal_checked_sub(num1, num2)
        .map(|buffer| buffer.0)
}

pub(super) fn decimal_checked_mul(
    mut caller: Caller<'_, HostState>,
    num1_ptr: u32,
    num1_len: u32,
    num2_ptr: u32,
    num2_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let num1 = read_memory(caller.as_context_mut(), memory, num1_ptr, num1_len)?;
    let num2 = read_memory(caller.as_context_mut(), memory, num2_ptr, num2_len)?;

    runtime
        .decimal_checked_mul(num1, num2)
        .map(|buffer| buffer.0)
}

pub(super) fn decimal_checked_div(
    mut caller: Caller<'_, HostState>,
    num1_ptr: u32,
    num1_len: u32,
    num2_ptr: u32,
    num2_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let num1 = read_memory(caller.as_context_mut(), memory, num1_ptr, num1_len)?;
    let num2 = read_memory(caller.as_context_mut(), memory, num2_ptr, num2_len)?;

    runtime
        .decimal_checked_div(num1, num2)
        .map(|buffer| buffer.0)
}

pub(super) fn decimal_checked_neg(
    mut caller: Caller<'_, HostState>,
    num_ptr: u32,
    num_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let num = read_memory(caller.as_context_mut(), memory, num_ptr, num_len)?;

    runtime.decimal_checked_neg(num).map(|buffer| buffer.0)
}

pub(super) fn decimal_checked_round(
    mut caller: Caller<'_, HostState>,
    num_ptr: u32,
    num_len: u32,
    decimal_places_ptr: u32,
    decimal_places_len: u32,
    mode_ptr: u32,
    mode_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let num = read_memory(caller.as_context_mut(), memory, num_ptr, num_len)?;
    let decimal_places = read_memory(
        caller.as_context_mut(),
        memory,
        decimal_places_ptr,
        decimal_places_len,
    )?;
    let mode = read_memory(caller.as_context_mut(), memory, mode_ptr, mode_len)?;

    runtime
        .decimal_checked_round(num, decimal_places, mode)
        .map(|buffer| buffer.0)
}

pub(super) fn decimal_checked_powi(
    mut caller: Caller<'_, HostState>,
    num_ptr: u32,
    num_len: u32,
    exp_ptr: u32,
    exp_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let num = read_memory(caller.as_context_mut(), memory, num_ptr, num_len)?;
    let exp = read_memory(caller.as_context_mut(), memory, exp_ptr, exp_len)?;

    runtime
        .decimal_checked_powi(num, exp)
        .map(|buffer| buffer.0)
}

pub(super) fn decimal_checked_sqrt(
    mut caller: Caller<'_, HostState>,
    num_ptr: u32,
    num_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let num = read_memory(caller.as_context_mut(), memory, num_ptr, num_len)?;

    runtime.decimal_checked_sqrt(num).map(|buffer| buffer.0)
}

pub(super) fn decimal_checked_cbrt(
    mut caller: Caller<'_, HostState>,
    num_ptr: u32,
    num_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let num = read_memory(caller.as_context_mut(), memory, num_ptr, num_len)?;

    runtime.decimal_checked_cbrt(num).map(|buffer| buffer.0)
}

pub(super) fn decimal_checked_nth_root(
    mut caller: Caller<'_, HostState>,
    num_ptr: u32,
    num_len: u32,
    n_ptr: u32,
    n_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let num = read_memory(caller.as_context_mut(), memory, num_ptr, num_len)?;
    let n = read_memory(caller.as_context_mut(), memory, n_ptr, n_len)?;

    runtime
        .decimal_checked_nth_root(num, n)
        .map(|buffer| buffer.0)
}

pub(super) fn precise_decimal_checked_add(
    mut caller: Caller<'_, HostState>,
    num1_ptr: u32,
    num1_len: u32,
    num2_ptr: u32,
    num2_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let num1 = read_memory(caller.as_context_mut(), memory, num1_ptr, num1_len)?;
    let num2 = read_memory(caller.as_context_mut(), memory, num2_ptr, num2_len)?;

    runtime
        .precise_decimal_checked_add(num1, num2)
        .map(|buffer| buffer.0)
}

pub(super) fn precise_decimal_checked_sub(
    mut caller: Caller<'_, HostState>,
    num1_ptr: u32,
    num1_len: u32,
    num2_ptr: u32,
    num2_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let num1 = read_memory(caller.as_context_mut(), memory, num1_ptr, num1_len)?;
    let num2 = read_memory(caller.as_context_mut(), memory, num2_ptr, num2_len)?;

    runtime
        .precise_decimal_checked_sub(num1, num2)
        .map(|buffer| buffer.0)
}

pub(super) fn precise_decimal_checked_mul(
    mut caller: Caller<'_, HostState>,
    num1_ptr: u32,
    num1_len: u32,
    num2_ptr: u32,
    num2_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let num1 = read_memory(caller.as_context_mut(), memory, num1_ptr, num1_len)?;
    let num2 = read_memory(caller.as_context_mut(), memory, num2_ptr, num2_len)?;

    runtime
        .precise_decimal_checked_mul(num1, num2)
        .map(|buffer| buffer.0)
}

pub(super) fn precise_decimal_checked_div(
    mut caller: Caller<'_, HostState>,
    num1_ptr: u32,
    num1_len: u32,
    num2_ptr: u32,
    num2_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let num1 = read_memory(caller.as_context_mut(), memory, num1_ptr, num1_len)?;
    let num2 = read_memory(caller.as_context_mut(), memory, num2_ptr, num2_len)?;

    runtime
        .precise_decimal_checked_div(num1, num2)
        .map(|buffer| buffer.0)
}

pub(super) fn precise_decimal_checked_neg(
    mut caller: Caller<'_, HostState>,
    num_ptr: u32,
    num_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let num = read_memory(caller.as_context_mut(), memory, num_ptr, num_len)?;

    runtime
        .precise_decimal_checked_neg(num)
        .map(|buffer| buffer.0)
}

pub(super) fn precise_decimal_checked_round(
    mut caller: Caller<'_, HostState>,
    num_ptr: u32,
    num_len: u32,
    precise_decimal_places_ptr: u32,
    precise_decimal_places_len: u32,
    mode_ptr: u32,
    mode_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let num = read_memory(caller.as_context_mut(), memory, num_ptr, num_len)?;
    let precise_decimal_places = read_memory(
        caller.as_context_mut(),
        memory,
        precise_decimal_places_ptr,
        precise_decimal_places_len,
    )?;
    let mode = read_memory(caller.as_context_mut(), memory, mode_ptr, mode_len)?;

    runtime
        .precise_decimal_checked_round(num, precise_decimal_places, mode)
        .map(|buffer| buffer.0)
}

pub(super) fn precise_decimal_checked_powi(
    mut caller: Caller<'_, HostState>,
    num_ptr: u32,
    num_len: u32,
    exp_ptr: u32,
    exp_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let num = read_memory(caller.as_context_mut(), memory, num_ptr, num_len)?;
    let exp = read_memory(caller.as_context_mut(), memory, exp_ptr, exp_len)?;

    runtime
        .precise_decimal_checked_powi(num, exp)
        .map(|buffer| buffer.0)
}

pub(super) fn precise_decimal_checked_sqrt(
    mut caller: Caller<'_, HostState>,
    num_ptr: u32,
    num_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let num = read_memory(caller.as_context_mut(), memory, num_ptr, num_len)?;

    runtime
        .precise_decimal_checked_sqrt(num)
        .map(|buffer| buffer.0)
}

pub(super) fn precise_decimal_checked_cbrt(
    mut caller: Caller<'_, HostState>,
    num_ptr: u32,
    num_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let num = read_memory(caller.as_context_mut(), memory, num_ptr, num_len)?;

    runtime
        .precise_decimal_checked_cbrt(num)
        .map(|buffer| buffer.0)
}

pub(super) fn precise_decimal_checked_nth_root(
    mut caller: Caller<'_, HostState>,
    num_ptr: u32,
    num_len: u32,
    n_ptr: u32,
    n_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (memory, runtime) = grab_runtime!(caller);

    let num = read_memory(caller.as_context_mut(), memory, num_ptr, num_len)?;
    let n = read_memory(caller.as_context_mut(), memory, n_ptr, n_len)?;

    runtime
        .precise_decimal_checked_nth_root(num, n)
        .map(|buffer| buffer.0)
}

pub(super) fn read_memory(
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

pub(super) fn write_memory(
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
        .map_err(|_| InvokeError::SelfError(WasmRuntimeError::MemoryAccessError))
}

pub(super) fn read_slice(
    store: impl AsContextMut,
    memory: Memory,
    v: Slice,
) -> Result<Vec<u8>, InvokeError<WasmRuntimeError>> {
    let ptr = v.ptr();
    let len = v.len();

    read_memory(store, memory, ptr, len)
}
