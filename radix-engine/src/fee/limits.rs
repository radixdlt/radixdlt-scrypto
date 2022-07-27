// Note: we may consider loading the parameters from configuration/embedders, if customization is needed.

/// The amount of system loan granted to a transaction to boostrap execution.
/// TODO: implement system loan
pub const DEFAULT_SYSTEM_LOAN_AMOUNT: u32 = 10_000_000;

/// The maximum number of cost units that a transaction can consume.
pub const DEFAULT_MAX_TRANSACTION_COST: u32 = 10_000_000;

/// The maximum number of cost units to be used when extracting blueprint ABIs.
pub const MAX_EXTRACT_ABI_COST: u32 = 10_000_000;

/// The maximum number of recursive function/method calls.
pub const MAX_CALL_DEPTH: usize = 16;
