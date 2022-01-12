use scrypto::prelude::*;

// PriceOrable component. Taken from https://github.com/radixdlt/radixdlt-scrypto examples.
// I removed the admin badge for simplicity.
blueprint! {
    struct PriceOracle {
        /// Last price of each resource pair
        prices: LazyMap<(Address, Address), Decimal>,
        usd: Vault
    }

    impl PriceOracle {
        /// Creates a PriceOracle component, along with admin badges.
        pub fn new() -> Component {
            // Create usd tokens
            let usd = ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                        .metadata("name", "USD")
                        .initial_supply_fungible(100000);

            Self {
                prices: LazyMap::new(),
                usd: Vault::with_bucket(usd)
            }
            .instantiate()
        }

        /// Returns the current price of a resource pair BASE/QUOTE.
        pub fn get_price(&self, base: Address, quote: Address) -> Option<Decimal> {
            self.prices.get(&(base, quote))
        }

        // Return the address of USD token
        pub fn get_usd_address(&self) -> Address {
            self.usd.resource_address()
        }

        /// Updates the price of a resource pair BASE/QUOTE and its inverse.
        pub fn update_price(&self, base: Address, quote: Address, price: Decimal) {
            self.prices.insert((base, quote), price);
            self.prices.insert((quote, base), Decimal::from(1) / price);
        }
    }
}
