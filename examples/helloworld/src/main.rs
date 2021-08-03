// There is no main function in Scrypto.
#![no_main]

use scrypto::*;

component! {
    struct Greeting {
        counter:  u32
    }

    impl Greeting {
        pub fn new() -> Self {
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
