pub const TRANSACTION_VERSION_V1: u8 = 1;

pub const MAX_NUMBER_OF_INTENT_SIGNATURES: usize = 16;

pub const MAX_NUMBER_OF_BLOBS: usize = 64;

/// The minimum value of tip percentage
pub const MIN_TIP_PERCENTAGE: u16 = 0;

/// The maximum value of tip percentage
pub const MAX_TIP_PERCENTAGE: u16 = u16::MAX;

/// The max epoch range
/// Should be ~ 1 month. The below is ~30 days given 5 minute epochs.
pub const MAX_EPOCH_RANGE: u64 = 12 * 24 * 30;

/// The max transaction size
pub const MAX_TRANSACTION_SIZE: usize = 1 * 1024 * 1024;
