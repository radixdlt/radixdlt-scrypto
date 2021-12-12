use scrypto::prelude::*;

blueprint! {
    struct ChangeMe {}

    impl ChangeMe {
        pub fn new() -> Component {
            Self{}.instantiate()
        }
    }
}
