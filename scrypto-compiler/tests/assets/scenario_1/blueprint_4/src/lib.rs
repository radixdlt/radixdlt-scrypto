use scrypto::prelude::*;

#[blueprint]
mod test_blueprint_4 {
    struct Test4 {}

    impl Test4 {
        pub fn new() -> Global<Test4> {
            Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }
    }
}
