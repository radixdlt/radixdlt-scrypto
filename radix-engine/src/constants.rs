/// The default cost units loaned from the system to bootstrap execution (lock fee).
pub const DEFAULT_SYSTEM_LOAN: u32 = 1_000_000;

/// The default max cost unit limit for a transaction, used by transaction validator.
pub const DEFAULT_MAX_COST_UNIT_LIMIT: u32 = 100_000_000;

/// The default cost unit limit for a transaction.
pub const DEFAULT_COST_UNIT_LIMIT: u32 = 100_000_000;

/// The default cost unit price.
pub const DEFAULT_COST_UNIT_PRICE: &'static str = "0.0000001";

/// The default max call depth.
pub const DEFAULT_MAX_CALL_DEPTH: usize = 16;

pub const EXTRACT_ABI_CREDIT: u32 = 100_000_000;
pub const PREVIEW_CREDIT: u32 = 100_000_000;
pub const GENESIS_CREATION_CREDIT: u32 = 100_000_000;
