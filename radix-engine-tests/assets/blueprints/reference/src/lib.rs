use scrypto::prelude::*;

#[blueprint]
mod reference_test {
    use child_reference_holder::*;

    struct ReferenceTest {
        reference: Option<Reference>,
        vault: Option<Vault>,
        kv_store: Option<KeyValueStore<u32, Reference>>,
    }

    impl ReferenceTest {
        pub fn create_global_node_with_local_ref() {
            let bucket = Bucket::new(XRD.into());

            Self {
                reference: Some(Reference(bucket.0.as_node_id().clone())),
                vault: None,
                kv_store: None,
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
                kv_store: None,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn new_with_bucket(bucket: Bucket) -> Global<ReferenceTest> {
            Self {
                reference: Some(Reference(XRD.as_node_id().clone())),
                vault: Some(Vault::with_bucket(bucket)),
                kv_store: None,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn add_local_ref_to_stored_substate(&mut self) {
            let bucket = Bucket::new(XRD.into());

            self.reference = Some(Reference(bucket.0.as_node_id().clone()));
        }

        pub fn add_direct_access_ref_to_stored_substate(&mut self, address: InternalAddress) {
            self.reference = Some(Reference(address.as_node_id().clone()));
        }

        pub fn add_direct_access_ref_to_heap_substate(&self, address: InternalAddress) {
            let instance = Self {
                reference: None,
                vault: None,
                kv_store: None,
            }
            .instantiate();

            instance.add_direct_access_ref_to_stored_substate(address);

            instance.prepare_to_globalize(OwnerRole::None).globalize();
        }

        pub fn add_direct_access_ref_to_kv_store_substate(&self, address: InternalAddress) {
            let kv_store = KeyValueStore::new();

            kv_store.insert(1, address.into());

            Self {
                reference: None,
                vault: None,
                kv_store: Some(kv_store),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize();
        }

        pub fn recall(reference: TypedInternalReference) -> Bucket {
            scrypto_decode(&ScryptoVmV1Api::object_call_direct(
                &reference.0.as_node_id(),
                VAULT_RECALL_IDENT,
                scrypto_args!(Decimal::ONE),
            ))
            .unwrap()
        }

        pub fn send_and_receive_reference() {
            let bucket = Bucket::new(XRD.into());
            Blueprint::<ChildReferenceHolder>::bounce_back(Reference(bucket.0.as_node_id().clone()));
            bucket.drop_empty();
        }
    }
}

#[blueprint]
mod child_reference_holder {
    struct ChildReferenceHolder {}

    impl ChildReferenceHolder {
        pub fn bounce_back(reference: Reference) -> Reference {
            reference
        }
    }
}

#[derive(ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
#[sbor(transparent)]
pub struct TypedInternalReference(Reference);

impl Describe<ScryptoCustomTypeKind> for TypedInternalReference {
    const TYPE_ID: RustTypeId = RustTypeId::Novel([123u8; 20]);

    fn type_data() -> TypeData<ScryptoCustomTypeKind, RustTypeId> {
        TypeData {
            kind: TypeKind::Custom(ScryptoCustomTypeKind::Reference),
            metadata: TypeMetadata::no_child_names("TypedInternalReference"),
            validation: TypeValidation::Custom(ScryptoCustomTypeValidation::Reference(
                ReferenceValidation::IsInternalTyped(
                    Some(RESOURCE_PACKAGE),
                    FUNGIBLE_VAULT_BLUEPRINT.to_string(),
                ),
            )),
        }
    }

    fn add_all_dependencies(_aggregator: &mut TypeAggregator<ScryptoCustomTypeKind>) {}
}
