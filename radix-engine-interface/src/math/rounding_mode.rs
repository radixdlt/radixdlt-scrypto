/// Defines how rounding should be done.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RoundingMode {
    /// Rounds towards positive infinity, e.g. `3.1 -> 4`, `-3.1 -> -3`.
    TowardsPositiveInfinity,
    /// Rounds towards negative infinity, e.g. `3.1 -> 3`, `-3.1 -> -4`.
    TowardsNegativeInfinity,
    /// Rounds towards zero, e.g. `3.1 -> 3`, `-3.1 -> -3`.
    TowardsZero,
    /// Rounds away from zero, e.g. `3.1 -> 4`, `-3.1 -> -4`.
    AwayFromZero,
    /// Rounds to the nearest and when a number is halfway between two others, it's rounded towards zero, e.g. `3.5 -> 3`, `-3.5 -> -3`.
    TowardsNearestAndHalfTowardsZero,
    /// Rounds to the nearest and when a number is halfway between two others, it's rounded away zero, e.g. `3.5 -> 4`, `-3.5 -> -4`.
    TowardsNearestAndHalfAwayFromZero,
}
