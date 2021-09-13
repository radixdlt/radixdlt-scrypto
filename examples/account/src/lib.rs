use scrypto::prelude::*;

blueprint! {
    struct Account {
        resources: Map,
    }

    impl Account {
        pub fn new() -> Address {
            Account {
                resources: Map::new(),
            }
            .instantiate()
        }

        fn deposit(&mut self, bucket: BID) {
            let resource = bucket.resource();
            match self.resources.get::<Address, BID>(&resource) {
                Some(b) => {
                    b.put(bucket);
                }
                None => {
                    let b = BID::new_empty(resource);
                    b.put(bucket);
                    self.resources.insert(resource, b);
                }
            }
        }

        fn withdraw(&mut self, amount: U256, resource: Address) -> BID {
            let bucket = self.resources.get::<Address, BID>(&resource).unwrap();
            bucket.take(amount)
        }

        /// Publish a code package.
        pub fn publish_package(&self, code: Vec<u8>) -> Address {
            let package = Package::new(&code);
            package.into()
        }

        /// Create a resource with mutable supply.
        pub fn create_resource_mutable(
            &self,
            symbol: String,
            name: String,
            description: String,
            url: String,
            icon_url: String,
            minter: Address,
        ) -> Address {
            let resource = Resource::new_mutable( &symbol, &name, &description, &url, &icon_url, minter);
            resource.into()
        }

        /// Create a resource with fixed supply.
        pub fn create_resource_fixed(
            &mut self,
            symbol: String,
            name: String,
            description: String,
            url: String,
            icon_url: String,
            supply: U256,
        ) -> Address {
            let tokens: Tokens = Resource::new_fixed(&symbol, &name, &description, &url, &icon_url, supply);
            let address = tokens.resource();
            self.deposit_tokens(tokens);
            address
        }

        /// Deposit buckets into this account
        pub fn deposit_buckets(&mut self, buckets: Vec<BID>) {
            for bucket in buckets {
                self.deposit(bucket);
            }
        }

        /// Deposit tokens into this account
        pub fn deposit_tokens(&mut self, tokens: Tokens) {
            self.deposit(tokens.into());
        }

        /// Deposit badges into this account
        pub fn deposit_badges(&mut self, badges: Badges) {
            self.deposit(badges.into());
        }

        /// Withdraw tokens from this account
        pub fn withdraw_tokens(&mut self, amount: U256, resource: Address) -> Tokens {
          self.withdraw(amount, resource).into()
        }

        /// Withdraw badges from this account
        pub fn withdraw_badges(&mut self, amount: U256, resource: Address) -> Badges {
            self.withdraw(amount, resource).into()
        }
    }
}
