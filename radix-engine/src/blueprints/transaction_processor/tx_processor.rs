use crate::blueprints::resource::WorktopSubstate;
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::errors::SystemUpstreamError;
use crate::kernel::kernel_api::KernelNodeApi;
use crate::system::node_init::ModuleInit;
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::types::*;
use native_sdk::resource::{ComponentAuthZone, SysBucket, SysProof, Worktop};
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::node_modules::auth::{
    AccessRulesSetMethodAccessRuleInput, ACCESS_RULES_SET_METHOD_ACCESS_RULE_IDENT,
};
use radix_engine_interface::api::node_modules::metadata::{
    MetadataRemoveInput, MetadataSetInput, METADATA_REMOVE_IDENT, METADATA_SET_IDENT,
};
use radix_engine_interface::api::node_modules::royalty::{
    ComponentClaimRoyaltyInput, ComponentSetRoyaltyConfigInput,
    COMPONENT_ROYALTY_CLAIM_ROYALTY_IDENT, COMPONENT_ROYALTY_SET_ROYALTY_CONFIG_IDENT,
};
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::blueprints::transaction_processor::*;
use sbor::rust::prelude::*;
use transaction::data::transform;
use transaction::data::TransformHandler;
use transaction::errors::ManifestIdAllocationError;
use transaction::model::*;
use transaction::validation::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum TransactionProcessorError {
    TransactionEpochNotYetValid {
        valid_from: u64,
        current_epoch: u64,
    },
    TransactionEpochNoLongerValid {
        valid_until: u64,
        current_epoch: u64,
    },
    BucketNotFound(u32),
    ProofNotFound(u32),
    BlobNotFound(Hash),
    IdAllocationError(ManifestIdAllocationError),
    InvalidCallData(DecodeError),
    InvalidPackageSchema(DecodeError),
}

pub struct TransactionProcessorBlueprint;

impl TransactionProcessorBlueprint {
    pub(crate) fn run<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let input: TransactionProcessorRunInput = input.as_typed().map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;

        // Runtime transaction validation
        for request in &input.runtime_validations {
            TransactionProcessor::perform_validation(request, api)?;
        }

        // Create a worktop
        let worktop_node_id = api.kernel_allocate_node_id(EntityType::InternalGenericComponent)?;
        api.kernel_create_node(
            worktop_node_id,
            btreemap!(
                SysModuleId::Object.into() => btreemap!(
                    WorktopOffset::Worktop.into() => IndexedScryptoValue::from_typed(&WorktopSubstate::new())
                ),
                SysModuleId::TypeInfo.into() => ModuleInit::TypeInfo(
                    TypeInfoSubstate::Object(ObjectInfo {
                        blueprint: Blueprint::new(&RESOURCE_PACKAGE, WORKTOP_BLUEPRINT),
                        global: false,
                        outer_object: None,
                    })
                ).to_substates()
            ),
        )?;
        let worktop = Worktop(Own(worktop_node_id));

        // Decode instructions
        let instructions: Vec<Instruction> = manifest_decode(&input.instructions).unwrap();

        let mut processor = TransactionProcessor::new(input.blobs);
        let mut outputs = Vec::new();
        for (index, inst) in instructions.into_iter().enumerate() {
            api.update_instruction_index(index)?;

            let result = match inst {
                Instruction::TakeFromWorktop { resource_address } => {
                    let bucket = worktop.sys_take_all(resource_address, api)?;
                    processor.create_manifest_bucket(bucket)?;
                    InstructionOutput::None
                }
                Instruction::TakeFromWorktopByAmount {
                    amount,
                    resource_address,
                } => {
                    let bucket = worktop.sys_take(resource_address, amount, api)?;
                    processor.create_manifest_bucket(bucket)?;
                    InstructionOutput::None
                }
                Instruction::TakeFromWorktopByIds {
                    ids,
                    resource_address,
                } => {
                    let bucket = worktop.sys_take_non_fungibles(resource_address, ids, api)?;
                    processor.create_manifest_bucket(bucket)?;
                    InstructionOutput::None
                }
                Instruction::ReturnToWorktop { bucket_id } => {
                    let bucket = processor.take_bucket(&bucket_id)?;
                    worktop.sys_put(bucket, api)?;
                    InstructionOutput::None
                }
                Instruction::AssertWorktopContains { resource_address } => {
                    worktop.sys_assert_contains(resource_address, api)?;
                    InstructionOutput::None
                }
                Instruction::AssertWorktopContainsByAmount {
                    amount,
                    resource_address,
                } => {
                    worktop.sys_assert_contains_amount(resource_address, amount, api)?;
                    InstructionOutput::None
                }
                Instruction::AssertWorktopContainsByIds {
                    ids,
                    resource_address,
                } => {
                    worktop.sys_assert_contains_non_fungibles(resource_address, ids, api)?;
                    InstructionOutput::None
                }
                Instruction::PopFromAuthZone {} => {
                    let proof = ComponentAuthZone::sys_pop(api)?;
                    processor.create_manifest_proof(proof)?;
                    InstructionOutput::None
                }
                Instruction::ClearAuthZone => {
                    ComponentAuthZone::sys_clear(api)?;
                    InstructionOutput::None
                }
                Instruction::ClearSignatureProofs => {
                    ComponentAuthZone::sys_clear_signature_proofs(api)?;
                    InstructionOutput::None
                }
                Instruction::PushToAuthZone { proof_id } => {
                    let proof = processor.take_proof(&proof_id)?;
                    ComponentAuthZone::sys_push(proof, api)?;
                    InstructionOutput::None
                }
                Instruction::CreateProofFromAuthZone { resource_address } => {
                    let proof = ComponentAuthZone::sys_create_proof(resource_address, api)?;
                    processor.create_manifest_proof(proof)?;
                    InstructionOutput::None
                }
                Instruction::CreateProofFromAuthZoneByAmount {
                    amount,
                    resource_address,
                } => {
                    let proof = ComponentAuthZone::sys_create_proof_by_amount(
                        amount,
                        resource_address,
                        api,
                    )?;
                    processor.create_manifest_proof(proof)?;
                    InstructionOutput::None
                }
                Instruction::CreateProofFromAuthZoneByIds {
                    ids,
                    resource_address,
                } => {
                    let proof =
                        ComponentAuthZone::sys_create_proof_by_ids(&ids, resource_address, api)?;
                    processor.create_manifest_proof(proof)?;
                    InstructionOutput::None
                }
                Instruction::CreateProofFromBucket { bucket_id } => {
                    let bucket = processor.get_bucket(&bucket_id)?;
                    let proof = bucket.sys_create_proof(api)?;
                    processor.create_manifest_proof(proof)?;
                    InstructionOutput::None
                }
                Instruction::CloneProof { proof_id } => {
                    let proof = processor.get_proof(&proof_id)?;
                    let proof = proof.sys_clone(api)?;
                    processor.create_manifest_proof(proof)?;
                    InstructionOutput::None
                }
                Instruction::DropProof { proof_id } => {
                    let proof = processor.take_proof(&proof_id)?;
                    proof.sys_drop(api)?;
                    InstructionOutput::None
                }
                Instruction::DropAllProofs => {
                    // NB: the difference between DROP_ALL_PROOFS and CLEAR_AUTH_ZONE is that
                    // the former will drop all named proofs before clearing the auth zone.

                    for (_, real_id) in processor.proof_id_mapping.drain(..) {
                        let proof = Proof(Own(real_id));
                        proof.sys_drop(api).map(|_| IndexedScryptoValue::unit())?;
                    }
                    ComponentAuthZone::sys_clear(api)?;
                    InstructionOutput::None
                }
                Instruction::CallFunction {
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
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result, &worktop, api,
                    )?;
                    InstructionOutput::CallReturn(result.into())
                }
                Instruction::CallMethod {
                    component_address,
                    method_name,
                    args,
                } => {
                    let mut processor_with_api = TransactionProcessorWithApi {
                        worktop,
                        processor,
                        api,
                    };
                    let scrypto_value = transform(args, &mut processor_with_api)?;
                    processor = processor_with_api.processor;

                    let rtn = api.call_method(
                        component_address.as_node_id(),
                        &method_name,
                        scrypto_encode(&scrypto_value).unwrap(),
                    )?;
                    let result = IndexedScryptoValue::from_vec(rtn).unwrap();
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result, &worktop, api,
                    )?;
                    InstructionOutput::CallReturn(result.into())
                }
                Instruction::PublishPackage {
                    code,
                    schema,
                    royalty_config,
                    metadata,
                } => {
                    let code = processor.get_blob(&code)?;

                    // TODO: remove clone by allowing invocation to have references, like in TransactionProcessorRunInvocation.
                    let result = api.call_function(
                        PACKAGE_PACKAGE,
                        PACKAGE_BLUEPRINT,
                        PACKAGE_PUBLISH_WASM_IDENT,
                        scrypto_encode(&PackagePublishWasmInput {
                            code: code.to_vec(), // TODO: cow?
                            schema: schema.clone(),
                            royalty_config: royalty_config.clone(),
                            metadata: metadata.clone(),
                        })
                        .unwrap(),
                    )?;

                    let result_indexed = IndexedScryptoValue::from_vec(result).unwrap();
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result_indexed,
                        &worktop,
                        api,
                    )?;

                    InstructionOutput::CallReturn(result_indexed.into())
                }
                Instruction::PublishPackageAdvanced {
                    code,
                    schema,
                    royalty_config,
                    metadata,
                    access_rules,
                } => {
                    let code = processor.get_blob(&code)?;

                    // TODO: remove clone by allowing invocation to have references, like in TransactionProcessorRunInvocation.
                    let result = api.call_function(
                        PACKAGE_PACKAGE,
                        PACKAGE_BLUEPRINT,
                        PACKAGE_PUBLISH_WASM_ADVANCED_IDENT,
                        scrypto_encode(&PackagePublishWasmAdvancedInput {
                            package_address: None,
                            code: code.to_vec(), // TODO: cow?
                            schema: schema.clone(),
                            access_rules: access_rules.clone(),
                            royalty_config: royalty_config.clone(),
                            metadata: metadata.clone(),
                        })
                        .unwrap(),
                    )?;

                    let result_indexed = IndexedScryptoValue::from_vec(result).unwrap();
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result_indexed,
                        &worktop,
                        api,
                    )?;

                    InstructionOutput::CallReturn(result_indexed.into())
                }
                Instruction::BurnResource { bucket_id } => {
                    let bucket = processor.take_bucket(&bucket_id)?;
                    let rtn = bucket.sys_burn(api)?;

                    let result = IndexedScryptoValue::from_typed(&rtn);
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result, &worktop, api,
                    )?;
                    InstructionOutput::CallReturn(result.into())
                }
                Instruction::MintFungible {
                    resource_address,
                    amount,
                } => {
                    let rtn = api.call_method(
                        resource_address.as_node_id(),
                        FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT,
                        scrypto_encode(&FungibleResourceManagerMintInput { amount }).unwrap(),
                    )?;

                    let result = IndexedScryptoValue::from_vec(rtn).unwrap();
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result, &worktop, api,
                    )?;
                    InstructionOutput::CallReturn(result.into())
                }
                Instruction::MintNonFungible {
                    resource_address,
                    args,
                } => {
                    let mut processor_with_api = TransactionProcessorWithApi {
                        worktop,
                        processor,
                        api,
                    };
                    let scrypto_value = transform(args, &mut processor_with_api)?;
                    processor = processor_with_api.processor;

                    let rtn = api.call_method(
                        resource_address.as_node_id(),
                        NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT,
                        scrypto_encode(&scrypto_value).unwrap(),
                    )?;
                    let result = IndexedScryptoValue::from_vec(rtn).unwrap();
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result, &worktop, api,
                    )?;
                    InstructionOutput::CallReturn(result.into())
                }
                Instruction::MintUuidNonFungible {
                    resource_address,
                    args,
                } => {
                    let mut processor_with_api = TransactionProcessorWithApi {
                        worktop,
                        processor,
                        api,
                    };
                    let scrypto_value = transform(args, &mut processor_with_api)?;
                    processor = processor_with_api.processor;
                    let rtn = api.call_method(
                        resource_address.as_node_id(),
                        NON_FUNGIBLE_RESOURCE_MANAGER_MINT_UUID_IDENT,
                        scrypto_encode(&scrypto_value).unwrap(),
                    )?;

                    let result = IndexedScryptoValue::from_vec(rtn).unwrap();
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result, &worktop, api,
                    )?;
                    InstructionOutput::CallReturn(result.into())
                }
                Instruction::RecallResource { vault_id, amount } => {
                    let rtn = api.call_method(
                        vault_id.as_node_id(),
                        VAULT_RECALL_IDENT,
                        scrypto_encode(&VaultRecallInput { amount }).unwrap(),
                    )?;

                    let result = IndexedScryptoValue::from_vec(rtn).unwrap();
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result, &worktop, api,
                    )?;
                    InstructionOutput::CallReturn(result.into())
                }
                Instruction::SetMetadata {
                    entity_address,
                    key,
                    value,
                } => {
                    let receiver = entity_address.into();
                    let result = api.call_module_method(
                        &receiver,
                        ObjectModuleId::Metadata,
                        METADATA_SET_IDENT,
                        scrypto_encode(&MetadataSetInput {
                            key: key.clone(),
                            value: scrypto_decode(&scrypto_encode(&value).unwrap()).unwrap(),
                        })
                        .unwrap(),
                    )?;

                    let result_indexed = IndexedScryptoValue::from_vec(result).unwrap();
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result_indexed,
                        &worktop,
                        api,
                    )?;

                    InstructionOutput::CallReturn(result_indexed.into())
                }
                Instruction::RemoveMetadata {
                    entity_address,
                    key,
                } => {
                    let receiver = entity_address.into();
                    let result = api.call_module_method(
                        &receiver,
                        ObjectModuleId::Metadata,
                        METADATA_REMOVE_IDENT,
                        scrypto_encode(&MetadataRemoveInput { key: key.clone() }).unwrap(),
                    )?;

                    let result_indexed = IndexedScryptoValue::from_vec(result).unwrap();
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result_indexed,
                        &worktop,
                        api,
                    )?;

                    InstructionOutput::CallReturn(result_indexed.into())
                }
                Instruction::SetPackageRoyaltyConfig {
                    package_address,
                    royalty_config,
                } => {
                    let result = api.call_module_method(
                        package_address.as_node_id(),
                        ObjectModuleId::SELF,
                        PACKAGE_SET_ROYALTY_CONFIG_IDENT,
                        scrypto_encode(&PackageSetRoyaltyConfigInput {
                            royalty_config: royalty_config.clone(),
                        })
                        .unwrap(),
                    )?;

                    let result_indexed = IndexedScryptoValue::from_vec(result).unwrap();
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result_indexed,
                        &worktop,
                        api,
                    )?;

                    InstructionOutput::CallReturn(result_indexed.into())
                }
                Instruction::SetComponentRoyaltyConfig {
                    component_address,
                    royalty_config,
                } => {
                    let result = api.call_module_method(
                        component_address.as_node_id(),
                        ObjectModuleId::Royalty,
                        COMPONENT_ROYALTY_SET_ROYALTY_CONFIG_IDENT,
                        scrypto_encode(&ComponentSetRoyaltyConfigInput {
                            royalty_config: royalty_config.clone(),
                        })
                        .unwrap(),
                    )?;

                    let result_indexed = IndexedScryptoValue::from_vec(result).unwrap();
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result_indexed,
                        &worktop,
                        api,
                    )?;

                    InstructionOutput::CallReturn(result_indexed.into())
                }
                Instruction::ClaimPackageRoyalty { package_address } => {
                    let result = api.call_module_method(
                        package_address.as_node_id(),
                        ObjectModuleId::SELF,
                        PACKAGE_CLAIM_ROYALTY_IDENT,
                        scrypto_encode(&PackageClaimRoyaltyInput {}).unwrap(),
                    )?;

                    let result_indexed = IndexedScryptoValue::from_vec(result).unwrap();
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result_indexed,
                        &worktop,
                        api,
                    )?;

                    InstructionOutput::CallReturn(result_indexed.into())
                }
                Instruction::ClaimComponentRoyalty { component_address } => {
                    let result = api.call_module_method(
                        component_address.as_node_id(),
                        ObjectModuleId::Royalty,
                        COMPONENT_ROYALTY_CLAIM_ROYALTY_IDENT,
                        scrypto_encode(&ComponentClaimRoyaltyInput {}).unwrap(),
                    )?;

                    let result_indexed = IndexedScryptoValue::from_vec(result).unwrap();
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result_indexed,
                        &worktop,
                        api,
                    )?;

                    InstructionOutput::CallReturn(result_indexed.into())
                }
                Instruction::SetMethodAccessRule {
                    entity_address,
                    key,
                    rule,
                } => {
                    let receiver = entity_address.into();
                    let result = api.call_module_method(
                        &receiver,
                        ObjectModuleId::AccessRules,
                        ACCESS_RULES_SET_METHOD_ACCESS_RULE_IDENT,
                        scrypto_encode(&AccessRulesSetMethodAccessRuleInput {
                            object_key: ObjectKey::SELF,
                            method_key: key.clone(),
                            rule: AccessRuleEntry::AccessRule(rule.clone()),
                        })
                        .unwrap(),
                    )?;

                    let result_indexed = IndexedScryptoValue::from_vec(result).unwrap();
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result_indexed,
                        &worktop,
                        api,
                    )?;

                    InstructionOutput::CallReturn(result_indexed.into())
                }
            };
            outputs.push(result);
        }

        worktop.sys_drop(api)?;

        Ok(IndexedScryptoValue::from_typed(&outputs))
    }
}

struct TransactionProcessor<'blob> {
    proof_id_mapping: IndexMap<ManifestProof, NodeId>,
    bucket_id_mapping: NonIterMap<ManifestBucket, NodeId>,
    id_allocator: ManifestIdAllocator,
    blobs_by_hash: BTreeMap<Hash, Cow<'blob, [u8]>>,
}

impl<'blob> TransactionProcessor<'blob> {
    fn new(blobs_by_hash: BTreeMap<Hash, Cow<'blob, [u8]>>) -> Self {
        Self {
            proof_id_mapping: index_map_new(),
            bucket_id_mapping: NonIterMap::new(),
            id_allocator: ManifestIdAllocator::new(),
            blobs_by_hash,
        }
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
        let new_id = self.id_allocator.new_bucket_id().map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::TransactionProcessorError(
                TransactionProcessorError::IdAllocationError(e),
            ))
        })?;
        self.bucket_id_mapping
            .insert(new_id.clone(), bucket.0.into());
        Ok(new_id)
    }

    fn create_manifest_proof(&mut self, proof: Proof) -> Result<ManifestProof, RuntimeError> {
        let new_id = self.id_allocator.new_proof_id().map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::TransactionProcessorError(
                TransactionProcessorError::IdAllocationError(e),
            ))
        })?;
        self.proof_id_mapping.insert(new_id.clone(), proof.0.into());
        Ok(new_id)
    }

    fn move_proofs_to_authzone_and_buckets_to_worktop<Y>(
        value: &IndexedScryptoValue,
        worktop: &Worktop,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        // Auto move into worktop & auth_zone
        for owned_node in value.owned_nodes() {
            let info = api.get_object_info(owned_node)?;
            match (
                info.blueprint.package_address,
                info.blueprint.blueprint_name.as_str(),
            ) {
                (RESOURCE_PACKAGE, FUNGIBLE_BUCKET_BLUEPRINT)
                | (RESOURCE_PACKAGE, NON_FUNGIBLE_BUCKET_BLUEPRINT) => {
                    let bucket = Bucket(Own(owned_node.clone()));
                    worktop.sys_put(bucket, api)?;
                }
                (RESOURCE_PACKAGE, FUNGIBLE_PROOF_BLUEPRINT)
                | (RESOURCE_PACKAGE, NON_FUNGIBLE_PROOF_BLUEPRINT) => {
                    let proof = Proof(Own(owned_node.clone()));
                    ComponentAuthZone::sys_push(proof, api)?;
                }
                _ => {}
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
                let current_epoch = Runtime::sys_current_epoch(env)?;

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
                                valid_until: *end_epoch_exclusive - 1,
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

struct TransactionProcessorWithApi<'blob, 'a, Y: ClientApi<RuntimeError>> {
    worktop: Worktop,
    processor: TransactionProcessor<'blob>,
    api: &'a mut Y,
}

impl<'blob, 'a, Y: ClientApi<RuntimeError>> TransformHandler<RuntimeError>
    for TransactionProcessorWithApi<'blob, 'a, Y>
{
    fn replace_bucket(&mut self, b: ManifestBucket) -> Result<Own, RuntimeError> {
        self.processor.take_bucket(&b).map(|x| x.0)
    }

    fn replace_proof(&mut self, p: ManifestProof) -> Result<Own, RuntimeError> {
        self.processor.take_proof(&p).map(|x| x.0)
    }

    fn replace_expression(&mut self, e: ManifestExpression) -> Result<Vec<Own>, RuntimeError> {
        match e {
            ManifestExpression::EntireWorktop => {
                let buckets = self.worktop.sys_drain(self.api)?;
                Ok(buckets.into_iter().map(|b| b.0).collect())
            }
            ManifestExpression::EntireAuthZone => {
                let proofs = ComponentAuthZone::sys_drain(self.api)?;
                Ok(proofs.into_iter().map(|p| p.0).collect())
            }
        }
    }

    fn replace_blob(&mut self, b: ManifestBlobRef) -> Result<Vec<u8>, RuntimeError> {
        Ok(self.processor.get_blob(&b)?.to_vec())
    }
}
