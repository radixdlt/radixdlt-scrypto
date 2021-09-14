use scrypto::prelude::*;

blueprint! {
    struct Greeting {
        count: u32
    }

    impl Greeting {
        pub fn new() -> Address {
            let component = Self {
                count: 0
            }.instantiate();

            debug!("New component: {}", component);
            component
        }

        pub fn say_hello(&mut self) -> u32 {
            info!("Hello, visitor #{}.", self.count);
            self.count += 1;
            self.count
        }
    }
}
