use scrypto::prelude::*;

blueprint! {
    struct CoffeeMachine {}

    impl CoffeeMachine {
        pub fn new() -> Component{
            Self{}.instantiate()
        }

        pub fn make_coffee() {
            info!("Brewing coffee !");
        }
    }
}