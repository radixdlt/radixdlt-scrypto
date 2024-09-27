use scrypto::prelude::*;

#[blueprint]
mod factory {
    struct Factory;

    impl Factory {
        pub fn new() -> Global<Factory> {
            Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn create(&self) -> FungibleBucket {
            Blueprint::<AccountLocker>::instantiate_simple(true).1
        }
    }
}
