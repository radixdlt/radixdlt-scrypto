use scrypto::prelude::*;

#[blueprint]
mod move_test {
    struct MoveTest {
        vaults: Vec<Vault>,
    }

    impl MoveTest {
        fn create_test_token(amount: u32) -> Bucket {
            ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata(metadata! {
                    init {
                        "name" => "TestToken".to_owned(), locked;
                    }
                })
                .mint_initial_supply(amount)
        }

        pub fn receive_bucket(&mut self, t: Bucket) {
            self.vaults.push(Vault::with_bucket(t));
        }

        pub fn receive_proof(&self, t: Proof) {
            t.drop();
        }

        pub fn move_bucket() {
            let bucket = Self::create_test_token(1000);
            let component = MoveTest { vaults: Vec::new() }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize();
            component.receive_bucket(bucket);
        }

        pub fn move_proof() -> Bucket {
            let bucket = Self::create_test_token(1000);
            let component = MoveTest { vaults: Vec::new() }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize();
            component.receive_proof(bucket.create_proof());

            bucket
        }
    }
}

#[blueprint]
mod core_test {
    struct CoreTest;

    impl CoreTest {
        pub fn query() -> (PackageAddress, Hash, Epoch) {
            (
                Runtime::package_address(),
                Runtime::transaction_hash(),
                Runtime::current_epoch(),
            )
        }
    }
}
