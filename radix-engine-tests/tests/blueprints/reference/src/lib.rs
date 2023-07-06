use scrypto::prelude::*;

#[blueprint]
mod reference_test {
    struct ReferenceTest {
        reference: Reference,
    }

    impl ReferenceTest {
        pub fn create_global_node_with_local_ref() {
            let bucket = Bucket::new(XRD);

            Self {
                reference: Reference(bucket.0.as_node_id().clone()),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize();

            bucket.drop_empty();
        }

        pub fn new() -> Global<ReferenceTest> {
            Self {
                reference: Reference(XRD.as_node_id().clone()),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn add_local_ref_to_stored_substate(&mut self) {
            let bucket = Bucket::new(XRD);

            self.reference = Reference(bucket.0.as_node_id().clone());
        }
    }
}
