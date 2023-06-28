use sbor::*;
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
                reference: RADIX_TOKEN,
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
            let proof = self.vault.create_proof();
            Bucket(proof.0)
        }

        pub fn return_bucket_for_proof(&mut self) -> Proof {
            let bucket = self.vault.take(1);
            Proof(bucket.0)
        }

        pub fn create_object_with_illegal_data() {
            let bucket = Bucket::new(RADIX_TOKEN);

            Self {
                vault: Vault(bucket.0),
                reference: RADIX_TOKEN,
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
            let proof = self.vault.create_proof();
            Reference(proof.0.into())
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
