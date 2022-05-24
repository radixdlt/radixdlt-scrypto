// We are testing if builtin types panic on overflow. They should as this creates a safe math
// environment for Scrypto
#[cfg(test)]
mod tests {
    macro_rules! overflow {
        ($t:ty, $b:literal) => (
            paste::paste! {
                    #[test]
                    #[should_panic]
                    fn [<overflow_test_add_$t>]() {
                        let a = 1$t;
                        let b = <$t>::MAX;
                        let c = a + b;         
                    }


                    #[test]
                    #[should_panic]
                    fn [<overflow_test_mul_$t>]() {
                        let a = 2$t;
                        let b = <$t>::MAX;
                        let c = a * b;         
                    }

                    #[test]
                    #[should_panic]
                    fn [<overflow_test_div_$t>]() {
                        let a = 0$t;
                        let b = <$t>::MAX;
                        let c = b / a;         
                    }

                    #[test]
                    #[should_panic]
                    fn [<overflow_test_powi_$t>]() {
                        let a = 2$t;
                        let b = <$t>::MAX;
                        let c = b.powi(a);         
                    }

                    #[test]
                    #[should_panic]
                    fn [<overflow_test_shl_$t>]() {
                        let a = $b$t + 1;
                        let b = <$t>::MAX;
                        let c = b.shl(a);         
                    }

                    #[test]
                    #[should_panic]
                    fn [<overflow_test_shl_$t>]() {
                        let a = $b$t + 1;
                        let b = <$t>::MAX;
                        let c = b.shr(a);         
                    }
            }
        )
    }
    macro_rules! overflow_signed {
        ($($t:ty)*) => (
            $(
                paste::paste! {
                    #[test]
                    #[should_panic]
                    fn [<overflow_test_sub_$t>]() {
                        let a = -1$t;
                        let b = <$t>::MAX;
                        let c = a - b;         
                    }
                }
            )*
        )
    }

    overflow!{ i8, 8 }
    overflow_signed! { i8 i16 i32 i64 i128 }
}
