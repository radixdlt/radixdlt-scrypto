use scrypto::core::{call_method, State};
use scrypto::resource::{Bucket, BucketRef, ResourceBuilder, Vault};
use scrypto::{args, blueprint};

blueprint! {
    struct MoveTest {
        vaults: Vec<Vault>
    }

    impl MoveTest {

        pub fn receive_bucket(&mut self, t: Bucket) {
            self.vaults.push(Vault::with_bucket(t));
        }

        pub fn receive_bucket_ref(&self, t: BucketRef) {
            t.drop();
        }

        pub fn move_bucket() {
            let bucket = ResourceBuilder::new()
                .metadata("name", "TestToken")
                .create_fixed(1000);

            let component = MoveTest {
                vaults: Vec::new()
            }
            .instantiate();
            call_method(component.address(), "receive_bucket", args!(bucket));
        }

        pub fn move_bucket_ref() -> Bucket {
            let bucket = ResourceBuilder::new()
                .metadata("name", "TestToken")
                .create_fixed(1000);

            let component = MoveTest {
                vaults: Vec::new()
            }
            .instantiate();
            call_method(component.address(), "receive_bucket_ref", args!(bucket.borrow()));

            bucket
        }
    }
}
