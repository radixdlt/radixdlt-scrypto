use scrypto::prelude::*;

blueprint! {
    struct PriceOracle {
        prices_in_thousandths: LazyMap<String, u64>,
    }

    impl PriceOracle {
        pub fn new() -> Component {
            Self {
                prices_in_thousandths: LazyMap::new(),
            }
            .instantiate()
        }

        pub fn get_price(&self, pair: String) -> Option<u64> {
            self.prices_in_thousandths.get(&pair)
        }

        pub fn put_price(&self, pair: String, price_in_thousandths: u64) {
            self.prices_in_thousandths.insert(pair, price_in_thousandths);
        }
    }
}
