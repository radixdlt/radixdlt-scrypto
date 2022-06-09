// Note: we may consider loading the parameters from configuration/embedders, if customization is needed.

/// The amount of system loan granted to a transaction to boostrap execution.
/// TODO: implement system loan
pub const SYSTEM_LOAN_AMOUNT: u32 = 5_000_000;

/// The maximum number of cost units that a transaction can consume.
pub const MAX_TRANSACTION_COST: u32 = 5_000_000;

/// The maximum number of cost units to be used when extracting blueprint ABIs.
pub const MAX_EXTRACT_ABI_COST: u32 = 5_000_000;

/// The maximum number of recursive function/method calls.
/// TODO: track call depth
pub const MAX_CALL_DEPTH: u32 = 16;
