use scrypto::prelude::*;

blueprint!{
    struct Numbers {}

    impl Numbers {
        pub fn test_input(
            _: Decimal,
            _: PreciseDecimal,
        ) {
            info!("Call succeeded");
        }
    }
}