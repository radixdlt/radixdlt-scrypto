use scrypto::prelude::*;

#[blueprint]
mod reference_test {
    struct ReferenceTest {
        reference: Reference,
    }

    impl ReferenceTest {
        pub fn new() {
            let bucket = Bucket::new(RADIX_TOKEN);

            Self {
                reference: Reference(bucket.0.as_node_id().clone()),
            }
            .instantiate()
            .globalize();

            bucket.drop_empty();
        }
    }
}
