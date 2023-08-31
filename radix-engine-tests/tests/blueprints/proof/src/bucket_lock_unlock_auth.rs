use scrypto::prelude::*;

#[derive(Debug, PartialEq, Eq, ScryptoSbor, NonFungibleData)]
pub struct Example {
    pub name: String,
    #[mutable]
    pub available: bool,
}

#[blueprint]
mod bucket_lock_unlock_auth {
    struct BucketLockUnlockAuth {
        bucket: Bucket,
    }

    impl BucketLockUnlockAuth {
        pub fn call_lock_fungible_amount_directly() {
            let bucket: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .mint_initial_supply(100)
                .into();

            ScryptoVmV1Api::object_call(
                bucket.0.as_node_id(),
                FUNGIBLE_BUCKET_LOCK_AMOUNT_IDENT,
                scrypto_args!(Decimal::from(1)),
            );
        }

        pub fn call_unlock_fungible_amount_directly() {
            let bucket: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .mint_initial_supply(100)
                .into();

            let _proof = bucket.create_proof_of_all();

            ScryptoVmV1Api::object_call(
                bucket.0.as_node_id(),
                FUNGIBLE_BUCKET_UNLOCK_AMOUNT_IDENT,
                scrypto_args!(Decimal::from(1)),
            );
        }

        pub fn call_lock_non_fungibles_directly() {
            let bucket: Bucket = ResourceBuilder::new_integer_non_fungible(OwnerRole::None)
                .mint_initial_supply([(
                    1u64.into(),
                    Example {
                        name: "One".to_owned(),
                        available: true,
                    },
                )])
                .into();

            ScryptoVmV1Api::object_call(
                bucket.0.as_node_id(),
                NON_FUNGIBLE_BUCKET_LOCK_NON_FUNGIBLES_IDENT,
                scrypto_args!([NonFungibleLocalId::integer(1)]),
            );
        }

        pub fn call_unlock_non_fungibles_directly() {
            let bucket: Bucket = ResourceBuilder::new_integer_non_fungible(OwnerRole::None)
                .mint_initial_supply([(
                    1u64.into(),
                    Example {
                        name: "One".to_owned(),
                        available: true,
                    },
                )])
                .into();

            let _proof = bucket.create_proof_of_all();

            ScryptoVmV1Api::object_call(
                bucket.0.as_node_id(),
                NON_FUNGIBLE_BUCKET_UNLOCK_NON_FUNGIBLES_IDENT,
                scrypto_args!([NonFungibleLocalId::integer(1)]),
            );
        }
    }
}
