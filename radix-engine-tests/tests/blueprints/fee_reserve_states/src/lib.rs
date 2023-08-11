use scrypto::prelude::*;

#[blueprint]
mod fee_reserve {
    struct FeeReserveChecker {}

    impl FeeReserveChecker {
        pub fn check() -> (u32, Decimal, u32, Decimal) {
            (
                Runtime::execution_cost_unit_limit(),
                Runtime::execution_cost_unit_price(),
                Runtime::tip_percentage(),
                Runtime::fee_balance(),
            )
        }
    }
}
