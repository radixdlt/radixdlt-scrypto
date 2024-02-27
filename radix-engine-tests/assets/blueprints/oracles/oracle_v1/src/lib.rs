use scrypto::prelude::*;

#[blueprint]
mod oracle {
    enable_method_auth! {
        roles {
            oracle_manager_auth => updatable_by: [];
        },
        methods {
            set_price => restrict_to: [oracle_manager_auth, OWNER];
            get_oracle_info => PUBLIC;
            get_price => PUBLIC;
        }
    }

    struct Oracle {
        info: String,
        prices: KeyValueStore<(ResourceAddress, ResourceAddress), Decimal>,
    }

    impl Oracle {
        pub fn instantiate_owned() -> Owned<Oracle> {
            Self {
                info: "Oracle v1".to_string(),
                prices: KeyValueStore::new(),
            }
            .instantiate()
        }

        pub fn instantiate_and_globalize(
            owner_badge: NonFungibleGlobalId,
            manager_badge: NonFungibleGlobalId,
        ) -> Global<Oracle> {
            let owner_role = OwnerRole::Fixed(rule!(require(owner_badge)));
            let manager_rule = rule!(require(manager_badge));

            Self::instantiate_owned()
                .prepare_to_globalize(owner_role)
                .roles(roles! {
                    oracle_manager_auth => manager_rule;
                })
                .globalize()
        }

        pub fn get_oracle_info(&self) -> String {
            self.info.clone()
        }

        pub fn set_price(&mut self, base: ResourceAddress, quote: ResourceAddress, price: Decimal) {
            self.prices.insert((base, quote), price);
        }

        pub fn get_price(&self, base: ResourceAddress, quote: ResourceAddress) -> Option<Decimal> {
            self.prices.get(&(base, quote)).map(|price| *price)
        }
    }
}
