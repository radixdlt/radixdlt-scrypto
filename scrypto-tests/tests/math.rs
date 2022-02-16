use scrypto::prelude::*;

blueprint! {
    struct TestDecimal {
    }

    impl TestDecimal {
        pub fn test_dec_macro() -> Decimal {
            dec!(1) + dec!("2") - dec!("3.5") * dec!(5, 6) / dec!("7", -8)
        }

        pub fn test_bdec_macro() -> BigDecimal {
            bdec!(1) + bdec!("2") - bdec!("3.5") * bdec!(5, 6) / bdec!("7", -8)
        }
    }
}

#[test]
fn it_compiles() {}
