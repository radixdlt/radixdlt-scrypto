use scrypto::prelude::*;

#[blueprint]
mod badge_test {
    struct BadgeTest;

    impl BadgeTest {
        fn create_test_badge(amount: u32) -> Bucket {
            ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
                .metadata(metadata! {
                    init {
                        "name" => "TestToken".to_owned(), locked;
                    }
                })
                .mint_initial_supply(amount)
                .into()
        }

        pub fn combine() -> Bucket {
            let mut bucket1 = Self::create_test_badge(100);
            let bucket2 = bucket1.take(50);
            bucket1.put(bucket2);
            bucket1
        }

        pub fn split() -> (Bucket, Bucket) {
            let mut bucket1 = Self::create_test_badge(100);
            let bucket2 = bucket1.take(Decimal::from(5));
            (bucket1, bucket2)
        }

        pub fn borrow() -> Bucket {
            let bucket = Self::create_test_badge(100);
            let proof = bucket.create_proof_of_all();
            proof.drop();
            bucket
        }

        pub fn query() -> (Decimal, ResourceAddress, Bucket) {
            let bucket = Self::create_test_badge(100);
            (bucket.amount(), bucket.resource_address(), bucket)
        }
    }
}
