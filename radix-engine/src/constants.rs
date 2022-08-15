/// The default cost units loaned from the system to bootstrap execution (lock fee).
/// TODO: reduce this number by crazy optimization
pub const DEFAULT_SYSTEM_LOAN: u32 = u32::MAX; // TODO: Temporary fix to be able to publish large packages

/// The default cost unit limit for extracting blueprint ABIs.
pub const DEFAULT_EXTRACT_ABI_COST_UNIT_LIMIT: u32 = 10_000_000;

/// The default max cost unit limit for a transaction, used by transaction validator.
pub const DEFAULT_MAX_COST_UNIT_LIMIT: u32 = u32::MAX; // TODO: Temporary fix to be able to publish large packages

/// The default cost unit limit for a transaction.
pub const DEFAULT_COST_UNIT_LIMIT: u32 = u32::MAX; // TODO: Temporary fix to be able to publish large packages

/// The default cost unit price.
pub const DEFAULT_COST_UNIT_PRICE: &'static str = "0.000001";

/// The default max call depth.
pub const DEFAULT_MAX_CALL_DEPTH: usize = 16;
