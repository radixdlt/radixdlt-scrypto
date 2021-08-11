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
        pub fn new() -> Address {
            Component::new("Greeting", Self {
                counter: 0
            }).into()
        }

        pub fn say_hello(&mut self) -> String {
            self.counter += 1;
            "hello".to_string()
        }
    }
}
