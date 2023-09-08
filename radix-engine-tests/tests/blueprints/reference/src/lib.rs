use scrypto::prelude::*;

#[blueprint]
mod reference_test {
    struct ReferenceTest {
        reference: Option<Reference>,
        vault: Option<Vault>,
    }

    impl ReferenceTest {
        pub fn create_global_node_with_local_ref() {
            let bucket = Bucket::new(XRD);

            Self {
                reference: Some(Reference(bucket.0.as_node_id().clone())),
                vault: None,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize();

            bucket.drop_empty();
        }

        pub fn new() -> Global<ReferenceTest> {
            Self {
                reference: Some(Reference(XRD.as_node_id().clone())),
                vault: None,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn new_with_bucket(bucket: Bucket) -> Global<ReferenceTest> {
            Self {
                reference: Some(Reference(XRD.as_node_id().clone())),
                vault: Some(Vault::with_bucket(bucket)),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn add_local_ref_to_stored_substate(&mut self) {
            let bucket = Bucket::new(XRD);

            self.reference = Some(Reference(bucket.0.as_node_id().clone()));
        }

        pub fn add_direct_access_ref_to_stored_substate(&mut self, address: InternalAddress) {
            self.reference = Some(Reference(address.as_node_id().clone()));
        }

        pub fn add_direct_access_ref_to_heap_substate(&mut self, address: InternalAddress) {
            let instance = Self {
                reference: None,
                vault: None,
            }
            .instantiate();

            instance.add_direct_access_ref_to_stored_substate(address);

            instance.prepare_to_globalize(OwnerRole::None).globalize();
        }
    }
}
