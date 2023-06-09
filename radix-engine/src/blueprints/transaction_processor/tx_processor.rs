use crate::blueprints::resource::WorktopSubstate;
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::KernelNodeApi;
use crate::kernel::kernel_api::KernelSubstateApi;
use crate::system::node_init::ModuleInit;
use crate::system::node_modules::type_info::TypeInfoBlueprint;
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::types::*;
use native_sdk::resource::NativeNonFungibleBucket;
use native_sdk::resource::{NativeBucket, NativeProof, Worktop};
use native_sdk::runtime::{LocalAuthZone, Runtime};
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::blueprints::transaction_processor::*;
use sbor::rust::prelude::*;
use transaction::data::transform;
use transaction::data::TransformHandler;
use transaction::model::*;
use transaction::validation::*;

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct TransactionProcessorRunInput {
    pub transaction_hash: Hash,
    pub runtime_validations: Vec<RuntimeValidationRequest>,
    pub manifest_encoded_instructions: Vec<u8>,
    pub global_address_reservations: Vec<GlobalAddressReservation>,
    pub references: Vec<Reference>, // Required so that the kernel passes the references to the processor frame
    pub blobs: IndexMap<Hash, Vec<u8>>,
}

// This needs to match the above, but is easily encodable to avoid cloning from the transaction payload to encode
#[derive(Debug, Eq, PartialEq, ScryptoEncode)]
pub struct TransactionProcessorRunInputEfficientEncodable<'a> {
    pub transaction_hash: &'a Hash,
    pub runtime_validations: &'a [RuntimeValidationRequest],
    pub manifest_encoded_instructions: &'a [u8],
    pub global_address_reservations: Vec<GlobalAddressReservation>,
    pub references: &'a IndexSet<Reference>,
    pub blobs: &'a IndexMap<Hash, Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum TransactionProcessorError {
    TransactionEpochNotYetValid {
        valid_from: Epoch,
        current_epoch: Epoch,
    },
    TransactionEpochNoLongerValid {
        valid_until: Epoch,
        current_epoch: Epoch,
    },
    BucketNotFound(u32),
    ProofNotFound(u32),
    OwnedNotFound(u32),
    BlobNotFound(Hash),
    InvalidCallData(DecodeError),
    InvalidPackageSchema(DecodeError),
}

pub struct TransactionProcessorBlueprint;

macro_rules! handle_call_method {
    ($module_id:expr, $node_id:expr, $direct_access:expr, $method_name:expr, $args:expr, $worktop:expr, $processor:expr, $api:expr) => {{
        let mut processor_with_api = TransactionProcessorWithApi {
            worktop: $worktop,
            processor: $processor,
            api: $api,
        };
        let scrypto_value = transform($args, &mut processor_with_api)?;
        $processor = processor_with_api.processor;

        let rtn = $api.call_method_advanced(
            $node_id,
            $direct_access,
            $module_id,
            &$method_name,
            scrypto_encode(&scrypto_value).unwrap(),
        )?;
        let result = IndexedScryptoValue::from_vec(rtn).unwrap();
        $processor.handle_call_return_data(&result, &$worktop, $api)?;
        InstructionOutput::CallReturn(result.into())
    }};
}

impl TransactionProcessorBlueprint {
    pub(crate) fn run<Y, L: Default>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi<L> + ClientApi<RuntimeError>,
    {
        let input: TransactionProcessorRunInput = input
            .as_typed()
            .map_err(|e| RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e)))?;

        // Runtime transaction validation
        for request in &input.runtime_validations {
            TransactionProcessor::perform_validation(request, api)?;
        }

        // Create a worktop
        let worktop_node_id = api.kernel_allocate_node_id(EntityType::InternalGenericComponent)?;
        api.kernel_create_node(
            worktop_node_id,
            btreemap!(
                OBJECT_BASE_PARTITION => btreemap!(
                    WorktopField::Worktop.into() => IndexedScryptoValue::from_typed(&WorktopSubstate::new())
                ),
                TYPE_INFO_FIELD_PARTITION => ModuleInit::TypeInfo(
                    TypeInfoSubstate::Object(ObjectInfo {
                        blueprint: BlueprintId::new(&RESOURCE_PACKAGE, WORKTOP_BLUEPRINT),
                        global: false,
                        outer_object: None,
                        instance_schema: None,
                    })
                ).to_substates()
            ),
        )?;
        let worktop = Worktop(Own(worktop_node_id));
        let instructions =
            manifest_decode::<Vec<InstructionV1>>(&input.manifest_encoded_instructions)
                .expect("Instructions could not be decoded");
        let mut processor =
            TransactionProcessor::new(input.blobs, input.global_address_reservations);
        let mut outputs = Vec::new();
        for (index, inst) in instructions.into_iter().enumerate() {
            api.update_instruction_index(index)?;

            let result = match inst {
                InstructionV1::TakeAllFromWorktop { resource_address } => {
                    let bucket = worktop.take_all(resource_address, api)?;
                    processor.create_manifest_bucket(bucket)?;
                    InstructionOutput::None
                }
                InstructionV1::TakeFromWorktop {
                    amount,
                    resource_address,
                } => {
                    let bucket = worktop.take(resource_address, amount, api)?;
                    processor.create_manifest_bucket(bucket)?;
                    InstructionOutput::None
                }
                InstructionV1::TakeNonFungiblesFromWorktop {
                    ids,
                    resource_address,
                } => {
                    let bucket = worktop.take_non_fungibles(
                        resource_address,
                        ids.into_iter().collect(),
                        api,
                    )?;
                    processor.create_manifest_bucket(bucket)?;
                    InstructionOutput::None
                }
                InstructionV1::ReturnToWorktop { bucket_id } => {
                    let bucket = processor.take_bucket(&bucket_id)?;
                    worktop.put(bucket, api)?;
                    InstructionOutput::None
                }
                InstructionV1::AssertWorktopContains {
                    amount,
                    resource_address,
                } => {
                    worktop.assert_contains_amount(resource_address, amount, api)?;
                    InstructionOutput::None
                }
                InstructionV1::AssertWorktopContainsNonFungibles {
                    ids,
                    resource_address,
                } => {
                    worktop.assert_contains_non_fungibles(
                        resource_address,
                        ids.into_iter().collect(),
                        api,
                    )?;
                    InstructionOutput::None
                }
                InstructionV1::PopFromAuthZone {} => {
                    let proof = LocalAuthZone::pop(api)?;
                    processor.create_manifest_proof(proof)?;
                    InstructionOutput::None
                }
                InstructionV1::ClearAuthZone => {
                    LocalAuthZone::clear(api)?;
                    InstructionOutput::None
                }
                InstructionV1::ClearSignatureProofs => {
                    LocalAuthZone::clear_signature_proofs(api)?;
                    InstructionOutput::None
                }
                InstructionV1::PushToAuthZone { proof_id } => {
                    let proof = processor.take_proof(&proof_id)?;
                    LocalAuthZone::push(proof, api)?;
                    InstructionOutput::None
                }
                InstructionV1::CreateProofFromAuthZone { resource_address } => {
                    let proof = LocalAuthZone::create_proof(resource_address, api)?;
                    processor.create_manifest_proof(proof)?;
                    InstructionOutput::None
                }
                InstructionV1::CreateProofFromAuthZoneOfAmount {
                    amount,
                    resource_address,
                } => {
                    let proof =
                        LocalAuthZone::create_proof_of_amount(amount, resource_address, api)?;
                    processor.create_manifest_proof(proof)?;
                    InstructionOutput::None
                }
                InstructionV1::CreateProofFromAuthZoneOfNonFungibles {
                    ids,
                    resource_address,
                } => {
                    let proof = LocalAuthZone::create_proof_of_non_fungibles(
                        &ids.into_iter().collect(),
                        resource_address,
                        api,
                    )?;
                    processor.create_manifest_proof(proof)?;
                    InstructionOutput::None
                }
                InstructionV1::CreateProofFromAuthZoneOfAll { resource_address } => {
                    let proof = LocalAuthZone::create_proof_of_all(resource_address, api)?;
                    processor.create_manifest_proof(proof)?;
                    InstructionOutput::None
                }
                InstructionV1::CreateProofFromBucket { bucket_id } => {
                    let bucket = processor.get_bucket(&bucket_id)?;
                    let proof = bucket.create_proof(api)?;
                    processor.create_manifest_proof(proof)?;
                    InstructionOutput::None
                }
                InstructionV1::CreateProofFromBucketOfAmount { bucket_id, amount } => {
                    let bucket = processor.get_bucket(&bucket_id)?;
                    let proof = bucket.create_proof_of_amount(amount, api)?;
                    processor.create_manifest_proof(proof)?;
                    InstructionOutput::None
                }
                InstructionV1::CreateProofFromBucketOfNonFungibles { bucket_id, ids } => {
                    let bucket = processor.get_bucket(&bucket_id)?;
                    let proof =
                        bucket.create_proof_of_non_fungibles(ids.into_iter().collect(), api)?;
                    processor.create_manifest_proof(proof)?;
                    InstructionOutput::None
                }
                InstructionV1::CreateProofFromBucketOfAll { bucket_id } => {
                    let bucket = processor.get_bucket(&bucket_id)?;
                    let proof = bucket.create_proof_of_all(api)?;
                    processor.create_manifest_proof(proof)?;
                    InstructionOutput::None
                }
                InstructionV1::BurnResource { bucket_id } => {
                    let bucket = processor.take_bucket(&bucket_id)?;
                    let rtn = bucket.burn(api)?;

                    let result = IndexedScryptoValue::from_typed(&rtn);
                    processor.handle_call_return_data(&result, &worktop, api)?;
                    InstructionOutput::CallReturn(result.into())
                }
                InstructionV1::CloneProof { proof_id } => {
                    let proof = processor.get_proof(&proof_id)?;
                    let proof = proof.clone(api)?;
                    processor.create_manifest_proof(proof)?;
                    InstructionOutput::None
                }
                InstructionV1::DropProof { proof_id } => {
                    let proof = processor.take_proof(&proof_id)?;
                    proof.drop(api)?;
                    InstructionOutput::None
                }
                InstructionV1::CallFunction {
                    package_address,
                    blueprint_name,
                    function_name,
                    args,
                } => {
                    let mut processor_with_api = TransactionProcessorWithApi {
                        worktop,
                        processor,
                        api,
                    };
                    let scrypto_value = transform(args, &mut processor_with_api)?;
                    processor = processor_with_api.processor;

                    let rtn = api.call_function(
                        package_address,
                        &blueprint_name,
                        &function_name,
                        scrypto_encode(&scrypto_value).unwrap(),
                    )?;

                    let result = IndexedScryptoValue::from_vec(rtn).unwrap();
                    processor.handle_call_return_data(&result, &worktop, api)?;
                    InstructionOutput::CallReturn(result.into())
                }
                InstructionV1::CallMethod {
                    address,
                    method_name,
                    args,
                } => {
                    handle_call_method!(
                        ObjectModuleId::Main,
                        address.as_node_id(),
                        false,
                        method_name,
                        args,
                        worktop,
                        processor,
                        api
                    )
                }
                InstructionV1::CallRoyaltyMethod {
                    address,
                    method_name,
                    args,
                } => {
                    handle_call_method!(
                        ObjectModuleId::Royalty,
                        address.as_node_id(),
                        false,
                        method_name,
                        args,
                        worktop,
                        processor,
                        api
                    )
                }
                InstructionV1::CallMetadataMethod {
                    address,
                    method_name,
                    args,
                } => {
                    handle_call_method!(
                        ObjectModuleId::Metadata,
                        address.as_node_id(),
                        false,
                        method_name,
                        args,
                        worktop,
                        processor,
                        api
                    )
                }
                InstructionV1::CallAccessRulesMethod {
                    address,
                    method_name,
                    args,
                } => {
                    handle_call_method!(
                        ObjectModuleId::AccessRules,
                        address.as_node_id(),
                        false,
                        method_name,
                        args,
                        worktop,
                        processor,
                        api
                    )
                }
                InstructionV1::CallDirectVaultMethod {
                    address,
                    method_name,
                    args,
                } => {
                    handle_call_method!(
                        ObjectModuleId::Main,
                        address.as_node_id(),
                        true,
                        method_name,
                        args,
                        worktop,
                        processor,
                        api
                    )
                }
                InstructionV1::DropAllProofs => {
                    // NB: the difference between DROP_ALL_PROOFS and CLEAR_AUTH_ZONE is that
                    // the former will drop all named proofs before clearing the auth zone.

                    for (_, real_id) in processor.proof_id_mapping.drain(..) {
                        let proof = Proof(Own(real_id));
                        proof.drop(api).map(|_| IndexedScryptoValue::unit())?;
                    }
                    LocalAuthZone::clear(api)?;
                    InstructionOutput::None
                }
            };
            outputs.push(result);
        }

        worktop.drop(api)?;

        Ok(IndexedScryptoValue::from_typed(&outputs))
    }
}

struct TransactionProcessor {
    proof_id_mapping: IndexMap<ManifestProof, NodeId>,
    bucket_id_mapping: NonIterMap<ManifestBucket, NodeId>,
    own_id_mapping: NonIterMap<ManifestReservation, NodeId>,
    id_allocator: ManifestIdAllocator,
    blobs_by_hash: IndexMap<Hash, Vec<u8>>,
}

impl TransactionProcessor {
    fn new(
        blobs_by_hash: IndexMap<Hash, Vec<u8>>,
        global_address_reservations: Vec<GlobalAddressReservation>,
    ) -> Self {
        let mut processor = Self {
            proof_id_mapping: index_map_new(),
            bucket_id_mapping: NonIterMap::new(),
            own_id_mapping: NonIterMap::new(),
            id_allocator: ManifestIdAllocator::new(),
            blobs_by_hash,
        };
        for reservation in global_address_reservations {
            processor.create_manifest_own(reservation.0 .0).unwrap();
        }
        processor
    }

    fn take_own(&mut self, own_id: &ManifestReservation) -> Result<Own, RuntimeError> {
        let real_id = self
            .own_id_mapping
            .remove(own_id)
            .ok_or(RuntimeError::ApplicationError(
                ApplicationError::TransactionProcessorError(
                    TransactionProcessorError::OwnedNotFound(own_id.0),
                ),
            ))?;
        Ok(Own(real_id))
    }

    fn get_bucket(&mut self, bucket_id: &ManifestBucket) -> Result<Bucket, RuntimeError> {
        let real_id = self.bucket_id_mapping.get(bucket_id).cloned().ok_or(
            RuntimeError::ApplicationError(ApplicationError::TransactionProcessorError(
                TransactionProcessorError::BucketNotFound(bucket_id.0),
            )),
        )?;
        Ok(Bucket(Own(real_id)))
    }

    fn take_bucket(&mut self, bucket_id: &ManifestBucket) -> Result<Bucket, RuntimeError> {
        let real_id =
            self.bucket_id_mapping
                .remove(bucket_id)
                .ok_or(RuntimeError::ApplicationError(
                    ApplicationError::TransactionProcessorError(
                        TransactionProcessorError::BucketNotFound(bucket_id.0),
                    ),
                ))?;
        Ok(Bucket(Own(real_id)))
    }

    fn get_blob(&mut self, blob_ref: &ManifestBlobRef) -> Result<&[u8], RuntimeError> {
        let hash = Hash(blob_ref.0);
        self.blobs_by_hash
            .get(&hash)
            .map(|x| x.as_ref())
            .ok_or(RuntimeError::ApplicationError(
                ApplicationError::TransactionProcessorError(
                    TransactionProcessorError::BlobNotFound(hash),
                ),
            ))
    }

    fn get_proof(&mut self, proof_id: &ManifestProof) -> Result<Proof, RuntimeError> {
        let real_id =
            self.proof_id_mapping
                .get(proof_id)
                .cloned()
                .ok_or(RuntimeError::ApplicationError(
                    ApplicationError::TransactionProcessorError(
                        TransactionProcessorError::ProofNotFound(proof_id.0),
                    ),
                ))?;
        Ok(Proof(Own(real_id)))
    }

    fn take_proof(&mut self, proof_id: &ManifestProof) -> Result<Proof, RuntimeError> {
        let real_id =
            self.proof_id_mapping
                .remove(proof_id)
                .ok_or(RuntimeError::ApplicationError(
                    ApplicationError::TransactionProcessorError(
                        TransactionProcessorError::ProofNotFound(proof_id.0),
                    ),
                ))?;
        Ok(Proof(Own(real_id)))
    }

    fn create_manifest_bucket(&mut self, bucket: Bucket) -> Result<ManifestBucket, RuntimeError> {
        let new_id = self.id_allocator.new_bucket_id();
        self.bucket_id_mapping
            .insert(new_id.clone(), bucket.0.into());
        Ok(new_id)
    }

    fn create_manifest_proof(&mut self, proof: Proof) -> Result<ManifestProof, RuntimeError> {
        let new_id = self.id_allocator.new_proof_id();
        self.proof_id_mapping.insert(new_id.clone(), proof.0.into());
        Ok(new_id)
    }

    fn create_manifest_own(
        &mut self,
        node_id: NodeId,
    ) -> Result<ManifestReservation, RuntimeError> {
        let new_id = self.id_allocator.new_own_id();
        self.own_id_mapping.insert(new_id, node_id);
        Ok(new_id)
    }

    fn handle_call_return_data<Y, L: Default>(
        &mut self,
        value: &IndexedScryptoValue,
        worktop: &Worktop,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi<L> + ClientApi<RuntimeError>,
    {
        // Auto move into worktop & auth_zone
        for node_id in value.owned_nodes() {
            let info = TypeInfoBlueprint::get_type(node_id, api)?;
            match info {
                TypeInfoSubstate::Object(info) => match (
                    info.blueprint.package_address,
                    info.blueprint.blueprint_name.as_str(),
                ) {
                    (RESOURCE_PACKAGE, FUNGIBLE_BUCKET_BLUEPRINT)
                    | (RESOURCE_PACKAGE, NON_FUNGIBLE_BUCKET_BLUEPRINT) => {
                        let bucket = Bucket(Own(node_id.clone()));
                        worktop.put(bucket, api)?;
                    }
                    (RESOURCE_PACKAGE, FUNGIBLE_PROOF_BLUEPRINT)
                    | (RESOURCE_PACKAGE, NON_FUNGIBLE_PROOF_BLUEPRINT) => {
                        let proof = Proof(Own(node_id.clone()));
                        LocalAuthZone::push(proof, api)?;
                    }
                    _ => {
                        self.create_manifest_own(node_id.clone())?;
                    }
                },
                TypeInfoSubstate::KeyValueStore(_)
                | TypeInfoSubstate::GlobalAddressReservation(_) => {
                    self.create_manifest_own(node_id.clone())?;
                }
                TypeInfoSubstate::GlobalAddressPhantom(_) => unreachable!(),
            }
        }

        Ok(())
    }

    fn perform_validation<'a, Y>(
        request: &RuntimeValidationRequest,
        env: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientObjectApi<RuntimeError>,
    {
        let should_skip_assertion = request.skip_assertion;
        match &request.validation {
            RuntimeValidation::WithinEpochRange {
                start_epoch_inclusive,
                end_epoch_exclusive,
            } => {
                // TODO - Instead of doing a check of the exact epoch, we could do a check in range [X, Y]
                //        Which could allow for better caching of transaction validity over epoch boundaries
                let current_epoch = Runtime::current_epoch(env)?;

                if !should_skip_assertion && current_epoch < *start_epoch_inclusive {
                    return Err(RuntimeError::ApplicationError(
                        ApplicationError::TransactionProcessorError(
                            TransactionProcessorError::TransactionEpochNotYetValid {
                                valid_from: *start_epoch_inclusive,
                                current_epoch,
                            },
                        ),
                    ));
                }
                if !should_skip_assertion && current_epoch >= *end_epoch_exclusive {
                    return Err(RuntimeError::ApplicationError(
                        ApplicationError::TransactionProcessorError(
                            TransactionProcessorError::TransactionEpochNoLongerValid {
                                valid_until: end_epoch_exclusive.previous(),
                                current_epoch,
                            },
                        ),
                    ));
                }

                Ok(())
            }
            RuntimeValidation::IntentHashUniqueness { .. } => {
                // TODO - Add intent hash replay prevention here
                // This will to enable its removal from the node
                Ok(())
            }
        }
    }
}

struct TransactionProcessorWithApi<'a, Y: ClientApi<RuntimeError>> {
    worktop: Worktop,
    processor: TransactionProcessor,
    api: &'a mut Y,
}

impl<'a, Y: ClientApi<RuntimeError>> TransformHandler<RuntimeError>
    for TransactionProcessorWithApi<'a, Y>
{
    fn replace_bucket(&mut self, b: ManifestBucket) -> Result<Own, RuntimeError> {
        self.processor.take_bucket(&b).map(|x| x.0)
    }

    fn replace_proof(&mut self, p: ManifestProof) -> Result<Own, RuntimeError> {
        self.processor.take_proof(&p).map(|x| x.0)
    }

    fn replace_reservation(&mut self, p: ManifestReservation) -> Result<Own, RuntimeError> {
        self.processor.take_own(&p)
    }

    fn replace_allocated_address(
        &mut self,
        p: ManifestNamedAddress,
    ) -> Result<Reference, RuntimeError> {
        todo!()
    }

    fn replace_expression(&mut self, e: ManifestExpression) -> Result<Vec<Own>, RuntimeError> {
        match e {
            ManifestExpression::EntireWorktop => {
                let buckets = self.worktop.drain(self.api)?;
                Ok(buckets.into_iter().map(|b| b.0).collect())
            }
            ManifestExpression::EntireAuthZone => {
                let proofs = LocalAuthZone::drain(self.api)?;
                Ok(proofs.into_iter().map(|p| p.0).collect())
            }
        }
    }

    fn replace_blob(&mut self, b: ManifestBlobRef) -> Result<Vec<u8>, RuntimeError> {
        Ok(self.processor.get_blob(&b)?.to_vec())
    }
}
