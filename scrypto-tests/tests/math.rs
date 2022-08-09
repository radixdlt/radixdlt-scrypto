use scrypto::prelude::*;

blueprint! {
    struct TestDecimal {
    }

    impl TestDecimal {
        pub fn test_dec_macro() -> Decimal {
            dec!(1) + dec!("2") - dec!("3.5") * dec!(5, 6) / dec!("7", -8)
        }

        pub fn test_to_primitive() -> i128 {
            let mut a: I128 = 2.into();
            a = a.pow(10);
            a.to_i128().unwrap()
        }
    }


}

#[test]
fn it_compiles() {}
