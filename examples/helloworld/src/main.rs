// There is no main function in Scrypto.
#![no_main]

use scrypto::constructs::*;
use scrypto::*;

blueprint! {
    struct Greeting {
        counter:  u32
    }

    impl Greeting {
        pub fn new() -> Component {
            Self {
                counter: 0
            }.into()
        }

        pub fn say_hello(&mut self) {
            info!("Hello, {}th visitor!", self.counter);
            self.counter += 1;
        }
    }
}
