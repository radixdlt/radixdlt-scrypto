use scrypto::prelude::*;

#[blueprint]
mod leaks {
    struct Leaks {}

    impl Leaks {
        pub fn dangling_component() {
            Self {}.instantiate();
        }

        pub fn dangling_bucket() {
            let _bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata(metadata! {
                    init {
                        "name" => "TestToken".to_owned(), locked;
                    }
                })
                .mint_initial_supply(1);
        }

        pub fn dangling_vault() {
            let bucket: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata(metadata! {
                    init {
                        "name" => "TestToken".to_owned(), locked;
                    }
                })
                .mint_initial_supply(1)
                .into();
            let _vault = Vault::with_bucket(bucket);
        }

        pub fn get_bucket() -> Bucket {
            let bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata(metadata! {
                    init {
                        "name" => "TestToken".to_owned(), locked;
                    }
                })
                .mint_initial_supply(1)
                .into();
            bucket
        }

        pub fn dangling_kv_store() {
            let map = KeyValueStore::new();
            map.insert("hello".to_owned(), "world".to_owned());
            map.get(&"hello".to_owned());
        }

        pub fn dangling_bucket_with_proof() -> Proof {
            let bucket: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata(metadata! {
                    init {
                        "name" => "TestToken".to_owned(), locked;
                    }
                })
                .mint_initial_supply(1)
                .into();

            bucket.create_proof_of_all()
        }
    }
}
