#[cfg(feature = "fuzzing")]
use arbitrary::Arbitrary;
use core::cmp::Ordering;
use sbor::Sbor;
#[cfg(feature = "fuzzing")]
use serde::{Deserialize, Serialize};

/// Defines the rounding strategy.
///
/// Following the same naming convention as https://docs.rs/rust_decimal/latest/rust_decimal/enum.RoundingStrategy.html.
#[cfg_attr(feature = "fuzzing", derive(Arbitrary, Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Sbor)]
pub enum RoundingMode {
    /// The number is always rounded toward positive infinity, e.g. `3.1 -> 4`, `-3.1 -> -3`.
    ToPositiveInfinity,
    /// The number is always rounded toward negative infinity, e.g. `3.1 -> 3`, `-3.1 -> -4`.
    ToNegativeInfinity,
    /// The number is always rounded toward zero, e.g. `3.1 -> 3`, `-3.1 -> -3`.
    ToZero,
    /// The number is always rounded away from zero, e.g. `3.1 -> 4`, `-3.1 -> -4`.
    AwayFromZero,

    /// The number is rounded to the nearest, and when it is halfway between two others, it's rounded toward zero, e.g. `3.5 -> 3`, `-3.5 -> -3`.
    ToNearestMidpointTowardZero,
    /// The number is rounded to the nearest, and when it is halfway between two others, it's rounded away from zero, e.g. `3.5 -> 4`, `-3.5 -> -4`.
    ToNearestMidpointAwayFromZero,
    /// The number is rounded to the nearest, and when it is halfway between two others, it's rounded toward the nearest even number. Also known as "Bankers Rounding".
    ToNearestMidpointToEven,
}

/// The resolved rounding strategy internal to the round method
pub(crate) enum ResolvedRoundingStrategy {
    RoundUp,
    RoundDown,
    RoundToEven,
}

impl ResolvedRoundingStrategy {
    pub fn from_mode(
        mode: RoundingMode,
        is_positive: bool,
        compare_to_midpoint: impl FnOnce() -> Ordering,
    ) -> Self {
        match mode {
            RoundingMode::ToPositiveInfinity => ResolvedRoundingStrategy::RoundUp,
            RoundingMode::ToNegativeInfinity => ResolvedRoundingStrategy::RoundDown,
            RoundingMode::ToZero => ResolvedRoundingStrategy::towards_zero(is_positive),
            RoundingMode::AwayFromZero => ResolvedRoundingStrategy::away_from_zero(is_positive),
            RoundingMode::ToNearestMidpointTowardZero => Self::from_midpoint_ordering(
                compare_to_midpoint(),
                ResolvedRoundingStrategy::towards_zero(is_positive),
            ),
            RoundingMode::ToNearestMidpointAwayFromZero => Self::from_midpoint_ordering(
                compare_to_midpoint(),
                ResolvedRoundingStrategy::away_from_zero(is_positive),
            ),
            RoundingMode::ToNearestMidpointToEven => Self::from_midpoint_ordering(
                compare_to_midpoint(),
                ResolvedRoundingStrategy::RoundToEven,
            ),
        }
    }

    fn from_midpoint_ordering(ordering: Ordering, equal_strategy: Self) -> Self {
        match ordering {
            Ordering::Less => Self::RoundDown,
            Ordering::Equal => equal_strategy,
            Ordering::Greater => Self::RoundUp,
        }
    }

    fn towards_zero(is_positive: bool) -> Self {
        if is_positive {
            Self::RoundDown
        } else {
            Self::RoundUp
        }
    }

    fn away_from_zero(is_positive: bool) -> Self {
        if is_positive {
            Self::RoundUp
        } else {
            Self::RoundDown
        }
    }
}
