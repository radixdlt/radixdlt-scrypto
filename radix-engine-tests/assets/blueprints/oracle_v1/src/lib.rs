use scrypto::prelude::*;

#[blueprint]
mod oracle {
    enable_method_auth! {
        methods {
            set_price => restrict_to: [OWNER];
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

        pub fn instantiate_global(owner_role: OwnerRole) -> Global<Oracle> {
            Self::instantiate_owned()
                .prepare_to_globalize(owner_role)
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
