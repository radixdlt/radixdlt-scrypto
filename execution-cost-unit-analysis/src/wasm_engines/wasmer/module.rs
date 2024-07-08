use super::env::*;
use super::instance::WasmerInstance;
use radix_common::prelude::*;
use radix_engine::vm::wasm::*;
use sbor::rust::sync::*;
use wasmer::*;

// IMPORTANT:
// The below integration of Wasmer is not yet checked rigorously enough for production use
// TODO: Address the below issues before considering production use.

/// A `WasmerModule` defines a parsed WASM module, which is a template which can be instantiated.
///
/// Unlike `WasmerInstance`, this is correctly `Send + Sync` - which is good, because this is the
/// thing which is cached in the ScryptoInterpreter caches.
pub struct WasmerModule {
    pub(super) module: Module,
    #[allow(dead_code)]
    pub(super) code_size_bytes: usize,
}

impl WasmerModule {
    pub fn instantiate(&self) -> WasmerInstance {
        // env
        let env = WasmerInstanceEnv {
            instance: LazyInit::new(),
            runtime_ptr: Arc::new(Mutex::new(0)),
        };

        // imports
        let import_object = imports! {
            MODULE_ENV_NAME => {
                BLUEPRINT_CALL_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::blueprint_call
                ),
                ADDRESS_ALLOCATE_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::address_allocate
                ),
                ADDRESS_GET_RESERVATION_ADDRESS_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::address_get_reservation_address
                ),
                OBJECT_NEW_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::object_new
                ),
                OBJECT_GLOBALIZE_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::object_globalize
                ),
                OBJECT_INSTANCE_OF_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::object_instance_of
                ),
                OBJECT_GET_BLUEPRINT_ID_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::object_get_blueprint_id
                ),
                OBJECT_GET_OUTER_OBJECT_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::object_get_outer_object
                ),
                OBJECT_CALL_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::object_call
                ),
                OBJECT_CALL_MODULE_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::object_call_module
                ),
                OBJECT_CALL_DIRECT_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::object_call_direct
                ),
                KEY_VALUE_STORE_NEW_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::key_value_store_new
                ),
                KEY_VALUE_STORE_OPEN_ENTRY_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::key_value_store_open_entry
                ),
                KEY_VALUE_STORE_REMOVE_ENTRY_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::key_value_store_remove_entry
                ),
                KEY_VALUE_ENTRY_READ_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::key_value_entry_read
                ),
                KEY_VALUE_ENTRY_WRITE_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::key_value_entry_write
                ),
                KEY_VALUE_ENTRY_REMOVE_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::key_value_entry_remove
                ),
                KEY_VALUE_ENTRY_CLOSE_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::key_value_entry_close
                ),
                FIELD_ENTRY_READ_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::field_entry_read
                ),
                FIELD_ENTRY_WRITE_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::field_entry_write
                ),
                FIELD_ENTRY_CLOSE_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::field_entry_close
                ),
                ACTOR_OPEN_FIELD_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::actor_open_field
                ),
                ACTOR_GET_OBJECT_ID_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::actor_get_node_id
                ),
                ACTOR_GET_PACKAGE_ADDRESS_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::actor_get_package_address
                ),
                ACTOR_GET_BLUEPRINT_NAME_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::actor_get_blueprint_name
                ),
                ACTOR_EMIT_EVENT_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::actor_emit_event
                ),
                COSTING_CONSUME_WASM_EXECUTION_UNITS_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::consume_wasm_execution_units
                ),
                COSTING_GET_EXECUTION_COST_UNIT_LIMIT_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::costing_get_execution_cost_unit_limit
                ),
                COSTING_GET_EXECUTION_COST_UNIT_PRICE_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::costing_get_execution_cost_unit_price
                ),
                COSTING_GET_FINALIZATION_COST_UNIT_LIMIT_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::costing_get_finalization_cost_unit_limit
                ),
                COSTING_GET_FINALIZATION_COST_UNIT_PRICE_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::costing_get_finalization_cost_unit_price
                ),
                COSTING_GET_USD_PRICE_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::costing_get_usd_price
                ),
                COSTING_GET_TIP_PERCENTAGE_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::costing_get_tip_percentage
                ),
                COSTING_GET_FEE_BALANCE_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::costing_get_fee_balance
                ),
                SYS_LOG_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::sys_log
                ),
                SYS_BECH32_ENCODE_ADDRESS_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::sys_bech32_encode_address
                ),
                SYS_PANIC_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::sys_panic
                ),
                SYS_GET_TRANSACTION_HASH_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::sys_get_transaction_hash
                ),
                SYS_GENERATE_RUID_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::sys_generate_ruid
                ),
                BUFFER_CONSUME_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::buffer_consume
                ),
                CRYPTO_UTILS_BLS12381_V1_VERIFY_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::bls12381_v1_verify
                ),
                CRYPTO_UTILS_BLS12381_V1_AGGREGATE_VERIFY_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::bls12381_v1_aggregate_verify
                ),
                CRYPTO_UTILS_BLS12381_V1_FAST_AGGREGATE_VERIFY_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::bls12381_v1_fast_aggregate_verify
                ),
                CRYPTO_UTILS_BLS12381_G2_SIGNATURE_AGGREGATE_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::bls12381_g2_signature_aggregate
                ),
                CRYPTO_UTILS_KECCAK256_HASH_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::keccak256_hash
                ),
                DECIMAL_CHECKED_ADD_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::decimal_checked_add
                ),
                DECIMAL_CHECKED_SUB_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::decimal_checked_sub
                ),
                DECIMAL_CHECKED_MUL_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::decimal_checked_mul
                ),
                DECIMAL_CHECKED_DIV_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::decimal_checked_div
                ),
                DECIMAL_CHECKED_NEG_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::decimal_checked_neg
                ),
                DECIMAL_CHECKED_ROUND_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::decimal_checked_round
                ),
                DECIMAL_CHECKED_POWI_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::decimal_checked_powi
                ),
                DECIMAL_CHECKED_SQRT_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::decimal_checked_sqrt
                ),
                DECIMAL_CHECKED_CBRT_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::decimal_checked_cbrt
                ),
                DECIMAL_CHECKED_NTH_ROOT_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::decimal_checked_nth_root
                ),
                PRECISE_DECIMAL_CHECKED_ADD_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::precise_decimal_checked_add
                ),
                PRECISE_DECIMAL_CHECKED_SUB_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::precise_decimal_checked_sub
                ),
                PRECISE_DECIMAL_CHECKED_MUL_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::precise_decimal_checked_mul
                ),
                PRECISE_DECIMAL_CHECKED_DIV_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::precise_decimal_checked_div
                ),
                PRECISE_DECIMAL_CHECKED_NEG_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::precise_decimal_checked_neg
                ),
                PRECISE_DECIMAL_CHECKED_ROUND_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::precise_decimal_checked_round
                ),
                PRECISE_DECIMAL_CHECKED_POWI_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::precise_decimal_checked_powi
                ),
                PRECISE_DECIMAL_CHECKED_SQRT_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::precise_decimal_checked_sqrt
                ),
                PRECISE_DECIMAL_CHECKED_CBRT_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::precise_decimal_checked_cbrt
                ),
                PRECISE_DECIMAL_CHECKED_NTH_ROOT_FUNCTION_NAME => Function::new_native_with_env(
                    self.module.store(),
                    env.clone(),
                    super::host_functions::precise_decimal_checked_nth_root
                ),
            }
        };

        // instantiate
        let instance =
            Instance::new(&self.module, &import_object).expect("Failed to instantiate module");

        WasmerInstance {
            instance,
            runtime_ptr: env.runtime_ptr,
        }
    }
}
