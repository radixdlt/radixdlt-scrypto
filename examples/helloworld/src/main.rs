#![no_main]

use scrypto::prelude::*;

blueprint! {
    struct Greeting {
        cnt:  u32
    }

    impl Greeting {
        pub fn new() -> Address {
            let component = Self {
                cnt: 0
            }.instantiate();

            debug!("New component: {}", component);
            component
        }

        pub fn say_hello(&mut self) {
            info!("Hello, visitor #{}.", self.cnt);
            self.cnt += 1;
        }
    }
}
