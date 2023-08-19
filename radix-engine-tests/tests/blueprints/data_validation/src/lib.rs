use sbor::*;
use scrypto::api::key_value_entry_api::*;
use scrypto::api::key_value_store_api::*;
use scrypto::api::LockFlags;
use scrypto::prelude::*;

#[blueprint]
mod data_validation {

    struct DataValidation {
        vault: Vault,
        reference: ResourceAddress,
    }

    impl DataValidation {
        pub fn new() -> Global<DataValidation> {
            let resource = ResourceBuilder::new_fungible(OwnerRole::None).mint_initial_supply(100);

            Self {
                vault: Vault::with_bucket(resource),
                reference: XRD,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn accept_empty_bucket(&self, bucket: Bucket) {
            bucket.drop_empty()
        }

        pub fn accept_non_empty_bucket(&self, bucket: Bucket) -> Bucket {
            bucket
        }

        pub fn accept_proof(&self, proof: Proof) {
            proof.drop()
        }

        pub fn return_proof_for_bucket(&self) -> Bucket {
            let proof = self.vault.as_fungible().create_proof_of_amount(dec!(1));
            Bucket(proof.0 .0)
        }

        pub fn return_bucket_for_proof(&mut self) -> Proof {
            let bucket = self.vault.take(1);
            Proof(bucket.0)
        }

        pub fn create_object_with_illegal_data() {
            let bucket = Bucket::new(XRD);

            Self {
                vault: Vault(bucket.0),
                reference: XRD,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize();
        }

        pub fn update_state_with_illegal_data(&mut self) {
            let actor = Runtime::global_address();
            self.reference = unsafe { ResourceAddress::new_unchecked(actor.into()) };
        }

        pub fn can_pass_own_as_reference(&mut self) -> Reference {
            let proof = self.vault.as_fungible().create_proof_of_amount(dec!(1));
            Reference(proof.0 .0.into())
        }

        pub fn accept_custom_reference(&self, _: CustomReference) {}
    }
}

#[derive(ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
#[sbor(transparent)]
pub struct CustomReference(Reference);

impl Describe<ScryptoCustomTypeKind> for CustomReference {
    const TYPE_ID: GlobalTypeId = GlobalTypeId::Novel([123u8; 20]);

    fn type_data() -> TypeData<ScryptoCustomTypeKind, GlobalTypeId> {
        TypeData {
            kind: TypeKind::Custom(ScryptoCustomTypeKind::Reference),
            metadata: TypeMetadata::no_child_names("CustomReference"),
            validation: TypeValidation::Custom(ScryptoCustomTypeValidation::Reference(
                ReferenceValidation::IsGlobalTyped(
                    Some(RESOURCE_PACKAGE),
                    FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                ),
            )),
        }
    }
}

#[blueprint]
mod vec_of_u8_underflow {
    struct VecOfU8Underflow {
        kv_store: KeyValueStore<u32, Vec<u8>>,
    }

    impl VecOfU8Underflow {
        pub fn write_vec_u8_underflow_to_key_value_store() -> Global<VecOfU8Underflow> {
            // Construct large SBOR payload
            let mut vec = Vec::<u8>::with_capacity(1 * 1024 * 1024);
            unsafe {
                vec.set_len(1 * 1024 * 1024);
            }
            (&mut vec[0..7]).copy_from_slice(&[
                // 92 = Scrypto SBOR
                92, // 32 = VALUE_KIND_ARRAY
                32, // 7 = U8 in the array
                7,
                // Length of 99999993 expressed as VLQ.
                // NOTE: This is longer than the buffer length of 1048576
                // Essentially to the engine this looks like this SBOR payload has been truncated, ie underflow
                249, 193, 215, 47,
            ]);

            // Create a KVStore
            let kv_store = KeyValueStore::<u32, Vec<u8>>::new();

            // Insert into store
            let key_payload = scrypto_encode(&1u32).unwrap();
            let value_payload = vec;
            let handle = ScryptoVmV1Api
                .key_value_store_open_entry(
                    kv_store.id.as_node_id(),
                    &key_payload,
                    LockFlags::MUTABLE,
                )
                .unwrap();
            ScryptoVmV1Api
                .key_value_entry_set(handle, value_payload)
                .unwrap();
            ScryptoVmV1Api.key_value_entry_close(handle).unwrap();

            // Put the kv store into a component
            VecOfU8Underflow { kv_store }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }
    }
}
