//=================
// Blueprint
//=================
pub const BLUEPRINT_CALL_FUNCTION_NAME: &str = "blueprint_call";

//=================
// Address
//=================
pub const ADDRESS_ALLOCATE_FUNCTION_NAME: &str = "address_allocate";
pub const ADDRESS_GET_RESERVATION_ADDRESS_FUNCTION_NAME: &str = "address_get_reservation_address";

//=================
// Object
//=================
pub const OBJECT_NEW_FUNCTION_NAME: &str = "object_new";
pub const OBJECT_GLOBALIZE_FUNCTION_NAME: &str = "object_globalize";
pub const OBJECT_GET_BLUEPRINT_ID_FUNCTION_NAME: &str = "object_get_blueprint_id";
pub const OBJECT_GET_OUTER_OBJECT_FUNCTION_NAME: &str = "object_get_outer_object";
pub const OBJECT_CALL_FUNCTION_NAME: &str = "object_call";
pub const OBJECT_CALL_DIRECT_FUNCTION_NAME: &str = "object_call_direct";
pub const OBJECT_CALL_MODULE_FUNCTION_NAME: &str = "object_call_module";

//=================
// Actor
//=================
pub const ACTOR_OPEN_FIELD_FUNCTION_NAME: &str = "actor_open_field";
pub const ACTOR_CALL_MODULE_FUNCTION_NAME: &str = "actor_call_module";
pub const ACTOR_GET_GLOBAL_ADDRESS_FUNCTION_NAME: &str = "actor_get_global_address";
pub const ACTOR_GET_BLUEPRINT_ID_FUNCTION_NAME: &str = "actor_get_blueprint_id";
pub const ACTOR_GET_AUTH_ZONE_FUNCTION_NAME: &str = "actor_get_auth_zone";
pub const ACTOR_GET_OBJECT_ID_FUNCTION_NAME: &str = "actor_get_object_id";
pub const ACTOR_EMIT_EVENT_FUNCTION_NAME: &str = "actor_emit_event";

//=================
// Key Value Store
//=================
pub const KEY_VALUE_STORE_NEW_FUNCTION_NAME: &str = "kv_store_new";
pub const KEY_VALUE_STORE_OPEN_ENTRY_FUNCTION_NAME: &str = "kv_store_open_entry";
pub const KEY_VALUE_STORE_REMOVE_ENTRY_FUNCTION_NAME: &str = "kv_store_remove_entry";

//=================
// KV Entry
//=================
pub const KEY_VALUE_ENTRY_READ_FUNCTION_NAME: &str = "kv_entry_read";
pub const KEY_VALUE_ENTRY_WRITE_FUNCTION_NAME: &str = "kv_entry_write";
pub const KEY_VALUE_ENTRY_REMOVE_FUNCTION_NAME: &str = "kv_entry_remove";
pub const KEY_VALUE_ENTRY_CLOSE_FUNCTION_NAME: &str = "kv_entry_close";

//=================
// Field Entry
//=================
pub const FIELD_ENTRY_READ_FUNCTION_NAME: &str = "field_entry_read";
pub const FIELD_ENTRY_WRITE_FUNCTION_NAME: &str = "field_entry_write";
pub const FIELD_ENTRY_CLOSE_FUNCTION_NAME: &str = "field_entry_close";

//=================
// Costing
//=================
pub const COSTING_CONSUME_WASM_EXECUTION_UNITS_FUNCTION_NAME: &str = "gas";
pub const COSTING_EXECUTION_COST_UNIT_LIMIT_FUNCTION_NAME: &str =
    "costing_execution_cost_unit_limit";
pub const COSTING_EXECUTION_COST_UNIT_PRICE_FUNCTION_NAME: &str =
    "costing_execution_cost_unit_price";
pub const COSTING_FINALIZATION_COST_UNIT_LIMIT_FUNCTION_NAME: &str =
    "costing_finalization_cost_unit_limit";
pub const COSTING_FINALIZATION_COST_UNIT_PRICE_FUNCTION_NAME: &str =
    "costing_finalization_cost_unit_price";
pub const COSTING_USD_PRICE_FUNCTION_NAME: &str = "costing_usd_price";
pub const COSTING_TIP_PERCENTAGE_FUNCTION_NAME: &str = "costing_tip_percentage";
pub const COSTING_FEE_BALANCE_FUNCTION_NAME: &str = "costing_fee_balance";

//=================
// System
//=================
pub const SYS_LOG_FUNCTION_NAME: &str = "sys_log";
pub const SYS_GET_TRANSACTION_HASH_FUNCTION_NAME: &str = "sys_get_transaction_hash";
pub const SYS_GENERATE_RUID_FUNCTION_NAME: &str = "sys_generate_ruid";
pub const SYS_PANIC_FUNCTION_NAME: &str = "sys_panic";

//=================
// WASM Shim
//=================
pub const BUFFER_CONSUME_FUNCTION_NAME: &str = "buffer_consume";

pub const MODULE_ENV_NAME: &str = "env";
pub const EXPORT_MEMORY: &str = "memory";
