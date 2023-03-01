use crate::blueprints::resource::{WorktopBlueprint, WorktopSubstate};
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::kernel::actor::Actor;
use crate::kernel::call_frame::CallFrameUpdate;
use crate::kernel::kernel_api::{
    ExecutableInvocation, Executor, KernelNodeApi, KernelSubstateApi, TemporaryResolvedInvocation,
};
use crate::system::node::{RENodeInit, RENodeModuleInit};
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::system::package::PackageError;
use crate::types::*;
use crate::wasm::WasmEngine;
use native_sdk::resource::{ComponentAuthZone, SysBucket, SysProof, Worktop};
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::node_modules::auth::{
    AccessRulesSetMethodAccessRuleInput, ACCESS_RULES_SET_METHOD_ACCESS_RULE_IDENT,
};
use radix_engine_interface::api::node_modules::metadata::{MetadataSetInput, METADATA_SET_IDENT};
use radix_engine_interface::api::node_modules::royalty::{
    ComponentClaimRoyaltyInput, ComponentSetRoyaltyConfigInput, PackageClaimRoyaltyInput,
    PackageSetRoyaltyConfigInput, COMPONENT_ROYALTY_CLAIM_ROYALTY_IDENT,
    COMPONENT_ROYALTY_SET_ROYALTY_CONFIG_IDENT, PACKAGE_ROYALTY_CLAIM_ROYALTY_IDENT,
    PACKAGE_ROYALTY_SET_ROYALTY_CONFIG_IDENT,
};
use radix_engine_interface::api::package::*;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::{IndexedScryptoValue, ScryptoValue};
use sbor::rust::borrow::Cow;
use transaction::data::model::*;
use transaction::data::to_address;
use transaction::data::transform;
use transaction::data::ManifestCustomValue;
use transaction::data::ManifestValue;
use transaction::data::TransformHandler;
use transaction::data::{manifest_decode, manifest_encode};
use transaction::errors::ManifestIdAllocationError;
use transaction::model::*;
use transaction::validation::*;

#[derive(Debug, ScryptoSbor)]
pub struct TransactionProcessorRunInvocation<'a> {
    pub transaction_hash: Hash,
    pub runtime_validations: Cow<'a, [RuntimeValidationRequest]>,
    pub instructions: Cow<'a, Vec<u8>>,
    pub blobs: Cow<'a, [Vec<u8>]>,
}

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
}

pub trait NativeOutput: ScryptoEncode + Debug + Send + Sync {}
impl<T: ScryptoEncode + Debug + Send + Sync> NativeOutput for T {}

#[derive(Debug, Clone)]
pub enum InstructionOutput {
    CallReturn(IndexedScryptoValue),
    None,
}

impl<'a> Invocation for TransactionProcessorRunInvocation<'a> {
    type Output = Vec<InstructionOutput>;

    fn debug_identifier(&self) -> InvocationDebugIdentifier {
        InvocationDebugIdentifier::Transaction
    }
}

fn extract_refs_from_instruction(instruction: &Instruction, update: &mut CallFrameUpdate) {
    match instruction {
        Instruction::CallFunction {
            package_address,
            args,
            ..
        } => {
            update.add_ref(RENodeId::GlobalPackage(*package_address));
            let value: ManifestValue =
                manifest_decode(args).expect("Invalid CALL_FUNCTION arguments");
            extract_refs_from_value(&value, update);

            if package_address.eq(&EPOCH_MANAGER_PACKAGE) {
                update.add_ref(RENodeId::GlobalResourceManager(PACKAGE_TOKEN));
            }
        }
        Instruction::PublishPackage { access_rules, .. } => {
            update.add_ref(RENodeId::GlobalPackage(PACKAGE_LOADER));

            // TODO: Remove and cleanup
            let value: ManifestValue = manifest_decode(&manifest_encode(access_rules).unwrap())
                .expect("Invalid CALL_FUNCTION arguments");
            extract_refs_from_value(&value, update);
        }
        Instruction::CallMethod {
            component_address,
            args,
            ..
        } => {
            update.add_ref(RENodeId::GlobalComponent(*component_address));
            let value: ManifestValue =
                manifest_decode(args).expect("Invalid CALL_METHOD arguments");
            extract_refs_from_value(&value, update);
        }

        Instruction::SetMetadata { entity_address, .. }
        | Instruction::SetMethodAccessRule { entity_address, .. } => {
            let address = to_address(entity_address.clone());
            let node_id = address.into();
            update.add_ref(node_id);
        }
        Instruction::RecallResource { vault_id, .. } => {
            // TODO: This needs to be cleaned up
            // TODO: How does this relate to newly created vaults in the transaction frame?
            // TODO: Will probably want different spacing for refed vs. owned nodes
            update.add_ref(RENodeId::Vault(*vault_id));
        }

        Instruction::SetPackageRoyaltyConfig {
            package_address, ..
        }
        | Instruction::ClaimPackageRoyalty {
            package_address, ..
        } => {
            update.add_ref(RENodeId::GlobalPackage(*package_address));
        }
        Instruction::SetComponentRoyaltyConfig {
            component_address, ..
        }
        | Instruction::ClaimComponentRoyalty {
            component_address, ..
        } => {
            update.add_ref(RENodeId::GlobalComponent(*component_address));
        }
        Instruction::TakeFromWorktop {
            resource_address, ..
        }
        | Instruction::TakeFromWorktopByAmount {
            resource_address, ..
        }
        | Instruction::TakeFromWorktopByIds {
            resource_address, ..
        }
        | Instruction::AssertWorktopContains {
            resource_address, ..
        }
        | Instruction::AssertWorktopContainsByAmount {
            resource_address, ..
        }
        | Instruction::AssertWorktopContainsByIds {
            resource_address, ..
        }
        | Instruction::CreateProofFromAuthZone {
            resource_address, ..
        }
        | Instruction::CreateProofFromAuthZoneByAmount {
            resource_address, ..
        }
        | Instruction::CreateProofFromAuthZoneByIds {
            resource_address, ..
        }
        | Instruction::MintFungible {
            resource_address, ..
        }
        | Instruction::MintNonFungible {
            resource_address, ..
        }
        | Instruction::MintUuidNonFungible {
            resource_address, ..
        } => {
            update.add_ref(RENodeId::GlobalResourceManager(resource_address.clone()));
        }
        Instruction::ReturnToWorktop { .. }
        | Instruction::PopFromAuthZone { .. }
        | Instruction::PushToAuthZone { .. }
        | Instruction::ClearAuthZone { .. }
        | Instruction::CreateProofFromBucket { .. }
        | Instruction::CloneProof { .. }
        | Instruction::DropProof { .. }
        | Instruction::DropAllProofs { .. }
        | Instruction::BurnResource { .. }
        | Instruction::AssertAccessRule { .. } => {}
    }
}

fn extract_refs_from_value(value: &ManifestValue, collector: &mut CallFrameUpdate) {
    match value {
        Value::Bool { .. }
        | Value::I8 { .. }
        | Value::I16 { .. }
        | Value::I32 { .. }
        | Value::I64 { .. }
        | Value::I128 { .. }
        | Value::U8 { .. }
        | Value::U16 { .. }
        | Value::U32 { .. }
        | Value::U64 { .. }
        | Value::U128 { .. }
        | Value::String { .. } => {}
        Value::Enum { fields, .. } => {
            for f in fields {
                extract_refs_from_value(f, collector);
            }
        }
        Value::Array { elements, .. } => {
            for f in elements {
                extract_refs_from_value(f, collector);
            }
        }
        Value::Tuple { fields } => {
            for f in fields {
                extract_refs_from_value(f, collector);
            }
        }
        Value::Map { entries, .. } => {
            for f in entries {
                extract_refs_from_value(&f.0, collector);
                extract_refs_from_value(&f.1, collector);
            }
        }
        Value::Custom { value } => match value {
            ManifestCustomValue::Address(a) => {
                let address = to_address(a.clone());
                let node_id = match address {
                    Address::Package(package_address) => RENodeId::GlobalPackage(package_address),
                    Address::Component(component_address) => {
                        RENodeId::GlobalComponent(component_address)
                    }
                    Address::Resource(resource_address) => {
                        RENodeId::GlobalResourceManager(resource_address)
                    }
                };
                collector.add_ref(node_id)
            }
            _ => {}
        },
    }
}

impl<'a> ExecutableInvocation for TransactionProcessorRunInvocation<'a> {
    type Exec = Self;

    fn resolve<D: KernelSubstateApi>(
        self,
        _api: &mut D,
    ) -> Result<TemporaryResolvedInvocation<Self::Exec>, RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();
        // TODO: This can be refactored out once any type in sbor is implemented
        let instructions: Vec<Instruction> = manifest_decode(&self.instructions).unwrap();
        for instruction in instructions {
            extract_refs_from_instruction(&instruction, &mut call_frame_update);
        }
        call_frame_update.add_ref(RENodeId::GlobalResourceManager(RADIX_TOKEN));
        call_frame_update.add_ref(RENodeId::GlobalResourceManager(PACKAGE_TOKEN));
        call_frame_update.add_ref(RENodeId::GlobalComponent(EPOCH_MANAGER));
        call_frame_update.add_ref(RENodeId::GlobalComponent(CLOCK));
        call_frame_update.add_ref(RENodeId::GlobalResourceManager(ECDSA_SECP256K1_TOKEN));
        call_frame_update.add_ref(RENodeId::GlobalResourceManager(EDDSA_ED25519_TOKEN));

        let actor = Actor::function(FnIdentifier {
            package_address: PACKAGE_LOADER,
            blueprint_name: TRANSACTION_PROCESSOR_BLUEPRINT.to_string(),
            ident: "run".to_string(),
        });

        let resolved = TemporaryResolvedInvocation {
            resolved_actor: actor,
            update: call_frame_update,
            executor: self,
            args: ScryptoValue::Tuple { fields: vec![] },
        };

        Ok(resolved)
    }

    fn payload_size(&self) -> usize {
        0
    }
}

impl<'a> Executor for TransactionProcessorRunInvocation<'a> {
    type Output = Vec<InstructionOutput>;

    fn execute<Y, W: WasmEngine>(
        self,
        _args: ScryptoValue,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        for request in self.runtime_validations.as_ref() {
            TransactionProcessor::perform_validation(request, api)?;
        }

        let worktop_node_id = api.kernel_allocate_node_id(RENodeType::Worktop)?;
        api.kernel_create_node(
            worktop_node_id,
            RENodeInit::Worktop(WorktopSubstate::new()),
            btreemap!(
                NodeModuleId::TypeInfo => RENodeModuleInit::TypeInfo(TypeInfoSubstate {
                    package_address: RESOURCE_MANAGER_PACKAGE,
                    blueprint_name: WORKTOP_BLUEPRINT.to_string(),
                    global: false,
                })
            ),
        )?;

        let instructions: Vec<Instruction> = manifest_decode(&self.instructions).unwrap();

        // TODO: defer blob hashing to post fee payments as it's computationally costly
        let mut blobs_by_hash = HashMap::new();
        for blob in self.blobs.as_ref() {
            blobs_by_hash.insert(hash(blob), blob);
        }

        let mut processor = TransactionProcessor::new(blobs_by_hash);
        let mut outputs = Vec::new();
        for (index, inst) in instructions.into_iter().enumerate() {
            api.update_instruction_index(index)?;

            let result = match inst {
                Instruction::TakeFromWorktop { resource_address } => {
                    let bucket = Worktop::sys_take_all(resource_address, api)?;
                    processor.create_manifest_bucket(bucket)?;
                    InstructionOutput::None
                }
                Instruction::TakeFromWorktopByAmount {
                    amount,
                    resource_address,
                } => {
                    let bucket = Worktop::sys_take(resource_address, amount, api)?;
                    processor.create_manifest_bucket(bucket)?;
                    InstructionOutput::None
                }
                Instruction::TakeFromWorktopByIds {
                    ids,
                    resource_address,
                } => {
                    let bucket = Worktop::sys_take_non_fungibles(resource_address, ids, api)?;
                    processor.create_manifest_bucket(bucket)?;
                    InstructionOutput::None
                }
                Instruction::ReturnToWorktop { bucket_id } => {
                    let bucket = processor.take_bucket(&bucket_id)?;
                    Worktop::sys_put(bucket, api)?;
                    InstructionOutput::None
                }
                Instruction::AssertWorktopContains { resource_address } => {
                    Worktop::sys_assert_contains(resource_address, api)?;
                    InstructionOutput::None
                }
                Instruction::AssertWorktopContainsByAmount {
                    amount,
                    resource_address,
                } => {
                    Worktop::sys_assert_contains_amount(resource_address, amount, api)?;
                    InstructionOutput::None
                }
                Instruction::AssertWorktopContainsByIds {
                    ids,
                    resource_address,
                } => {
                    Worktop::sys_assert_contains_non_fungibles(resource_address, ids, api)?;
                    InstructionOutput::None
                }
                Instruction::PopFromAuthZone {} => {
                    let proof = ComponentAuthZone::sys_pop(api)?;
                    processor.create_manifest_proof(proof)?;
                    InstructionOutput::None
                }
                Instruction::ClearAuthZone => {
                    processor.proof_id_mapping.clear();
                    ComponentAuthZone::sys_clear(api)?;
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
                    for (_, real_id) in processor.proof_id_mapping.drain() {
                        let proof = Proof(real_id);
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
                    let value: ManifestValue =
                        manifest_decode(&args).expect("Invalid CALL_FUNCTION arguments");
                    let mut processor_with_api = TransactionProcessorWithApi { processor, api };
                    let scrypto_value = transform(value, &mut processor_with_api)?;
                    processor = processor_with_api.processor;

                    let rtn = api.call_function(
                        package_address,
                        &blueprint_name,
                        &function_name,
                        scrypto_encode(&scrypto_value).unwrap(),
                    )?;

                    let result = IndexedScryptoValue::from_vec(rtn).unwrap();
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result, api,
                    )?;
                    InstructionOutput::CallReturn(result)
                }
                Instruction::CallMethod {
                    component_address,
                    method_name,
                    args,
                } => {
                    let value: ManifestValue =
                        manifest_decode(&args).expect("Invalid CALL_METHOD arguments");
                    let mut processor_with_api = TransactionProcessorWithApi { processor, api };
                    let scrypto_value = transform(value, &mut processor_with_api)?;
                    processor = processor_with_api.processor;

                    let rtn = api.call_method(
                        RENodeId::GlobalComponent(component_address.into()),
                        &method_name,
                        scrypto_encode(&scrypto_value).unwrap(),
                    )?;
                    let result = IndexedScryptoValue::from_vec(rtn).unwrap();
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result, api,
                    )?;
                    InstructionOutput::CallReturn(result)
                }
                Instruction::PublishPackage {
                    code,
                    abi,
                    royalty_config,
                    metadata,
                    access_rules,
                } => {
                    let code = processor.get_blob(&code)?;
                    let abi = processor.get_blob(&abi)?;
                    let abi = scrypto_decode(abi).map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::PackageError(
                            PackageError::InvalidAbi(e),
                        ))
                    })?;

                    // TODO: remove clone by allowing invocation to have references, like in TransactionProcessorRunInvocation.
                    let result = api.call_function(
                        PACKAGE_LOADER,
                        PACKAGE_LOADER_BLUEPRINT,
                        PACKAGE_LOADER_PUBLISH_WASM_IDENT,
                        scrypto_encode(&PackageLoaderPublishWasmInput {
                            package_address: None,
                            code: code.clone(),
                            abi,
                            access_rules: access_rules.clone(),
                            royalty_config: royalty_config.clone(),
                            metadata: metadata.clone(),
                        })
                        .unwrap(),
                    )?;

                    let result_indexed = IndexedScryptoValue::from_vec(result).unwrap();
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result_indexed,
                        api,
                    )?;

                    InstructionOutput::CallReturn(result_indexed)
                }
                Instruction::BurnResource { bucket_id } => {
                    let bucket = processor.take_bucket(&bucket_id)?;
                    let rtn = api.call_function(
                        RESOURCE_MANAGER_PACKAGE,
                        RESOURCE_MANAGER_BLUEPRINT,
                        RESOURCE_MANAGER_BURN_BUCKET_IDENT,
                        scrypto_encode(&ResourceManagerBurnBucketInput { bucket }).unwrap(),
                    )?;

                    let result = IndexedScryptoValue::from_vec(rtn).unwrap();
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result, api,
                    )?;
                    InstructionOutput::CallReturn(result)
                }
                Instruction::MintFungible {
                    resource_address,
                    amount,
                } => {
                    let rtn = api.call_method(
                        RENodeId::GlobalResourceManager(resource_address),
                        RESOURCE_MANAGER_MINT_FUNGIBLE,
                        scrypto_encode(&ResourceManagerMintFungibleInput { amount }).unwrap(),
                    )?;

                    let result = IndexedScryptoValue::from_vec(rtn).unwrap();
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result, api,
                    )?;
                    InstructionOutput::CallReturn(result)
                }
                Instruction::MintNonFungible {
                    resource_address,
                    entries,
                } => {
                    let rtn = api.call_method(
                        RENodeId::GlobalResourceManager(resource_address),
                        RESOURCE_MANAGER_MINT_NON_FUNGIBLE,
                        scrypto_encode(&ResourceManagerMintNonFungibleInput { entries: entries })
                            .unwrap(),
                    )?;

                    let result = IndexedScryptoValue::from_vec(rtn).unwrap();
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result, api,
                    )?;
                    InstructionOutput::CallReturn(result)
                }
                Instruction::MintUuidNonFungible {
                    resource_address,
                    entries,
                } => {
                    let rtn = api.call_method(
                        RENodeId::GlobalResourceManager(resource_address),
                        RESOURCE_MANAGER_MINT_UUID_NON_FUNGIBLE,
                        scrypto_encode(&ResourceManagerMintUuidNonFungibleInput {
                            entries: entries,
                        })
                        .unwrap(),
                    )?;

                    let result = IndexedScryptoValue::from_vec(rtn).unwrap();
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result, api,
                    )?;
                    InstructionOutput::CallReturn(result)
                }
                Instruction::RecallResource { vault_id, amount } => {
                    let rtn = api.call_method(
                        RENodeId::Vault(vault_id),
                        VAULT_RECALL_IDENT,
                        scrypto_encode(&VaultRecallInput { amount }).unwrap(),
                    )?;

                    let result = IndexedScryptoValue::from_vec(rtn).unwrap();
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result, api,
                    )?;
                    InstructionOutput::CallReturn(result)
                }
                Instruction::SetMetadata {
                    entity_address,
                    key,
                    value,
                } => {
                    let address = to_address(entity_address);
                    let receiver = address.into();
                    let result = api.call_module_method(
                        receiver,
                        NodeModuleId::Metadata,
                        METADATA_SET_IDENT,
                        scrypto_encode(&MetadataSetInput {
                            key: key.clone(),
                            value: value.clone(),
                        })
                        .unwrap(),
                    )?;

                    let result_indexed = IndexedScryptoValue::from_vec(result).unwrap();
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result_indexed,
                        api,
                    )?;

                    InstructionOutput::CallReturn(result_indexed)
                }
                Instruction::SetPackageRoyaltyConfig {
                    package_address,
                    royalty_config,
                } => {
                    let result = api.call_module_method(
                        RENodeId::GlobalPackage(package_address),
                        NodeModuleId::PackageRoyalty,
                        PACKAGE_ROYALTY_SET_ROYALTY_CONFIG_IDENT,
                        scrypto_encode(&PackageSetRoyaltyConfigInput {
                            royalty_config: royalty_config.clone(),
                        })
                        .unwrap(),
                    )?;

                    let result_indexed = IndexedScryptoValue::from_vec(result).unwrap();
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result_indexed,
                        api,
                    )?;

                    InstructionOutput::CallReturn(result_indexed)
                }
                Instruction::SetComponentRoyaltyConfig {
                    component_address,
                    royalty_config,
                } => {
                    let result = api.call_module_method(
                        RENodeId::GlobalComponent(component_address.into()),
                        NodeModuleId::ComponentRoyalty,
                        COMPONENT_ROYALTY_SET_ROYALTY_CONFIG_IDENT,
                        scrypto_encode(&ComponentSetRoyaltyConfigInput {
                            royalty_config: royalty_config.clone(),
                        })
                        .unwrap(),
                    )?;

                    let result_indexed = IndexedScryptoValue::from_vec(result).unwrap();
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result_indexed,
                        api,
                    )?;

                    InstructionOutput::CallReturn(result_indexed)
                }
                Instruction::ClaimPackageRoyalty { package_address } => {
                    let result = api.call_module_method(
                        RENodeId::GlobalPackage(package_address),
                        NodeModuleId::PackageRoyalty,
                        PACKAGE_ROYALTY_CLAIM_ROYALTY_IDENT,
                        scrypto_encode(&PackageClaimRoyaltyInput {}).unwrap(),
                    )?;

                    let result_indexed = IndexedScryptoValue::from_vec(result).unwrap();
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result_indexed,
                        api,
                    )?;

                    InstructionOutput::CallReturn(result_indexed)
                }
                Instruction::ClaimComponentRoyalty { component_address } => {
                    let result = api.call_module_method(
                        RENodeId::GlobalComponent(component_address.into()),
                        NodeModuleId::ComponentRoyalty,
                        COMPONENT_ROYALTY_CLAIM_ROYALTY_IDENT,
                        scrypto_encode(&ComponentClaimRoyaltyInput {}).unwrap(),
                    )?;

                    let result_indexed = IndexedScryptoValue::from_vec(result).unwrap();
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result_indexed,
                        api,
                    )?;

                    InstructionOutput::CallReturn(result_indexed)
                }
                Instruction::SetMethodAccessRule {
                    entity_address,
                    key,
                    rule,
                } => {
                    let address = to_address(entity_address);
                    let receiver = address.into();
                    let result = api.call_module_method(
                        receiver,
                        NodeModuleId::AccessRules,
                        ACCESS_RULES_SET_METHOD_ACCESS_RULE_IDENT,
                        scrypto_encode(&AccessRulesSetMethodAccessRuleInput {
                            key: key.clone(),
                            rule: AccessRuleEntry::AccessRule(rule.clone()),
                        })
                        .unwrap(),
                    )?;

                    let result_indexed = IndexedScryptoValue::from_vec(result).unwrap();
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result_indexed,
                        api,
                    )?;

                    InstructionOutput::CallReturn(result_indexed)
                }
                Instruction::AssertAccessRule { access_rule } => {
                    let rtn = ComponentAuthZone::sys_assert_access_rule(access_rule, api)?;

                    let result = IndexedScryptoValue::from_typed(&rtn);
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result, api,
                    )?;
                    InstructionOutput::CallReturn(result)
                }
            };
            outputs.push(result);
        }

        WorktopBlueprint::drop(
            IndexedScryptoValue::from_typed(&WorktopDropInput {}).into(),
            api,
        )?;
        // Can't use native-sdk yet since there is no way to express moving the worktop
        /*
        Worktop::sys_drop(api)?;
         */

        Ok((outputs, CallFrameUpdate::empty()))
    }
}

struct TransactionProcessor<'blob> {
    proof_id_mapping: HashMap<ManifestProof, ProofId>,
    bucket_id_mapping: HashMap<ManifestBucket, ObjectId>,
    id_allocator: ManifestIdAllocator,
    blobs_by_hash: HashMap<Hash, &'blob Vec<u8>>,
}

impl<'blob> TransactionProcessor<'blob> {
    fn new(blobs_by_hash: HashMap<Hash, &'blob Vec<u8>>) -> Self {
        Self {
            proof_id_mapping: HashMap::new(),
            bucket_id_mapping: HashMap::new(),
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
        Ok(Bucket(real_id))
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
        Ok(Bucket(real_id))
    }

    fn get_blob(&mut self, blob_ref: &ManifestBlobRef) -> Result<&'blob Vec<u8>, RuntimeError> {
        let hash = Hash(blob_ref.0);
        self.blobs_by_hash
            .get(&hash)
            .cloned()
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
        Ok(Proof(real_id))
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
        Ok(Proof(real_id))
    }

    fn create_manifest_bucket(&mut self, bucket: Bucket) -> Result<ManifestBucket, RuntimeError> {
        let new_id = self.id_allocator.new_bucket_id().map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::TransactionProcessorError(
                TransactionProcessorError::IdAllocationError(e),
            ))
        })?;
        self.bucket_id_mapping.insert(new_id.clone(), bucket.0);
        Ok(new_id)
    }

    fn create_manifest_proof(&mut self, proof: Proof) -> Result<ManifestProof, RuntimeError> {
        let new_id = self.id_allocator.new_proof_id().map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::TransactionProcessorError(
                TransactionProcessorError::IdAllocationError(e),
            ))
        })?;
        self.proof_id_mapping.insert(new_id.clone(), proof.0);
        Ok(new_id)
    }

    fn move_proofs_to_authzone_and_buckets_to_worktop<Y>(
        value: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        // Auto move into worktop & auth_zone
        for owned_node in value.owned_node_ids() {
            let (package_address, blueprint) = api.get_object_type_info(*owned_node)?;
            match (package_address, blueprint.as_str()) {
                (RESOURCE_MANAGER_PACKAGE, BUCKET_BLUEPRINT) => {
                    let bucket = Bucket(owned_node.clone().into());
                    Worktop::sys_put(bucket, api)?;
                }
                (RESOURCE_MANAGER_PACKAGE, PROOF_BLUEPRINT) => {
                    let proof = Proof(owned_node.clone().into());
                    ComponentAuthZone::sys_push(proof, api)?;
                }
                _ => {
                }
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
    processor: TransactionProcessor<'blob>,
    api: &'a mut Y,
}

impl<'blob, 'a, Y: ClientApi<RuntimeError>> TransformHandler<RuntimeError>
    for TransactionProcessorWithApi<'blob, 'a, Y>
{
    fn replace_bucket(&mut self, b: ManifestBucket) -> Result<Own, RuntimeError> {
        self.processor.take_bucket(&b).map(|x| Own::Bucket(x.0))
    }

    fn replace_proof(&mut self, p: ManifestProof) -> Result<Own, RuntimeError> {
        self.processor.take_proof(&p).map(|x| Own::Proof(x.0))
    }

    fn replace_expression(&mut self, e: ManifestExpression) -> Result<Vec<Own>, RuntimeError> {
        match e {
            ManifestExpression::EntireWorktop => {
                let buckets = Worktop::sys_drain(self.api)?;
                Ok(buckets.into_iter().map(|b| Own::Bucket(b.0)).collect())
            }
            ManifestExpression::EntireAuthZone => {
                let proofs = ComponentAuthZone::sys_drain(self.api)?;
                Ok(proofs.into_iter().map(|p| Own::Proof(p.0)).collect())
            }
        }
    }

    fn replace_blob(&mut self, b: ManifestBlobRef) -> Result<Vec<u8>, RuntimeError> {
        Ok(self.processor.get_blob(&b)?.clone())
    }
}
