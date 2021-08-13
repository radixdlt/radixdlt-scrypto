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

        pub fn say_hello(&mut self) {
            info!("Hello, {}th visitor!", self.counter);
            self.counter += 1;
        }
    }
}
