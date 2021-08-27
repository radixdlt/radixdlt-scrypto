#![no_main]

use scrypto::prelude::*;

blueprint! {
    struct Account {
        resources: HashMap<Address, BID>,
    }

    impl Account {
        pub fn new() -> Address {
            Account {
                resources: HashMap::new(),
            }
            .instantiate()
            .into()
        }

        /// Deposit tokens into this account
        pub fn deposit_bucket(&mut self, bucket: BID) {
            let resource = bucket.resource();
            self.resources
                .entry(resource)
                .or_insert(BID::new_empty(resource))
                .put(bucket);
        }

        /// Deposit tokens into this account
        pub fn deposit_tokens(&mut self, tokens: Tokens) {
            let resource = tokens.resource();
            self.resources
                .entry(resource)
                .or_insert(BID::new_empty(resource))
                .put(tokens.into());
        }

        /// Deposit badges into this account
        pub fn deposit_badges(&mut self, badges: Badges) {
            let resource = badges.resource();
            self.resources
                .entry(resource)
                .or_insert(BID::new_empty(resource))
                .put(badges.into());
        }

        /// Withdraw tokens from this account
        pub fn withdraw_tokens(&mut self, amount: U256, resource: Address) -> Tokens {
            self.resources
                .get_mut(&resource)
                .unwrap()
                .take(amount)
                .into()
        }

        /// Withdraw badges from this account
        pub fn withdraw_badges(&mut self, amount: U256, resource: Address) -> Badges {
            self.resources
                .get_mut(&resource)
                .unwrap()
                .take(amount)
                .into()
        }
    }
}
