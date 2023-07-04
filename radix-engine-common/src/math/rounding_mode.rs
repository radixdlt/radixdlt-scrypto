/// Defines the rounding strategy.
///
/// Following the same naming convention as https://docs.rs/rust_decimal/latest/rust_decimal/enum.RoundingStrategy.html.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    MidpointTowardZero,
    /// The number is rounded to the nearest, and when it is halfway between two others, it's rounded away from zero, e.g. `3.5 -> 4`, `-3.5 -> -4`.
    MidpointAwayFromZero,
    /// The number is rounded to the nearest, and when it is halfway between two others, it's rounded toward the nearest even number. Also known as "Bankers Rounding".
    MidpointNearestEven,
}
