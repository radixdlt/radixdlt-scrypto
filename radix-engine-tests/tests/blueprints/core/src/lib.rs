use scrypto::prelude::*;

#[blueprint]
mod move_test {
    struct MoveTest {
        vaults: Vec<Vault>,
    }

    impl MoveTest {
        fn create_test_token(amount: u32) -> Bucket {
            ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
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
            let component_address = MoveTest { vaults: Vec::new() }.instantiate().globalize();

            Runtime::call_method(component_address, "receive_bucket", scrypto_args!(bucket))
        }

        pub fn move_proof() -> Bucket {
            let bucket = Self::create_test_token(1000);
            let component_address = MoveTest { vaults: Vec::new() }.instantiate().globalize();

            let _: () = Runtime::call_method(
                component_address,
                "receive_proof",
                scrypto_args!(bucket.create_proof()),
            );

            bucket
        }
    }
}

#[blueprint]
mod core_test {
    struct CoreTest;

    impl CoreTest {
        pub fn query() -> (PackageAddress, Hash, u64, u128) {
            (
                Runtime::package_address(),
                Runtime::transaction_hash(),
                Runtime::current_epoch(),
                Runtime::generate_uuid(),
            )
        }
    }
}
