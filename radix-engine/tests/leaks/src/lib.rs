use scrypto::prelude::*;

blueprint! {
    struct Leaks {}

    impl Leaks {
        pub fn leaky_component() {
            Self {}.instantiate();
        }
    }
}
