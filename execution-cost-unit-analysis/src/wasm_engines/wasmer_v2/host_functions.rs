use super::env::*;
use radix_common::prelude::*;
use radix_engine::errors::InvokeError;
use radix_engine::vm::wasm::*;
use radix_engine_interface::api::actor_api::EventFlags;
use wasmer::*;

macro_rules! grab_runtime {
    ($env: expr) => {{
        let instance = unsafe { $env.instance.get_unchecked() };
        let ptr = $env.runtime_ptr.lock().expect("Runtime ptr unavailable");
        let runtime: &mut Box<dyn WasmRuntime> = unsafe { &mut *(*ptr as *mut _) };
        (instance, runtime)
    }};
}

// native functions starts
pub(super) fn buffer_consume(
    env: &WasmerV2InstanceEnv,
    buffer_id: BufferId,
    destination_ptr: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    let result = runtime.buffer_consume(buffer_id);
    match result {
        Ok(slice) => {
            write_memory(instance, destination_ptr, &slice)?;
            Ok(())
        }
        Err(e) => Err(e),
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn blueprint_call(
    env: &WasmerV2InstanceEnv,
    package_address_ptr: u32,
    package_address_len: u32,
    blueprint_name_ptr: u32,
    blueprint_name_len: u32,
    ident_ptr: u32,
    ident_len: u32,
    args_ptr: u32,
    args_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    let package_address = read_memory(instance, package_address_ptr, package_address_len)?;
    let blueprint_name = read_memory(instance, blueprint_name_ptr, blueprint_name_len)?;
    let ident = read_memory(instance, ident_ptr, ident_len)?;
    let args = read_memory(instance, args_ptr, args_len)?;

    runtime
        .blueprint_call(package_address, blueprint_name, ident, args)
        .map(|buffer| buffer.0)
}

pub(super) fn address_allocate(
    env: &WasmerV2InstanceEnv,
    package_address_ptr: u32,
    package_address_len: u32,
    blueprint_name_ptr: u32,
    blueprint_name_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    runtime
        .address_allocate(
            read_memory(instance, package_address_ptr, package_address_len)?,
            read_memory(instance, blueprint_name_ptr, blueprint_name_len)?,
        )
        .map(|buffer| buffer.0)
}

pub(super) fn address_get_reservation_address(
    env: &WasmerV2InstanceEnv,
    node_id_ptr: u32,
    node_id_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    runtime
        .address_get_reservation_address(read_memory(instance, node_id_ptr, node_id_len)?)
        .map(|buffer| buffer.0)
}

pub(super) fn object_call(
    env: &WasmerV2InstanceEnv,
    receiver_ptr: u32,
    receiver_len: u32,
    ident_ptr: u32,
    ident_len: u32,
    args_ptr: u32,
    args_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    let receiver = read_memory(instance, receiver_ptr, receiver_len)?;
    let ident = read_memory(instance, ident_ptr, ident_len)?;
    let args = read_memory(instance, args_ptr, args_len)?;

    runtime
        .object_call(receiver, ident, args)
        .map(|buffer| buffer.0)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn object_call_module(
    env: &WasmerV2InstanceEnv,
    receiver_ptr: u32,
    receiver_len: u32,
    module: u32,
    ident_ptr: u32,
    ident_len: u32,
    args_ptr: u32,
    args_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    let receiver = read_memory(instance, receiver_ptr, receiver_len)?;
    let ident = read_memory(instance, ident_ptr, ident_len)?;
    let args = read_memory(instance, args_ptr, args_len)?;

    runtime
        .object_call_module(receiver, module, ident, args)
        .map(|buffer| buffer.0)
}

pub(super) fn object_call_direct(
    env: &WasmerV2InstanceEnv,
    receiver_ptr: u32,
    receiver_len: u32,
    ident_ptr: u32,
    ident_len: u32,
    args_ptr: u32,
    args_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    let receiver = read_memory(instance, receiver_ptr, receiver_len)?;
    let ident = read_memory(instance, ident_ptr, ident_len)?;
    let args = read_memory(instance, args_ptr, args_len)?;

    runtime
        .object_call_direct(receiver, ident, args)
        .map(|buffer| buffer.0)
}

pub(super) fn object_new(
    env: &WasmerV2InstanceEnv,
    blueprint_name_ptr: u32,
    blueprint_name_len: u32,
    object_states_ptr: u32,
    object_states_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    runtime
        .object_new(
            read_memory(instance, blueprint_name_ptr, blueprint_name_len)?,
            read_memory(instance, object_states_ptr, object_states_len)?,
        )
        .map(|buffer| buffer.0)
}

pub(super) fn object_globalize(
    env: &WasmerV2InstanceEnv,
    obj_ptr: u32,
    obj_len: u32,
    modules_ptr: u32,
    modules_len: u32,
    address_ptr: u32,
    address_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    runtime
        .globalize_object(
            read_memory(instance, obj_ptr, obj_len)?,
            read_memory(instance, modules_ptr, modules_len)?,
            read_memory(instance, address_ptr, address_len)?,
        )
        .map(|buffer| buffer.0)
}

pub(super) fn object_instance_of(
    env: &WasmerV2InstanceEnv,
    component_id_ptr: u32,
    component_id_len: u32,
    package_address_ptr: u32,
    package_address_len: u32,
    blueprint_name_ptr: u32,
    blueprint_name_len: u32,
) -> Result<u32, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    runtime.instance_of(
        read_memory(instance, component_id_ptr, component_id_len)?,
        read_memory(instance, package_address_ptr, package_address_len)?,
        read_memory(instance, blueprint_name_ptr, blueprint_name_len)?,
    )
}

pub(super) fn object_get_blueprint_id(
    env: &WasmerV2InstanceEnv,
    component_id_ptr: u32,
    component_id_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    runtime
        .blueprint_id(read_memory(instance, component_id_ptr, component_id_len)?)
        .map(|buffer| buffer.0)
}

pub(super) fn object_get_outer_object(
    env: &WasmerV2InstanceEnv,
    component_id_ptr: u32,
    component_id_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    runtime
        .get_outer_object(read_memory(instance, component_id_ptr, component_id_len)?)
        .map(|buffer| buffer.0)
}

pub(super) fn key_value_store_new(
    env: &WasmerV2InstanceEnv,
    schema_id_ptr: u32,
    schema_id_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    runtime
        .key_value_store_new(read_memory(instance, schema_id_ptr, schema_id_len)?)
        .map(|buffer| buffer.0)
}

pub(super) fn key_value_store_open_entry(
    env: &WasmerV2InstanceEnv,
    node_id_ptr: u32,
    node_id_len: u32,
    key_ptr: u32,
    key_len: u32,
    flags: u32,
) -> Result<u32, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    runtime.key_value_store_open_entry(
        read_memory(instance, node_id_ptr, node_id_len)?,
        read_memory(instance, key_ptr, key_len)?,
        flags,
    )
}

pub(super) fn key_value_store_remove_entry(
    env: &WasmerV2InstanceEnv,
    node_id_ptr: u32,
    node_id_len: u32,
    key_ptr: u32,
    key_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    runtime
        .key_value_store_remove_entry(
            read_memory(instance, node_id_ptr, node_id_len)?,
            read_memory(instance, key_ptr, key_len)?,
        )
        .map(|buffer| buffer.0)
}

pub(super) fn key_value_entry_read(
    env: &WasmerV2InstanceEnv,
    handle: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (_instance, runtime) = grab_runtime!(env);

    runtime.key_value_entry_get(handle).map(|buffer| buffer.0)
}

pub(super) fn key_value_entry_write(
    env: &WasmerV2InstanceEnv,
    handle: u32,
    data_ptr: u32,
    data_len: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    let data = read_memory(instance, data_ptr, data_len)?;

    runtime.key_value_entry_set(handle, data)
}

pub(super) fn key_value_entry_remove(
    env: &WasmerV2InstanceEnv,
    handle: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (_instance, runtime) = grab_runtime!(env);

    runtime
        .key_value_entry_remove(handle)
        .map(|buffer| buffer.0)
}

pub(super) fn key_value_entry_close(
    env: &WasmerV2InstanceEnv,
    handle: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let (_instance, runtime) = grab_runtime!(env);

    runtime.key_value_entry_close(handle)
}

pub(super) fn field_entry_read(
    env: &WasmerV2InstanceEnv,
    handle: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (_instance, runtime) = grab_runtime!(env);

    runtime.field_entry_read(handle).map(|buffer| buffer.0)
}

pub(super) fn field_entry_write(
    env: &WasmerV2InstanceEnv,
    handle: u32,
    data_ptr: u32,
    data_len: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    let data = read_memory(instance, data_ptr, data_len)?;

    runtime.field_entry_write(handle, data)
}

pub(super) fn field_entry_close(
    env: &WasmerV2InstanceEnv,
    handle: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let (_instance, runtime) = grab_runtime!(env);

    runtime.field_entry_close(handle)
}

pub(super) fn actor_open_field(
    env: &WasmerV2InstanceEnv,
    object_handle: u32,
    field: u8,
    flags: u32,
) -> Result<u32, InvokeError<WasmRuntimeError>> {
    let (_instance, runtime) = grab_runtime!(env);

    runtime.actor_open_field(object_handle, field, flags)
}

pub(super) fn actor_get_node_id(
    env: &WasmerV2InstanceEnv,
    actor_ref_handle: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (_instance, runtime) = grab_runtime!(env);

    runtime
        .actor_get_node_id(actor_ref_handle)
        .map(|buffer| buffer.0)
}

pub(super) fn actor_get_package_address(
    env: &WasmerV2InstanceEnv,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (_instance, runtime) = grab_runtime!(env);

    runtime.actor_get_package_address().map(|buffer| buffer.0)
}

pub(super) fn actor_get_blueprint_name(
    env: &WasmerV2InstanceEnv,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (_instance, runtime) = grab_runtime!(env);

    runtime.actor_get_blueprint_name().map(|buffer| buffer.0)
}

pub(super) fn actor_emit_event(
    env: &WasmerV2InstanceEnv,
    event_name_ptr: u32,
    event_name_len: u32,
    event_data_ptr: u32,
    event_data_len: u32,
    flags: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    let event_name = read_memory(instance, event_name_ptr, event_name_len)?;
    let event_data = read_memory(instance, event_data_ptr, event_data_len)?;
    let event_flags = EventFlags::from_bits(flags).ok_or(InvokeError::SelfError(
        WasmRuntimeError::InvalidEventFlags(flags),
    ))?;

    runtime.actor_emit_event(event_name, event_data, event_flags)
}

pub(super) fn costing_get_execution_cost_unit_limit(
    env: &WasmerV2InstanceEnv,
) -> Result<u32, InvokeError<WasmRuntimeError>> {
    let (_instance, runtime) = grab_runtime!(env);

    runtime.costing_get_execution_cost_unit_limit()
}

pub(super) fn costing_get_execution_cost_unit_price(
    env: &WasmerV2InstanceEnv,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (_instance, runtime) = grab_runtime!(env);

    runtime
        .costing_get_execution_cost_unit_price()
        .map(|buffer| buffer.0)
}

pub(super) fn costing_get_finalization_cost_unit_limit(
    env: &WasmerV2InstanceEnv,
) -> Result<u32, InvokeError<WasmRuntimeError>> {
    let (_instance, runtime) = grab_runtime!(env);

    runtime.costing_get_finalization_cost_unit_limit()
}

pub(super) fn costing_get_finalization_cost_unit_price(
    env: &WasmerV2InstanceEnv,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (_instance, runtime) = grab_runtime!(env);

    runtime
        .costing_get_finalization_cost_unit_price()
        .map(|buffer| buffer.0)
}

pub(super) fn costing_get_tip_percentage(
    env: &WasmerV2InstanceEnv,
) -> Result<u32, InvokeError<WasmRuntimeError>> {
    let (_instance, runtime) = grab_runtime!(env);

    runtime.costing_get_tip_percentage()
}

pub(super) fn costing_get_fee_balance(
    env: &WasmerV2InstanceEnv,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (_instance, runtime) = grab_runtime!(env);

    runtime.costing_get_fee_balance().map(|buffer| buffer.0)
}

pub(super) fn costing_get_usd_price(
    env: &WasmerV2InstanceEnv,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (_instance, runtime) = grab_runtime!(env);

    runtime.costing_get_usd_price().map(|buffer| buffer.0)
}

pub(super) fn consume_wasm_execution_units(
    env: &WasmerV2InstanceEnv,
    n: u64,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let (_instance, runtime) = grab_runtime!(env);
    // TODO: wasm-instrument uses u64 for cost units. We need to decide if we want to move from u32
    // to u64 as well.
    runtime.consume_wasm_execution_units(n as u32)
}

pub(super) fn sys_log(
    env: &WasmerV2InstanceEnv,
    level_ptr: u32,
    level_len: u32,
    message_ptr: u32,
    message_len: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    let level = read_memory(instance, level_ptr, level_len)?;
    let message = read_memory(instance, message_ptr, message_len)?;

    runtime.sys_log(level, message)
}

pub(super) fn sys_bech32_encode_address(
    env: &WasmerV2InstanceEnv,
    address_ptr: u32,
    address_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    let address = read_memory(instance, address_ptr, address_len)?;

    runtime
        .sys_bech32_encode_address(address)
        .map(|buffer| buffer.0)
}

pub(super) fn sys_panic(
    env: &WasmerV2InstanceEnv,
    message_ptr: u32,
    message_len: u32,
) -> Result<(), InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    let message = read_memory(instance, message_ptr, message_len)?;

    runtime.sys_panic(message)
}

pub(super) fn sys_get_transaction_hash(
    env: &WasmerV2InstanceEnv,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (_instance, runtime) = grab_runtime!(env);

    runtime.sys_get_transaction_hash().map(|buffer| buffer.0)
}

pub(super) fn sys_generate_ruid(
    env: &WasmerV2InstanceEnv,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (_instance, runtime) = grab_runtime!(env);

    runtime.sys_generate_ruid().map(|buffer| buffer.0)
}

pub(super) fn bls12381_v1_verify(
    env: &WasmerV2InstanceEnv,
    message_ptr: u32,
    message_len: u32,
    public_key_ptr: u32,
    public_key_len: u32,
    signature_ptr: u32,
    signature_len: u32,
) -> Result<u32, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    let message = read_memory(instance, message_ptr, message_len)?;

    let public_key = read_memory(instance, public_key_ptr, public_key_len)?;
    let signature = read_memory(instance, signature_ptr, signature_len)?;

    runtime.crypto_utils_bls12381_v1_verify(message, public_key, signature)
}

pub(super) fn bls12381_v1_aggregate_verify(
    env: &WasmerV2InstanceEnv,
    pub_keys_and_msgs_ptr: u32,
    pub_keys_and_msgs_len: u32,
    signature_ptr: u32,
    signature_len: u32,
) -> Result<u32, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    let pub_keys_and_msgs = read_memory(instance, pub_keys_and_msgs_ptr, pub_keys_and_msgs_len)?;
    let signature = read_memory(instance, signature_ptr, signature_len)?;

    runtime.crypto_utils_bls12381_v1_aggregate_verify(pub_keys_and_msgs, signature)
}

pub(super) fn bls12381_v1_fast_aggregate_verify(
    env: &WasmerV2InstanceEnv,
    message_ptr: u32,
    message_len: u32,
    public_keys_ptr: u32,
    public_keys_len: u32,
    signature_ptr: u32,
    signature_len: u32,
) -> Result<u32, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    let message = read_memory(instance, message_ptr, message_len)?;

    let public_keys = read_memory(instance, public_keys_ptr, public_keys_len)?;
    let signature = read_memory(instance, signature_ptr, signature_len)?;

    runtime.crypto_utils_bls12381_v1_fast_aggregate_verify(message, public_keys, signature)
}

pub(super) fn bls12381_g2_signature_aggregate(
    env: &WasmerV2InstanceEnv,
    signatures_ptr: u32,
    signatures_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    let signatures = read_memory(instance, signatures_ptr, signatures_len)?;

    runtime
        .crypto_utils_bls12381_g2_signature_aggregate(signatures)
        .map(|buffer| buffer.0)
}

pub(super) fn keccak256_hash(
    env: &WasmerV2InstanceEnv,
    data_ptr: u32,
    data_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    let data = read_memory(instance, data_ptr, data_len)?;

    runtime
        .crypto_utils_keccak256_hash(data)
        .map(|buffer| buffer.0)
}

pub(super) fn decimal_checked_add(
    env: &WasmerV2InstanceEnv,
    num1_ptr: u32,
    num1_len: u32,
    num2_ptr: u32,
    num2_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    let num1 = read_memory(instance, num1_ptr, num1_len)?;
    let num2 = read_memory(instance, num2_ptr, num2_len)?;

    runtime
        .decimal_checked_add(num1, num2)
        .map(|buffer| buffer.0)
}

pub(super) fn decimal_checked_sub(
    env: &WasmerV2InstanceEnv,
    num1_ptr: u32,
    num1_len: u32,
    num2_ptr: u32,
    num2_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    let num1 = read_memory(instance, num1_ptr, num1_len)?;
    let num2 = read_memory(instance, num2_ptr, num2_len)?;

    runtime
        .decimal_checked_sub(num1, num2)
        .map(|buffer| buffer.0)
}

pub(super) fn decimal_checked_mul(
    env: &WasmerV2InstanceEnv,
    num1_ptr: u32,
    num1_len: u32,
    num2_ptr: u32,
    num2_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    let num1 = read_memory(instance, num1_ptr, num1_len)?;
    let num2 = read_memory(instance, num2_ptr, num2_len)?;

    runtime
        .decimal_checked_mul(num1, num2)
        .map(|buffer| buffer.0)
}

pub(super) fn decimal_checked_div(
    env: &WasmerV2InstanceEnv,
    num1_ptr: u32,
    num1_len: u32,
    num2_ptr: u32,
    num2_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    let num1 = read_memory(instance, num1_ptr, num1_len)?;
    let num2 = read_memory(instance, num2_ptr, num2_len)?;

    runtime
        .decimal_checked_div(num1, num2)
        .map(|buffer| buffer.0)
}

pub(super) fn decimal_checked_neg(
    env: &WasmerV2InstanceEnv,
    num_ptr: u32,
    num_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    let num = read_memory(instance, num_ptr, num_len)?;

    runtime.decimal_checked_neg(num).map(|buffer| buffer.0)
}

pub(super) fn decimal_checked_round(
    env: &WasmerV2InstanceEnv,
    num_ptr: u32,
    num_len: u32,
    decimal_places_ptr: u32,
    decimal_places_len: u32,
    mode_ptr: u32,
    mode_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    let num = read_memory(instance, num_ptr, num_len)?;
    let decimal_places = read_memory(instance, decimal_places_ptr, decimal_places_len)?;
    let mode = read_memory(instance, mode_ptr, mode_len)?;

    runtime
        .decimal_checked_round(num, decimal_places, mode)
        .map(|buffer| buffer.0)
}

pub(super) fn decimal_checked_powi(
    env: &WasmerV2InstanceEnv,
    num_ptr: u32,
    num_len: u32,
    exp_ptr: u32,
    exp_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    let num = read_memory(instance, num_ptr, num_len)?;
    let exp = read_memory(instance, exp_ptr, exp_len)?;

    runtime
        .decimal_checked_powi(num, exp)
        .map(|buffer| buffer.0)
}

pub(super) fn decimal_checked_sqrt(
    env: &WasmerV2InstanceEnv,
    num_ptr: u32,
    num_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    let num = read_memory(instance, num_ptr, num_len)?;

    runtime.decimal_checked_sqrt(num).map(|buffer| buffer.0)
}

pub(super) fn decimal_checked_cbrt(
    env: &WasmerV2InstanceEnv,
    num_ptr: u32,
    num_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    let num = read_memory(instance, num_ptr, num_len)?;

    runtime.decimal_checked_cbrt(num).map(|buffer| buffer.0)
}

pub(super) fn decimal_checked_nth_root(
    env: &WasmerV2InstanceEnv,
    num_ptr: u32,
    num_len: u32,
    n_ptr: u32,
    n_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    let num = read_memory(instance, num_ptr, num_len)?;
    let n = read_memory(instance, n_ptr, n_len)?;

    runtime
        .decimal_checked_nth_root(num, n)
        .map(|buffer| buffer.0)
}

pub(super) fn precise_decimal_checked_add(
    env: &WasmerV2InstanceEnv,
    num1_ptr: u32,
    num1_len: u32,
    num2_ptr: u32,
    num2_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    let num1 = read_memory(instance, num1_ptr, num1_len)?;
    let num2 = read_memory(instance, num2_ptr, num2_len)?;

    runtime
        .precise_decimal_checked_add(num1, num2)
        .map(|buffer| buffer.0)
}

pub(super) fn precise_decimal_checked_sub(
    env: &WasmerV2InstanceEnv,
    num1_ptr: u32,
    num1_len: u32,
    num2_ptr: u32,
    num2_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    let num1 = read_memory(instance, num1_ptr, num1_len)?;
    let num2 = read_memory(instance, num2_ptr, num2_len)?;

    runtime
        .precise_decimal_checked_sub(num1, num2)
        .map(|buffer| buffer.0)
}

pub(super) fn precise_decimal_checked_mul(
    env: &WasmerV2InstanceEnv,
    num1_ptr: u32,
    num1_len: u32,
    num2_ptr: u32,
    num2_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    let num1 = read_memory(instance, num1_ptr, num1_len)?;
    let num2 = read_memory(instance, num2_ptr, num2_len)?;

    runtime
        .precise_decimal_checked_mul(num1, num2)
        .map(|buffer| buffer.0)
}

pub(super) fn precise_decimal_checked_div(
    env: &WasmerV2InstanceEnv,
    num1_ptr: u32,
    num1_len: u32,
    num2_ptr: u32,
    num2_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    let num1 = read_memory(instance, num1_ptr, num1_len)?;
    let num2 = read_memory(instance, num2_ptr, num2_len)?;

    runtime
        .precise_decimal_checked_div(num1, num2)
        .map(|buffer| buffer.0)
}

pub(super) fn precise_decimal_checked_neg(
    env: &WasmerV2InstanceEnv,
    num_ptr: u32,
    num_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    let num = read_memory(instance, num_ptr, num_len)?;

    runtime
        .precise_decimal_checked_neg(num)
        .map(|buffer| buffer.0)
}

pub(super) fn precise_decimal_checked_round(
    env: &WasmerV2InstanceEnv,
    num_ptr: u32,
    num_len: u32,
    precise_decimal_places_ptr: u32,
    precise_decimal_places_len: u32,
    mode_ptr: u32,
    mode_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    let num = read_memory(instance, num_ptr, num_len)?;
    let precise_decimal_places = read_memory(
        instance,
        precise_decimal_places_ptr,
        precise_decimal_places_len,
    )?;
    let mode = read_memory(instance, mode_ptr, mode_len)?;

    runtime
        .precise_decimal_checked_round(num, precise_decimal_places, mode)
        .map(|buffer| buffer.0)
}

pub(super) fn precise_decimal_checked_powi(
    env: &WasmerV2InstanceEnv,
    num_ptr: u32,
    num_len: u32,
    exp_ptr: u32,
    exp_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    let num = read_memory(instance, num_ptr, num_len)?;
    let exp = read_memory(instance, exp_ptr, exp_len)?;

    runtime
        .precise_decimal_checked_powi(num, exp)
        .map(|buffer| buffer.0)
}

pub(super) fn precise_decimal_checked_sqrt(
    env: &WasmerV2InstanceEnv,
    num_ptr: u32,
    num_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    let num = read_memory(instance, num_ptr, num_len)?;

    runtime
        .precise_decimal_checked_sqrt(num)
        .map(|buffer| buffer.0)
}

pub(super) fn precise_decimal_checked_cbrt(
    env: &WasmerV2InstanceEnv,
    num_ptr: u32,
    num_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    let num = read_memory(instance, num_ptr, num_len)?;

    runtime
        .precise_decimal_checked_cbrt(num)
        .map(|buffer| buffer.0)
}

pub(super) fn precise_decimal_checked_nth_root(
    env: &WasmerV2InstanceEnv,
    num_ptr: u32,
    num_len: u32,
    n_ptr: u32,
    n_len: u32,
) -> Result<u64, InvokeError<WasmRuntimeError>> {
    let (instance, runtime) = grab_runtime!(env);

    let num = read_memory(instance, num_ptr, num_len)?;
    let n = read_memory(instance, n_ptr, n_len)?;

    runtime
        .precise_decimal_checked_nth_root(num, n)
        .map(|buffer| buffer.0)
}

pub(super) fn read_memory(
    instance: &Instance,
    ptr: u32,
    len: u32,
) -> Result<Vec<u8>, WasmRuntimeError> {
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

pub(super) fn write_memory(
    instance: &Instance,
    ptr: u32,
    data: &[u8],
) -> Result<(), WasmRuntimeError> {
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

pub(super) fn read_slice(instance: &Instance, v: Slice) -> Result<Vec<u8>, WasmRuntimeError> {
    let ptr = v.ptr();
    let len = v.len();

    read_memory(instance, ptr, len)
}
