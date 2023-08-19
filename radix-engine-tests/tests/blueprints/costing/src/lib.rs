use scrypto::prelude::*;

#[blueprint]
mod costing_test {
    struct CostingTest {}

    impl CostingTest {
        pub fn usd_price() -> Decimal {
            Runtime::usd_price()
        }
    }
}
