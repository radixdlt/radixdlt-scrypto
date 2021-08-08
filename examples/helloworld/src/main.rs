// There is no main function in Scrypto.
#![no_main]

use scrypto::constructs::*;
use scrypto::types::*;
use scrypto::*;

component! {
    struct Greeting {
        counter:  u32
    }

    impl Greeting {
        pub fn new() -> Self {
            let owner = Context::address();
            let resource = Resource::new("symbol", "name", "description", "url", "icon_url",  Some(owner), Some(U256::from(1000)));
            let tokens = Tokens::new(U256::from(100), resource.address());
            let mut account = Account::from(owner);
            account.deposit_tokens(tokens);

            Self {
                counter: 0
            }
        }

        pub fn say_hello(&mut self) -> String {
            self.counter += 1;
            "hello".to_string()
        }
    }
}
