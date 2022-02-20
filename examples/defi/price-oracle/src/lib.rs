use scrypto::prelude::*;

blueprint! {
    struct PriceOracle {
        /// Last price of each resource pair
        prices: LazyMap<(ResourceDefId, ResourceDefId), Decimal>,
        /// The admin badge resource definition ID
        admin_badge: ResourceDefId,
    }

    impl PriceOracle {
        /// Creates a PriceOracle component, along with admin badges.
        pub fn instantiate_oracle(num_of_admins: u32) -> (Bucket, ComponentId) {
            assert!(num_of_admins >= 1);

            let badges = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", "Price Oracle Admin Badge")
                .initial_supply_fungible(num_of_admins);

            let component = Self {
                prices: LazyMap::new(),
                admin_badge: badges.resource_def_id(),
            }
            .instantiate();

            (badges, component)
        }

        /// Returns the current price of a resource pair BASE/QUOTE.
        pub fn get_price(&self, base: ResourceDefId, quote: ResourceDefId) -> Option<Decimal> {
            self.prices.get(&(base, quote))
        }

        /// Updates the price of a resource pair BASE/QUOTE and its inverse.
        #[auth(admin_badge)]
        pub fn update_price(&self, base: ResourceDefId, quote: ResourceDefId, price: Decimal) {
            self.prices.insert((base, quote), price);
            self.prices.insert((quote, base), Decimal::from(1) / price);
        }

        /// Returns the admin badge resource definition ID.
        pub fn admin_badge(&self) -> ResourceDefId {
            self.admin_badge
        }
    }
}
