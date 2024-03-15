use scrypto::prelude::*;

#[blueprint]
mod test_blueprint_2 {
    struct Test2 {}

    impl Test2 {
        pub fn new() -> Global<Test2> {
            Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }
    }
}
