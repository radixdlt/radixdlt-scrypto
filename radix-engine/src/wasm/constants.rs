pub const CONSUME_BUFFER_FUNCTION_ID: usize = 0;
pub const CONSUME_BUFFER_FUNCTION_NAME: &str = "consume_buffer";
pub const CALL_METHOD_FUNCTION_ID: usize = 1;
pub const CALL_METHOD_FUNCTION_NAME: &str = "call_method";
pub const CALL_FUNCTION_FUNCTION_ID: usize = 2;
pub const CALL_FUNCTION_FUNCTION_NAME: &str = "call_function";
pub const CALL_NATIVE_FUNCTION_ID: usize = 3;
pub const CALL_NATIVE_FUNCTION_NAME: &str = "call_native";
pub const NEW_PACKAGE_FUNCTION_ID: usize = 13;
pub const NEW_PACKAGE_FUNCTION_NAME: &str = "new_package";
pub const NEW_COMPONENT_FUNCTION_ID: usize = 14;
pub const NEW_COMPONENT_FUNCTION_NAME: &str = "new_component";
pub const NEW_KEY_VALUE_STORE_FUNCTION_ID: usize = 16;
pub const NEW_KEY_VALUE_STORE_FUNCTION_NAME: &str = "new_key_value_store";
pub const GLOBALIZE_COMPONENT_FUNCTION_ID: usize = 15;
pub const GLOBALIZE_COMPONENT_FUNCTION_NAME: &str = "globalize_component";
pub const GET_VISIBLE_NODES_FUNCTION_ID: usize = 5;
pub const GET_VISIBLE_NODES_FUNCTION_NAME: &str = "get_visible_nodes";
pub const DROP_NODE_FUNCTION_ID: usize = 6;
pub const DROP_NODE_FUNCTION_NAME: &str = "drop_node";
pub const LOCK_SUBSTATE_FUNCTION_ID: usize = 7;
pub const LOCK_SUBSTATE_FUNCTION_NAME: &str = "lock_substate";
pub const READ_SUBSTATE_FUNCTION_ID: usize = 8;
pub const READ_SUBSTATE_FUNCTION_NAME: &str = "read_substate";
pub const WRITE_SUBSTATE_FUNCTION_ID: usize = 9;
pub const WRITE_SUBSTATE_FUNCTION_NAME: &str = "write_substate";
pub const DROP_LOCK_FUNCTION_ID: usize = 10;
pub const DROP_LOCK_FUNCTION_NAME: &str = "drop_lock";
pub const GET_ACTOR_FUNCTION_ID: usize = 11;
pub const GET_ACTOR_FUNCTION_NAME: &str = "get_actor";
pub const CONSUME_COST_UNITS_FUNCTION_ID: usize = 12;
pub const CONSUME_COST_UNITS_FUNCTION_NAME: &str = "gas";

pub const MODULE_ENV_NAME: &str = "env";
pub const EXPORT_MEMORY: &str = "memory";

/// The maximum initial memory size: `64 Pages * 64 KiB per Page = 4 MiB`
pub const DEFAULT_MAX_INITIAL_MEMORY_SIZE_PAGES: u32 = 64;

/// The maximum initial table size
pub const DEFAULT_MAX_INITIAL_TABLE_SIZE: u32 = 1024;

/// The max number of labels of a table jump, excluding the default
pub const DEFAULT_MAX_NUMBER_OF_BR_TABLE_TARGETS: u32 = 256;

/// The max number of global variables
pub const DEFAULT_MAX_NUMBER_OF_GLOBALS: u32 = 512;

/// The max number of functions
pub const DEFAULT_MAX_NUMBER_OF_FUNCTIONS: u32 = 64 * 1024;
