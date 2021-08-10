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
            })
        }

        pub fn say_hello(&mut self) -> String {
            self.counter += 1;
            "hello".to_string()
        }
    }
}
