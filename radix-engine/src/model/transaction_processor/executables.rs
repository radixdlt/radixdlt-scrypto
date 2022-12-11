use crate::model::resolve_native_method;
use crate::model::{parse_and_invoke_native_fn, resolve_native_function};
use native_sdk::resource::{ComponentAuthZone, SysBucket, SysProof, Worktop};
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::api::{EngineApi, Invocation, Invokable, InvokableModel};
use radix_engine_interface::api::types::{
    BucketId, GlobalAddress, NativeFunction, NativeFunctionIdent, NativeMethodIdent, ProofId,
    RENodeId, TransactionProcessorFunction,
};
use radix_engine_interface::data::{IndexedScryptoValue, ValueReplacingError};
use radix_engine_interface::model::*;
use sbor::rust::borrow::Cow;
use transaction::errors::IdAllocationError;
use transaction::model::*;
use transaction::validation::*;

use crate::engine::*;
use crate::model::WorktopSubstate;
use crate::types::*;
use crate::wasm::WasmEngine;

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct TransactionProcessorRunInvocation<'a> {
    pub runtime_validations: Cow<'a, [RuntimeValidationRequest]>,
    pub instructions: Cow<'a, [Instruction]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub enum TransactionProcessorError {
    TransactionEpochNotYetValid {
        valid_from: u64,
        current_epoch: u64,
    },
    TransactionEpochNoLongerValid {
        valid_until: u64,
        current_epoch: u64,
    },
    InvalidRequestData(DecodeError),
    InvalidGetEpochResponseData(DecodeError),
    InvalidMethod,
    BucketNotFound(BucketId),
    ProofNotFound(ProofId),
    NativeFunctionNotFound(NativeFunctionIdent),
    NativeMethodNotFound(NativeMethodIdent),
    IdAllocationError(IdAllocationError),
}

pub trait NativeOutput: ScryptoEncode + Debug {}
impl<T: ScryptoEncode + Debug> NativeOutput for T {}

#[derive(Debug)]
pub enum InstructionOutput {
    Native(Box<dyn NativeOutput>),
    Scrypto(IndexedScryptoValue),
}

impl InstructionOutput {
    pub fn as_vec(&self) -> Vec<u8> {
        match self {
            InstructionOutput::Native(o) => IndexedScryptoValue::from_typed(o.as_ref()).raw,
            InstructionOutput::Scrypto(value) => value.raw.clone(),
        }
    }
}

impl<'a> Invocation for TransactionProcessorRunInvocation<'a> {
    type Output = Vec<InstructionOutput>;
}

impl<'a, W: WasmEngine> ExecutableInvocation<W> for TransactionProcessorRunInvocation<'a> {
    type Exec = Self;

    fn resolve<D: ResolverApi<W>>(
        self,
        _api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let input = IndexedScryptoValue::from_typed(&self);
        let mut call_frame_update = CallFrameUpdate::empty();

        // TODO: Remove serialization
        for global_address in input.global_references() {
            call_frame_update
                .node_refs_to_copy
                .insert(RENodeId::Global(global_address));
        }

        // TODO: This can be refactored out once any type in sbor is implemented
        for instruction in self.instructions.as_ref() {
            match instruction {
                Instruction::Basic(BasicInstruction::CallFunction { args, .. })
                | Instruction::Basic(BasicInstruction::CallMethod { args, .. })
                | Instruction::System(SystemInstruction::CallNativeFunction { args, .. })
                | Instruction::System(SystemInstruction::CallNativeMethod { args, .. }) => {
                    let scrypto_value =
                        IndexedScryptoValue::from_slice(&args).expect("Invalid CALL arguments");
                    for global_address in scrypto_value.global_references() {
                        call_frame_update
                            .node_refs_to_copy
                            .insert(RENodeId::Global(global_address));
                    }
                }
                Instruction::Basic(BasicInstruction::RecallResource { vault_id, .. }) => {
                    // TODO: This needs to be cleaned up
                    // TODO: How does this relate to newly created vaults in the transaction frame?
                    // TODO: Will probably want different spacing for refed vs. owned nodes
                    call_frame_update
                        .node_refs_to_copy
                        .insert(RENodeId::Vault(*vault_id));
                }
                _ => {}
            }
        }
        call_frame_update
            .node_refs_to_copy
            .insert(RENodeId::Global(GlobalAddress::Resource(RADIX_TOKEN)));
        call_frame_update
            .node_refs_to_copy
            .insert(RENodeId::Global(GlobalAddress::System(EPOCH_MANAGER)));
        call_frame_update
            .node_refs_to_copy
            .insert(RENodeId::Global(GlobalAddress::System(CLOCK)));
        call_frame_update
            .node_refs_to_copy
            .insert(RENodeId::Global(GlobalAddress::Resource(
                ECDSA_SECP256K1_TOKEN,
            )));
        call_frame_update
            .node_refs_to_copy
            .insert(RENodeId::Global(GlobalAddress::Resource(
                EDDSA_ED25519_TOKEN,
            )));
        call_frame_update
            .node_refs_to_copy
            .insert(RENodeId::Global(GlobalAddress::Package(ACCOUNT_PACKAGE)));

        let actor = ResolvedActor::function(NativeFunction::TransactionProcessor(
            TransactionProcessorFunction::Run,
        ));

        Ok((actor, call_frame_update, self))
    }
}

impl<'a> Executor for TransactionProcessorRunInvocation<'a> {
    type Output = Vec<InstructionOutput>;

    fn execute<Y>(
        self,
        api: &mut Y,
    ) -> Result<(Vec<InstructionOutput>, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi
            + Invokable<ScryptoInvocation, RuntimeError>
            + EngineApi<RuntimeError>
            + InvokableModel<RuntimeError>,
    {
        for request in self.runtime_validations.as_ref() {
            TransactionProcessor::perform_validation(request, api)?;
        }
        let mut processor = TransactionProcessor::new();
        let mut outputs = Vec::new();
        let node_id = api.allocate_node_id(RENodeType::Worktop)?;
        let _worktop_id = api.create_node(node_id, RENode::Worktop(WorktopSubstate::new()))?;

        api.emit_event(Event::Runtime(RuntimeEvent::PreExecuteManifest))?;

        for (idx, inst) in self.instructions.into_iter().enumerate() {
            api.emit_event(Event::Runtime(RuntimeEvent::PreExecuteInstruction {
                instruction_index: idx,
                instruction: &inst,
            }))?;

            let result = match inst {
                Instruction::Basic(BasicInstruction::TakeFromWorktop { resource_address }) => {
                    let bucket = Worktop::sys_take_all(*resource_address, api)?;
                    let bucket = processor.next_static_bucket(bucket)?;
                    InstructionOutput::Native(Box::new(bucket))
                }
                Instruction::Basic(BasicInstruction::TakeFromWorktopByAmount {
                    amount,
                    resource_address,
                }) => {
                    let bucket = Worktop::sys_take_amount(*resource_address, *amount, api)?;
                    let bucket = processor.next_static_bucket(bucket)?;
                    InstructionOutput::Native(Box::new(bucket))
                }
                Instruction::Basic(BasicInstruction::TakeFromWorktopByIds {
                    ids,
                    resource_address,
                }) => {
                    let bucket =
                        Worktop::sys_take_non_fungibles(*resource_address, ids.clone(), api)?;
                    let bucket = processor.next_static_bucket(bucket)?;
                    InstructionOutput::Native(Box::new(bucket))
                }
                Instruction::Basic(BasicInstruction::ReturnToWorktop { bucket_id }) => {
                    let bucket = processor.take_bucket(bucket_id)?;
                    let rtn = Worktop::sys_put(bucket, api)?;
                    InstructionOutput::Native(Box::new(rtn))
                }
                Instruction::Basic(BasicInstruction::AssertWorktopContains {
                    resource_address,
                }) => {
                    let rtn = Worktop::sys_assert_contains(*resource_address, api)?;
                    InstructionOutput::Native(Box::new(rtn))
                }
                Instruction::Basic(BasicInstruction::AssertWorktopContainsByAmount {
                    amount,
                    resource_address,
                }) => {
                    let rtn = Worktop::sys_assert_contains_amount(*resource_address, *amount, api)?;
                    InstructionOutput::Native(Box::new(rtn))
                }
                Instruction::Basic(BasicInstruction::AssertWorktopContainsByIds {
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
                Instruction::Basic(BasicInstruction::PopFromAuthZone {}) => {
                    let proof = ComponentAuthZone::sys_pop(api)?;
                    let proof = processor.next_static_proof(proof)?;
                    InstructionOutput::Native(Box::new(proof))
                }
                Instruction::Basic(BasicInstruction::ClearAuthZone) => {
                    processor.proof_id_mapping.clear();
                    let rtn = ComponentAuthZone::sys_clear(api)?;
                    InstructionOutput::Native(Box::new(rtn))
                }
                Instruction::Basic(BasicInstruction::PushToAuthZone { proof_id }) => {
                    let proof = processor.take_proof(proof_id)?;
                    let rtn = ComponentAuthZone::sys_push(proof, api)?;
                    InstructionOutput::Native(Box::new(rtn))
                }
                Instruction::Basic(BasicInstruction::CreateProofFromAuthZone {
                    resource_address,
                }) => {
                    let proof = ComponentAuthZone::sys_create_proof(*resource_address, api)?;
                    let proof = processor.next_static_proof(proof)?;
                    InstructionOutput::Native(Box::new(proof))
                }
                Instruction::Basic(BasicInstruction::CreateProofFromAuthZoneByAmount {
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
                Instruction::Basic(BasicInstruction::CreateProofFromAuthZoneByIds {
                    ids,
                    resource_address,
                }) => {
                    let proof =
                        ComponentAuthZone::sys_create_proof_by_ids(ids, *resource_address, api)?;
                    let proof = processor.next_static_proof(proof)?;
                    InstructionOutput::Native(Box::new(proof))
                }
                Instruction::Basic(BasicInstruction::CreateProofFromBucket { bucket_id }) => {
                    let bucket = processor.get_bucket(bucket_id)?;
                    let proof = bucket.sys_create_proof(api)?;
                    let proof = processor.next_static_proof(proof)?;
                    InstructionOutput::Native(Box::new(proof))
                }
                Instruction::Basic(BasicInstruction::CloneProof { proof_id }) => {
                    let proof = processor.get_proof(proof_id)?;
                    let proof = proof.sys_clone(api)?;
                    let proof = processor.next_static_proof(proof)?;
                    InstructionOutput::Native(Box::new(proof))
                }
                Instruction::Basic(BasicInstruction::DropProof { proof_id }) => {
                    let proof = processor.take_proof(proof_id)?;
                    let rtn = proof.sys_drop(api)?;
                    InstructionOutput::Native(Box::new(rtn))
                }
                Instruction::Basic(BasicInstruction::DropAllProofs) => {
                    for (_, real_id) in processor.proof_id_mapping.drain() {
                        let proof = Proof(real_id);
                        proof.sys_drop(api).map(|_| IndexedScryptoValue::unit())?;
                    }
                    let rtn = ComponentAuthZone::sys_clear(api)?;
                    InstructionOutput::Native(Box::new(rtn))
                }
                Instruction::Basic(BasicInstruction::CallFunction {
                    package_address,
                    blueprint_name,
                    function_name,
                    args,
                }) => {
                    let args = processor
                        .replace_ids(
                            IndexedScryptoValue::from_slice(args)
                                .expect("Invalid CALL_FUNCTION arguments"),
                        )
                        .map_err(|e| {
                            RuntimeError::ApplicationError(
                                ApplicationError::TransactionProcessorError(e),
                            )
                        })
                        .and_then(|args| TransactionProcessor::process_expressions(args, api))?;

                    let result = api.invoke(ParsedScryptoInvocation::Function(
                        ScryptoFunctionIdent {
                            package: ScryptoPackage::Global(package_address.clone()),
                            blueprint_name: blueprint_name.clone(),
                            function_name: function_name.clone(),
                        },
                        args,
                    ))?;

                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result, api,
                    )?;

                    InstructionOutput::Scrypto(result)
                }
                Instruction::Basic(BasicInstruction::CallMethod {
                    component_address,
                    method_name,
                    args,
                }) => {
                    let args = processor
                        .replace_ids(
                            IndexedScryptoValue::from_slice(args)
                                .expect("Invalid CALL_METHOD arguments"),
                        )
                        .map_err(|e| {
                            RuntimeError::ApplicationError(
                                ApplicationError::TransactionProcessorError(e),
                            )
                        })
                        .and_then(|args| TransactionProcessor::process_expressions(args, api))?;

                    let result = api.invoke(ParsedScryptoInvocation::Method(
                        ScryptoMethodIdent {
                            receiver: ScryptoReceiver::Global(component_address.clone()),
                            method_name: method_name.clone(),
                        },
                        args,
                    ))?;

                    TransactionProcessor::move_proofs_to_authzone_and_buckets_to_worktop(
                        &result, api,
                    )?;

                    InstructionOutput::Scrypto(result)
                }
                Instruction::Basic(BasicInstruction::PublishPackage {
                    code,
                    abi,
                    royalty_config,
                    metadata,
                    access_rules,
                }) => {
                    let rtn = api.invoke(PackagePublishInvocation {
                        code: code.clone(),
                        abi: abi.clone(),
                        royalty_config: royalty_config.clone(),
                        metadata: metadata.clone(),
                        access_rules: access_rules.clone(),
                    })?;

                    InstructionOutput::Native(Box::new(rtn))
                }
                Instruction::Basic(BasicInstruction::PublishPackageWithOwner {
                    code,
                    abi,
                    owner_badge,
                }) => {
                    let rtn = api.invoke(PackagePublishWithOwnerInvocation {
                        code: code.clone(),
                        abi: abi.clone(),
                        royalty_config: BTreeMap::new(),
                        metadata: BTreeMap::new(),
                        owner_badge: owner_badge.clone(),
                    })?;

                    InstructionOutput::Native(Box::new(rtn))
                }
                Instruction::Basic(BasicInstruction::CreateResource {
                    resource_type,
                    metadata,
                    access_rules,
                    mint_params,
                }) => {
                    let rtn = api.invoke(ResourceManagerCreateInvocation {
                        resource_type: resource_type.clone(),
                        metadata: metadata.clone(),
                        access_rules: access_rules.clone(),
                        mint_params: mint_params.clone(),
                    })?;

                    if let (_, Some(bucket)) = &rtn {
                        Worktop::sys_put(Bucket(bucket.0), api)?;
                    }

                    InstructionOutput::Native(Box::new(rtn))
                }
                Instruction::Basic(BasicInstruction::CreateResourceWithOwner {
                    resource_type,
                    metadata,
                    owner_badge,
                    mint_params,
                }) => {
                    let rtn = api.invoke(ResourceManagerCreateWithOwnerInvocation {
                        resource_type: resource_type.clone(),
                        metadata: metadata.clone(),
                        owner_badge: owner_badge.clone(),
                        mint_params: mint_params.clone(),
                    })?;

                    if let (_, Some(bucket)) = &rtn {
                        Worktop::sys_put(Bucket(bucket.0), api)?;
                    }

                    InstructionOutput::Native(Box::new(rtn))
                }
                Instruction::Basic(BasicInstruction::BurnResource { bucket_id }) => {
                    let bucket = processor.take_bucket(bucket_id)?;
                    let rtn = api.invoke(ResourceManagerBucketBurnInvocation { bucket })?;
                    InstructionOutput::Native(Box::new(rtn))
                }
                Instruction::Basic(BasicInstruction::MintResource {
                    amount,
                    resource_address,
                }) => {
                    let rtn = api.invoke(ResourceManagerMintInvocation {
                        receiver: resource_address.clone(),
                        mint_params: MintParams::Fungible {
                            amount: amount.clone(),
                        },
                    })?;

                    Worktop::sys_put(Bucket(rtn.0), api)?;

                    InstructionOutput::Native(Box::new(rtn))
                }
                Instruction::Basic(BasicInstruction::RecallResource { vault_id, amount }) => {
                    let rtn = api.invoke(VaultRecallInvocation {
                        receiver: vault_id.clone(),
                        amount: amount.clone(),
                    })?;

                    Worktop::sys_put(Bucket(rtn.0), api)?;

                    InstructionOutput::Native(Box::new(rtn))
                }
                Instruction::Basic(BasicInstruction::SetMetadata {
                    entity_address,
                    key,
                    value,
                }) => {
                    let rtn = api.invoke(MetadataSetInvocation {
                        receiver: RENodeId::Global(entity_address.clone()),
                        key: key.clone(),
                        value: value.clone(),
                    })?;

                    InstructionOutput::Native(Box::new(rtn))
                }
                Instruction::Basic(BasicInstruction::SetPackageRoyaltyConfig {
                    package_address,
                    royalty_config,
                }) => {
                    let rtn = api.invoke(PackageSetRoyaltyConfigInvocation {
                        receiver: package_address.clone(),
                        royalty_config: royalty_config.clone(),
                    })?;
                    InstructionOutput::Native(Box::new(rtn))
                }
                Instruction::Basic(BasicInstruction::SetComponentRoyaltyConfig {
                    component_address,
                    royalty_config,
                }) => {
                    let rtn = api.invoke(ComponentSetRoyaltyConfigInvocation {
                        receiver: RENodeId::Global(GlobalAddress::Component(
                            component_address.clone(),
                        )),
                        royalty_config: royalty_config.clone(),
                    })?;

                    InstructionOutput::Native(Box::new(rtn))
                }
                Instruction::Basic(BasicInstruction::ClaimPackageRoyalty { package_address }) => {
                    let rtn = api.invoke(PackageClaimRoyaltyInvocation {
                        receiver: package_address.clone(),
                    })?;

                    Worktop::sys_put(Bucket(rtn.0), api)?;

                    InstructionOutput::Native(Box::new(rtn))
                }
                Instruction::Basic(BasicInstruction::ClaimComponentRoyalty {
                    component_address,
                }) => {
                    let rtn = api.invoke(ComponentClaimRoyaltyInvocation {
                        receiver: RENodeId::Global(GlobalAddress::Component(
                            component_address.clone(),
                        )),
                    })?;

                    Worktop::sys_put(Bucket(rtn.0), api)?;

                    InstructionOutput::Native(Box::new(rtn))
                }
                Instruction::Basic(BasicInstruction::SetMethodAccessRule {
                    entity_address,
                    index,
                    key,
                    rule,
                }) => {
                    let rtn = api.invoke(AccessRulesSetMethodAccessRuleInvocation {
                        receiver: RENodeId::Global(entity_address.clone()),
                        index: index.clone(),
                        key: key.clone(),
                        rule: rule.clone(),
                    })?;

                    InstructionOutput::Native(Box::new(rtn))
                }
                Instruction::System(SystemInstruction::CallNativeFunction {
                    function_ident,
                    args,
                }) => {
                    let args = processor
                        .replace_ids(
                            IndexedScryptoValue::from_slice(args)
                                .expect("Invalid CALL_NATIVE_FUNCTION arguments"),
                        )
                        .map_err(|e| {
                            RuntimeError::ApplicationError(
                                ApplicationError::TransactionProcessorError(e),
                            )
                        })
                        .and_then(|args| TransactionProcessor::process_expressions(args, api))?;

                    let native_function = resolve_native_function(
                        &function_ident.blueprint_name,
                        &function_ident.function_name,
                    )
                    .ok_or(RuntimeError::ApplicationError(
                        ApplicationError::TransactionProcessorError(
                            TransactionProcessorError::NativeFunctionNotFound(
                                function_ident.clone(),
                            ),
                        ),
                    ))?;

                    let rtn = parse_and_invoke_native_fn(
                        NativeFn::Function(native_function),
                        args.raw,
                        api,
                    )?;

                    InstructionOutput::Native(rtn)
                }
                Instruction::System(SystemInstruction::CallNativeMethod { method_ident, args }) => {
                    let args = processor
                        .replace_ids(
                            IndexedScryptoValue::from_slice(args)
                                .expect("Invalid CALL_NATIVE_METHOD arguments"),
                        )
                        .map_err(|e| {
                            RuntimeError::ApplicationError(
                                ApplicationError::TransactionProcessorError(e),
                            )
                        })
                        .and_then(|args| TransactionProcessor::process_expressions(args, api))?;

                    let native_method =
                        resolve_native_method(method_ident.receiver, &method_ident.method_name)
                            .ok_or(RuntimeError::ApplicationError(
                                ApplicationError::TransactionProcessorError(
                                    TransactionProcessorError::NativeMethodNotFound(
                                        method_ident.clone(),
                                    ),
                                ),
                            ))?;

                    let rtn =
                        parse_and_invoke_native_fn(NativeFn::Method(native_method), args.raw, api)?;

                    InstructionOutput::Native(rtn)
                }
            };
            outputs.push(result);

            api.emit_event(Event::Runtime(RuntimeEvent::PostExecuteInstruction {
                instruction_index: idx,
                instruction: &inst,
            }))?;
        }

        api.emit_event(Event::Runtime(RuntimeEvent::PostExecuteManifest))?;

        Ok((outputs, CallFrameUpdate::empty()))
    }
}

struct TransactionProcessor {
    proof_id_mapping: HashMap<ProofId, ProofId>,
    bucket_id_mapping: HashMap<BucketId, BucketId>,
    id_allocator: IdAllocator,
}

impl TransactionProcessor {
    fn new() -> Self {
        Self {
            proof_id_mapping: HashMap::new(),
            bucket_id_mapping: HashMap::new(),
            id_allocator: IdAllocator::new(IdSpace::Transaction),
        }
    }

    fn get_bucket(&mut self, bucket_id: &BucketId) -> Result<Bucket, RuntimeError> {
        let real_id = self.bucket_id_mapping.get(bucket_id).cloned().ok_or(
            RuntimeError::ApplicationError(ApplicationError::TransactionProcessorError(
                TransactionProcessorError::BucketNotFound(*bucket_id),
            )),
        )?;
        Ok(Bucket(real_id))
    }

    fn take_bucket(&mut self, bucket_id: &BucketId) -> Result<Bucket, RuntimeError> {
        let real_id =
            self.bucket_id_mapping
                .remove(bucket_id)
                .ok_or(RuntimeError::ApplicationError(
                    ApplicationError::TransactionProcessorError(
                        TransactionProcessorError::BucketNotFound(*bucket_id),
                    ),
                ))?;
        Ok(Bucket(real_id))
    }

    fn get_proof(&mut self, proof_id: &ProofId) -> Result<Proof, RuntimeError> {
        let real_id =
            self.proof_id_mapping
                .get(proof_id)
                .cloned()
                .ok_or(RuntimeError::ApplicationError(
                    ApplicationError::TransactionProcessorError(
                        TransactionProcessorError::ProofNotFound(*proof_id),
                    ),
                ))?;
        Ok(Proof(real_id))
    }

    fn take_proof(&mut self, proof_id: &ProofId) -> Result<Proof, RuntimeError> {
        let real_id =
            self.proof_id_mapping
                .remove(proof_id)
                .ok_or(RuntimeError::ApplicationError(
                    ApplicationError::TransactionProcessorError(
                        TransactionProcessorError::ProofNotFound(*proof_id),
                    ),
                ))?;
        Ok(Proof(real_id))
    }

    fn next_static_bucket(&mut self, bucket: Bucket) -> Result<Bucket, RuntimeError> {
        let new_id = self.id_allocator.new_bucket_id().map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::TransactionProcessorError(
                TransactionProcessorError::IdAllocationError(e),
            ))
        })?;
        self.bucket_id_mapping.insert(new_id, bucket.0);
        Ok(Bucket(new_id))
    }

    fn next_static_proof(&mut self, proof: Proof) -> Result<Proof, RuntimeError> {
        let new_id = self.id_allocator.new_proof_id().map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::TransactionProcessorError(
                TransactionProcessorError::IdAllocationError(e),
            ))
        })?;
        self.proof_id_mapping.insert(new_id, proof.0);
        Ok(Proof(new_id))
    }

    fn move_proofs_to_authzone_and_buckets_to_worktop<Y>(
        value: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: SystemApi
            + Invokable<ScryptoInvocation, RuntimeError>
            + EngineApi<RuntimeError>
            + InvokableModel<RuntimeError>,
    {
        // Auto move into auth_zone
        for (proof_id, _) in &value.proof_ids {
            let proof = Proof(*proof_id);
            ComponentAuthZone::sys_push(proof, api)?;
        }
        // Auto move into worktop
        for (bucket_id, _) in &value.bucket_ids {
            Worktop::sys_put(Bucket(*bucket_id), api)?;
        }

        Ok(())
    }

    fn replace_ids(
        &mut self,
        mut value: IndexedScryptoValue,
    ) -> Result<IndexedScryptoValue, TransactionProcessorError> {
        value
            .replace_ids(&mut self.proof_id_mapping, &mut self.bucket_id_mapping)
            .map_err(|e| match e {
                ValueReplacingError::BucketIdNotFound(bucket_id) => {
                    TransactionProcessorError::BucketNotFound(bucket_id)
                }
                ValueReplacingError::ProofIdNotFound(proof_id) => {
                    TransactionProcessorError::ProofNotFound(proof_id)
                }
            })?;
        Ok(value)
    }

    fn process_expressions<'a, Y>(
        args: IndexedScryptoValue,
        env: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        let mut value = args.dom;
        for (expression, path) in args.expressions {
            match expression.0.as_str() {
                "ENTIRE_WORKTOP" => {
                    let buckets = Worktop::sys_drain(env)?;

                    let val = path
                        .get_from_value_mut(&mut value)
                        .expect("Failed to locate an expression value using SBOR path");
                    *val = scrypto_decode(
                        &scrypto_encode(&buckets).expect("Failed to encode Vec<Bucket>"),
                    )
                    .expect("Failed to decode Vec<Bucket>")
                }
                "ENTIRE_AUTH_ZONE" => {
                    let proofs = ComponentAuthZone::sys_drain(env)?;

                    let val = path
                        .get_from_value_mut(&mut value)
                        .expect("Failed to locate an expression value using SBOR path");
                    *val = scrypto_decode(
                        &scrypto_encode(&proofs).expect("Failed to encode Vec<Proof>"),
                    )
                    .expect("Failed to decode Vec<Proof>")
                }
                _ => {} // no-op
            }
        }

        Ok(IndexedScryptoValue::from_value(value)
            .expect("SborValue became invalid post expression transformation"))
    }

    fn perform_validation<'a, Y>(
        request: &RuntimeValidationRequest,
        env: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: InvokableModel<RuntimeError>,
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
