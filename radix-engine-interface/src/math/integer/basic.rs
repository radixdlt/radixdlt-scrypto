use super::*;

macro_rules! checked_int_impl_large {
    (type_id: $t:ident, bytes_len: $bytes_len:literal, MIN: $min: expr, MAX: $max: expr) => {
        paste! {
            impl $t {
                /// Returns the smallest value that can be represented by this integer type.
                ///
                pub const MIN: Self = $min;

                /// Returns the largest value that can be represented by this integer type.
                ///
                pub const MAX: Self = $max;

                /// Returns the size of this integer type in bits.
                ///
                pub const BITS: u32 = $bytes_len * 8;

            }
        }
    };
}

macro_rules! checked_unsigned_large {
    ($($t:ident, $bytes_len:literal),*) => {
        $(
            checked_int_impl_large! {
                type_id: $t,
                bytes_len: $bytes_len,
                MIN: $t([0u8; $bytes_len]),
                MAX: $t([0xffu8; $bytes_len])
            }
        )*
    }
}

macro_rules! checked_signed_large {
    ( $($t:ident, $bytes_len:literal),* ) => {
        $(
            checked_int_impl_large! {
                type_id: $t,
                bytes_len: $bytes_len,
                MIN: {
                    let mut arr = [0u8; $bytes_len];
                    arr[$bytes_len - 1] = 0x80;
                    $t(arr)
                },
                MAX: {
                    let mut arr = [0xff; $bytes_len];
                    arr[$bytes_len - 1] = 0x7f;
                    $t(arr)
                }
            }
        )*
    }
}

checked_signed_large! {
    I256, 32,
    I320, 40,
    I384, 48,
    I512, 64,
    I728, 91
}

checked_unsigned_large! {
    U256, 32,
    U320, 40,
    U384, 48,
    U512, 64,
    U728, 91
}

macro_rules! checked_int_impl_small {
    ($($t:ident),*) => {$(
        paste! {
            impl $t {
                /// Returns the smallest value that can be represented by this integer type.
                ///
                pub const MIN: Self = Self([<$t:lower>]::MIN);

                /// Returns the largest value that can be represented by this integer type.
                ///
                pub const MAX: Self = Self([<$t:lower>]::MAX);

                /// Returns the size of this integer type in bits.
                ///
                pub const BITS: u32 = [<$t:lower>]::BITS;
            }
        }
        )*
    }
}

checked_int_impl_small! { I8, I16, I32, I64, I128, U8, U16, U32, U64, U128 }
