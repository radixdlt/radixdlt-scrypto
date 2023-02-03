use scrypto::prelude::*;

#[blueprint]
mod test_decimal {
    struct TestDecimal {}

    impl TestDecimal {
        pub fn test_dec_macro() -> Decimal {
            dec!(1) + dec!("2") - dec!("3.5") * dec!(5, 6) / dec!("7", -8)
        }
    }
}

#[test]
fn it_compiles() {}

#[test]
fn test_to_primitive() {
    let mut a: I128 = 2.into();
    a = a.pow(10);
    assert_eq!(a.to_i128().unwrap(), 1024i128);
}
