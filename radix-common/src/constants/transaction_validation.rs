/// The minimum value of tip percentage
///
/// 100 means 100%
pub const MIN_TIP_PERCENTAGE: u16 = 0;

/// The maximum value of tip percentage
///
/// 100 means 100%
pub const MAX_TIP_PERCENTAGE: u16 = u16::MAX;

/// The max epoch range
/// Should be ~ 1 month. The below is ~30 days given 5 minute epochs.
pub const MAX_EPOCH_RANGE: u64 = 12 * 24 * 30;
