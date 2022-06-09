// Note: we may consider loading the parameters from configuration/embedders, if customization is needed.

/// The amount of system loan granted to a transaction to boostrap execution.
pub const SYSTEM_LOAN_AMOUNT: u32 = 100_000;

/// The maximum number of cost units that a transaction can consume.
pub const MAX_TRANSACTION_COST: u32 = 2_000_000;

/// The maximum number of recursive function/method calls.
pub const MAX_CALL_DEPTH: u32 = 16;
