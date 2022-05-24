// We are testing if builtin types panic on overflow. They should as this creates a safe math
// environment for Scrypto
// Whenever these tests fail, Cargo.toml file shold be doulblechecked for following lines:
// [profile.release]
// overflow-checks = true

#![allow(arithmetic_overflow)]

#[cfg(test)]
mod tests {
    macro_rules! overflow {
        ( $( $t:ty, $b:literal ),* ) => {
            $(
                paste::paste! {
                    #[test]
                    #[should_panic]
                    fn [<overflow_test_add_$t>]() {
                        let a: $t = 1;
                        let b = <$t>::MAX;
                        let _c = a + b;
                    }


                    #[test]
                    #[should_panic]
                    fn [<overflow_test_mul_$t>]() {
                        let a: $t = 2;
                        let b = <$t>::MAX;
                        let _c = a * b;
                    }

                    #[test]
                    #[should_panic]
                    fn [<overflow_test_pow_$t>]() {
                        let a: $t = 2;
                        let b = <$t>::MAX;
                        let _c = b.pow(a.try_into().unwrap());
                    }

                    #[test]
                    #[should_panic]
                    fn [<overflow_test_shl_$t>]() {
                        let a: $t = $b + 1;
                        let b: $t = <$t>::MAX;
                        let _c = b << a;
                    }

                    #[test]
                    #[should_panic]
                    fn [<overflow_test_shr_$t>]() {
                        let a: $t = $b + 1;
                        let b: $t = <$t>::MAX;
                        let _c = b >> a;
                    }
                }
            )*
        };
    }
    macro_rules! overflow_signed {
        ($( $t:ty ),*) => {
            $(
                paste::paste! {
                    #[test]
                    #[should_panic]
                    fn [<overflow_test_sub_$t>]() {
                        let a: $t = -1;
                        let b = <$t>::MAX;
                        let _c = b - a;
                    }
                }
            )*
        };
    }

    macro_rules! underflow_unsigned {
        ($( $t:ty ),*) => {
            $(
                paste::paste! {
                    #[test]
                    #[should_panic]
                    fn [<overflow_test_unsigned_sub_$t>]() {
                        let a: $t = 1;
                        let b: $t = 0;
                        let _c = b - a;
                    }
                }
            )*
        };
    }

    overflow! { i8, 8, i16, 16, i32, 32, i64, 64, i128, 128, u8, 8, u16, 16, u32, 32, u64, 64, u128, 128 }
    overflow_signed! { i8, i16, i32, i64, i128 }
    underflow_unsigned!{ u8, u16, u32, u64, u128 }
}
