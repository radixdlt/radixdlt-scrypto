use radix_engine_constants::DEFAULT_MAX_WASM_MEM_PER_CALL_FRAME;

pub const CONSUME_BUFFER_FUNCTION_ID: usize = 0x10;
pub const CONSUME_BUFFER_FUNCTION_NAME: &str = "consume_buffer";
pub const CONSUME_COST_UNITS_FUNCTION_ID: usize = 0x11;
pub const CONSUME_COST_UNITS_FUNCTION_NAME: &str = "gas";

pub const NEW_COMPONENT_FUNCTION_ID: usize = 0x30;
pub const NEW_COMPONENT_FUNCTION_NAME: &str = "new_component";
pub const NEW_KEY_VALUE_STORE_FUNCTION_ID: usize = 0x31;
pub const NEW_KEY_VALUE_STORE_FUNCTION_NAME: &str = "new_key_value_store";
pub const GLOBALIZE_COMPONENT_FUNCTION_ID: usize = 0x32;
pub const GLOBALIZE_COMPONENT_FUNCTION_NAME: &str = "globalize_component";
pub const CALL_METHOD_FUNCTION_ID: usize = 0x33;
pub const CALL_METHOD_FUNCTION_NAME: &str = "call_method";
pub const CALL_FUNCTION_FUNCTION_ID: usize = 0x34;
pub const CALL_FUNCTION_FUNCTION_NAME: &str = "call_function";
pub const GET_COMPONENT_TYPE_INFO_FUNCTION_ID: usize = 0x35;
pub const GET_COMPONENT_TYPE_INFO_FUNCTION_NAME: &str = "get_component_type_info";

pub const LOCK_SUBSTATE_FUNCTION_ID: usize = 0x40;
pub const LOCK_SUBSTATE_FUNCTION_NAME: &str = "lock_substate";
pub const READ_SUBSTATE_FUNCTION_ID: usize = 0x41;
pub const READ_SUBSTATE_FUNCTION_NAME: &str = "read_substate";
pub const WRITE_SUBSTATE_FUNCTION_ID: usize = 0x42;
pub const WRITE_SUBSTATE_FUNCTION_NAME: &str = "write_substate";
pub const DROP_LOCK_FUNCTION_ID: usize = 0x43;
pub const DROP_LOCK_FUNCTION_NAME: &str = "drop_lock";

pub const EMIT_EVENT_FUNCTION_ID: usize = 0x50;
pub const EMIT_EVENT_FUNCTION_NAME: &str = "emit_event";
pub const LOG_FUNCTION_ID: usize = 0x51;
pub const LOG_FUNCTION_NAME: &str = "log_message";
pub const GET_TRANSACTION_HASH_FUNCTION_ID: usize = 0x52;
pub const GET_TRANSACTION_HASH_FUNCTION_NAME: &str = "get_transaction_hash";
pub const GENERATE_UUID_FUNCTION_ID: usize = 0x53;
pub const GENERATE_UUID_FUNCTION_NAME: &str = "generate_uuid";

// Under active refactoring
pub const GET_ACTOR_FUNCTION_ID: usize = 0xf0;
pub const GET_ACTOR_FUNCTION_NAME: &str = "get_actor";
pub const DROP_NODE_FUNCTION_ID: usize = 0xf1;
pub const DROP_NODE_FUNCTION_NAME: &str = "drop_node";

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
