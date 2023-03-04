use scrypto::prelude::*;

#[blueprint]
mod test_decimal {
    struct TestDecimal {}

    impl TestDecimal {
        pub fn test_dec_macro() -> Decimal {
            dec!(1) + dec!("2") - dec!("3.5") * dec!(5, 6) / dec!("7", -8)
        }
        pub fn test_pdec_macro() -> PreciseDecimal {
            pdec!(1) + pdec!("2") - pdec!("3.5") * pdec!(5, 6) / pdec!("7", -8)
        }
    }
}
