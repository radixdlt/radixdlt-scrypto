use radix_engine_constants::DEFAULT_MAX_WASM_MEM_PER_CALL_FRAME;

pub const CONSUME_BUFFER_FUNCTION_ID: usize = 0x10;
pub const CONSUME_BUFFER_FUNCTION_NAME: &str = "consume_buffer";
pub const CONSUME_COST_UNITS_FUNCTION_ID: usize = 0x11;
pub const CONSUME_COST_UNITS_FUNCTION_NAME: &str = "gas";

pub const NEW_OBJECT_FUNCTION_ID: usize = 0x30;
pub const NEW_OBJECT_FUNCTION_NAME: &str = "new_object";
pub const GLOBALIZE_OBJECT_FUNCTION_ID: usize = 0x31;
pub const GLOBALIZE_OBJECT_FUNCTION_NAME: &str = "globalize_object";
pub const GLOBALIZE_OBJECT_WITH_ADDRESS_FUNCTION_ID: usize = 0x32;
pub const GLOBALIZE_OBJECT_WITH_ADDRESS_FUNCTION_NAME: &str = "globalize_with_address";
pub const CALL_METHOD_FUNCTION_ID: usize = 0x33;
pub const CALL_METHOD_FUNCTION_NAME: &str = "call_method";
pub const CALL_FUNCTION_FUNCTION_ID: usize = 0x34;
pub const CALL_FUNCTION_FUNCTION_NAME: &str = "call_function";
pub const GET_OBJECT_INFO_FUNCTION_ID: usize = 0x35;
pub const GET_OBJECT_INFO_FUNCTION_NAME: &str = "get_object_info";
pub const DROP_OBJECT_FUNCTION_ID: usize = 0x36;
pub const DROP_OBJECT_FUNCTION_NAME: &str = "drop_object";

pub const KEY_VALUE_STORE_NEW_FUNCTION_ID: usize = 0x37;
pub const KEY_VALUE_STORE_NEW_FUNCTION_NAME: &str = "kv_store_new";
pub const KEY_VALUE_STORE_GET_INFO_FUNCTION_ID: usize = 0x38;
pub const KEY_VALUE_STORE_GET_INFO_FUNCTION_NAME: &str = "kv_store_get_info";
pub const KEY_VALUE_STORE_LOCK_ENTRY_FUNCTION_ID: usize = 0x39;
pub const KEY_VALUE_STORE_LOCK_ENTRY_FUNCTION_NAME: &str = "kv_store_lock_entry";
pub const KEY_VALUE_STORE_REMOVE_ENTRY_FUNCTION_ID: usize = 0x3a;
pub const KEY_VALUE_STORE_REMOVE_ENTRY_FUNCTION_NAME: &str = "kv_store_remove_entry";

pub const KEY_VALUE_ENTRY_GET_FUNCTION_ID: usize = 0x3b;
pub const KEY_VALUE_ENTRY_GET_FUNCTION_NAME: &str = "kv_entry_get";
pub const KEY_VALUE_ENTRY_SET_FUNCTION_ID: usize = 0x3c;
pub const KEY_VALUE_ENTRY_SET_FUNCTION_NAME: &str = "kv_entry_set";
pub const KEY_VALUE_ENTRY_RELEASE_FUNCTION_ID: usize = 0x3d;
pub const KEY_VALUE_ENTRY_RELEASE_FUNCTION_NAME: &str = "kv_entry_release";

pub const ACTOR_LOCK_FIELD_FUNCTION_ID: usize = 0x43;
pub const ACTOR_LOCK_FIELD_FUNCTION_NAME: &str = "actor_lock_field";

pub const FIELD_LOCK_READ_FUNCTION_ID: usize = 0x44;
pub const FIELD_LOCK_READ_FUNCTION_NAME: &str = "field_lock_read";
pub const FIELD_LOCK_WRITE_FUNCTION_ID: usize = 0x45;
pub const FIELD_LOCK_WRITE_FUNCTION_NAME: &str = "field_lock_write";
pub const FIELD_LOCK_RELEASE_FUNCTION_ID: usize = 0x46;
pub const FIELD_LOCK_RELEASE_FUNCTION_NAME: &str = "field_lock_release";

pub const EMIT_EVENT_FUNCTION_ID: usize = 0x50;
pub const EMIT_EVENT_FUNCTION_NAME: &str = "emit_event";
pub const LOG_FUNCTION_ID: usize = 0x51;
pub const LOG_FUNCTION_NAME: &str = "log_message";
pub const GET_TRANSACTION_HASH_FUNCTION_ID: usize = 0x52;
pub const GET_TRANSACTION_HASH_FUNCTION_NAME: &str = "get_transaction_hash";
pub const GENERATE_UUID_FUNCTION_ID: usize = 0x53;
pub const GENERATE_UUID_FUNCTION_NAME: &str = "generate_uuid";
pub const GET_GLOBAL_ADDRESS_FUNCTION_ID: usize = 0x54;
pub const GET_GLOBAL_ADDRESS_FUNCTION_NAME: &str = "get_global_address";
pub const GET_BLUEPRINT_FUNCTION_ID: usize = 0x55;
pub const GET_BLUEPRINT_FUNCTION_NAME: &str = "get_blueprint";
pub const GET_AUTH_ZONE_FUNCTION_ID: usize = 0x56;
pub const GET_AUTH_ZONE_FUNCTION_NAME: &str = "get_auth_zone";
pub const ASSERT_ACCESS_RULE_FUNCTION_ID: usize = 0x57;
pub const ASSERT_ACCESS_RULE_FUNCTION_NAME: &str = "assert_access_rule";

pub const MODULE_ENV_NAME: &str = "env";
pub const EXPORT_MEMORY: &str = "memory";

pub const WASM_MEMORY_PAGE_SIZE: u32 = 64 * 1024;

/// The maximum initial memory size calculated basing on Wasm call frame size: 4MiB
pub const DEFAULT_MAX_INITIAL_MEMORY_SIZE_PAGES: u32 =
    DEFAULT_MAX_WASM_MEM_PER_CALL_FRAME as u32 / WASM_MEMORY_PAGE_SIZE;

/// The maximum initial table size
pub const DEFAULT_MAX_INITIAL_TABLE_SIZE: u32 = 1024;

/// The max number of labels of a table jump, excluding the default
pub const DEFAULT_MAX_NUMBER_OF_BR_TABLE_TARGETS: u32 = 256;

/// The max number of global variables
pub const DEFAULT_MAX_NUMBER_OF_GLOBALS: u32 = 512;

/// The max number of functions
pub const DEFAULT_MAX_NUMBER_OF_FUNCTIONS: u32 = 64 * 1024;
