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
pub const DEFAULT_MIN_TIP_PERCENTAGE: u8 = 0;

/// The maximum value of tip percentage
pub const DEFAULT_MAX_TIP_PERCENTAGE: u8 = u8::MAX;

/// The max epoch range
pub const DEFAULT_MAX_EPOCH_RANGE: u64 = 100;

//==========================
// Transaction execution
//==========================

/// The default system loan amount, used by transaction executor.
pub const DEFAULT_SYSTEM_LOAN: u32 = 1_000_000;

/// The default max call depth, used by transaction executor.
pub const DEFAULT_MAX_CALL_DEPTH: usize = 12;

/// The default cost unit price.
pub const DEFAULT_COST_UNIT_PRICE: u128 = 100_000_000_000u128;
