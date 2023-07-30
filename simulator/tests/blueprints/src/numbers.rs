use scrypto::prelude::*;

#[blueprint]
mod numbers {
    struct Numbers {}

    impl Numbers {
        pub fn test_input(_: Decimal, _: BalancedDecimal, _: PreciseDecimal) {
            info!("Call succeeded");
        }
    }
}
