use scrypto::prelude::*;

#[blueprint]
mod oracle {
    struct Oracle {
        info: String,
        symbol_map: IndexMap<String, ResourceAddress>,
        prices: KeyValueStore<(String, String), Decimal>,
    }

    // Unlike "oracle_v1" and "oracle_v2" this Oracle has different interface,
    // which makes it incompatible with "oracle_proxy_basic"
    impl Oracle {
        pub fn instantiate_owned() -> Owned<Oracle> {
            Self {
                info: "Oracle v3".to_string(),
                symbol_map: indexmap!(),
                prices: KeyValueStore::new(),
            }
            .instantiate()
        }

        pub fn instantiate_global() -> Global<Oracle> {
            Self::instantiate_owned()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn get_oracle_info(&self) -> String {
            self.info.clone()
        }

        pub fn add_symbol(&mut self, address: ResourceAddress, symbol: String) {
            self.symbol_map.insert(symbol, address);
        }

        pub fn set_price(&mut self, base: String, quote: String, price: Decimal) {
            self.prices.insert((base, quote), price);
        }

        pub fn get_price(&self, base: String, quote: String) -> Option<Decimal> {
            self.prices.get(&(base, quote)).map(|price| *price)
        }

        pub fn get_address(&self, symbol: String) -> Option<ResourceAddress> {
            self.symbol_map.get(&symbol).map(|v| *v)
        }
    }
}
