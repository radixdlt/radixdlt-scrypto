use scrypto::prelude::*;

#[blueprint]
mod invalid_init_stored_bucket {
    struct InvalidInitStoredBucket {
        bucket: Bucket,
    }

    impl InvalidInitStoredBucket {
        pub fn create() -> ComponentAddress {
            let bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .restrict_withdraw(rule!(allow_all), rule!(deny_all))
                .mint_initial_supply(Decimal::from(5));

            let component = InvalidInitStoredBucket { bucket }.instantiate();
            component.globalize()
        }
    }
}

#[blueprint]
mod invalid_stored_bucket_in_owned_component {
    struct InvalidStoredBucketInOwnedComponent {
        bucket: Option<Bucket>,
    }

    impl InvalidStoredBucketInOwnedComponent {
        pub fn put_bucket(&mut self, bucket: Bucket) {
            self.bucket = Option::Some(bucket);
        }

        pub fn create_bucket_in_owned_component() -> ComponentAddress {
            let bucket = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .restrict_withdraw(rule!(allow_all), rule!(deny_all))
                .mint_initial_supply(Decimal::from(5));

            let component = InvalidStoredBucketInOwnedComponent {
                bucket: Option::None,
            }
            .instantiate();
            component.put_bucket(bucket);
            component.globalize()
        }
    }
}
