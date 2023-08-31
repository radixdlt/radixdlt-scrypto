use scrypto::prelude::*;

#[blueprint]
mod test_decimal {
    struct TestDecimal {}

    impl TestDecimal {
        pub fn test_dec_macro() -> Decimal {
            dec!(1)
                .safe_add(dec!("2"))
                .unwrap()
                .safe_sub(
                    dec!("3.5")
                        .safe_mul(dec!(5, 6))
                        .unwrap()
                        .safe_div(dec!("7", -8))
                        .unwrap(),
                )
                .unwrap()
        }
        pub fn test_pdec_macro() -> PreciseDecimal {
            pdec!(1)
                .safe_add(pdec!("2"))
                .unwrap()
                .safe_sub(
                    pdec!("3.5")
                        .safe_mul(pdec!(5, 6))
                        .unwrap()
                        .safe_div(pdec!("7", -8))
                        .unwrap(),
                )
                .unwrap()
        }
    }
}
