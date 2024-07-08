use super::env::{FakeHostState, HostState, WasmiInstanceEnv};
use super::error::WasmiInstantiationError;
use super::instance::WasmiInstance;
use radix_common::prelude::*;
use radix_engine::vm::wasm::*;
use radix_wasmi::core::Trap;
use radix_wasmi::*;
use sbor::rust::mem::transmute;

macro_rules! linker_define {
    ($linker: expr, $name: expr, $var: expr) => {
        $linker
            .define(MODULE_ENV_NAME, $name, $var)
            .expect(stringify!("Failed to define new linker item {}", $name));
    };
}

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
            #[allow(clippy::missing_transmute_annotations)]
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
                super::host_functions::consume_buffer(caller, buffer_id, destination_ptr)
                    .map_err(|e| e.into())
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
             -> Result<u64, Trap> {
                super::host_functions::call_method(
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
             -> Result<u64, Trap> {
                super::host_functions::call_module_method(
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

        let host_call_direct_method = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             receiver_ptr: u32,
             receiver_len: u32,
             ident_ptr: u32,
             ident_len: u32,
             args_ptr: u32,
             args_len: u32|
             -> Result<u64, Trap> {
                super::host_functions::call_direct_method(
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
             -> Result<u64, Trap> {
                super::host_functions::call_function(
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
                .map_err(|e| e.into())
            },
        );

        let host_new_component = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             blueprint_name_ptr: u32,
             blueprint_name_len: u32,
             object_states_ptr: u32,
             object_states_len: u32|
             -> Result<u64, Trap> {
                super::host_functions::new_object(
                    caller,
                    blueprint_name_ptr,
                    blueprint_name_len,
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
                super::host_functions::new_key_value_store(caller, schema_ptr, schema_len)
                    .map_err(|e| e.into())
            },
        );

        let host_allocate_global_address = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             package_address_ptr: u32,
             package_address_len: u32,
             blueprint_name_ptr: u32,
             blueprint_name_len: u32|
             -> Result<u64, Trap> {
                super::host_functions::allocate_global_address(
                    caller,
                    package_address_ptr,
                    package_address_len,
                    blueprint_name_ptr,
                    blueprint_name_len,
                )
                .map_err(|e| e.into())
            },
        );

        let host_get_reservation_address = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             node_id_ptr: u32,
             node_id_len: u32|
             -> Result<u64, Trap> {
                super::host_functions::get_reservation_address(caller, node_id_ptr, node_id_len)
                    .map_err(|e| e.into())
            },
        );

        let host_execution_cost_unit_limit = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>| -> Result<u32, Trap> {
                super::host_functions::execution_cost_unit_limit(caller).map_err(|e| e.into())
            },
        );

        let host_execution_cost_unit_price = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>| -> Result<u64, Trap> {
                super::host_functions::execution_cost_unit_price(caller).map_err(|e| e.into())
            },
        );

        let host_finalization_cost_unit_limit = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>| -> Result<u32, Trap> {
                super::host_functions::finalization_cost_unit_limit(caller).map_err(|e| e.into())
            },
        );

        let host_finalization_cost_unit_price = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>| -> Result<u64, Trap> {
                super::host_functions::finalization_cost_unit_price(caller).map_err(|e| e.into())
            },
        );

        let host_usd_price = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>| -> Result<u64, Trap> {
                super::host_functions::usd_price(caller).map_err(|e| e.into())
            },
        );

        let host_tip_percentage = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>| -> Result<u32, Trap> {
                super::host_functions::tip_percentage(caller).map_err(|e| e.into())
            },
        );

        let host_fee_balance = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>| -> Result<u64, Trap> {
                super::host_functions::fee_balance(caller).map_err(|e| e.into())
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
             -> Result<u64, Trap> {
                super::host_functions::globalize_object(
                    caller,
                    obj_ptr,
                    obj_len,
                    modules_ptr,
                    modules_len,
                    address_ptr,
                    address_len,
                )
                .map_err(|e| e.into())
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
             -> Result<u32, Trap> {
                super::host_functions::instance_of(
                    caller,
                    object_id_ptr,
                    object_id_len,
                    package_address_ptr,
                    package_address_len,
                    blueprint_name_ptr,
                    blueprint_name_len,
                )
                .map_err(|e| e.into())
            },
        );

        let host_get_blueprint_id = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             object_id_ptr: u32,
             object_id_len: u32|
             -> Result<u64, Trap> {
                super::host_functions::blueprint_id(caller, object_id_ptr, object_id_len)
                    .map_err(|e| e.into())
            },
        );

        let host_get_outer_object = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             object_id_ptr: u32,
             object_id_len: u32|
             -> Result<u64, Trap> {
                super::host_functions::get_outer_object(caller, object_id_ptr, object_id_len)
                    .map_err(|e| e.into())
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
                super::host_functions::lock_key_value_store_entry(
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
                super::host_functions::key_value_entry_get(caller, handle).map_err(|e| e.into())
            },
        );

        let host_key_value_entry_set = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             handle: u32,
             buffer_ptr: u32,
             buffer_len: u32|
             -> Result<(), Trap> {
                super::host_functions::key_value_entry_set(caller, handle, buffer_ptr, buffer_len)
                    .map_err(|e| e.into())
            },
        );

        let host_key_value_entry_remove = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>, handle: u32| -> Result<u64, Trap> {
                super::host_functions::key_value_entry_remove(caller, handle).map_err(|e| e.into())
            },
        );

        let host_unlock_key_value_entry = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>, handle: u32| -> Result<(), Trap> {
                super::host_functions::unlock_key_value_entry(caller, handle).map_err(|e| e.into())
            },
        );

        let host_key_value_store_remove = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             node_id_ptr: u32,
             node_id_len: u32,
             key_ptr: u32,
             key_len: u32|
             -> Result<u64, Trap> {
                super::host_functions::key_value_store_remove(
                    caller,
                    node_id_ptr,
                    node_id_len,
                    key_ptr,
                    key_len,
                )
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
                super::host_functions::lock_field(caller, object_handle, field, lock_flags)
                    .map_err(|e| e.into())
            },
        );

        let host_field_lock_read = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>, handle: u32| -> Result<u64, Trap> {
                super::host_functions::field_lock_read(caller, handle).map_err(|e| e.into())
            },
        );

        let host_field_lock_write = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             handle: u32,
             data_ptr: u32,
             data_len: u32|
             -> Result<(), Trap> {
                super::host_functions::field_lock_write(caller, handle, data_ptr, data_len)
                    .map_err(|e| e.into())
            },
        );

        let host_field_lock_release = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>, handle: u32| -> Result<(), Trap> {
                super::host_functions::field_lock_release(caller, handle).map_err(|e| e.into())
            },
        );

        let host_actor_get_node_id = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>, handle: u32| -> Result<u64, Trap> {
                super::host_functions::actor_get_node_id(caller, handle).map_err(|e| e.into())
            },
        );

        let host_get_package_address = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>| -> Result<u64, Trap> {
                super::host_functions::get_package_address(caller).map_err(|e| e.into())
            },
        );

        let host_get_blueprint_name = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>| -> Result<u64, Trap> {
                super::host_functions::get_blueprint_name(caller).map_err(|e| e.into())
            },
        );

        let host_consume_wasm_execution_units = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>, n: u64| -> Result<(), Trap> {
                super::host_functions::consume_wasm_execution_units(caller, n).map_err(|e| e.into())
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
             -> Result<(), Trap> {
                super::host_functions::emit_event(
                    caller,
                    event_name_ptr,
                    event_name_len,
                    event_data_ptr,
                    event_data_len,
                    flags,
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
                super::host_functions::emit_log(
                    caller,
                    level_ptr,
                    level_len,
                    message_ptr,
                    message_len,
                )
                .map_err(|e| e.into())
            },
        );

        let host_panic = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             message_ptr: u32,
             message_len: u32|
             -> Result<(), Trap> {
                super::host_functions::panic(caller, message_ptr, message_len).map_err(|e| e.into())
            },
        );

        let host_bech32_encode_address = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             address_ptr: u32,
             address_len: u32|
             -> Result<u64, Trap> {
                super::host_functions::bech32_encode_address(caller, address_ptr, address_len)
                    .map_err(|e| e.into())
            },
        );

        let host_get_transaction_hash = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>| -> Result<u64, Trap> {
                super::host_functions::get_transaction_hash(caller).map_err(|e| e.into())
            },
        );

        let host_generate_ruid = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>| -> Result<u64, Trap> {
                super::host_functions::generate_ruid(caller).map_err(|e| e.into())
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
             -> Result<u32, Trap> {
                super::host_functions::bls12381_v1_verify(
                    caller,
                    message_ptr,
                    message_len,
                    public_key_ptr,
                    public_key_len,
                    signature_ptr,
                    signature_len,
                )
                .map_err(|e| e.into())
            },
        );

        let host_bls12381_v1_aggregate_verify = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             pub_keys_and_msgs_ptr: u32,
             pub_keys_and_msgs_len: u32,
             signature_ptr: u32,
             signature_len: u32|
             -> Result<u32, Trap> {
                super::host_functions::bls12381_v1_aggregate_verify(
                    caller,
                    pub_keys_and_msgs_ptr,
                    pub_keys_and_msgs_len,
                    signature_ptr,
                    signature_len,
                )
                .map_err(|e| e.into())
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
             -> Result<u32, Trap> {
                super::host_functions::bls12381_v1_fast_aggregate_verify(
                    caller,
                    message_ptr,
                    message_len,
                    public_keys_ptr,
                    public_keys_len,
                    signature_ptr,
                    signature_len,
                )
                .map_err(|e| e.into())
            },
        );

        let host_bls12381_g2_signature_aggregate = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             signatures_ptr: u32,
             signatures_len: u32|
             -> Result<u64, Trap> {
                super::host_functions::bls12381_g2_signature_aggregate(
                    caller,
                    signatures_ptr,
                    signatures_len,
                )
                .map_err(|e| e.into())
            },
        );

        let host_keccak256_hash = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>, data_ptr: u32, data_len: u32| -> Result<u64, Trap> {
                super::host_functions::keccak256_hash(caller, data_ptr, data_len)
                    .map_err(|e| e.into())
            },
        );

        let decimal_checked_add = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             num1_ptr: u32,
             num1_len: u32,
             num2_ptr: u32,
             num2_len: u32|
             -> Result<u64, Trap> {
                super::host_functions::decimal_checked_add(
                    caller, num1_ptr, num1_len, num2_ptr, num2_len,
                )
                .map_err(Trap::from)
            },
        );

        let decimal_checked_sub = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             num1_ptr: u32,
             num1_len: u32,
             num2_ptr: u32,
             num2_len: u32|
             -> Result<u64, Trap> {
                super::host_functions::decimal_checked_sub(
                    caller, num1_ptr, num1_len, num2_ptr, num2_len,
                )
                .map_err(Trap::from)
            },
        );

        let decimal_checked_mul = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             num1_ptr: u32,
             num1_len: u32,
             num2_ptr: u32,
             num2_len: u32|
             -> Result<u64, Trap> {
                super::host_functions::decimal_checked_mul(
                    caller, num1_ptr, num1_len, num2_ptr, num2_len,
                )
                .map_err(Trap::from)
            },
        );

        let decimal_checked_div = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             num1_ptr: u32,
             num1_len: u32,
             num2_ptr: u32,
             num2_len: u32|
             -> Result<u64, Trap> {
                super::host_functions::decimal_checked_div(
                    caller, num1_ptr, num1_len, num2_ptr, num2_len,
                )
                .map_err(Trap::from)
            },
        );

        let decimal_checked_neg = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>, num_ptr: u32, num_len: u32| -> Result<u64, Trap> {
                super::host_functions::decimal_checked_neg(caller, num_ptr, num_len)
                    .map_err(Trap::from)
            },
        );

        let decimal_checked_round = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             num_ptr: u32,
             num_len: u32,
             decimal_places_ptr: u32,
             decimal_places_len: u32,
             mode_ptr: u32,
             mode_len: u32|
             -> Result<u64, Trap> {
                super::host_functions::decimal_checked_round(
                    caller,
                    num_ptr,
                    num_len,
                    decimal_places_ptr,
                    decimal_places_len,
                    mode_ptr,
                    mode_len,
                )
                .map_err(Trap::from)
            },
        );

        let decimal_checked_powi = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             num_ptr: u32,
             num_len: u32,
             exp_ptr: u32,
             exp_len: u32|
             -> Result<u64, Trap> {
                super::host_functions::decimal_checked_powi(
                    caller, num_ptr, num_len, exp_ptr, exp_len,
                )
                .map_err(Trap::from)
            },
        );

        let decimal_checked_sqrt = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>, num_ptr: u32, num_len: u32| -> Result<u64, Trap> {
                super::host_functions::decimal_checked_sqrt(caller, num_ptr, num_len)
                    .map_err(Trap::from)
            },
        );

        let decimal_checked_cbrt = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>, num_ptr: u32, num_len: u32| -> Result<u64, Trap> {
                super::host_functions::decimal_checked_cbrt(caller, num_ptr, num_len)
                    .map_err(Trap::from)
            },
        );

        let decimal_checked_nth_root = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             num_ptr: u32,
             num_len: u32,
             n_ptr: u32,
             n_len: u32|
             -> Result<u64, Trap> {
                super::host_functions::decimal_checked_nth_root(
                    caller, num_ptr, num_len, n_ptr, n_len,
                )
                .map_err(Trap::from)
            },
        );

        let precise_decimal_checked_add = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             num1_ptr: u32,
             num1_len: u32,
             num2_ptr: u32,
             num2_len: u32|
             -> Result<u64, Trap> {
                super::host_functions::precise_decimal_checked_add(
                    caller, num1_ptr, num1_len, num2_ptr, num2_len,
                )
                .map_err(Trap::from)
            },
        );

        let precise_decimal_checked_sub = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             num1_ptr: u32,
             num1_len: u32,
             num2_ptr: u32,
             num2_len: u32|
             -> Result<u64, Trap> {
                super::host_functions::precise_decimal_checked_sub(
                    caller, num1_ptr, num1_len, num2_ptr, num2_len,
                )
                .map_err(Trap::from)
            },
        );

        let precise_decimal_checked_mul = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             num1_ptr: u32,
             num1_len: u32,
             num2_ptr: u32,
             num2_len: u32|
             -> Result<u64, Trap> {
                super::host_functions::precise_decimal_checked_mul(
                    caller, num1_ptr, num1_len, num2_ptr, num2_len,
                )
                .map_err(Trap::from)
            },
        );

        let precise_decimal_checked_div = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             num1_ptr: u32,
             num1_len: u32,
             num2_ptr: u32,
             num2_len: u32|
             -> Result<u64, Trap> {
                super::host_functions::precise_decimal_checked_div(
                    caller, num1_ptr, num1_len, num2_ptr, num2_len,
                )
                .map_err(Trap::from)
            },
        );

        let precise_decimal_checked_neg = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>, num_ptr: u32, num_len: u32| -> Result<u64, Trap> {
                super::host_functions::precise_decimal_checked_neg(caller, num_ptr, num_len)
                    .map_err(Trap::from)
            },
        );

        let precise_decimal_checked_round = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             num_ptr: u32,
             num_len: u32,
             decimal_places_ptr: u32,
             decimal_places_len: u32,
             mode_ptr: u32,
             mode_len: u32|
             -> Result<u64, Trap> {
                super::host_functions::precise_decimal_checked_round(
                    caller,
                    num_ptr,
                    num_len,
                    decimal_places_ptr,
                    decimal_places_len,
                    mode_ptr,
                    mode_len,
                )
                .map_err(Trap::from)
            },
        );

        let precise_decimal_checked_powi = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             num_ptr: u32,
             num_len: u32,
             exp_ptr: u32,
             exp_len: u32|
             -> Result<u64, Trap> {
                super::host_functions::precise_decimal_checked_powi(
                    caller, num_ptr, num_len, exp_ptr, exp_len,
                )
                .map_err(Trap::from)
            },
        );

        let precise_decimal_checked_sqrt = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>, num_ptr: u32, num_len: u32| -> Result<u64, Trap> {
                super::host_functions::precise_decimal_checked_sqrt(caller, num_ptr, num_len)
                    .map_err(Trap::from)
            },
        );

        let precise_decimal_checked_cbrt = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>, num_ptr: u32, num_len: u32| -> Result<u64, Trap> {
                super::host_functions::precise_decimal_checked_cbrt(caller, num_ptr, num_len)
                    .map_err(Trap::from)
            },
        );

        let precise_decimal_checked_nth_root = Func::wrap(
            store.as_context_mut(),
            |caller: Caller<'_, HostState>,
             num_ptr: u32,
             num_len: u32,
             n_ptr: u32,
             n_len: u32|
             -> Result<u64, Trap> {
                super::host_functions::precise_decimal_checked_nth_root(
                    caller, num_ptr, num_len, n_ptr, n_len,
                )
                .map_err(Trap::from)
            },
        );

        let mut linker = <Linker<HostState>>::new();

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
            DECIMAL_CHECKED_ADD_FUNCTION_NAME,
            decimal_checked_add
        );
        linker_define!(
            linker,
            DECIMAL_CHECKED_SUB_FUNCTION_NAME,
            decimal_checked_sub
        );
        linker_define!(
            linker,
            DECIMAL_CHECKED_MUL_FUNCTION_NAME,
            decimal_checked_mul
        );
        linker_define!(
            linker,
            DECIMAL_CHECKED_DIV_FUNCTION_NAME,
            decimal_checked_div
        );
        linker_define!(
            linker,
            DECIMAL_CHECKED_NEG_FUNCTION_NAME,
            decimal_checked_neg
        );
        linker_define!(
            linker,
            DECIMAL_CHECKED_ROUND_FUNCTION_NAME,
            decimal_checked_round
        );
        linker_define!(
            linker,
            DECIMAL_CHECKED_POWI_FUNCTION_NAME,
            decimal_checked_powi
        );
        linker_define!(
            linker,
            DECIMAL_CHECKED_SQRT_FUNCTION_NAME,
            decimal_checked_sqrt
        );
        linker_define!(
            linker,
            DECIMAL_CHECKED_CBRT_FUNCTION_NAME,
            decimal_checked_cbrt
        );
        linker_define!(
            linker,
            DECIMAL_CHECKED_NTH_ROOT_FUNCTION_NAME,
            decimal_checked_nth_root
        );
        linker_define!(
            linker,
            PRECISE_DECIMAL_CHECKED_ADD_FUNCTION_NAME,
            precise_decimal_checked_add
        );
        linker_define!(
            linker,
            PRECISE_DECIMAL_CHECKED_SUB_FUNCTION_NAME,
            precise_decimal_checked_sub
        );
        linker_define!(
            linker,
            PRECISE_DECIMAL_CHECKED_MUL_FUNCTION_NAME,
            precise_decimal_checked_mul
        );
        linker_define!(
            linker,
            PRECISE_DECIMAL_CHECKED_DIV_FUNCTION_NAME,
            precise_decimal_checked_div
        );
        linker_define!(
            linker,
            PRECISE_DECIMAL_CHECKED_NEG_FUNCTION_NAME,
            precise_decimal_checked_neg
        );
        linker_define!(
            linker,
            PRECISE_DECIMAL_CHECKED_ROUND_FUNCTION_NAME,
            precise_decimal_checked_round
        );
        linker_define!(
            linker,
            PRECISE_DECIMAL_CHECKED_POWI_FUNCTION_NAME,
            precise_decimal_checked_powi
        );
        linker_define!(
            linker,
            PRECISE_DECIMAL_CHECKED_SQRT_FUNCTION_NAME,
            precise_decimal_checked_sqrt
        );
        linker_define!(
            linker,
            PRECISE_DECIMAL_CHECKED_CBRT_FUNCTION_NAME,
            precise_decimal_checked_cbrt
        );
        linker_define!(
            linker,
            PRECISE_DECIMAL_CHECKED_NTH_ROOT_FUNCTION_NAME,
            precise_decimal_checked_nth_root
        );

        linker.instantiate(store.as_context_mut(), module)
    }

    pub fn instantiate(&self) -> WasmiInstance {
        let instance = self.template_instance;
        let mut store = self.template_store.clone();
        let memory = match instance.get_export(store.as_context_mut(), EXPORT_MEMORY) {
            Some(Extern::Memory(memory)) => memory,
            _ => panic!("Failed to find memory export"),
        };

        WasmiInstance {
            instance,
            #[allow(clippy::missing_transmute_annotations)]
            store: unsafe { transmute(store) },
            memory,
        }
    }
}
