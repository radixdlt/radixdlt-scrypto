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

blueprint! {
    struct InvalidStoredBucketInOwnedComponent {
        bucket: Option<Bucket>
    }

    impl InvalidStoredBucketInOwnedComponent {
        pub fn put_bucket(&mut self, bucket: Bucket) {
            self.bucket = Option::Some(bucket);
        }

        pub fn create_bucket_in_owned_component() -> ComponentAddress {
            let bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .restrict_withdraw(rule!(allow_all), LOCKED)
                .initial_supply(Decimal::from(5));

            let component = InvalidStoredBucketInOwnedComponent {
                bucket: Option::None,
            }.instantiate();
            component.put_bucket(bucket);
            component.globalize()
        }
    }
}
