use scrypto::prelude::*;

blueprint! {
    struct MoveTest {
        vaults: Vec<Vault>,
    }

    impl MoveTest {
        fn create_test_token(amount: u32) -> Bucket {
            ResourceBuilder::new_fungible(0)
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
            let component = MoveTest { vaults: Vec::new() }.instantiate();
            call_method(component.address(), "receive_bucket", args!(bucket));
        }

        pub fn move_bucket_ref() -> Bucket {
            let bucket = Self::create_test_token(1000);
            let component = MoveTest { vaults: Vec::new() }.instantiate();
            call_method(
                component.address(),
                "receive_bucket_ref",
                args!(bucket.present()),
            );

            bucket
        }
    }
}
