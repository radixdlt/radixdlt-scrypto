use scrypto::prelude::*;

blueprint! {
    struct MoveTest {
        vaults: Vec<Vault>,
    }

    impl MoveTest {
        fn create_test_token(amount: u32) -> Bucket {
            ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                .metadata("name", "TestToken")
                .initial_supply_fungible(amount)
        }

        pub fn receive_bucket(&mut self, t: Bucket) {
            self.vaults.push(Vault::with_bucket(t));
        }

        pub fn receive_bucket_ref(&self, t: BucketRef) {
            t.drop();
        }

        pub fn move_bucket() {
            let bucket = Self::create_test_token(1000);
            let component_ref = MoveTest { vaults: Vec::new() }.instantiate();
            Context::call_method(component_ref, "receive_bucket", args!(bucket));
        }

        pub fn move_bucket_ref() -> Bucket {
            let bucket = Self::create_test_token(1000);
            let component_ref = MoveTest { vaults: Vec::new() }.instantiate();
            Context::call_method(component_ref, "receive_bucket_ref", args!(bucket.present()));

            bucket
        }
    }
}
