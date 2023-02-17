use crate::blueprints::resource::WorktopSubstate;
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::kernel::actor::ResolvedActor;
use crate::kernel::call_frame::CallFrameUpdate;
use crate::kernel::kernel_api::{ExecutableInvocation, Executor, KernelNodeApi, KernelSubstateApi};
use crate::system::node::RENodeInit;
use crate::types::*;
use crate::wasm::WasmEngine;
use native_sdk::resource::{ComponentAuthZone, SysBucket, SysProof, Worktop};
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::node_modules::auth::AccessRulesSetMethodAccessRuleInvocation;
use radix_engine_interface::api::node_modules::metadata::MetadataSetInvocation;
use radix_engine_interface::api::node_modules::royalty::{
    ComponentClaimRoyaltyInvocation, ComponentSetRoyaltyConfigInvocation,
};
use radix_engine_interface::api::package::*;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::api::{ClientComponentApi, ClientDerefApi};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::{IndexedScryptoValue, ReadOwnedNodesError, ScryptoValue};
use sbor::rust::borrow::Cow;
use transaction::errors::ManifestIdAllocationError;
use transaction::model::*;
use transaction::validation::*;
use transaction_data::model::*;

#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct TransactionProcessorRunInvocation<'a> {
    pub transaction_hash: Hash,
    pub runtime_validations: Cow<'a, [RuntimeValidationRequest]>,
    pub instructions: Cow<'a, Vec<u8>>,
    pub blobs: Cow<'a, [Vec<u8>]>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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
    BlobNotFound(u32),
    IdAllocationError(ManifestIdAllocationError),
    InvalidCallData(DecodeError),
}

pub trait NativeOutput: ScryptoEncode + Debug + Send + Sync {}
impl<T: ScryptoEncode + Debug + Send + Sync> NativeOutput for T {}

#[derive(Debug)]
pub enum InstructionOutput {
    Native(Box<dyn NativeOutput>),
    Scrypto(IndexedScryptoValue),
}

impl InstructionOutput {
    pub fn as_vec(&self) -> Vec<u8> {
        match self {
            InstructionOutput::Native(o) => IndexedScryptoValue::from_typed(o.as_ref()).into_vec(),
            InstructionOutput::Scrypto(value) => value.as_slice().to_owned(),
        }
    }
}

impl Clone for InstructionOutput {
    fn clone(&self) -> Self {
        match self {
            InstructionOutput::Scrypto(output) => InstructionOutput::Scrypto(output.clone()),
            InstructionOutput::Native(output) => {
                // SBOR Encode the output
                let encoded_output = scrypto_encode(&**output)
                    .expect("Impossible Case! Instruction output is not SBOR encodable!");

                // Decode to a ScryptoValue
                let decoded = scrypto_decode::<ScryptoValue>(&encoded_output)
                    .expect("Impossible Case! We literally just encoded this above");

                InstructionOutput::Native(Box::new(decoded))
            }
        }
    }
}

impl<'a> Invocation for TransactionProcessorRunInvocation<'a> {
    type Output = Vec<InstructionOutput>;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::TransactionProcessor(TransactionProcessorFn::Run))
    }
}

fn instruction_get_update(instruction: &Instruction, update: &mut CallFrameUpdate) {
    match instruction {
        Instruction::Basic(basic_function) => match basic_function {
            Instruction::CallFunction {
                args,
                package_address,
                ..
            } => {
                update.add_ref(RENodeId::Global(Address::Package(*package_address)));
                for node_id in slice_to_global_references(args) {
                    update.add_ref(node_id);
                }

                if package_address.eq(&EPOCH_MANAGER_PACKAGE) {
                    update.add_ref(RENodeId::Global(Address::Resource(PACKAGE_TOKEN)));
                }
            }
            Instruction::CallMethod {
                args,
                component_address,
                ..
            } => {
                update.add_ref(RENodeId::Global(Address::Component(*component_address)));
                for node_id in slice_to_global_references(args) {
                    update.add_ref(node_id);
                }
            }

            Instruction::SetMetadata { entity_address, .. }
            | Instruction::SetMethodAccessRule { entity_address, .. } => {
                update.add_ref(RENodeId::Global(*entity_address));
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
                update.add_ref(RENodeId::Global(Address::Package(*package_address)));
            }
            Instruction::SetComponentRoyaltyConfig {
                component_address, ..
            }
            | Instruction::ClaimComponentRoyalty {
                component_address, ..
            } => {
                update.add_ref(RENodeId::Global(Address::Component(*component_address)));
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
                update.add_ref(RENodeId::Global(Address::Resource(*resource_address)));
            }
            Instruction::ReturnToWorktop { .. }
            | Instruction::PopFromAuthZone { .. }
            | Instruction::PushToAuthZone { .. }
            | Instruction::ClearAuthZone { .. }
            | Instruction::CreateProofFromBucket { .. }
            | Instruction::CloneProof { .. }
            | Instruction::DropProof { .. }
            | Instruction::DropAllProofs { .. }
            | Instruction::PublishPackage { .. }
            | Instruction::PublishPackageWithOwner { .. }
            | Instruction::BurnResource { .. }
            | Instruction::AssertAccessRule { .. } => {}
        },
        Instruction::NativeInvocation(invocation) => {
            for node_id in invocation.refs() {
                update.add_ref(node_id);
            }
        }
    }
}

fn slice_to_global_references(slice: &[u8]) -> Vec<RENodeId> {
    let scrypto_value = IndexedScryptoValue::from_slice(slice).expect("Invalid CALL arguments");
    scrypto_value
        .global_references()
        .into_iter()
        .map(|addr| RENodeId::Global(addr))
        .collect()
}

impl<'a> ExecutableInvocation for TransactionProcessorRunInvocation<'a> {
    type Exec = Self;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();
        // TODO: This can be refactored out once any type in sbor is implemented
        for instruction in self.instructions.as_ref() {
            instruction_get_update(instruction, &mut call_frame_update);
        }
        call_frame_update.add_ref(RENodeId::Global(Address::Resource(RADIX_TOKEN)));
        call_frame_update.add_ref(RENodeId::Global(Address::Resource(PACKAGE_TOKEN)));
        call_frame_update.add_ref(RENodeId::Global(Address::Component(EPOCH_MANAGER)));
        call_frame_update.add_ref(RENodeId::Global(Address::Component(CLOCK)));
        call_frame_update.add_ref(RENodeId::Global(Address::Resource(ECDSA_SECP256K1_TOKEN)));
        call_frame_update.add_ref(RENodeId::Global(Address::Resource(EDDSA_ED25519_TOKEN)));

        let actor =
            ResolvedActor::function(NativeFn::TransactionProcessor(TransactionProcessorFn::Run));

        Ok((actor, call_frame_update, self))
    }
}

impl<'a> Executor for TransactionProcessorRunInvocation<'a> {
    type Output = Vec<InstructionOutput>;

    fn execute<Y, W: WasmEngine>(
        self,
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
            BTreeMap::new(),
        )?;

        // TODO: defer blob hashing to post fee payments as it's computationally costly
        let mut blobs_by_hash = HashMap::new();
        for blob in self.blobs.as_ref() {
            blobs_by_hash.insert(hash(blob), blob);
        }

        let mut processor = TransactionProcessor::new();
        let mut outputs = Vec::new();
        for (index, inst) in self.instructions.into_iter().enumerate() {
            api.update_instruction_index(index)?;

            let result = match inst {
                Instruction::Basic(Instruction::TakeFromWorktop { resource_address }) => {
                    let bucket = Worktop::sys_take_all(*resource_address, api)?;
                    let bucket = processor.next_static_bucket(bucket)?;
                    InstructionOutput::Native(Box::new(bucket))
                }
                Instruction::Basic(Instruction::TakeFromWorktopByAmount {
                    amount,
                    resource_address,
                }) => {
                    let bucket = Worktop::sys_take(*resource_address, *amount, api)?;
                    let bucket = processor.next_static_bucket(bucket)?;
                    InstructionOutput::Native(Box::new(bucket))
                }
                Instruction::Basic(Instruction::TakeFromWorktopByIds {
                    ids,
                    resource_address,
                }) => {
                    let bucket =
                        Worktop::sys_take_non_fungibles(*resource_address, ids.clone(), api)?;
                    let bucket = processor.next_static_bucket(bucket)?;
                    InstructionOutput::Native(Box::new(bucket))
                }
                Instruction::Basic(Instruction::ReturnToWorktop { bucket_id }) => {
                    let bucket = processor.take_bucket(bucket_id)?;
                    let rtn = Worktop::sys_put(bucket, api)?;
                    InstructionOutput::Native(Box::new(rtn))
                }
                Instruction::Basic(Instruction::AssertWorktopContains { resource_address }) => {
                    let rtn = Worktop::sys_assert_contains(*resource_address, api)?;
                    InstructionOutput::Native(Box::new(rtn))
                }
                Instruction::Basic(Instruction::AssertWorktopContainsByAmount {
                    amount,
                    resource_address,
                }) => {
                    let rtn = Worktop::sys_assert_contains_amount(*resource_address, *amount, api)?;
                    InstructionOutput::Native(Box::new(rtn))
                }
                Instruction::Basic(Instruction::AssertWorktopContainsByIds {
                    ids,
                    resource_address,
                }) => {
                    let rtn = Worktop::sys_assert_contains_non_fungibles(
                        *resource_address,
                        ids.clone(),
                        api,
                    )?;
                    InstructionOutput::Native(Box::new(rtn))
                }
                Instruction::Basic(Instruction::PopFromAuthZone {}) => {
                    let proof = ComponentAuthZone::sys_pop(api)?;
                    let proof = processor.next_static_proof(proof)?;
                    InstructionOutput::Native(Box::new(proof))
                }
                Instruction::Basic(Instruction::ClearAuthZone) => {
                    processor.proof_id_mapping.clear();
                    let rtn = ComponentAuthZone::sys_clear(api)?;
                    InstructionOutput::Native(Box::new(rtn))
                }
                Instruction::Basic(Instruction::PushToAuthZone { proof_id }) => {
                    let proof = processor.take_proof(proof_id)?;
                    let rtn = ComponentAuthZone::sys_push(proof, api)?;
                    InstructionOutput::Native(Box::new(rtn))
                }
                Instruction::Basic(Instruction::CreateProofFromAuthZone { resource_address }) => {
                    let proof = ComponentAuthZone::sys_create_proof(*resource_address, api)?;
                    let proof = processor.next_static_proof(proof)?;
                    InstructionOutput::Native(Box::new(proof))
                }
                Instruction::Basic(Instruction::CreateProofFromAuthZoneByAmount {
                    amount,
                    resource_address,
                }) => {
                    let proof = ComponentAuthZone::sys_create_proof_by_amount(
                        *amount,
                        *resource_address,
                        api,
                    )?;
                    let proof = processor.next_static_proof(proof)?;
                    InstructionOutput::Native(Box::new(proof))
                }
                Instruction::Basic(Instruction::CreateProofFromAuthZoneByIds {
                    ids,
                    resource_address,
                }) => {
                    let proof =
                        ComponentAuthZone::sys_create_proof_by_ids(ids, *resource_address, api)?;
                    let proof = processor.next_static_proof(proof)?;
                    InstructionOutput::Native(Box::new(proof))
                }
                Instruction::Basic(Instruction::CreateProofFromBucket { bucket_id }) => {
                    let bucket = processor.get_bucket(bucket_id)?;
                    let proof = bucket.sys_create_proof(api)?;
                    let proof = processor.next_static_proof(proof)?;
                    InstructionOutput::Native(Box::new(proof))
                }
                Instruction::Basic(Instruction::CloneProof { proof_id }) => {
                    let proof = processor.get_proof(proof_id)?;
                    let proof = proof.sys_clone(api)?;
                    let proof = processor.next_static_proof(proof)?;
                    InstructionOutput::Native(Box::new(proof))
                }
                Instruction::Basic(Instruction::DropProof { proof_id }) => {
                    let proof = processor.take_proof(proof_id)?;
                    let rtn = proof.sys_drop(api)?;
                    InstructionOutput::Native(Box::new(rtn))
                }
                Instruction::Basic(Instruction::DropAllProofs) => {
                    for (_, real_id) in processor.proof_id_mapping.drain() {
                        let proof = Proof(real_id);
                        proof.sys_drop(api).map(|_| IndexedScryptoValue::unit())?;
                    }
                    let rtn = ComponentAuthZone::sys_clear(api)?;
                    InstructionOutput::Native(Box::new(rtn))
                }
                Instruction::Basic(Instruction::CallFunction {
                    package_address,
                    blueprint_name,
                    function_name,
                    args,
                }) => {
                    let args = processor.replace_manifest_values(
                        IndexedScryptoValue::from_slice(args)
                            .expect("Invalid CALL_FUNCTION arguments"),
                        api,
                    )?;

                    let result = api.call_function(
                        package_address.clone(),
                        blueprint_name,
                        function_name,
                        args.to_vec(),
                    )?;
                    let result = IndexedScryptoValue::from_vec(result).unwrap();
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result, api,
                    )?;
                    InstructionOutput::Scrypto(result)
                }
                Instruction::Basic(Instruction::CallMethod {
                    component_address,
                    method_name,
                    args,
                }) => {
                    let args = processor.replace_manifest_values(
                        IndexedScryptoValue::from_slice(args)
                            .expect("Invalid CALL_METHOD arguments"),
                        api,
                    )?;

                    let result = api.call_method(
                        ScryptoReceiver::Global(*component_address),
                        method_name,
                        args.into_vec(),
                    )?;
                    let result_indexed = IndexedScryptoValue::from_vec(result).unwrap();
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result_indexed,
                        api,
                    )?;

                    InstructionOutput::Scrypto(result_indexed)
                }
                Instruction::Basic(Instruction::PublishPackage {
                    code,
                    abi,
                    royalty_config,
                    metadata,
                    access_rules,
                }) => {
                    let code = blobs_by_hash
                        .get(&code.0)
                        .ok_or(RuntimeError::ApplicationError(
                            ApplicationError::TransactionProcessorError(
                                TransactionProcessorError::BlobNotFound(code.clone()),
                            ),
                        ))?;
                    let abi = blobs_by_hash
                        .get(&abi.0)
                        .ok_or(RuntimeError::ApplicationError(
                            ApplicationError::TransactionProcessorError(
                                TransactionProcessorError::BlobNotFound(abi.clone()),
                            ),
                        ))?;
                    // TODO: remove clone by allowing invocation to have references, like in TransactionProcessorRunInvocation.
                    let rtn = api.call_native(PackagePublishInvocation {
                        package_address: None,
                        code: code.clone().clone(),
                        abi: abi.clone().clone(),
                        royalty_config: royalty_config.clone(),
                        metadata: metadata.clone(),
                        access_rules: access_rules.clone(),
                    })?;

                    InstructionOutput::Native(Box::new(rtn))
                }
                Instruction::Basic(Instruction::PublishPackageWithOwner {
                    code,
                    abi,
                    owner_badge,
                }) => {
                    let code = blobs_by_hash
                        .get(&code.0)
                        .ok_or(RuntimeError::ApplicationError(
                            ApplicationError::TransactionProcessorError(
                                TransactionProcessorError::BlobNotFound(code.clone()),
                            ),
                        ))?;
                    let abi = blobs_by_hash
                        .get(&abi.0)
                        .ok_or(RuntimeError::ApplicationError(
                            ApplicationError::TransactionProcessorError(
                                TransactionProcessorError::BlobNotFound(abi.clone()),
                            ),
                        ))?;
                    // TODO: remove clone by allowing invocation to have references, like in TransactionProcessorRunInvocation.
                    let rtn = api.call_native(PackagePublishInvocation {
                        package_address: None,
                        code: code.clone().clone(),
                        abi: abi.clone().clone(),
                        royalty_config: BTreeMap::new(),
                        metadata: BTreeMap::new(),
                        access_rules: package_access_rules_from_owner_badge(owner_badge),
                    })?;

                    InstructionOutput::Native(Box::new(rtn))
                }
                Instruction::Basic(Instruction::BurnResource { bucket_id }) => {
                    let bucket = processor.take_bucket(bucket_id)?;
                    let result = api.call_function(
                        RESOURCE_MANAGER_PACKAGE,
                        RESOURCE_MANAGER_BLUEPRINT,
                        RESOURCE_MANAGER_BURN_BUCKET_IDENT,
                        scrypto_encode(&ResourceManagerBurnBucketInput { bucket }).unwrap(),
                    )?;

                    let result_indexed = IndexedScryptoValue::from_vec(result).unwrap();
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result_indexed,
                        api,
                    )?;

                    InstructionOutput::Scrypto(result_indexed)
                }
                Instruction::Basic(Instruction::MintFungible {
                    resource_address,
                    amount,
                }) => {
                    let result = api.call_method(
                        ScryptoReceiver::Resource(*resource_address),
                        RESOURCE_MANAGER_MINT_FUNGIBLE,
                        scrypto_encode(&ResourceManagerMintFungibleInput {
                            amount: amount.clone(),
                        })
                        .unwrap(),
                    )?;

                    let result_indexed = IndexedScryptoValue::from_vec(result).unwrap();
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result_indexed,
                        api,
                    )?;

                    InstructionOutput::Scrypto(result_indexed)
                }
                Instruction::Basic(Instruction::MintNonFungible {
                    resource_address,
                    entries,
                }) => {
                    let result = api.call_method(
                        ScryptoReceiver::Resource(*resource_address),
                        RESOURCE_MANAGER_MINT_NON_FUNGIBLE,
                        scrypto_encode(&ResourceManagerMintNonFungibleInput {
                            entries: entries.clone(),
                        })
                        .unwrap(),
                    )?;

                    let result_indexed = IndexedScryptoValue::from_vec(result).unwrap();
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result_indexed,
                        api,
                    )?;

                    InstructionOutput::Scrypto(result_indexed)
                }
                Instruction::Basic(Instruction::MintUuidNonFungible {
                    resource_address,
                    entries,
                }) => {
                    let result = api.call_method(
                        ScryptoReceiver::Resource(*resource_address),
                        RESOURCE_MANAGER_MINT_UUID_NON_FUNGIBLE,
                        scrypto_encode(&ResourceManagerMintUuidNonFungibleInput {
                            entries: entries.clone(),
                        })
                        .unwrap(),
                    )?;

                    let result_indexed = IndexedScryptoValue::from_vec(result).unwrap();
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result_indexed,
                        api,
                    )?;

                    InstructionOutput::Scrypto(result_indexed)
                }
                Instruction::Basic(Instruction::RecallResource { vault_id, amount }) => {
                    let result = api.call_method(
                        ScryptoReceiver::Vault(*vault_id),
                        VAULT_RECALL_IDENT,
                        scrypto_encode(&VaultRecallInput {
                            amount: amount.clone(),
                        })
                        .unwrap(),
                    )?;

                    let result_indexed = IndexedScryptoValue::from_vec(result).unwrap();
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result_indexed,
                        api,
                    )?;

                    InstructionOutput::Scrypto(result_indexed)
                }
                Instruction::Basic(Instruction::SetMetadata {
                    entity_address,
                    key,
                    value,
                }) => {
                    let rtn = api.call_native(MetadataSetInvocation {
                        receiver: RENodeId::Global(entity_address.clone()),
                        key: key.clone(),
                        value: value.clone(),
                    })?;

                    InstructionOutput::Native(Box::new(rtn))
                }
                Instruction::Basic(Instruction::SetPackageRoyaltyConfig {
                    package_address,
                    royalty_config,
                }) => {
                    let rtn = api.call_native(PackageSetRoyaltyConfigInvocation {
                        receiver: package_address.clone(),
                        royalty_config: royalty_config.clone(),
                    })?;
                    InstructionOutput::Native(Box::new(rtn))
                }
                Instruction::Basic(Instruction::SetComponentRoyaltyConfig {
                    component_address,
                    royalty_config,
                }) => {
                    let rtn = api.call_native(ComponentSetRoyaltyConfigInvocation {
                        receiver: RENodeId::Global(Address::Component(component_address.clone())),
                        royalty_config: royalty_config.clone(),
                    })?;

                    InstructionOutput::Native(Box::new(rtn))
                }
                Instruction::Basic(Instruction::ClaimPackageRoyalty { package_address }) => {
                    let rtn = api.call_native(PackageClaimRoyaltyInvocation {
                        receiver: package_address.clone(),
                    })?;

                    Worktop::sys_put(Bucket(rtn.0), api)?;

                    InstructionOutput::Native(Box::new(rtn))
                }
                Instruction::Basic(Instruction::ClaimComponentRoyalty { component_address }) => {
                    let rtn = api.call_native(ComponentClaimRoyaltyInvocation {
                        receiver: RENodeId::Global(Address::Component(component_address.clone())),
                    })?;

                    Worktop::sys_put(Bucket(rtn.0), api)?;

                    InstructionOutput::Native(Box::new(rtn))
                }
                Instruction::Basic(Instruction::SetMethodAccessRule {
                    entity_address,
                    index,
                    key,
                    rule,
                }) => {
                    let rtn = api.call_native(AccessRulesSetMethodAccessRuleInvocation {
                        receiver: RENodeId::Global(entity_address.clone()),
                        index: index.clone(),
                        key: key.clone(),
                        rule: AccessRuleEntry::AccessRule(rule.clone()),
                    })?;

                    InstructionOutput::Native(Box::new(rtn))
                }
                Instruction::Basic(Instruction::AssertAccessRule { access_rule }) => {
                    let rtn = ComponentAuthZone::sys_assert_access_rule(access_rule.clone(), api)?;
                    InstructionOutput::Native(Box::new(rtn))
                }
                Instruction::NativeInvocation(invocation) => {
                    let invocation = invocation.clone();
                    let (fn_identifier, invocation) = invocation.flatten();
                    let rtn = api.call_native_raw(fn_identifier, invocation)?;

                    // TODO: Move buckets/proofs to worktop/authzone without serialization
                    let result = IndexedScryptoValue::from_vec(rtn.clone()).unwrap();
                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result, api,
                    )?;
                    InstructionOutput::Native(Box::new(result.as_value().clone()))
                }
            };
            outputs.push(result);
        }

        api.kernel_drop_node(worktop_node_id)?;

        Ok((outputs, CallFrameUpdate::empty()))
    }
}

struct TransactionProcessor {
    proof_id_mapping: HashMap<ManifestProof, ProofId>,
    bucket_id_mapping: HashMap<ManifestBucket, BucketId>,
    id_allocator: ManifestIdAllocator,
}

impl TransactionProcessor {
    fn new() -> Self {
        Self {
            proof_id_mapping: HashMap::new(),
            bucket_id_mapping: HashMap::new(),
            id_allocator: ManifestIdAllocator::new(),
        }
    }

    fn get_bucket(&mut self, bucket_id: &ManifestBucket) -> Result<Bucket, RuntimeError> {
        let real_id = self.bucket_id_mapping.get(bucket_id).cloned().ok_or(
            RuntimeError::ApplicationError(ApplicationError::TransactionProcessorError(
                TransactionProcessorError::BucketNotFound(bucket_id.clone()),
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
                        TransactionProcessorError::BucketNotFound(bucket_id.clone()),
                    ),
                ))?;
        Ok(Bucket(real_id))
    }

    fn get_proof(&mut self, proof_id: &ManifestProof) -> Result<Proof, RuntimeError> {
        let real_id =
            self.proof_id_mapping
                .get(proof_id)
                .cloned()
                .ok_or(RuntimeError::ApplicationError(
                    ApplicationError::TransactionProcessorError(
                        TransactionProcessorError::ProofNotFound(proof_id.clone()),
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
                        TransactionProcessorError::ProofNotFound(proof_id.clone()),
                    ),
                ))?;
        Ok(Proof(real_id))
    }

    fn next_static_bucket(&mut self, bucket: Bucket) -> Result<ManifestBucket, RuntimeError> {
        let new_id = self.id_allocator.new_bucket_id().map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::TransactionProcessorError(
                TransactionProcessorError::IdAllocationError(e),
            ))
        })?;
        self.bucket_id_mapping.insert(new_id.clone(), bucket.0);
        Ok(new_id)
    }

    fn next_static_proof(&mut self, proof: Proof) -> Result<ManifestProof, RuntimeError> {
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
        for owned_node in &value
            .owned_node_ids()
            .expect("Duplication checked by engine")
        {
            match owned_node {
                RENodeId::Bucket(bucket_id) => {
                    Worktop::sys_put(Bucket(*bucket_id), api)?;
                }
                RENodeId::Proof(proof_id) => {
                    let proof = Proof(*proof_id);
                    ComponentAuthZone::sys_push(proof, api)?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn replace_manifest_values<'a, Y>(
        &mut self,
        value: IndexedScryptoValue,
        env: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let mut expression_replacements = Vec::<Vec<Own>>::new();
        for (expression, _) in value.expressions() {
            match expression {
                ManifestExpression::EntireWorktop => {
                    let buckets = Worktop::sys_drain(env)?;
                    expression_replacements.push(buckets.into_iter().map(Into::into).collect())
                }
                ManifestExpression::EntireAuthZone => {
                    let proofs = ComponentAuthZone::sys_drain(env)?;
                    expression_replacements.push(proofs.into_iter().map(Into::into).collect())
                }
            }
        }

        value
            .replace_manifest_values(
                &mut self.proof_id_mapping,
                &mut self.bucket_id_mapping,
                expression_replacements,
            )
            .map_err(|e| {
                RuntimeError::ApplicationError(ApplicationError::TransactionProcessorError(
                    TransactionProcessorError::ReplaceManifestValuesError(e),
                ))
            })
    }

    fn perform_validation<'a, Y>(
        request: &RuntimeValidationRequest,
        env: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientComponentApi<RuntimeError>,
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
