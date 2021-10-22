use scrypto::prelude::*;

blueprint! {
    struct PriceOracle {
        /// Last price of each resource pair
        prices: LazyMap<(Address, Address), u128>,
        /// The number of decimal places
        decimals: u8,
        /// The admin badge resource def address
        admin: Address
    }

    impl PriceOracle {
        /// Creates a PriceOracle component, along with necessary badges.
        pub fn new(decimals: u8, num_of_admins: u32) -> (Component, Bucket) {
            scrypto_assert!(decimals >= 2 && decimals <= 18);
            scrypto_assert!(num_of_admins >= 1);

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

        /// Returns the number of decimal places.
        pub fn decimals(&self) -> u8 {
            self.decimals
        }

        /// Returns the admin badge resource def address.
        pub fn admin(&self) -> Address {
            self.admin
        }
    }
}
