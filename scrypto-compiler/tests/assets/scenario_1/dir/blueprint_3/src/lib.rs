use scrypto::prelude::*;

#[blueprint]
mod test_blueprint_3 {
    struct Test3 {}

    impl Test3 {
        pub fn new() -> Global<Test3> {
            Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }
    }
}
