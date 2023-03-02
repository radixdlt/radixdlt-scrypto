//==========================
// Transaction construction
//==========================

/// The default cost unit limit.
pub const DEFAULT_COST_UNIT_LIMIT: u32 = 100_000_000;

//==========================
// Transaction validation
//==========================

pub const TRANSACTION_VERSION_V1: u8 = 1;
pub const MAX_NUMBER_OF_INTENT_SIGNATURES: usize = 16;

/// The minimum value of cost unit limit
pub const DEFAULT_MIN_COST_UNIT_LIMIT: u32 = 1_000_000;

/// The maximum value of cost unit limit
pub const DEFAULT_MAX_COST_UNIT_LIMIT: u32 = 100_000_000;

/// The minimum value of tip percentage
pub const DEFAULT_MIN_TIP_PERCENTAGE: u16 = 0;

/// The maximum value of tip percentage
pub const DEFAULT_MAX_TIP_PERCENTAGE: u16 = u16::MAX;

/// The max epoch range
pub const DEFAULT_MAX_EPOCH_RANGE: u64 = 100;

/// The max transaction size
pub const MAX_TRANSACTION_SIZE: usize = 1 * 1024 * 1024;

//==========================
// Transaction execution
//==========================

/// The default system loan amount, used by transaction executor.
pub const DEFAULT_SYSTEM_LOAN: u32 = 10_000_000;

/// The default max call depth, used by transaction executor.
pub const DEFAULT_MAX_CALL_DEPTH: usize = 8;

/// The default cost unit price.
pub const DEFAULT_COST_UNIT_PRICE: u128 = 100_000_000_000u128;

/// The default maximum WASM memory per transaction (multiple WASM instances up to call depth).
pub const DEFAULT_MAX_WASM_MEM_PER_TRANSACTION: usize = 10 * 1024 * 1024;

/// The default maximum WASM memory per WASM call frame.
pub const DEFAULT_MAX_WASM_MEM_PER_CALL_FRAME: usize = 4 * 1024 * 1024;

/// The default maximum substates reads count per transaction.
pub const DEFAULT_MAX_SUBSTATE_READS_PER_TRANSACTION: usize = 20_000;

/// The default maximum substates writes count per transaction.
pub const DEFAULT_MAX_SUBSTATE_WRITES_PER_TRANSACTION: usize = 5_000;

/// The default maximum substate read and write size.
/// TODO: Apply this limit in create_node too
pub const DEFAULT_MAX_SUBSTATE_SIZE: usize = 4 * 1024 * 1024;

/// The default maximum invoke input args size.
pub const DEFAULT_MAX_INVOKE_INPUT_SIZE: usize = 4 * 1024 * 1024;
