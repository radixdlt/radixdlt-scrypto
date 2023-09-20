use scrypto::prelude::*;

#[blueprint]
mod test_decimal {
    struct TestDecimal {}

    impl TestDecimal {
        pub fn test_dec_macro() -> Decimal {
            dec!(1)
                .checked_add(dec!("2"))
                .unwrap()
                .checked_sub(
                    dec!("3.5")
                        .checked_mul(dec!(5_000_000))
                        .unwrap()
                        .checked_div(dec!("0.00000007"))
                        .unwrap(),
                )
                .unwrap()
        }
        pub fn test_pdec_macro() -> PreciseDecimal {
            pdec!(1)
                .checked_add(pdec!("2"))
                .unwrap()
                .checked_sub(
                    pdec!("3.5")
                        .checked_mul(pdec!(5_000_000))
                        .unwrap()
                        .checked_div(pdec!("0.00000007"))
                        .unwrap(),
                )
                .unwrap()
        }
    }
}
