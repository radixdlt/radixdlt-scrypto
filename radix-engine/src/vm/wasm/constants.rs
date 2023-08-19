//=================
// WASM Shim
//=================
pub const CONSUME_BUFFER_FUNCTION_NAME: &str = "consume_buffer";

//=================
// Costing
//=================
pub const CONSUME_WASM_EXECUTION_UNITS_FUNCTION_NAME: &str = "gas";
pub const EXECUTION_COST_UNIT_LIMIT_FUNCTION_NAME: &str = "execution_cost_unit_limit";
pub const EXECUTION_COST_UNIT_PRICE_FUNCTION_NAME: &str = "execution_cost_unit_price";
pub const FINALIZATION_COST_UNIT_LIMIT_FUNCTION_NAME: &str = "finalization_cost_unit_limit";
pub const FINALIZATION_COST_UNIT_PRICE_FUNCTION_NAME: &str = "finalization_cost_unit_price";
pub const USD_PRICE_FUNCTION_NAME: &str = "usd_price";
pub const TIP_PERCENTAGE_FUNCTION_NAME: &str = "tip_percentage";
pub const FEE_BALANCE_FUNCTION_NAME: &str = "fee_balance";

//=================
// Blueprint/Object
//=================
pub const ALLOCATE_GLOBAL_ADDRESS_FUNCTION_NAME: &str = "allocate_global_address";
pub const GET_RESERVATION_ADDRESS_FUNCTION_NAME: &str = "get_reservation_address";
pub const NEW_OBJECT_FUNCTION_NAME: &str = "new_object";
pub const GLOBALIZE_FUNCTION_NAME: &str = "globalize";
pub const CALL_METHOD_FUNCTION_NAME: &str = "call_method";
pub const CALL_FUNCTION_FUNCTION_NAME: &str = "call_function";
pub const GET_BLUEPRINT_ID_FUNCTION_NAME: &str = "get_blueprint_id";
pub const GET_OUTER_OBJECT_FUNCTION_NAME: &str = "get_outer_object";
pub const DROP_OBJECT_FUNCTION_NAME: &str = "drop_object";

//=================
// Key Value Store
//=================
pub const KEY_VALUE_STORE_NEW_FUNCTION_NAME: &str = "kv_store_new";
pub const KEY_VALUE_STORE_OPEN_ENTRY_FUNCTION_NAME: &str = "kv_store_open_entry";
pub const KEY_VALUE_STORE_REMOVE_ENTRY_FUNCTION_NAME: &str = "kv_store_remove_entry";

//=================
// KV Entry Handle
//=================
pub const KEY_VALUE_ENTRY_GET_FUNCTION_NAME: &str = "kv_entry_get";
pub const KEY_VALUE_ENTRY_SET_FUNCTION_NAME: &str = "kv_entry_set";
pub const KEY_VALUE_ENTRY_RELEASE_FUNCTION_NAME: &str = "kv_entry_release";

//=================
// Field Handle
//=================
pub const FIELD_LOCK_READ_FUNCTION_NAME: &str = "field_lock_read";
pub const FIELD_LOCK_WRITE_FUNCTION_NAME: &str = "field_lock_write";
pub const FIELD_LOCK_RELEASE_FUNCTION_NAME: &str = "field_lock_release";

//=================
// Actor
//=================
pub const ACTOR_OPEN_FIELD_FUNCTION_NAME: &str = "actor_open_field";
pub const ACTOR_CALL_MODULE_METHOD_FUNCTION_NAME: &str = "actor_call_module_method";
pub const GET_GLOBAL_ADDRESS_FUNCTION_NAME: &str = "get_global_address";
pub const GET_BLUEPRINT_FUNCTION_NAME: &str = "get_blueprint";
pub const GET_AUTH_ZONE_FUNCTION_NAME: &str = "get_auth_zone";
pub const GET_NODE_ID_FUNCTION_NAME: &str = "get_node_id";

//=================
// Environment
//=================
pub const EMIT_EVENT_FUNCTION_NAME: &str = "emit_event";
pub const EMIT_LOG_FUNCTION_NAME: &str = "emit_log";
pub const GET_TRANSACTION_HASH_FUNCTION_NAME: &str = "get_transaction_hash";
pub const GENERATE_RUID_FUNCTION_NAME: &str = "generate_ruid";
pub const PANIC_FUNCTION_NAME: &str = "panic";

pub const MODULE_ENV_NAME: &str = "env";
pub const EXPORT_MEMORY: &str = "memory";
