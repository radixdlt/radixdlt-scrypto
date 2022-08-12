use scrypto::prelude::*;

blueprint! {
    struct TestDecimal {
    }

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

#[test]
fn test_info_primitive() {
    let c: U32 = U32(5);
    info!("Safe integer: {}", c);
    assert_eq!(format!("{}", c), "5");
}
