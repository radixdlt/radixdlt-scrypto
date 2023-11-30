use scrypto::prelude::*;

#[blueprint]
mod invalid_init_stored_bucket {
    struct InvalidInitStoredBucket {
        bucket: Bucket,
    }

    impl InvalidInitStoredBucket {
        pub fn create() -> Global<InvalidInitStoredBucket> {
            let bucket: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
                .withdraw_roles(withdraw_roles! {
                    withdrawer => rule!(allow_all);
                    withdrawer_updater => rule!(deny_all);
                })
                .mint_initial_supply(Decimal::from(5))
                .into();

            let component = InvalidInitStoredBucket { bucket }.instantiate();
            component.prepare_to_globalize(OwnerRole::None).globalize()
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

        pub fn create_bucket_in_owned_component() -> Global<InvalidStoredBucketInOwnedComponent> {
            let bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
                .withdraw_roles(withdraw_roles! {
                    withdrawer => rule!(allow_all);
                    withdrawer_updater => rule!(deny_all);
                })
                .mint_initial_supply(Decimal::from(5));

            let component = InvalidStoredBucketInOwnedComponent {
                bucket: Option::None,
            }
            .instantiate();
            component.put_bucket(bucket.into());
            component.prepare_to_globalize(OwnerRole::None).globalize()
        }
    }
}
