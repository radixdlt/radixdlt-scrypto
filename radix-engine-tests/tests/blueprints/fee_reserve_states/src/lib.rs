use scrypto::prelude::*;

#[blueprint]
mod fee_reserve {
    struct FeeReserveChecker {}

    impl FeeReserveChecker {
        pub fn check() -> (u32, Decimal, u32, Decimal, u32, Decimal) {
            (
                Runtime::get_execution_cost_unit_limit(),
                Runtime::get_execution_cost_unit_price(),
                Runtime::get_finalization_cost_unit_limit(),
                Runtime::get_finalization_cost_unit_price(),
                Runtime::get_tip_percentage(),
                Runtime::get_fee_balance(),
            )
        }
    }
}
