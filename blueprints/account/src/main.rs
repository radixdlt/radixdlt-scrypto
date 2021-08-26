#![no_main]

use scrypto::prelude::*;

blueprint! {
    struct Account {
        tokens: HashMap<Address, Tokens>,
        badges: HashMap<Address, Badges>,
    }

    impl Account {
        pub fn new() -> Account {
            Account {
                tokens: HashMap::new(),
                badges: HashMap::new(),
            }
        }

        /// Deposit tokens into this account
        pub fn deposit_tokens(&mut self, tokens: Tokens) {
            let resource = tokens.resource();
            self.tokens
                .entry(resource)
                .or_insert(Tokens::new_empty(resource))
                .put(tokens);
        }

        /// Deposit badges into this account
        pub fn deposit_badges(&mut self, badges: Badges) {
            let resource = badges.resource();
            self.badges
                .entry(resource)
                .or_insert(Badges::new_empty(resource))
                .put(badges);
        }

        /// Withdraw tokens from this account
        pub fn withdraw_tokens(&mut self, amount: U256, resource: Address) -> Tokens {
            self.tokens.get_mut(&resource).unwrap().take(amount)
        }

        /// Withdraw badges from this account
        pub fn withdraw_badges(&mut self, amount: U256, resource: Address) -> Badges {
            self.badges.get_mut(&resource).unwrap().take(amount)
        }
    }
}
