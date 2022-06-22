use scrypto::prelude::*;

blueprint! {
    struct Leaks {}

    impl Leaks {
        pub fn dangling_component() {
            Self {}.instantiate();
        }

        pub fn dangling_vault() -> () {
            let bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .initial_supply(1);
            let _vault = Vault::with_bucket(bucket);
        }
    }
}
