use scrypto::prelude::*;

blueprint! {
    struct MutualFarm {
        
    }

    impl MutualFarm {
        pub fn new() -> Component {
            Self {
            }
            .instantiate()
        }
    }
}
