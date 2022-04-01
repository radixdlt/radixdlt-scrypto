use scrypto::prelude::*;

blueprint! {
    struct MoveTest {
        vaults: Vec<Vault>,
    }

    impl MoveTest {
        fn create_test_token(amount: u32) -> Bucket {
            ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .initial_supply(amount)
        }

        pub fn receive_bucket(&mut self, t: Bucket) {
            self.vaults.push(Vault::with_bucket(t));
        }

        pub fn receive_proof(&self, t: Proof) {
            t.drop();
        }

        pub fn move_bucket() {
            let bucket = Self::create_test_token(1000);
            let component_address = MoveTest { vaults: Vec::new() }.instantiate();
            Process::call_method(component_address, "receive_bucket", args!(bucket));
        }

        pub fn move_proof() -> Bucket {
            let bucket = Self::create_test_token(1000);
            let component_address = MoveTest { vaults: Vec::new() }.instantiate();
            Process::call_method(
                component_address,
                "receive_proof",
                args!(bucket.create_proof()),
            );

            bucket
        }
    }
}
