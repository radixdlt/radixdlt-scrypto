use scrypto::prelude::*;

blueprint! {
    struct PriceOracle {
        prices: LazyMap<(Address, Address), u128>,
        decimals: u8,
        admin: Address,
    }

    impl PriceOracle {
        /// Creates a PriceOracle component, along with necessary badges.
        pub fn new(decimals: u8, num_of_admins: u32) -> (Component, Bucket) {
            scrypto_assert!(decimals >= 2 && decimals <= 18);

            let badges = ResourceBuilder::new()
                .metadata("name", "Admin Badge")
                .create_fixed(num_of_admins);

            let component = Self {
                prices: LazyMap::new(),
                decimals,
                admin: badges.resource_def().address()
            }
            .instantiate();

            (component, badges)
        }

        /// Returns the current price of a resource pair.
        pub fn get_price(&self, base: Address, quote: Address) -> Option<u128> {
            self.prices.get(&(base, quote))
        }

        /// Updates the price of a resource pair and its inverse.
        #[auth(admin)]
        pub fn update_price(&self, base: Address, quote: Address, price: u128) {
            let scale = 10u128.pow(self.decimals as u32);
            self.prices.insert((base, quote), price);
            self.prices.insert((quote, base), scale * scale / price);
        }

        /// Returns the decimal
        pub fn decimals(&self) -> u8 {
            self.decimals
        }

        /// Returns the admin badge address
        pub fn admin(&self) -> Address {
            self.admin
        }
    }
}
