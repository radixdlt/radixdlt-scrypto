use scrypto::prelude::*;

blueprint! {
    struct PriceOracle {
        prices_in_billionth: LazyMap<(Address, Address), u128>,
    }

    impl PriceOracle {
        /// Creates a PriceOracle component.
        pub fn new() -> Component {
            Self {
                prices_in_billionth: LazyMap::new(),
            }
            .instantiate()
        }

        /// Returns the price (in billionth) of pair BASE/QUOTE.
        pub fn get_price(&self, base: Address, quote: Address) -> Option<u128> {
            self.prices_in_billionth.get(&(base, quote))
        }

        /// Updates the price (in billionth) of pair BASE/QUOTE.
        pub fn update_price(&self, base: Address, quote: Address, price_in_billionth: u128) {
            self.prices_in_billionth.insert((base, quote), price_in_billionth);
            self.prices_in_billionth.insert((base, quote), 1_000_000_000 / price_in_billionth);
        }
    }
}
