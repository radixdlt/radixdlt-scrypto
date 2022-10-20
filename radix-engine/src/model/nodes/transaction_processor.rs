use scrypto::resource::AuthZoneDrainInput;
use transaction::errors::IdAllocationError;
use transaction::model::*;
use transaction::validation::*;

use crate::engine::{RENode, SystemApi};
use crate::fee::FeeReserve;
use crate::model::resolve_native_function;
use crate::model::resolve_native_method;
use crate::model::{InvokeError, WorktopSubstate};
use crate::model::{
    WorktopAssertContainsAmountInput, WorktopAssertContainsInput,
    WorktopAssertContainsNonFungiblesInput, WorktopDrainInput, WorktopPutInput,
    WorktopTakeAllInput, WorktopTakeAmountInput, WorktopTakeNonFungiblesInput,
};
use crate::types::*;

#[derive(Debug, TypeId, Encode, Decode)]
pub struct TransactionProcessorRunInput {
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
pub enum TransactionProcessorError {
    InvalidRequestData(DecodeError),
    InvalidMethod,
    BucketNotFound(BucketId),
    ProofNotFound(ProofId),
    NativeFunctionNotFound(NativeFunctionIdent),
    NativeMethodNotFound(NativeMethodIdent),
    IdAllocationError(IdAllocationError),
}

pub struct TransactionProcessor {}

impl TransactionProcessor {
    fn replace_node_id(
        node_id: RENodeId,
        proof_id_mapping: &mut HashMap<ProofId, ProofId>,
        bucket_id_mapping: &mut HashMap<BucketId, BucketId>,
    ) -> Result<RENodeId, InvokeError<TransactionProcessorError>> {
        match node_id {
            RENodeId::Bucket(bucket_id) => bucket_id_mapping
                .get(&bucket_id)
                .cloned()
                .map(RENodeId::Bucket)
                .ok_or(InvokeError::Error(
                    TransactionProcessorError::BucketNotFound(bucket_id),
                )),
            RENodeId::Proof(proof_id) => proof_id_mapping
                .get(&proof_id)
                .cloned()
                .map(RENodeId::Proof)
                .ok_or(InvokeError::Error(
                    TransactionProcessorError::ProofNotFound(proof_id),
                )),
            _ => Ok(node_id),
        }
    }

    fn replace_receiver(
        receiver: Receiver,
        proof_id_mapping: &mut HashMap<ProofId, ProofId>,
        bucket_id_mapping: &mut HashMap<BucketId, BucketId>,
    ) -> Result<Receiver, InvokeError<TransactionProcessorError>> {
        let receiver = match receiver {
            Receiver::Ref(node_id) => Receiver::Ref(Self::replace_node_id(
                node_id,
                proof_id_mapping,
                bucket_id_mapping,
            )?),
            Receiver::Consumed(node_id) => Receiver::Consumed(Self::replace_node_id(
                node_id,
                proof_id_mapping,
                bucket_id_mapping,
            )?),
        };

        Ok(receiver)
    }

    fn replace_ids(
        proof_id_mapping: &mut HashMap<ProofId, ProofId>,
        bucket_id_mapping: &mut HashMap<BucketId, BucketId>,
        mut value: ScryptoValue,
    ) -> Result<ScryptoValue, InvokeError<TransactionProcessorError>> {
        value
            .replace_ids(proof_id_mapping, bucket_id_mapping)
            .map_err(|e| match e {
                ScryptoValueReplaceError::BucketIdNotFound(bucket_id) => {
                    InvokeError::Error(TransactionProcessorError::BucketNotFound(bucket_id))
                }
                ScryptoValueReplaceError::ProofIdNotFound(proof_id) => {
                    InvokeError::Error(TransactionProcessorError::ProofNotFound(proof_id))
                }
            })?;
        Ok(value)
    }

    fn process_expressions<'s, Y, R>(
        args: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, InvokeError<TransactionProcessorError>>
    where
        Y: SystemApi<'s, R>,
        R: FeeReserve,
    {
        let mut value = args.dom;
        for (expression, path) in args.expressions {
            match expression.0.as_str() {
                "ENTIRE_WORKTOP" => {
                    let buckets = system_api
                        .invoke_native(NativeInvocation::Method(
                            NativeMethod::Worktop(WorktopMethod::Drain),
                            Receiver::Ref(RENodeId::Worktop),
                            ScryptoValue::from_typed(&WorktopDrainInput {}),
                        ))
                        .map_err(InvokeError::Downstream)
                        .map(|result| {
                            let mut buckets = Vec::new();
                            for (bucket_id, _) in result.bucket_ids {
                                buckets.push(scrypto::resource::Bucket(bucket_id));
                            }
                            buckets
                        })?;

                    let val = path
                        .get_from_value_mut(&mut value)
                        .expect("Failed to locate an expression value using SBOR path");
                    *val =
                        decode_any(&scrypto_encode(&buckets)).expect("Failed to decode Vec<Bucket>")
                }
                "ENTIRE_AUTH_ZONE" => {
                    let node_ids = system_api
                        .get_visible_node_ids()
                        .map_err(InvokeError::Downstream)?;
                    let auth_zone_node_id = node_ids
                        .into_iter()
                        .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
                        .expect("AuthZone does not exist");

                    let proofs = system_api
                        .invoke_native(NativeInvocation::Method(
                            NativeMethod::AuthZone(AuthZoneMethod::Drain),
                            Receiver::Ref(auth_zone_node_id),
                            ScryptoValue::from_typed(&AuthZoneDrainInput {}),
                        ))
                        .map_err(InvokeError::Downstream)
                        .map(|result| {
                            let mut proofs = Vec::new();
                            for (proof_id, _) in result.proof_ids {
                                proofs.push(scrypto::resource::Proof(proof_id));
                            }
                            proofs
                        })?;

                    let val = path
                        .get_from_value_mut(&mut value)
                        .expect("Failed to locate an expression value using SBOR path");
                    *val =
                        decode_any(&scrypto_encode(&proofs)).expect("Failed to decode Vec<Proof>")
                }
                _ => {} // no-op
            }
        }

        Ok(ScryptoValue::from_value(value)
            .expect("Value became invalid post expression transformation"))
    }

    fn first_bucket(value: &ScryptoValue) -> BucketId {
        *value
            .bucket_ids
            .iter()
            .next()
            .expect("No bucket found in value")
            .0
    }

    fn first_proof(value: &ScryptoValue) -> ProofId {
        *value
            .proof_ids
            .iter()
            .next()
            .expect("No proof found in value")
            .0
    }

    pub fn static_main<'s, Y, R>(
        func: TransactionProcessorFunction,
        args: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, InvokeError<TransactionProcessorError>>
    where
        Y: SystemApi<'s, R>,
        R: FeeReserve,
    {
        match func {
            TransactionProcessorFunction::Run => {
                let input: TransactionProcessorRunInput =
                    scrypto_decode(&args.raw).map_err(|e| {
                        InvokeError::Error(TransactionProcessorError::InvalidRequestData(e))
                    })?;

                let mut proof_id_mapping = HashMap::new();
                let mut bucket_id_mapping = HashMap::new();
                let mut outputs = Vec::new();
                let mut id_allocator = IdAllocator::new(IdSpace::Transaction);

                let _worktop_id = system_api
                    .create_node(RENode::Worktop(WorktopSubstate::new()))
                    .map_err(InvokeError::Downstream)?;

                let owned_node_ids = system_api
                    .get_visible_node_ids()
                    .map_err(InvokeError::Downstream)?;
                let auth_zone_node_id = owned_node_ids
                    .into_iter()
                    .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
                    .expect("AuthZone does not exist");
                let auth_zone_ref = Receiver::Ref(auth_zone_node_id);

                for inst in &input.instructions {
                    let result = match inst {
                        Instruction::TakeFromWorktop { resource_address } => id_allocator
                            .new_bucket_id()
                            .map_err(|e| {
                                InvokeError::Error(TransactionProcessorError::IdAllocationError(e))
                            })
                            .and_then(|new_id| {
                                system_api
                                    .invoke_native(NativeInvocation::Method(
                                        NativeMethod::Worktop(WorktopMethod::TakeAll),
                                        Receiver::Ref(RENodeId::Worktop),
                                        ScryptoValue::from_typed(&WorktopTakeAllInput {
                                            resource_address: *resource_address,
                                        }),
                                    ))
                                    .map_err(InvokeError::Downstream)
                                    .map(|rtn| {
                                        let bucket_id = Self::first_bucket(&rtn);
                                        bucket_id_mapping.insert(new_id, bucket_id);
                                        ScryptoValue::from_typed(&scrypto::resource::Bucket(new_id))
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
                                system_api
                                    .invoke_native(NativeInvocation::Method(
                                        NativeMethod::Worktop(WorktopMethod::TakeAmount),
                                        Receiver::Ref(RENodeId::Worktop),
                                        ScryptoValue::from_typed(&WorktopTakeAmountInput {
                                            amount: *amount,
                                            resource_address: *resource_address,
                                        }),
                                    ))
                                    .map_err(InvokeError::Downstream)
                                    .map(|rtn| {
                                        let bucket_id = Self::first_bucket(&rtn);
                                        bucket_id_mapping.insert(new_id, bucket_id);
                                        ScryptoValue::from_typed(&scrypto::resource::Bucket(new_id))
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
                                system_api
                                    .invoke_native(NativeInvocation::Method(
                                        NativeMethod::Worktop(WorktopMethod::TakeNonFungibles),
                                        Receiver::Ref(RENodeId::Worktop),
                                        ScryptoValue::from_typed(&WorktopTakeNonFungiblesInput {
                                            ids: ids.clone(),
                                            resource_address: *resource_address,
                                        }),
                                    ))
                                    .map_err(InvokeError::Downstream)
                                    .map(|rtn| {
                                        let bucket_id = Self::first_bucket(&rtn);
                                        bucket_id_mapping.insert(new_id, bucket_id);
                                        ScryptoValue::from_typed(&scrypto::resource::Bucket(new_id))
                                    })
                            }),
                        Instruction::ReturnToWorktop { bucket_id } => bucket_id_mapping
                            .remove(bucket_id)
                            .map(|real_id| {
                                system_api
                                    .invoke_native(NativeInvocation::Method(
                                        NativeMethod::Worktop(WorktopMethod::Put),
                                        Receiver::Ref(RENodeId::Worktop),
                                        ScryptoValue::from_typed(&WorktopPutInput {
                                            bucket: scrypto::resource::Bucket(real_id),
                                        }),
                                    ))
                                    .map_err(InvokeError::Downstream)
                            })
                            .unwrap_or(Err(InvokeError::Error(
                                TransactionProcessorError::BucketNotFound(*bucket_id),
                            ))),
                        Instruction::AssertWorktopContains { resource_address } => system_api
                            .invoke_native(NativeInvocation::Method(
                                NativeMethod::Worktop(WorktopMethod::AssertContains),
                                Receiver::Ref(RENodeId::Worktop),
                                ScryptoValue::from_typed(&WorktopAssertContainsInput {
                                    resource_address: *resource_address,
                                }),
                            ))
                            .map_err(InvokeError::Downstream),
                        Instruction::AssertWorktopContainsByAmount {
                            amount,
                            resource_address,
                        } => system_api
                            .invoke_native(NativeInvocation::Method(
                                NativeMethod::Worktop(WorktopMethod::AssertContainsAmount),
                                Receiver::Ref(RENodeId::Worktop),
                                ScryptoValue::from_typed(&WorktopAssertContainsAmountInput {
                                    amount: *amount,
                                    resource_address: *resource_address,
                                }),
                            ))
                            .map_err(InvokeError::Downstream),
                        Instruction::AssertWorktopContainsByIds {
                            ids,
                            resource_address,
                        } => system_api
                            .invoke_native(NativeInvocation::Method(
                                NativeMethod::Worktop(WorktopMethod::AssertContainsNonFungibles),
                                Receiver::Ref(RENodeId::Worktop),
                                ScryptoValue::from_typed(&WorktopAssertContainsNonFungiblesInput {
                                    ids: ids.clone(),
                                    resource_address: *resource_address,
                                }),
                            ))
                            .map_err(InvokeError::Downstream),

                        Instruction::PopFromAuthZone {} => id_allocator
                            .new_proof_id()
                            .map_err(|e| {
                                InvokeError::Error(TransactionProcessorError::IdAllocationError(e))
                            })
                            .and_then(|new_id| {
                                system_api
                                    .invoke_native(NativeInvocation::Method(
                                        NativeMethod::AuthZone(AuthZoneMethod::Pop),
                                        auth_zone_ref,
                                        ScryptoValue::from_typed(&AuthZonePopInput {}),
                                    ))
                                    .map_err(InvokeError::Downstream)
                                    .map(|rtn| {
                                        let proof_id = Self::first_proof(&rtn);
                                        proof_id_mapping.insert(new_id, proof_id);
                                        ScryptoValue::from_typed(&scrypto::resource::Proof(new_id))
                                    })
                            }),
                        Instruction::ClearAuthZone => {
                            proof_id_mapping.clear();
                            system_api
                                .invoke_native(NativeInvocation::Method(
                                    NativeMethod::AuthZone(AuthZoneMethod::Clear),
                                    auth_zone_ref,
                                    ScryptoValue::from_typed(&AuthZoneClearInput {}),
                                ))
                                .map_err(InvokeError::Downstream)
                        }
                        Instruction::PushToAuthZone { proof_id } => proof_id_mapping
                            .remove(proof_id)
                            .ok_or(InvokeError::Error(
                                TransactionProcessorError::ProofNotFound(*proof_id),
                            ))
                            .and_then(|real_id| {
                                system_api
                                    .invoke_native(NativeInvocation::Method(
                                        NativeMethod::AuthZone(AuthZoneMethod::Push),
                                        auth_zone_ref,
                                        ScryptoValue::from_typed(&AuthZonePushInput {
                                            proof: scrypto::resource::Proof(real_id),
                                        }),
                                    ))
                                    .map_err(InvokeError::Downstream)
                            }),
                        Instruction::CreateProofFromAuthZone { resource_address } => id_allocator
                            .new_proof_id()
                            .map_err(|e| {
                                InvokeError::Error(TransactionProcessorError::IdAllocationError(e))
                            })
                            .and_then(|new_id| {
                                system_api
                                    .invoke_native(NativeInvocation::Method(
                                        NativeMethod::AuthZone(AuthZoneMethod::CreateProof),
                                        auth_zone_ref,
                                        ScryptoValue::from_typed(&AuthZoneCreateProofInput {
                                            resource_address: *resource_address,
                                        }),
                                    ))
                                    .map_err(InvokeError::Downstream)
                                    .map(|rtn| {
                                        let proof_id = Self::first_proof(&rtn);
                                        proof_id_mapping.insert(new_id, proof_id);
                                        ScryptoValue::from_typed(&scrypto::resource::Proof(new_id))
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
                                system_api
                                    .invoke_native(NativeInvocation::Method(
                                        NativeMethod::AuthZone(AuthZoneMethod::CreateProofByAmount),
                                        auth_zone_ref,
                                        ScryptoValue::from_typed(
                                            &AuthZoneCreateProofByAmountInput {
                                                amount: *amount,
                                                resource_address: *resource_address,
                                            },
                                        ),
                                    ))
                                    .map_err(InvokeError::Downstream)
                                    .map(|rtn| {
                                        let proof_id = Self::first_proof(&rtn);
                                        proof_id_mapping.insert(new_id, proof_id);
                                        ScryptoValue::from_typed(&scrypto::resource::Proof(new_id))
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
                                system_api
                                    .invoke_native(NativeInvocation::Method(
                                        NativeMethod::AuthZone(AuthZoneMethod::CreateProofByIds),
                                        auth_zone_ref,
                                        ScryptoValue::from_typed(&AuthZoneCreateProofByIdsInput {
                                            ids: ids.clone(),
                                            resource_address: *resource_address,
                                        }),
                                    ))
                                    .map_err(InvokeError::Downstream)
                                    .map(|rtn| {
                                        let proof_id = Self::first_proof(&rtn);
                                        proof_id_mapping.insert(new_id, proof_id);
                                        ScryptoValue::from_typed(&scrypto::resource::Proof(new_id))
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
                                system_api
                                    .invoke_native(NativeInvocation::Method(
                                        NativeMethod::Bucket(BucketMethod::CreateProof),
                                        Receiver::Ref(RENodeId::Bucket(real_bucket_id)),
                                        ScryptoValue::from_typed(&BucketCreateProofInput {}),
                                    ))
                                    .map_err(InvokeError::Downstream)
                                    .map(|rtn| {
                                        let proof_id = Self::first_proof(&rtn);
                                        proof_id_mapping.insert(new_id, proof_id);
                                        ScryptoValue::from_typed(&scrypto::resource::Proof(new_id))
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
                                        system_api
                                            .invoke_native(NativeInvocation::Method(
                                                NativeMethod::Proof(ProofMethod::Clone),
                                                Receiver::Ref(RENodeId::Proof(real_id)),
                                                ScryptoValue::from_typed(&ProofCloneInput {}),
                                            ))
                                            .map_err(InvokeError::Downstream)
                                            .map(|v| {
                                                let cloned_proof_id = Self::first_proof(&v);
                                                proof_id_mapping.insert(new_id, cloned_proof_id);
                                                ScryptoValue::from_typed(&scrypto::resource::Proof(
                                                    new_id,
                                                ))
                                            })
                                    })
                                    .unwrap_or(Err(InvokeError::Error(
                                        TransactionProcessorError::ProofNotFound(*proof_id),
                                    )))
                            }),
                        Instruction::DropProof { proof_id } => proof_id_mapping
                            .remove(proof_id)
                            .map(|real_id| {
                                system_api
                                    .invoke_native(NativeInvocation::Method(
                                        NativeMethod::Proof(ProofMethod::Drop),
                                        Receiver::Consumed(RENodeId::Proof(real_id)),
                                        ScryptoValue::from_typed(&ConsumingProofDropInput {}),
                                    ))
                                    .map_err(InvokeError::Downstream)
                            })
                            .unwrap_or(Err(InvokeError::Error(
                                TransactionProcessorError::ProofNotFound(*proof_id),
                            ))),
                        Instruction::DropAllProofs => {
                            for (_, real_id) in proof_id_mapping.drain() {
                                system_api
                                    .invoke_native(NativeInvocation::Method(
                                        NativeMethod::Proof(ProofMethod::Drop),
                                        Receiver::Consumed(RENodeId::Proof(real_id)),
                                        ScryptoValue::from_typed(&ConsumingProofDropInput {}),
                                    ))
                                    .map_err(InvokeError::Downstream)?;
                            }
                            system_api
                                .invoke_native(NativeInvocation::Method(
                                    NativeMethod::AuthZone(AuthZoneMethod::Clear),
                                    auth_zone_ref,
                                    ScryptoValue::from_typed(&AuthZoneClearInput {}),
                                ))
                                .map_err(InvokeError::Downstream)
                        }
                        Instruction::CallFunction {
                            function_ident,
                            args,
                        } => {
                            Self::replace_ids(
                                &mut proof_id_mapping,
                                &mut bucket_id_mapping,
                                ScryptoValue::from_slice(args)
                                    .expect("Invalid CALL_FUNCTION arguments"),
                            )
                            .and_then(|args| Self::process_expressions(args, system_api))
                            .and_then(|args| {
                                system_api
                                    .invoke_scrypto(ScryptoInvocation::Function(
                                        function_ident.clone(),
                                        args,
                                    ))
                                    .map_err(InvokeError::Downstream)
                            })
                            .and_then(|result| {
                                // Auto move into auth_zone
                                for (proof_id, _) in &result.proof_ids {
                                    system_api
                                        .invoke_native(NativeInvocation::Method(
                                            NativeMethod::AuthZone(AuthZoneMethod::Push),
                                            auth_zone_ref,
                                            ScryptoValue::from_typed(&AuthZonePushInput {
                                                proof: scrypto::resource::Proof(*proof_id),
                                            }),
                                        ))
                                        .map_err(InvokeError::Downstream)?;
                                }
                                // Auto move into worktop
                                for (bucket_id, _) in &result.bucket_ids {
                                    system_api
                                        .invoke_native(NativeInvocation::Method(
                                            NativeMethod::Worktop(WorktopMethod::Put),
                                            Receiver::Ref(RENodeId::Worktop),
                                            ScryptoValue::from_typed(&WorktopPutInput {
                                                bucket: scrypto::resource::Bucket(*bucket_id),
                                            }),
                                        ))
                                        .map_err(InvokeError::Downstream)?;
                                }
                                Ok(result)
                            })
                        }
                        Instruction::CallMethod { method_ident, args } => {
                            Self::replace_ids(
                                &mut proof_id_mapping,
                                &mut bucket_id_mapping,
                                ScryptoValue::from_slice(args)
                                    .expect("Invalid CALL_METHOD arguments"),
                            )
                            .and_then(|args| Self::process_expressions(args, system_api))
                            .and_then(|args| {
                                system_api
                                    .invoke_scrypto(ScryptoInvocation::Method(
                                        method_ident.clone(),
                                        args,
                                    ))
                                    .map_err(InvokeError::Downstream)
                            })
                            .and_then(|result| {
                                // Auto move into auth_zone
                                for (proof_id, _) in &result.proof_ids {
                                    system_api
                                        .invoke_native(NativeInvocation::Method(
                                            NativeMethod::AuthZone(AuthZoneMethod::Push),
                                            auth_zone_ref,
                                            ScryptoValue::from_typed(&AuthZonePushInput {
                                                proof: scrypto::resource::Proof(*proof_id),
                                            }),
                                        ))
                                        .map_err(InvokeError::Downstream)?;
                                }
                                // Auto move into worktop
                                for (bucket_id, _) in &result.bucket_ids {
                                    system_api
                                        .invoke_native(NativeInvocation::Method(
                                            NativeMethod::Worktop(WorktopMethod::Put),
                                            Receiver::Ref(RENodeId::Worktop),
                                            ScryptoValue::from_typed(&WorktopPutInput {
                                                bucket: scrypto::resource::Bucket(*bucket_id),
                                            }),
                                        ))
                                        .map_err(InvokeError::downstream)?;
                                }
                                Ok(result)
                            })
                        }
                        Instruction::PublishPackage { code, abi } => system_api
                            .invoke_native(NativeInvocation::Function(
                                NativeFunction::Package(PackageFunction::Publish),
                                ScryptoValue::from_typed(&PackagePublishInput {
                                    code: code.clone(),
                                    abi: abi.clone(),
                                }),
                            ))
                            .map_err(InvokeError::Downstream),
                        Instruction::CallNativeFunction {
                            function_ident,
                            args,
                        } => {
                            Self::replace_ids(
                                &mut proof_id_mapping,
                                &mut bucket_id_mapping,
                                ScryptoValue::from_slice(args)
                                    .expect("Invalid CALL_NATIVE_FUNCTION arguments"),
                            )
                            .and_then(|args| Self::process_expressions(args, system_api))
                            .and_then(|args| {
                                system_api
                                    .invoke_native(NativeInvocation::Function(
                                        resolve_native_function(
                                            &function_ident.blueprint_name,
                                            &function_ident.function_name,
                                        )
                                        .ok_or(
                                            InvokeError::Error(
                                                TransactionProcessorError::NativeFunctionNotFound(
                                                    function_ident.clone(),
                                                ),
                                            ),
                                        )?,
                                        args,
                                    ))
                                    .map_err(InvokeError::Downstream)
                            })
                            .and_then(|result| {
                                // Auto move into auth_zone
                                for (proof_id, _) in &result.proof_ids {
                                    system_api
                                        .invoke_native(NativeInvocation::Method(
                                            NativeMethod::AuthZone(AuthZoneMethod::Push),
                                            auth_zone_ref,
                                            ScryptoValue::from_typed(&AuthZonePushInput {
                                                proof: scrypto::resource::Proof(*proof_id),
                                            }),
                                        ))
                                        .map_err(InvokeError::Downstream)?;
                                }
                                // Auto move into worktop
                                for (bucket_id, _) in &result.bucket_ids {
                                    system_api
                                        .invoke_native(NativeInvocation::Method(
                                            NativeMethod::Worktop(WorktopMethod::Put),
                                            Receiver::Ref(RENodeId::Worktop),
                                            ScryptoValue::from_typed(&WorktopPutInput {
                                                bucket: scrypto::resource::Bucket(*bucket_id),
                                            }),
                                        ))
                                        .map_err(InvokeError::Downstream)?;
                                }
                                Ok(result)
                            })
                        }
                        Instruction::CallNativeMethod { method_ident, args } => {
                            Self::replace_ids(
                                &mut proof_id_mapping,
                                &mut bucket_id_mapping,
                                ScryptoValue::from_slice(args)
                                    .expect("Invalid CALL_NATIVE_METHOD arguments"),
                            )
                            .and_then(|args| Self::process_expressions(args, system_api))
                            .and_then(|args| {
                                system_api
                                    .invoke_native(NativeInvocation::Method(
                                        resolve_native_method(
                                            &method_ident.receiver,
                                            &method_ident.method_name,
                                        )
                                        .ok_or(
                                            InvokeError::Error(
                                                TransactionProcessorError::NativeMethodNotFound(
                                                    method_ident.clone(),
                                                ),
                                            ),
                                        )?,
                                        Self::replace_receiver(
                                            method_ident.receiver.clone(),
                                            &mut proof_id_mapping,
                                            &mut bucket_id_mapping,
                                        )?,
                                        args,
                                    ))
                                    .map_err(InvokeError::Downstream)
                            })
                            .and_then(|result| {
                                // Auto move into auth_zone
                                for (proof_id, _) in &result.proof_ids {
                                    system_api
                                        .invoke_native(NativeInvocation::Method(
                                            NativeMethod::AuthZone(AuthZoneMethod::Push),
                                            auth_zone_ref,
                                            ScryptoValue::from_typed(&AuthZonePushInput {
                                                proof: scrypto::resource::Proof(*proof_id),
                                            }),
                                        ))
                                        .map_err(InvokeError::Downstream)?;
                                }
                                // Auto move into worktop
                                for (bucket_id, _) in &result.bucket_ids {
                                    system_api
                                        .invoke_native(NativeInvocation::Method(
                                            NativeMethod::Worktop(WorktopMethod::Put),
                                            Receiver::Ref(RENodeId::Worktop),
                                            ScryptoValue::from_typed(&WorktopPutInput {
                                                bucket: scrypto::resource::Bucket(*bucket_id),
                                            }),
                                        ))
                                        .map_err(InvokeError::downstream)?;
                                }
                                Ok(result)
                            })
                        }
                    }?;
                    outputs.push(result);
                }

                Ok(ScryptoValue::from_typed(
                    &outputs
                        .into_iter()
                        .map(|sv| sv.raw)
                        .collect::<Vec<Vec<u8>>>(),
                ))
            }
        }
    }
}
