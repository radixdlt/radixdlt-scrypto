use scrypto::types::*;
use scrypto::*;

blueprint! {
    struct Greeting {
        counter:  u32
    }

    impl Greeting {
        pub fn new() -> Address {
            Self {
                counter: 0
            }.instantiate()
        }

        pub fn say_hello(&mut self) {
            info!("Hello, {}th visitor!", self.counter);
            self.counter += 1;
        }
    }
}
