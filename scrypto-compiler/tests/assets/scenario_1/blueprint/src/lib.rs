use scrypto::prelude::*;

#[blueprint]
mod test_blueprint {
    struct Test {}

    impl Test {
        pub fn new() -> Global<Test> {
            Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }
    }
}

#[test]
fn some_test() {}
