use scrypto::prelude::*;

#[blueprint]
mod reference_test {
    use child_reference_holder::*;
    use gated_target::*;

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

        pub fn take_via_normal_call(reference: TypedInternalReference) -> Bucket {
            scrypto_decode(&ScryptoVmV1Api::object_call(
                &reference.0.as_node_id(),
                VAULT_TAKE_IDENT,
                scrypto_args!(Decimal::ONE),
            ))
            .unwrap()
        }

        pub fn take_non_fungibles_via_normal_call(
            reference: TypedNonFungibleVaultReference,
        ) -> Bucket {
            let ids: IndexSet<NonFungibleLocalId> = scrypto_decode(&ScryptoVmV1Api::object_call(
                &reference.0.as_node_id(),
                NON_FUNGIBLE_VAULT_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT,
                scrypto_args!(u32::MAX),
            ))
            .unwrap();

            scrypto_decode(&ScryptoVmV1Api::object_call(
                &reference.0.as_node_id(),
                NON_FUNGIBLE_VAULT_TAKE_NON_FUNGIBLES_IDENT,
                scrypto_args!(ids),
            ))
            .unwrap()
        }

        pub fn take_via_own_kind(vault: FungibleVault) -> Bucket {
            let mut vault: Vault = vault.into();
            vault.take(Decimal::ONE)
        }

        pub fn forge_proof_via_normal_call(reference: TypedInternalReference) {
            let proof: Proof = scrypto_decode(&ScryptoVmV1Api::object_call(
                &reference.0.as_node_id(),
                FUNGIBLE_VAULT_CREATE_PROOF_OF_AMOUNT_IDENT,
                scrypto_args!(Decimal::ONE),
            ))
            .unwrap();

            proof.drop();
        }

        pub fn forge_nft_proof_and_call_gated(
            reference: TypedNonFungibleVaultReference,
            target: ComponentAddress,
        ) {
            let resource = ResourceAddress::try_from(ScryptoVmV1Api::object_get_outer_object(
                &reference.0.as_node_id(),
            ))
            .unwrap();
            let ids: IndexSet<NonFungibleLocalId> = scrypto_decode(&ScryptoVmV1Api::object_call(
                &reference.0.as_node_id(),
                NON_FUNGIBLE_VAULT_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT,
                scrypto_args!(u32::MAX),
            ))
            .unwrap();
            let proof: Proof = scrypto_decode(&ScryptoVmV1Api::object_call(
                &reference.0.as_node_id(),
                NON_FUNGIBLE_VAULT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT,
                scrypto_args!(ids),
            ))
            .unwrap();

            LocalAuthZone::push(proof);
            Global::<GatedTarget>::from(target).gated_action();
            LocalAuthZone::drop_proofs();
        }

        pub fn lock_fee_via_normal_call(reference: TypedInternalReference) {
            scrypto_decode::<()>(&ScryptoVmV1Api::object_call(
                &reference.0.as_node_id(),
                FUNGIBLE_VAULT_LOCK_FEE_IDENT,
                scrypto_args!(dec!("100"), false),
            ))
            .unwrap();
        }

        pub fn send_and_receive_reference() {
            let bucket = Bucket::new(XRD.into());
            Blueprint::<ChildReferenceHolder>::bounce_back_reference(Reference(
                bucket.0.as_node_id().clone(),
            ));
            bucket.drop_empty();
        }

        pub fn send_and_receive_reference_wrapped_in_owned() {
            let bucket = Bucket::new(XRD.into());
            let bucket_reference = Reference(bucket.0.as_node_id().clone());
            // Instantiating a new object is possible
            let wrapper = ChildReferenceHolder {
                reference: Some(bucket_reference),
            }
            .instantiate();
            let mut wrapper: Owned<ChildReferenceHolder> =
                Blueprint::<ChildReferenceHolder>::bounce_back_owned(wrapper);

            // CLEANUP
            wrapper.take_reference();
            // Q: Do we need to somehow drop the reference before dropping the bucket?
            bucket.drop_empty();
            // We have set `ChildReferenceHolder` to be transient, but we can't drop it, as it's not exposed to scrypto
            // So we expect some kind of undropped node error
        }
    }
}

#[blueprint]
mod child_reference_holder {
    struct ChildReferenceHolder {
        pub reference: Option<Reference>,
    }

    impl ChildReferenceHolder {
        pub fn bounce_back_reference(reference: Reference) -> Reference {
            reference
        }

        pub fn bounce_back_owned(
            owned: Owned<ChildReferenceHolder>,
        ) -> Owned<ChildReferenceHolder> {
            owned
        }

        pub fn new_with_reference(reference: Reference) -> Owned<ChildReferenceHolder> {
            ChildReferenceHolder {
                reference: Some(reference),
            }
            .instantiate()
        }

        pub fn take_reference(&mut self) -> Option<Reference> {
            self.reference.take()
        }
    }
}

#[blueprint]
mod gated_target {
    struct GatedTarget {
        resource: ResourceAddress,
    }

    impl GatedTarget {
        pub fn instantiate(resource: ResourceAddress) -> Global<GatedTarget> {
            Self { resource }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn gated_action(&self) {
            Runtime::assert_access_rule(rule!(require(self.resource)));
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

#[derive(ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
#[sbor(transparent)]
pub struct TypedNonFungibleVaultReference(Reference);

impl Describe<ScryptoCustomTypeKind> for TypedNonFungibleVaultReference {
    const TYPE_ID: RustTypeId = RustTypeId::Novel([124u8; 20]);

    fn type_data() -> TypeData<ScryptoCustomTypeKind, RustTypeId> {
        TypeData {
            kind: TypeKind::Custom(ScryptoCustomTypeKind::Reference),
            metadata: TypeMetadata::no_child_names("TypedNonFungibleVaultReference"),
            validation: TypeValidation::Custom(ScryptoCustomTypeValidation::Reference(
                ReferenceValidation::IsInternalTyped(
                    Some(RESOURCE_PACKAGE),
                    NON_FUNGIBLE_VAULT_BLUEPRINT.to_string(),
                ),
            )),
        }
    }

    fn add_all_dependencies(_aggregator: &mut TypeAggregator<ScryptoCustomTypeKind>) {}
}
