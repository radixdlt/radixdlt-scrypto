use crate::model::resolve_native_function;
use crate::model::resolve_native_method;
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
use crate::model::{InvokeError, WorktopSubstate};
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

impl<'a> Invocation for TransactionProcessorRunInvocation<'a> {
    type Output = Vec<Vec<u8>>;
}

impl<'a, W: WasmEngine> ExecutableInvocation<W> for TransactionProcessorRunInvocation<'a> {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: ResolverApi<W>>(
        self,
        _api: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
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
                | Instruction::System(SystemInstruction::CallNativeFunction { args, .. }) => {
                    let scrypto_value =
                        IndexedScryptoValue::from_slice(&args).expect("Invalid CALL arguments");
                    for global_address in scrypto_value.global_references() {
                        call_frame_update
                            .node_refs_to_copy
                            .insert(RENodeId::Global(global_address));
                    }
                }
                Instruction::System(SystemInstruction::CallNativeMethod { args, method_ident }) => {
                    let scrypto_value =
                        IndexedScryptoValue::from_slice(&args).expect("Invalid CALL arguments");
                    for global_address in scrypto_value.global_references() {
                        call_frame_update
                            .node_refs_to_copy
                            .insert(RENodeId::Global(global_address));
                    }

                    // TODO: This needs to be cleaned up
                    // TODO: How does this relate to newly created vaults in the transaction frame?
                    // TODO: Will probably want different spacing for refed vs. owned nodes
                    match method_ident.receiver {
                        RENodeId::Vault(..) => {
                            call_frame_update
                                .node_refs_to_copy
                                .insert(method_ident.receiver);
                        }
                        _ => {}
                    }
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

        let actor = REActor::Function(ResolvedFunction::Native(
            NativeFunction::TransactionProcessor(TransactionProcessorFunction::Run),
        ));
        let executor = NativeExecutor(self);
        Ok((actor, call_frame_update, executor))
    }
}

impl<'a> NativeProcedure for TransactionProcessorRunInvocation<'a> {
    type Output = Vec<Vec<u8>>;

    fn main<Y>(self, system_api: &mut Y) -> Result<(Vec<Vec<u8>>, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi
            + Invokable<ScryptoInvocation, RuntimeError>
            + EngineApi<RuntimeError>
            + InvokableModel<RuntimeError>,
    {
        TransactionProcessor::run(self, system_api)
            .map(|rtn| (rtn, CallFrameUpdate::empty()))
            .map_err(|e| e.into())
    }
}

pub struct TransactionProcessor {}

impl TransactionProcessor {
    fn replace_ids(
        proof_id_mapping: &mut HashMap<ProofId, ProofId>,
        bucket_id_mapping: &mut HashMap<BucketId, BucketId>,
        mut value: IndexedScryptoValue,
    ) -> Result<IndexedScryptoValue, InvokeError<TransactionProcessorError>> {
        value
            .replace_ids(proof_id_mapping, bucket_id_mapping)
            .map_err(|e| match e {
                ValueReplacingError::BucketIdNotFound(bucket_id) => {
                    InvokeError::Error(TransactionProcessorError::BucketNotFound(bucket_id))
                }
                ValueReplacingError::ProofIdNotFound(proof_id) => {
                    InvokeError::Error(TransactionProcessorError::ProofNotFound(proof_id))
                }
            })?;
        Ok(value)
    }

    fn process_expressions<'a, Y>(
        args: IndexedScryptoValue,
        env: &mut Y,
    ) -> Result<IndexedScryptoValue, InvokeError<TransactionProcessorError>>
    where
        Y: EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        let mut value = args.dom;
        for (expression, path) in args.expressions {
            match expression.0.as_str() {
                "ENTIRE_WORKTOP" => {
                    let buckets = Worktop::sys_drain(env).map_err(InvokeError::Downstream)?;

                    let val = path
                        .get_from_value_mut(&mut value)
                        .expect("Failed to locate an expression value using SBOR path");
                    *val = scrypto_decode(
                        &scrypto_encode(&buckets).expect("Failed to encode Vec<Bucket>"),
                    )
                    .expect("Failed to decode Vec<Bucket>")
                }
                "ENTIRE_AUTH_ZONE" => {
                    let proofs =
                        ComponentAuthZone::sys_drain(env).map_err(InvokeError::Downstream)?;

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
    ) -> Result<(), InvokeError<TransactionProcessorError>>
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
                    return Err(InvokeError::Error(
                        TransactionProcessorError::TransactionEpochNotYetValid {
                            valid_from: *start_epoch_inclusive,
                            current_epoch,
                        },
                    ));
                }
                if !should_skip_assertion && current_epoch >= *end_epoch_exclusive {
                    return Err(InvokeError::Error(
                        TransactionProcessorError::TransactionEpochNoLongerValid {
                            valid_until: *end_epoch_exclusive - 1,
                            current_epoch,
                        },
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

    pub fn run<Y>(
        input: TransactionProcessorRunInvocation,
        api: &mut Y,
    ) -> Result<Vec<Vec<u8>>, InvokeError<TransactionProcessorError>>
    where
        Y: SystemApi
            + EngineApi<RuntimeError>
            + Invokable<ScryptoInvocation, RuntimeError>
            + InvokableModel<RuntimeError>,
    {
        for request in input.runtime_validations.as_ref() {
            Self::perform_validation(request, api)?;
        }
        let mut proof_id_mapping = HashMap::new();
        let mut bucket_id_mapping = HashMap::new();
        let mut outputs = Vec::new();
        let mut id_allocator = IdAllocator::new(IdSpace::Transaction);

        let node_id = api.allocate_node_id(RENodeType::Worktop)?;
        let _worktop_id = api
            .create_node(node_id, RENode::Worktop(WorktopSubstate::new()))
            .map_err(InvokeError::Downstream)?;

        api.emit_event(Event::Runtime(RuntimeEvent::PreExecuteManifest))
            .map_err(InvokeError::Downstream)?;

        for (idx, inst) in input.instructions.as_ref().iter().enumerate() {
            api.emit_event(Event::Runtime(RuntimeEvent::PreExecuteInstruction {
                instruction_index: idx,
                instruction: &inst,
            }))
            .map_err(InvokeError::Downstream)?;

            let result = match inst {
                Instruction::Basic(BasicInstruction::TakeFromWorktop { resource_address }) => {
                    id_allocator
                        .new_bucket_id()
                        .map_err(|e| {
                            InvokeError::Error(TransactionProcessorError::IdAllocationError(e))
                        })
                        .and_then(|new_id| {
                            Worktop::sys_take_all(*resource_address, api)
                                .map_err(InvokeError::Downstream)
                                .map(|bucket| {
                                    bucket_id_mapping.insert(new_id, bucket.0);
                                    IndexedScryptoValue::from_typed(&bucket)
                                })
                        })
                }
                Instruction::Basic(BasicInstruction::TakeFromWorktopByAmount {
                    amount,
                    resource_address,
                }) => id_allocator
                    .new_bucket_id()
                    .map_err(|e| {
                        InvokeError::Error(TransactionProcessorError::IdAllocationError(e))
                    })
                    .and_then(|new_id| {
                        Worktop::sys_take_amount(*resource_address, *amount, api)
                            .map_err(InvokeError::Downstream)
                            .map(|bucket| {
                                bucket_id_mapping.insert(new_id, bucket.0);
                                IndexedScryptoValue::from_typed(&bucket)
                            })
                    }),
                Instruction::Basic(BasicInstruction::TakeFromWorktopByIds {
                    ids,
                    resource_address,
                }) => id_allocator
                    .new_bucket_id()
                    .map_err(|e| {
                        InvokeError::Error(TransactionProcessorError::IdAllocationError(e))
                    })
                    .and_then(|new_id| {
                        Worktop::sys_take_non_fungibles(*resource_address, ids.clone(), api)
                            .map_err(InvokeError::Downstream)
                            .map(|bucket| {
                                bucket_id_mapping.insert(new_id, bucket.0);
                                IndexedScryptoValue::from_typed(&bucket)
                            })
                    }),
                Instruction::Basic(BasicInstruction::ReturnToWorktop { bucket_id }) => {
                    bucket_id_mapping
                        .remove(bucket_id)
                        .map(|real_id| {
                            Worktop::sys_put(Bucket(real_id), api)
                                .map(|rtn| IndexedScryptoValue::from_typed(&rtn))
                                .map_err(InvokeError::Downstream)
                        })
                        .unwrap_or(Err(InvokeError::Error(
                            TransactionProcessorError::BucketNotFound(*bucket_id),
                        )))
                }
                Instruction::Basic(BasicInstruction::AssertWorktopContains {
                    resource_address,
                }) => Worktop::sys_assert_contains(*resource_address, api)
                    .map(|rtn| IndexedScryptoValue::from_typed(&rtn))
                    .map_err(InvokeError::Downstream),
                Instruction::Basic(BasicInstruction::AssertWorktopContainsByAmount {
                    amount,
                    resource_address,
                }) => Worktop::sys_assert_contains_amount(*resource_address, *amount, api)
                    .map(|rtn| IndexedScryptoValue::from_typed(&rtn))
                    .map_err(InvokeError::Downstream),
                Instruction::Basic(BasicInstruction::AssertWorktopContainsByIds {
                    ids,
                    resource_address,
                }) => {
                    Worktop::sys_assert_contains_non_fungibles(*resource_address, ids.clone(), api)
                        .map(|rtn| IndexedScryptoValue::from_typed(&rtn))
                        .map_err(InvokeError::Downstream)
                }

                Instruction::Basic(BasicInstruction::PopFromAuthZone {}) => id_allocator
                    .new_proof_id()
                    .map_err(|e| {
                        InvokeError::Error(TransactionProcessorError::IdAllocationError(e))
                    })
                    .and_then(|new_id| {
                        ComponentAuthZone::sys_pop(api)
                            .map_err(InvokeError::Downstream)
                            .map(|proof| {
                                proof_id_mapping.insert(new_id, proof.0);
                                IndexedScryptoValue::from_typed(&proof)
                            })
                    }),
                Instruction::Basic(BasicInstruction::ClearAuthZone) => {
                    proof_id_mapping.clear();
                    ComponentAuthZone::sys_clear(api)
                        .map(|rtn| IndexedScryptoValue::from_typed(&rtn))
                        .map_err(InvokeError::Downstream)
                }
                Instruction::Basic(BasicInstruction::PushToAuthZone { proof_id }) => {
                    proof_id_mapping
                        .remove(proof_id)
                        .ok_or(InvokeError::Error(
                            TransactionProcessorError::ProofNotFound(*proof_id),
                        ))
                        .and_then(|real_id| {
                            let proof = Proof(real_id);
                            ComponentAuthZone::sys_push(proof, api)
                                .map(|rtn| IndexedScryptoValue::from_typed(&rtn))
                                .map_err(InvokeError::Downstream)
                        })
                }
                Instruction::Basic(BasicInstruction::CreateProofFromAuthZone {
                    resource_address,
                }) => id_allocator
                    .new_proof_id()
                    .map_err(|e| {
                        InvokeError::Error(TransactionProcessorError::IdAllocationError(e))
                    })
                    .and_then(|new_id| {
                        ComponentAuthZone::sys_create_proof(*resource_address, api)
                            .map_err(InvokeError::Downstream)
                            .map(|proof| {
                                proof_id_mapping.insert(new_id, proof.0);
                                IndexedScryptoValue::from_typed(&proof)
                            })
                    }),
                Instruction::Basic(BasicInstruction::CreateProofFromAuthZoneByAmount {
                    amount,
                    resource_address,
                }) => id_allocator
                    .new_proof_id()
                    .map_err(|e| {
                        InvokeError::Error(TransactionProcessorError::IdAllocationError(e))
                    })
                    .and_then(|new_id| {
                        ComponentAuthZone::sys_create_proof_by_amount(
                            *amount,
                            *resource_address,
                            api,
                        )
                        .map_err(InvokeError::Downstream)
                        .map(|proof| {
                            proof_id_mapping.insert(new_id, proof.0);
                            IndexedScryptoValue::from_typed(&proof)
                        })
                    }),
                Instruction::Basic(BasicInstruction::CreateProofFromAuthZoneByIds {
                    ids,
                    resource_address,
                }) => id_allocator
                    .new_proof_id()
                    .map_err(|e| {
                        InvokeError::Error(TransactionProcessorError::IdAllocationError(e))
                    })
                    .and_then(|new_id| {
                        ComponentAuthZone::sys_create_proof_by_ids(ids, *resource_address, api)
                            .map_err(InvokeError::Downstream)
                            .map(|proof| {
                                proof_id_mapping.insert(new_id, proof.0);
                                IndexedScryptoValue::from_typed(&proof)
                            })
                    }),
                Instruction::Basic(BasicInstruction::CreateProofFromBucket { bucket_id }) => {
                    id_allocator
                        .new_proof_id()
                        .map_err(|e| {
                            InvokeError::Error(TransactionProcessorError::IdAllocationError(e))
                        })
                        .and_then(|new_id| {
                            bucket_id_mapping
                                .get(bucket_id)
                                .cloned()
                                .map(|real_bucket_id| (new_id, real_bucket_id))
                                .ok_or(InvokeError::Error(
                                    TransactionProcessorError::BucketNotFound(new_id),
                                ))
                        })
                        .and_then(|(new_id, real_bucket_id)| {
                            let bucket = Bucket(real_bucket_id);
                            bucket
                                .sys_create_proof(api)
                                .map_err(InvokeError::Downstream)
                                .map(|proof| {
                                    proof_id_mapping.insert(new_id, proof.0);
                                    IndexedScryptoValue::from_typed(&proof)
                                })
                        })
                }
                Instruction::Basic(BasicInstruction::CloneProof { proof_id }) => id_allocator
                    .new_proof_id()
                    .map_err(|e| {
                        InvokeError::Error(TransactionProcessorError::IdAllocationError(e))
                    })
                    .and_then(|new_id| {
                        proof_id_mapping
                            .get(proof_id)
                            .cloned()
                            .map(|real_id| {
                                let proof = Proof(real_id);
                                proof
                                    .sys_clone(api)
                                    .map_err(InvokeError::Downstream)
                                    .map(|proof| {
                                        proof_id_mapping.insert(new_id, proof.0);
                                        IndexedScryptoValue::from_typed(&proof)
                                    })
                            })
                            .unwrap_or(Err(InvokeError::Error(
                                TransactionProcessorError::ProofNotFound(*proof_id),
                            )))
                    }),
                Instruction::Basic(BasicInstruction::DropProof { proof_id }) => proof_id_mapping
                    .remove(proof_id)
                    .map(|real_id| {
                        let proof = Proof(real_id);
                        proof
                            .sys_drop(api)
                            .map(|_| IndexedScryptoValue::unit())
                            .map_err(InvokeError::Downstream)
                    })
                    .unwrap_or(Err(InvokeError::Error(
                        TransactionProcessorError::ProofNotFound(*proof_id),
                    ))),
                Instruction::Basic(BasicInstruction::DropAllProofs) => {
                    for (_, real_id) in proof_id_mapping.drain() {
                        let proof = Proof(real_id);
                        proof
                            .sys_drop(api)
                            .map(|_| IndexedScryptoValue::unit())
                            .map_err(InvokeError::Downstream)?;
                    }
                    ComponentAuthZone::sys_clear(api)
                        .map(|rtn| IndexedScryptoValue::from_typed(&rtn))
                        .map_err(InvokeError::Downstream)
                }
                Instruction::Basic(BasicInstruction::CallFunction {
                    package_address,
                    blueprint_name,
                    function_name,
                    args,
                }) => {
                    Self::replace_ids(
                        &mut proof_id_mapping,
                        &mut bucket_id_mapping,
                        IndexedScryptoValue::from_slice(args)
                            .expect("Invalid CALL_FUNCTION arguments"),
                    )
                    .and_then(|args| Self::process_expressions(args, api))
                    .and_then(|args| {
                        api.invoke(ParsedScryptoInvocation::Function(
                            ScryptoFunctionIdent {
                                package: ScryptoPackage::Global(package_address.clone()),
                                blueprint_name: blueprint_name.clone(),
                                function_name: function_name.clone(),
                            },
                            args,
                        ))
                        .map_err(InvokeError::Downstream)
                    })
                    .and_then(|result| {
                        // Auto move into auth_zone
                        for (proof_id, _) in &result.proof_ids {
                            let proof = Proof(*proof_id);
                            ComponentAuthZone::sys_push(proof, api)
                                .map_err(InvokeError::Downstream)?;
                        }
                        // Auto move into worktop
                        for (bucket_id, _) in &result.bucket_ids {
                            Worktop::sys_put(Bucket(*bucket_id), api)
                                .map_err(InvokeError::Downstream)?;
                        }
                        Ok(result)
                    })
                }
                Instruction::Basic(BasicInstruction::CallMethod {
                    component_address,
                    method_name,
                    args,
                }) => {
                    Self::replace_ids(
                        &mut proof_id_mapping,
                        &mut bucket_id_mapping,
                        IndexedScryptoValue::from_slice(args)
                            .expect("Invalid CALL_METHOD arguments"),
                    )
                    .and_then(|args| Self::process_expressions(args, api))
                    .and_then(|args| {
                        api.invoke(ParsedScryptoInvocation::Method(
                            ScryptoMethodIdent {
                                receiver: ScryptoReceiver::Global(component_address.clone()),
                                method_name: method_name.clone(),
                            },
                            args,
                        ))
                        .map_err(InvokeError::Downstream)
                    })
                    .and_then(|result| {
                        // Auto move into auth_zone
                        for (proof_id, _) in &result.proof_ids {
                            let proof = Proof(*proof_id);
                            ComponentAuthZone::sys_push(proof, api)
                                .map_err(InvokeError::Downstream)?;
                        }
                        // Auto move into worktop
                        for (bucket_id, _) in &result.bucket_ids {
                            Worktop::sys_put(Bucket(*bucket_id), api)
                                .map_err(InvokeError::downstream)?;
                        }
                        Ok(result)
                    })
                }
                Instruction::Basic(BasicInstruction::PublishPackage {
                    code,
                    abi,
                    royalty_config,
                    metadata,
                    access_rules,
                }) => api
                    .invoke(PackagePublishInvocation {
                        code: code.clone(),
                        abi: abi.clone(),
                        royalty_config: royalty_config.clone(),
                        metadata: metadata.clone(),
                        access_rules: access_rules.clone(),
                    })
                    .map(|rtn| IndexedScryptoValue::from_typed(&rtn))
                    .map_err(InvokeError::Downstream),
                Instruction::Basic(BasicInstruction::PublishPackageWithOwner {
                    code,
                    abi,
                    owner_badge,
                }) => api
                    .invoke(PackagePublishWithOwnerInvocation {
                        code: code.clone(),
                        abi: abi.clone(),
                        royalty_config: BTreeMap::new(),
                        metadata: BTreeMap::new(),
                        owner_badge: owner_badge.clone(),
                    })
                    .map(|rtn| IndexedScryptoValue::from_typed(&rtn))
                    .map_err(InvokeError::Downstream),
                Instruction::Basic(BasicInstruction::CreateResource {
                    resource_type,
                    metadata,
                    access_rules,
                    mint_params,
                }) => api
                    .invoke(ResourceManagerCreateInvocation {
                        resource_type: resource_type.clone(),
                        metadata: metadata.clone(),
                        access_rules: access_rules.clone(),
                        mint_params: mint_params.clone(),
                    })
                    .map(|rtn| IndexedScryptoValue::from_typed(&rtn))
                    .map_err(InvokeError::Downstream)
                    .and_then(|result| {
                        // Auto move into worktop
                        for (bucket_id, _) in &result.bucket_ids {
                            Worktop::sys_put(Bucket(*bucket_id), api)
                                .map_err(InvokeError::downstream)?;
                        }
                        Ok(result)
                    }),
                Instruction::Basic(BasicInstruction::CreateResourceWithOwner {
                    resource_type,
                    metadata,
                    owner_badge,
                    mint_params,
                }) => api
                    .invoke(ResourceManagerCreateWithOwnerInvocation {
                        resource_type: resource_type.clone(),
                        metadata: metadata.clone(),
                        owner_badge: owner_badge.clone(),
                        mint_params: mint_params.clone(),
                    })
                    .map(|rtn| IndexedScryptoValue::from_typed(&rtn))
                    .map_err(InvokeError::Downstream)
                    .and_then(|result| {
                        // Auto move into worktop
                        for (bucket_id, _) in &result.bucket_ids {
                            Worktop::sys_put(Bucket(*bucket_id), api)
                                .map_err(InvokeError::downstream)?;
                        }
                        Ok(result)
                    }),
                Instruction::Basic(BasicInstruction::BurnResource { bucket_id }) => {
                    bucket_id_mapping
                        .get(bucket_id)
                        .cloned()
                        .ok_or(InvokeError::Error(
                            TransactionProcessorError::BucketNotFound(*bucket_id),
                        ))
                        .and_then(|bucket_id| {
                            api.invoke(ResourceManagerBucketBurnInvocation {
                                bucket: Bucket(bucket_id.clone()),
                            })
                            .map(|rtn| IndexedScryptoValue::from_typed(&rtn))
                            .map_err(InvokeError::Downstream)
                        })
                }
                Instruction::Basic(BasicInstruction::MintFungible {
                    resource_address,
                    amount,
                }) => api
                    .invoke(ResourceManagerMintInvocation {
                        receiver: resource_address.clone(),
                        mint_params: MintParams::Fungible {
                            amount: amount.clone(),
                        },
                    })
                    .map(|rtn| IndexedScryptoValue::from_typed(&rtn))
                    .map_err(InvokeError::Downstream)
                    .and_then(|result| {
                        // Auto move into worktop
                        for (bucket_id, _) in &result.bucket_ids {
                            Worktop::sys_put(Bucket(*bucket_id), api)
                                .map_err(InvokeError::downstream)?;
                        }
                        Ok(result)
                    }),
                Instruction::Basic(BasicInstruction::SetMetadata {
                    entity_address,
                    key,
                    value,
                }) => api
                    .invoke(MetadataSetInvocation {
                        receiver: RENodeId::Global(entity_address.clone()),
                        key: key.clone(),
                        value: value.clone(),
                    })
                    .map(|rtn| IndexedScryptoValue::from_typed(&rtn))
                    .map_err(InvokeError::Downstream),
                Instruction::Basic(BasicInstruction::SetPackageRoyaltyConfig {
                    package_address,
                    royalty_config,
                }) => api
                    .invoke(PackageSetRoyaltyConfigInvocation {
                        receiver: package_address.clone(),
                        royalty_config: royalty_config.clone(),
                    })
                    .map(|rtn| IndexedScryptoValue::from_typed(&rtn))
                    .map_err(InvokeError::Downstream),
                Instruction::Basic(BasicInstruction::SetComponentRoyaltyConfig {
                    component_address,
                    royalty_config,
                }) => api
                    .invoke(ComponentSetRoyaltyConfigInvocation {
                        receiver: RENodeId::Global(GlobalAddress::Component(
                            component_address.clone(),
                        )),
                        royalty_config: royalty_config.clone(),
                    })
                    .map(|rtn| IndexedScryptoValue::from_typed(&rtn))
                    .map_err(InvokeError::Downstream),
                Instruction::Basic(BasicInstruction::ClaimPackageRoyalty { package_address }) => {
                    api.invoke(PackageClaimRoyaltyInvocation {
                        receiver: package_address.clone(),
                    })
                    .map(|rtn| IndexedScryptoValue::from_typed(&rtn))
                    .map_err(InvokeError::Downstream)
                    .and_then(|result| {
                        // Auto move into worktop
                        for (bucket_id, _) in &result.bucket_ids {
                            Worktop::sys_put(Bucket(*bucket_id), api)
                                .map_err(InvokeError::downstream)?;
                        }
                        Ok(result)
                    })
                }
                Instruction::Basic(BasicInstruction::ClaimComponentRoyalty {
                    component_address,
                }) => api
                    .invoke(ComponentClaimRoyaltyInvocation {
                        receiver: RENodeId::Global(GlobalAddress::Component(
                            component_address.clone(),
                        )),
                    })
                    .map(|rtn| IndexedScryptoValue::from_typed(&rtn))
                    .map_err(InvokeError::Downstream)
                    .and_then(|result| {
                        // Auto move into worktop
                        for (bucket_id, _) in &result.bucket_ids {
                            Worktop::sys_put(Bucket(*bucket_id), api)
                                .map_err(InvokeError::downstream)?;
                        }
                        Ok(result)
                    }),
                Instruction::System(SystemInstruction::CallNativeFunction {
                    function_ident,
                    args,
                }) => {
                    Self::replace_ids(
                        &mut proof_id_mapping,
                        &mut bucket_id_mapping,
                        IndexedScryptoValue::from_slice(args)
                            .expect("Invalid CALL_NATIVE_FUNCTION arguments"),
                    )
                    .and_then(|args| Self::process_expressions(args, api))
                    .and_then(|args| {
                        let native_function = resolve_native_function(
                            &function_ident.blueprint_name,
                            &function_ident.function_name,
                        )
                        .ok_or(InvokeError::Error(
                            TransactionProcessorError::NativeFunctionNotFound(
                                function_ident.clone(),
                            ),
                        ))?;
                        parse_and_invoke_native_fn(
                            NativeFn::Function(native_function),
                            args.raw,
                            api,
                        )
                        .map_err(InvokeError::Downstream)
                    })
                    .and_then(|result| {
                        // Auto move into auth_zone
                        for (proof_id, _) in &result.proof_ids {
                            let proof = Proof(*proof_id);
                            ComponentAuthZone::sys_push(proof, api)
                                .map_err(InvokeError::Downstream)?;
                        }
                        // Auto move into worktop
                        for (bucket_id, _) in &result.bucket_ids {
                            Worktop::sys_put(Bucket(*bucket_id), api)
                                .map_err(InvokeError::Downstream)?;
                        }
                        Ok(result)
                    })
                }
                Instruction::System(SystemInstruction::CallNativeMethod { method_ident, args }) => {
                    Self::replace_ids(
                        &mut proof_id_mapping,
                        &mut bucket_id_mapping,
                        IndexedScryptoValue::from_slice(args)
                            .expect("Invalid CALL_NATIVE_METHOD arguments"),
                    )
                    .and_then(|args| Self::process_expressions(args, api))
                    .and_then(|args| {
                        let native_method =
                            resolve_native_method(method_ident.receiver, &method_ident.method_name)
                                .ok_or(InvokeError::Error(
                                    TransactionProcessorError::NativeMethodNotFound(
                                        method_ident.clone(),
                                    ),
                                ))?;

                        parse_and_invoke_native_fn(NativeFn::Method(native_method), args.raw, api)
                            .map_err(InvokeError::Downstream)
                    })
                    .and_then(|result| {
                        // Auto move into auth_zone
                        for (proof_id, _) in &result.proof_ids {
                            let proof = Proof(*proof_id);
                            ComponentAuthZone::sys_push(proof, api)
                                .map_err(InvokeError::Downstream)?;
                        }
                        // Auto move into worktop
                        for (bucket_id, _) in &result.bucket_ids {
                            Worktop::sys_put(Bucket(*bucket_id), api)
                                .map_err(InvokeError::downstream)?;
                        }
                        Ok(result)
                    })
                }
            }?;
            outputs.push(result);

            api.emit_event(Event::Runtime(RuntimeEvent::PostExecuteInstruction {
                instruction_index: idx,
                instruction: &inst,
            }))
            .map_err(InvokeError::Downstream)?;
        }

        api.emit_event(Event::Runtime(RuntimeEvent::PostExecuteManifest))
            .map_err(InvokeError::Downstream)?;

        Ok(outputs
            .into_iter()
            .map(|sv| sv.raw)
            .collect::<Vec<Vec<u8>>>())
    }
}
