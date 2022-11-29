use radix_engine_interface::api::api::{EngineApi, Invocation, SysInvokableNative};
use radix_engine_interface::api::types::{
    BucketId, GlobalAddress, NativeFn, NativeFunction, NativeFunctionIdent, NativeMethodIdent,
    ProofId, RENodeId, TransactionProcessorFunction,
};
use radix_engine_interface::data::{IndexedScryptoValue, ValueReplacingError};
use radix_engine_interface::model::*;
use sbor::rust::borrow::Cow;
use scrypto::resource::Worktop;
use scrypto::resource::{ComponentAuthZone, SysBucket, SysProof};
use scrypto::runtime::Runtime;
use transaction::errors::IdAllocationError;
use transaction::model::*;
use transaction::validation::*;

use crate::engine::*;
use crate::model::resolve_native_function;
use crate::model::resolve_native_method;
use crate::model::{InvokeError, WorktopSubstate};
use crate::types::*;

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

impl<'a> ExecutableInvocation for TransactionProcessorRunInvocation<'a> {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        self,
        _deref: &mut D,
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
                Instruction::CallFunction { args, .. }
                | Instruction::CallMethod { args, .. }
                | Instruction::CallNativeFunction { args, .. } => {
                    let scrypto_value =
                        IndexedScryptoValue::from_slice(&args).expect("Invalid CALL arguments");
                    for global_address in scrypto_value.global_references() {
                        call_frame_update
                            .node_refs_to_copy
                            .insert(RENodeId::Global(global_address));
                    }
                }
                Instruction::CallNativeMethod { args, method_ident } => {
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
        let executor = NativeExecutor(self, input);
        Ok((actor, call_frame_update, executor))
    }
}

impl<'a> NativeProcedure for TransactionProcessorRunInvocation<'a> {
    type Output = Vec<Vec<u8>>;

    fn main<Y>(self, system_api: &mut Y) -> Result<(Vec<Vec<u8>>, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi
            + Invokable<ScryptoInvocation>
            + EngineApi<RuntimeError>
            + SysInvokableNative<RuntimeError>,
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
        Y: EngineApi<RuntimeError> + SysInvokableNative<RuntimeError>,
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
        Y: SysInvokableNative<RuntimeError>,
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
        env: &mut Y,
    ) -> Result<Vec<Vec<u8>>, InvokeError<TransactionProcessorError>>
    where
        Y: SystemApi
            + EngineApi<RuntimeError>
            + Invokable<ScryptoInvocation>
            + SysInvokableNative<RuntimeError>,
    {
        for request in input.runtime_validations.as_ref() {
            Self::perform_validation(request, env)?;
        }
        let mut proof_id_mapping = HashMap::new();
        let mut bucket_id_mapping = HashMap::new();
        let mut outputs = Vec::new();
        let mut id_allocator = IdAllocator::new(IdSpace::Transaction);

        let _worktop_id = env
            .create_node(RENode::Worktop(WorktopSubstate::new()))
            .map_err(InvokeError::Downstream)?;

        env.emit_event(Event::Runtime(RuntimeEvent::PreExecuteManifest))
            .map_err(InvokeError::Downstream)?;

        for (idx, inst) in input.instructions.as_ref().iter().enumerate() {
            env.emit_event(Event::Runtime(RuntimeEvent::PreExecuteInstruction {
                instruction_index: idx,
                instruction: &inst,
            }))
            .map_err(InvokeError::Downstream)?;

            let result = match inst {
                Instruction::TakeFromWorktop { resource_address } => id_allocator
                    .new_bucket_id()
                    .map_err(|e| {
                        InvokeError::Error(TransactionProcessorError::IdAllocationError(e))
                    })
                    .and_then(|new_id| {
                        Worktop::sys_take_all(*resource_address, env)
                            .map_err(InvokeError::Downstream)
                            .map(|bucket| {
                                bucket_id_mapping.insert(new_id, bucket.0);
                                IndexedScryptoValue::from_typed(&bucket)
                            })
                    }),
                Instruction::TakeFromWorktopByAmount {
                    amount,
                    resource_address,
                } => id_allocator
                    .new_bucket_id()
                    .map_err(|e| {
                        InvokeError::Error(TransactionProcessorError::IdAllocationError(e))
                    })
                    .and_then(|new_id| {
                        Worktop::sys_take_amount(*resource_address, *amount, env)
                            .map_err(InvokeError::Downstream)
                            .map(|bucket| {
                                bucket_id_mapping.insert(new_id, bucket.0);
                                IndexedScryptoValue::from_typed(&bucket)
                            })
                    }),
                Instruction::TakeFromWorktopByIds {
                    ids,
                    resource_address,
                } => id_allocator
                    .new_bucket_id()
                    .map_err(|e| {
                        InvokeError::Error(TransactionProcessorError::IdAllocationError(e))
                    })
                    .and_then(|new_id| {
                        Worktop::sys_take_non_fungibles(*resource_address, ids.clone(), env)
                            .map_err(InvokeError::Downstream)
                            .map(|bucket| {
                                bucket_id_mapping.insert(new_id, bucket.0);
                                IndexedScryptoValue::from_typed(&bucket)
                            })
                    }),
                Instruction::ReturnToWorktop { bucket_id } => bucket_id_mapping
                    .remove(bucket_id)
                    .map(|real_id| {
                        Worktop::sys_put(Bucket(real_id), env)
                            .map(|rtn| IndexedScryptoValue::from_typed(&rtn))
                            .map_err(InvokeError::Downstream)
                    })
                    .unwrap_or(Err(InvokeError::Error(
                        TransactionProcessorError::BucketNotFound(*bucket_id),
                    ))),
                Instruction::AssertWorktopContains { resource_address } => {
                    Worktop::sys_assert_contains(*resource_address, env)
                        .map(|rtn| IndexedScryptoValue::from_typed(&rtn))
                        .map_err(InvokeError::Downstream)
                }
                Instruction::AssertWorktopContainsByAmount {
                    amount,
                    resource_address,
                } => Worktop::sys_assert_contains_amount(*resource_address, *amount, env)
                    .map(|rtn| IndexedScryptoValue::from_typed(&rtn))
                    .map_err(InvokeError::Downstream),
                Instruction::AssertWorktopContainsByIds {
                    ids,
                    resource_address,
                } => {
                    Worktop::sys_assert_contains_non_fungibles(*resource_address, ids.clone(), env)
                        .map(|rtn| IndexedScryptoValue::from_typed(&rtn))
                        .map_err(InvokeError::Downstream)
                }

                Instruction::PopFromAuthZone {} => id_allocator
                    .new_proof_id()
                    .map_err(|e| {
                        InvokeError::Error(TransactionProcessorError::IdAllocationError(e))
                    })
                    .and_then(|new_id| {
                        ComponentAuthZone::sys_pop(env)
                            .map_err(InvokeError::Downstream)
                            .map(|proof| {
                                proof_id_mapping.insert(new_id, proof.0);
                                IndexedScryptoValue::from_typed(&proof)
                            })
                    }),
                Instruction::ClearAuthZone => {
                    proof_id_mapping.clear();
                    ComponentAuthZone::sys_clear(env)
                        .map(|rtn| IndexedScryptoValue::from_typed(&rtn))
                        .map_err(InvokeError::Downstream)
                }
                Instruction::PushToAuthZone { proof_id } => proof_id_mapping
                    .remove(proof_id)
                    .ok_or(InvokeError::Error(
                        TransactionProcessorError::ProofNotFound(*proof_id),
                    ))
                    .and_then(|real_id| {
                        let proof = Proof(real_id);
                        ComponentAuthZone::sys_push(proof, env)
                            .map(|rtn| IndexedScryptoValue::from_typed(&rtn))
                            .map_err(InvokeError::Downstream)
                    }),
                Instruction::CreateProofFromAuthZone { resource_address } => id_allocator
                    .new_proof_id()
                    .map_err(|e| {
                        InvokeError::Error(TransactionProcessorError::IdAllocationError(e))
                    })
                    .and_then(|new_id| {
                        ComponentAuthZone::sys_create_proof(*resource_address, env)
                            .map_err(InvokeError::Downstream)
                            .map(|proof| {
                                proof_id_mapping.insert(new_id, proof.0);
                                IndexedScryptoValue::from_typed(&proof)
                            })
                    }),
                Instruction::CreateProofFromAuthZoneByAmount {
                    amount,
                    resource_address,
                } => id_allocator
                    .new_proof_id()
                    .map_err(|e| {
                        InvokeError::Error(TransactionProcessorError::IdAllocationError(e))
                    })
                    .and_then(|new_id| {
                        ComponentAuthZone::sys_create_proof_by_amount(
                            *amount,
                            *resource_address,
                            env,
                        )
                        .map_err(InvokeError::Downstream)
                        .map(|proof| {
                            proof_id_mapping.insert(new_id, proof.0);
                            IndexedScryptoValue::from_typed(&proof)
                        })
                    }),
                Instruction::CreateProofFromAuthZoneByIds {
                    ids,
                    resource_address,
                } => id_allocator
                    .new_proof_id()
                    .map_err(|e| {
                        InvokeError::Error(TransactionProcessorError::IdAllocationError(e))
                    })
                    .and_then(|new_id| {
                        ComponentAuthZone::sys_create_proof_by_ids(ids, *resource_address, env)
                            .map_err(InvokeError::Downstream)
                            .map(|proof| {
                                proof_id_mapping.insert(new_id, proof.0);
                                IndexedScryptoValue::from_typed(&proof)
                            })
                    }),
                Instruction::CreateProofFromBucket { bucket_id } => id_allocator
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
                            .sys_create_proof(env)
                            .map_err(InvokeError::Downstream)
                            .map(|proof| {
                                proof_id_mapping.insert(new_id, proof.0);
                                IndexedScryptoValue::from_typed(&proof)
                            })
                    }),
                Instruction::CloneProof { proof_id } => id_allocator
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
                                    .sys_clone(env)
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
                Instruction::DropProof { proof_id } => proof_id_mapping
                    .remove(proof_id)
                    .map(|real_id| {
                        let proof = Proof(real_id);
                        proof
                            .sys_drop(env)
                            .map(|_| IndexedScryptoValue::unit())
                            .map_err(InvokeError::Downstream)
                    })
                    .unwrap_or(Err(InvokeError::Error(
                        TransactionProcessorError::ProofNotFound(*proof_id),
                    ))),
                Instruction::DropAllProofs => {
                    for (_, real_id) in proof_id_mapping.drain() {
                        let proof = Proof(real_id);
                        proof
                            .sys_drop(env)
                            .map(|_| IndexedScryptoValue::unit())
                            .map_err(InvokeError::Downstream)?;
                    }
                    ComponentAuthZone::sys_clear(env)
                        .map(|rtn| IndexedScryptoValue::from_typed(&rtn))
                        .map_err(InvokeError::Downstream)
                }
                Instruction::CallFunction {
                    function_ident,
                    args,
                } => {
                    Self::replace_ids(
                        &mut proof_id_mapping,
                        &mut bucket_id_mapping,
                        IndexedScryptoValue::from_slice(args)
                            .expect("Invalid CALL_FUNCTION arguments"),
                    )
                    .and_then(|args| Self::process_expressions(args, env))
                    .and_then(|args| {
                        env.invoke(ScryptoInvocation::Function(function_ident.clone(), args))
                            .map_err(InvokeError::Downstream)
                    })
                    .and_then(|result| {
                        // Auto move into auth_zone
                        for (proof_id, _) in &result.proof_ids {
                            let proof = Proof(*proof_id);
                            ComponentAuthZone::sys_push(proof, env)
                                .map_err(InvokeError::Downstream)?;
                        }
                        // Auto move into worktop
                        for (bucket_id, _) in &result.bucket_ids {
                            Worktop::sys_put(Bucket(*bucket_id), env)
                                .map_err(InvokeError::Downstream)?;
                        }
                        Ok(result)
                    })
                }
                Instruction::CallMethod { method_ident, args } => {
                    Self::replace_ids(
                        &mut proof_id_mapping,
                        &mut bucket_id_mapping,
                        IndexedScryptoValue::from_slice(args)
                            .expect("Invalid CALL_METHOD arguments"),
                    )
                    .and_then(|args| Self::process_expressions(args, env))
                    .and_then(|args| {
                        env.invoke(ScryptoInvocation::Method(method_ident.clone(), args))
                            .map_err(InvokeError::Downstream)
                    })
                    .and_then(|result| {
                        // Auto move into auth_zone
                        for (proof_id, _) in &result.proof_ids {
                            let proof = Proof(*proof_id);
                            ComponentAuthZone::sys_push(proof, env)
                                .map_err(InvokeError::Downstream)?;
                        }
                        // Auto move into worktop
                        for (bucket_id, _) in &result.bucket_ids {
                            Worktop::sys_put(Bucket(*bucket_id), env)
                                .map_err(InvokeError::downstream)?;
                        }
                        Ok(result)
                    })
                }
                Instruction::PublishPackage { code, abi } => env
                    .sys_invoke(PackagePublishNoOwnerInvocation {
                        code: code.clone(),
                        abi: abi.clone(),
                        metadata: HashMap::new(),
                    })
                    .map(|address| IndexedScryptoValue::from_typed(&address))
                    .map_err(InvokeError::Downstream),
                Instruction::CallNativeFunction {
                    function_ident,
                    args,
                } => {
                    Self::replace_ids(
                        &mut proof_id_mapping,
                        &mut bucket_id_mapping,
                        IndexedScryptoValue::from_slice(args)
                            .expect("Invalid CALL_NATIVE_FUNCTION arguments"),
                    )
                    .and_then(|args| Self::process_expressions(args, env))
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
                            env,
                        )
                        .map_err(InvokeError::Downstream)
                    })
                    .and_then(|result| {
                        // Auto move into auth_zone
                        for (proof_id, _) in &result.proof_ids {
                            let proof = Proof(*proof_id);
                            ComponentAuthZone::sys_push(proof, env)
                                .map_err(InvokeError::Downstream)?;
                        }
                        // Auto move into worktop
                        for (bucket_id, _) in &result.bucket_ids {
                            Worktop::sys_put(Bucket(*bucket_id), env)
                                .map_err(InvokeError::Downstream)?;
                        }
                        Ok(result)
                    })
                }
                Instruction::CallNativeMethod { method_ident, args } => {
                    Self::replace_ids(
                        &mut proof_id_mapping,
                        &mut bucket_id_mapping,
                        IndexedScryptoValue::from_slice(args)
                            .expect("Invalid CALL_NATIVE_METHOD arguments"),
                    )
                    .and_then(|args| Self::process_expressions(args, env))
                    .and_then(|args| {
                        let native_method =
                            resolve_native_method(method_ident.receiver, &method_ident.method_name)
                                .ok_or(InvokeError::Error(
                                    TransactionProcessorError::NativeMethodNotFound(
                                        method_ident.clone(),
                                    ),
                                ))?;

                        parse_and_invoke_native_fn(NativeFn::Method(native_method), args.raw, env)
                            .map_err(InvokeError::Downstream)
                    })
                    .and_then(|result| {
                        // Auto move into auth_zone
                        for (proof_id, _) in &result.proof_ids {
                            let proof = Proof(*proof_id);
                            ComponentAuthZone::sys_push(proof, env)
                                .map_err(InvokeError::Downstream)?;
                        }
                        // Auto move into worktop
                        for (bucket_id, _) in &result.bucket_ids {
                            Worktop::sys_put(Bucket(*bucket_id), env)
                                .map_err(InvokeError::downstream)?;
                        }
                        Ok(result)
                    })
                }
            }?;
            outputs.push(result);

            env.emit_event(Event::Runtime(RuntimeEvent::PostExecuteInstruction {
                instruction_index: idx,
                instruction: &inst,
            }))
            .map_err(InvokeError::Downstream)?;
        }

        env.emit_event(Event::Runtime(RuntimeEvent::PostExecuteManifest))
            .map_err(InvokeError::Downstream)?;

        Ok(outputs
            .into_iter()
            .map(|sv| sv.raw)
            .collect::<Vec<Vec<u8>>>())
    }
}
