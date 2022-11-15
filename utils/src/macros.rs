/// Creates a `Decimal` from literals.
///
/// # Example
/// ```no_run
/// use scrypto::prelude::*;
///
/// let a = dec!(1);
/// let b = dec!("1.1");
/// ```
#[macro_export]
macro_rules! dec {
    ($x:literal) => {
        radix_engine_lib::math::Decimal::from($x)
    };

    ($base:literal, $shift:literal) => {
        // Base can be any type that converts into a Decimal, and shift must support
        // comparison and `-` unary operation, enforced by rustc.
        {
            let base = radix_engine_lib::math::Decimal::from($base);
            if $shift >= 0 {
                base * radix_engine_lib::math::Decimal::try_from(
                    radix_engine_lib::math::I256::from(10u8)
                        .pow(u32::try_from($shift).expect("Shift overflow")),
                )
                .expect("Shift overflow")
            } else {
                base / radix_engine_lib::math::Decimal::try_from(
                    radix_engine_lib::math::I256::from(10u8)
                        .pow(u32::try_from(-$shift).expect("Shift overflow")),
                )
                .expect("Shift overflow")
            }
        }
    };
}

/// Creates a safe integer from literals.
/// You must specify the type of the
/// integer you want to create.
///
/// # Example
/// ```no_run
/// use scrypto::prelude::*;
///
/// let a: I256 = i!(21);
/// let b: U512 = i!("1156");
/// ```
#[macro_export]
macro_rules! i {
    ($x:expr) => {
        $x.try_into().expect("Parse Error")
    };
}

/// Creates a `PreciseDecimal` from literals.
///
/// # Example
/// ```no_run
/// use scrypto::prelude::*;
///
/// let a = pdec!(1);
/// let b = pdec!("1.1");
/// ```
#[macro_export]
macro_rules! pdec {
    ($x:literal) => {
        radix_engine_lib::math::PreciseDecimal::from($x)
    };

    ($base:literal, $shift:literal) => {
        // Base can be any type that converts into a PreciseDecimal, and shift must support
        // comparison and `-` unary operation, enforced by rustc.
        {
            let base = radix_engine_lib::math::PreciseDecimal::from($base);
            if $shift >= 0 {
                base * radix_engine_lib::math::PreciseDecimal::try_from(
                    radix_engine_lib::math::I512::from(10u8)
                        .pow(u32::try_from($shift).expect("Shift overflow")),
                )
                .expect("Shift overflow")
            } else {
                base / radix_engine_lib::math::PreciseDecimal::try_from(
                    radix_engine_lib::math::I512::from(10u8)
                        .pow(u32::try_from(-$shift).expect("Shift overflow")),
                )
                .expect("Shift overflow")
            }
        }
    };
}
