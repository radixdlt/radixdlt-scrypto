use scrypto::prelude::*;

blueprint! {
    struct InvalidInitStoredBucket {
        bucket: Bucket
    }

    impl InvalidInitStoredBucket {
        pub fn create() -> ComponentAddress {
            let bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .restrict_withdraw(rule!(allow_all), LOCKED)
                .initial_supply(Decimal::from(5));

            let component = InvalidInitStoredBucket {
                bucket
            }.instantiate();
            component.globalize()
        }
    }
}
