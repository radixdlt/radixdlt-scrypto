use transaction::errors::IdAllocationError;
use transaction::model::*;
use transaction::validation::*;

use crate::engine::{HeapRENode, SystemApi};
use crate::fee::FeeReserve;
use crate::model::worktop::{
    WorktopAssertContainsAmountInput, WorktopAssertContainsInput,
    WorktopAssertContainsNonFungiblesInput, WorktopDrainInput, WorktopPutInput,
    WorktopTakeAllInput, WorktopTakeAmountInput, WorktopTakeNonFungiblesInput,
};
use crate::model::InvokeError;
use crate::types::*;
use crate::wasm::*;

use super::Worktop;

#[derive(Debug, TypeId, Encode, Decode)]
pub struct TransactionProcessorRunInput {
    pub instructions: Vec<ExecutableInstruction>,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub enum TransactionProcessorError {
    InvalidRequestData(DecodeError),
    InvalidMethod,
    BucketNotFound(BucketId),
    ProofNotFound(ProofId),
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
            Receiver::CurrentAuthZone => Receiver::CurrentAuthZone,
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

    fn process_expressions<'s, Y, W, I, R>(
        args: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, InvokeError<TransactionProcessorError>>
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
    {
        let mut value = args.dom;
        for (expression, path) in args.expressions {
            match expression.0.as_str() {
                "ENTIRE_WORKTOP" => {
                    let buckets = system_api
                        .invoke_method(
                            Receiver::Ref(RENodeId::Worktop),
                            FnIdentifier::Native(NativeFnIdentifier::Worktop(
                                WorktopFnIdentifier::Drain,
                            )),
                            ScryptoValue::from_typed(&WorktopDrainInput {}),
                        )
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
                    let auth_zone = system_api.auth_zone(1);
                    let proofs = auth_zone.drain();
                    let node_ids: Result<Vec<RENodeId>, InvokeError<TransactionProcessorError>> =
                        proofs
                            .into_iter()
                            .map(|proof| {
                                system_api
                                    .node_create(HeapRENode::Proof(proof))
                                    .map_err(InvokeError::Downstream)
                            })
                            .collect();

                    let mut proofs = Vec::new();
                    for node_id in node_ids? {
                        let proof_id: ProofId = node_id.into();
                        proofs.push(scrypto::resource::Proof(proof_id));
                    }

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

    pub fn static_main<'s, Y, W, I, R>(
        transaction_processor_fn: TransactionProcessorFnIdentifier,
        call_data: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, InvokeError<TransactionProcessorError>>
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
    {
        match transaction_processor_fn {
            TransactionProcessorFnIdentifier::Run => {
                let input: TransactionProcessorRunInput =
                    scrypto_decode(&call_data.raw).map_err(|e| {
                        InvokeError::Error(TransactionProcessorError::InvalidRequestData(e))
                    })?;

                let mut proof_id_mapping = HashMap::new();
                let mut bucket_id_mapping = HashMap::new();
                let mut outputs = Vec::new();
                let mut id_allocator = IdAllocator::new(IdSpace::Transaction);

                let _worktop_id = system_api
                    .node_create(HeapRENode::Worktop(Worktop::new()))
                    .map_err(InvokeError::Downstream)?;

                for inst in &input.instructions.clone() {
                    let result = match inst {
                        ExecutableInstruction::TakeFromWorktop { resource_address } => id_allocator
                            .new_bucket_id()
                            .map_err(|e| {
                                InvokeError::Error(TransactionProcessorError::IdAllocationError(e))
                            })
                            .and_then(|new_id| {
                                system_api
                                    .invoke_method(
                                        Receiver::Ref(RENodeId::Worktop),
                                        FnIdentifier::Native(NativeFnIdentifier::Worktop(
                                            WorktopFnIdentifier::TakeAll,
                                        )),
                                        ScryptoValue::from_typed(&WorktopTakeAllInput {
                                            resource_address: *resource_address,
                                        }),
                                    )
                                    .map_err(InvokeError::Downstream)
                                    .map(|rtn| {
                                        let bucket_id = Self::first_bucket(&rtn);
                                        bucket_id_mapping.insert(new_id, bucket_id);
                                        ScryptoValue::from_typed(&scrypto::resource::Bucket(new_id))
                                    })
                            }),
                        ExecutableInstruction::TakeFromWorktopByAmount {
                            amount,
                            resource_address,
                        } => id_allocator
                            .new_bucket_id()
                            .map_err(|e| {
                                InvokeError::Error(TransactionProcessorError::IdAllocationError(e))
                            })
                            .and_then(|new_id| {
                                system_api
                                    .invoke_method(
                                        Receiver::Ref(RENodeId::Worktop),
                                        FnIdentifier::Native(NativeFnIdentifier::Worktop(
                                            WorktopFnIdentifier::TakeAmount,
                                        )),
                                        ScryptoValue::from_typed(&WorktopTakeAmountInput {
                                            amount: *amount,
                                            resource_address: *resource_address,
                                        }),
                                    )
                                    .map_err(InvokeError::Downstream)
                                    .map(|rtn| {
                                        let bucket_id = Self::first_bucket(&rtn);
                                        bucket_id_mapping.insert(new_id, bucket_id);
                                        ScryptoValue::from_typed(&scrypto::resource::Bucket(new_id))
                                    })
                            }),
                        ExecutableInstruction::TakeFromWorktopByIds {
                            ids,
                            resource_address,
                        } => id_allocator
                            .new_bucket_id()
                            .map_err(|e| {
                                InvokeError::Error(TransactionProcessorError::IdAllocationError(e))
                            })
                            .and_then(|new_id| {
                                system_api
                                    .invoke_method(
                                        Receiver::Ref(RENodeId::Worktop),
                                        FnIdentifier::Native(NativeFnIdentifier::Worktop(
                                            WorktopFnIdentifier::TakeNonFungibles,
                                        )),
                                        ScryptoValue::from_typed(&WorktopTakeNonFungiblesInput {
                                            ids: ids.clone(),
                                            resource_address: *resource_address,
                                        }),
                                    )
                                    .map_err(InvokeError::Downstream)
                                    .map(|rtn| {
                                        let bucket_id = Self::first_bucket(&rtn);
                                        bucket_id_mapping.insert(new_id, bucket_id);
                                        ScryptoValue::from_typed(&scrypto::resource::Bucket(new_id))
                                    })
                            }),
                        ExecutableInstruction::ReturnToWorktop { bucket_id } => bucket_id_mapping
                            .remove(bucket_id)
                            .map(|real_id| {
                                system_api
                                    .invoke_method(
                                        Receiver::Ref(RENodeId::Worktop),
                                        FnIdentifier::Native(NativeFnIdentifier::Worktop(
                                            WorktopFnIdentifier::Put,
                                        )),
                                        ScryptoValue::from_typed(&WorktopPutInput {
                                            bucket: scrypto::resource::Bucket(real_id),
                                        }),
                                    )
                                    .map_err(InvokeError::Downstream)
                            })
                            .unwrap_or(Err(InvokeError::Error(
                                TransactionProcessorError::BucketNotFound(*bucket_id),
                            ))),
                        ExecutableInstruction::AssertWorktopContains { resource_address } => {
                            system_api
                                .invoke_method(
                                    Receiver::Ref(RENodeId::Worktop),
                                    FnIdentifier::Native(NativeFnIdentifier::Worktop(
                                        WorktopFnIdentifier::AssertContains,
                                    )),
                                    ScryptoValue::from_typed(&WorktopAssertContainsInput {
                                        resource_address: *resource_address,
                                    }),
                                )
                                .map_err(InvokeError::Downstream)
                        }
                        ExecutableInstruction::AssertWorktopContainsByAmount {
                            amount,
                            resource_address,
                        } => system_api
                            .invoke_method(
                                Receiver::Ref(RENodeId::Worktop),
                                FnIdentifier::Native(NativeFnIdentifier::Worktop(
                                    WorktopFnIdentifier::AssertContainsAmount,
                                )),
                                ScryptoValue::from_typed(&WorktopAssertContainsAmountInput {
                                    amount: *amount,
                                    resource_address: *resource_address,
                                }),
                            )
                            .map_err(InvokeError::Downstream),
                        ExecutableInstruction::AssertWorktopContainsByIds {
                            ids,
                            resource_address,
                        } => system_api
                            .invoke_method(
                                Receiver::Ref(RENodeId::Worktop),
                                FnIdentifier::Native(NativeFnIdentifier::Worktop(
                                    WorktopFnIdentifier::AssertContainsNonFungibles,
                                )),
                                ScryptoValue::from_typed(&WorktopAssertContainsNonFungiblesInput {
                                    ids: ids.clone(),
                                    resource_address: *resource_address,
                                }),
                            )
                            .map_err(InvokeError::Downstream),

                        ExecutableInstruction::PopFromAuthZone {} => id_allocator
                            .new_proof_id()
                            .map_err(|e| {
                                InvokeError::Error(TransactionProcessorError::IdAllocationError(e))
                            })
                            .and_then(|new_id| {
                                system_api
                                    .invoke_method(
                                        Receiver::CurrentAuthZone,
                                        FnIdentifier::Native(NativeFnIdentifier::AuthZone(
                                            AuthZoneFnIdentifier::Pop,
                                        )),
                                        ScryptoValue::from_typed(&AuthZonePopInput {}),
                                    )
                                    .map_err(InvokeError::Downstream)
                                    .map(|rtn| {
                                        let proof_id = Self::first_proof(&rtn);
                                        proof_id_mapping.insert(new_id, proof_id);
                                        ScryptoValue::from_typed(&scrypto::resource::Proof(new_id))
                                    })
                            }),
                        ExecutableInstruction::ClearAuthZone => {
                            proof_id_mapping.clear();
                            system_api
                                .invoke_method(
                                    Receiver::CurrentAuthZone,
                                    FnIdentifier::Native(NativeFnIdentifier::AuthZone(
                                        AuthZoneFnIdentifier::Clear,
                                    )),
                                    ScryptoValue::from_typed(&AuthZoneClearInput {}),
                                )
                                .map_err(InvokeError::Downstream)
                        }
                        ExecutableInstruction::PushToAuthZone { proof_id } => proof_id_mapping
                            .remove(proof_id)
                            .ok_or(InvokeError::Error(
                                TransactionProcessorError::ProofNotFound(*proof_id),
                            ))
                            .and_then(|real_id| {
                                system_api
                                    .invoke_method(
                                        Receiver::CurrentAuthZone,
                                        FnIdentifier::Native(NativeFnIdentifier::AuthZone(
                                            AuthZoneFnIdentifier::Push,
                                        )),
                                        ScryptoValue::from_typed(&AuthZonePushInput {
                                            proof: scrypto::resource::Proof(real_id),
                                        }),
                                    )
                                    .map_err(InvokeError::Downstream)
                            }),
                        ExecutableInstruction::CreateProofFromAuthZone { resource_address } => {
                            id_allocator
                                .new_proof_id()
                                .map_err(|e| {
                                    InvokeError::Error(
                                        TransactionProcessorError::IdAllocationError(e),
                                    )
                                })
                                .and_then(|new_id| {
                                    system_api
                                        .invoke_method(
                                            Receiver::CurrentAuthZone,
                                            FnIdentifier::Native(NativeFnIdentifier::AuthZone(
                                                AuthZoneFnIdentifier::CreateProof,
                                            )),
                                            ScryptoValue::from_typed(&AuthZoneCreateProofInput {
                                                resource_address: *resource_address,
                                            }),
                                        )
                                        .map_err(InvokeError::Downstream)
                                        .map(|rtn| {
                                            let proof_id = Self::first_proof(&rtn);
                                            proof_id_mapping.insert(new_id, proof_id);
                                            ScryptoValue::from_typed(&scrypto::resource::Proof(
                                                new_id,
                                            ))
                                        })
                                })
                        }
                        ExecutableInstruction::CreateProofFromAuthZoneByAmount {
                            amount,
                            resource_address,
                        } => id_allocator
                            .new_proof_id()
                            .map_err(|e| {
                                InvokeError::Error(TransactionProcessorError::IdAllocationError(e))
                            })
                            .and_then(|new_id| {
                                system_api
                                    .invoke_method(
                                        Receiver::CurrentAuthZone,
                                        FnIdentifier::Native(NativeFnIdentifier::AuthZone(
                                            AuthZoneFnIdentifier::CreateProofByAmount,
                                        )),
                                        ScryptoValue::from_typed(
                                            &AuthZoneCreateProofByAmountInput {
                                                amount: *amount,
                                                resource_address: *resource_address,
                                            },
                                        ),
                                    )
                                    .map_err(InvokeError::Downstream)
                                    .map(|rtn| {
                                        let proof_id = Self::first_proof(&rtn);
                                        proof_id_mapping.insert(new_id, proof_id);
                                        ScryptoValue::from_typed(&scrypto::resource::Proof(new_id))
                                    })
                            }),
                        ExecutableInstruction::CreateProofFromAuthZoneByIds {
                            ids,
                            resource_address,
                        } => id_allocator
                            .new_proof_id()
                            .map_err(|e| {
                                InvokeError::Error(TransactionProcessorError::IdAllocationError(e))
                            })
                            .and_then(|new_id| {
                                system_api
                                    .invoke_method(
                                        Receiver::CurrentAuthZone,
                                        FnIdentifier::Native(NativeFnIdentifier::AuthZone(
                                            AuthZoneFnIdentifier::CreateProofByIds,
                                        )),
                                        ScryptoValue::from_typed(&AuthZoneCreateProofByIdsInput {
                                            ids: ids.clone(),
                                            resource_address: *resource_address,
                                        }),
                                    )
                                    .map_err(InvokeError::Downstream)
                                    .map(|rtn| {
                                        let proof_id = Self::first_proof(&rtn);
                                        proof_id_mapping.insert(new_id, proof_id);
                                        ScryptoValue::from_typed(&scrypto::resource::Proof(new_id))
                                    })
                            }),
                        ExecutableInstruction::CreateProofFromBucket { bucket_id } => id_allocator
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
                                    .invoke_method(
                                        Receiver::Ref(RENodeId::Bucket(real_bucket_id)),
                                        FnIdentifier::Native(NativeFnIdentifier::Bucket(
                                            BucketFnIdentifier::CreateProof,
                                        )),
                                        ScryptoValue::from_typed(&BucketCreateProofInput {}),
                                    )
                                    .map_err(InvokeError::Downstream)
                                    .map(|rtn| {
                                        let proof_id = Self::first_proof(&rtn);
                                        proof_id_mapping.insert(new_id, proof_id);
                                        ScryptoValue::from_typed(&scrypto::resource::Proof(new_id))
                                    })
                            }),
                        ExecutableInstruction::CloneProof { proof_id } => id_allocator
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
                                            .invoke_method(
                                                Receiver::Ref(RENodeId::Proof(real_id)),
                                                FnIdentifier::Native(NativeFnIdentifier::Proof(
                                                    ProofFnIdentifier::Clone,
                                                )),
                                                ScryptoValue::from_typed(&ProofCloneInput {}),
                                            )
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
                        ExecutableInstruction::DropProof { proof_id } => proof_id_mapping
                            .remove(proof_id)
                            .map(|real_id| {
                                system_api
                                    .invoke_method(
                                        Receiver::Consumed(RENodeId::Proof(real_id)),
                                        FnIdentifier::Native(NativeFnIdentifier::Proof(
                                            ProofFnIdentifier::Drop,
                                        )),
                                        ScryptoValue::from_typed(&ConsumingProofDropInput {}),
                                    )
                                    .map_err(InvokeError::Downstream)
                            })
                            .unwrap_or(Err(InvokeError::Error(
                                TransactionProcessorError::ProofNotFound(*proof_id),
                            ))),
                        ExecutableInstruction::DropAllProofs => {
                            for (_, real_id) in proof_id_mapping.drain() {
                                system_api
                                    .invoke_method(
                                        Receiver::Consumed(RENodeId::Proof(real_id)),
                                        FnIdentifier::Native(NativeFnIdentifier::Proof(
                                            ProofFnIdentifier::Drop,
                                        )),
                                        ScryptoValue::from_typed(&ConsumingProofDropInput {}),
                                    )
                                    .map_err(InvokeError::Downstream)?;
                            }
                            system_api
                                .invoke_method(
                                    Receiver::CurrentAuthZone,
                                    FnIdentifier::Native(NativeFnIdentifier::AuthZone(
                                        AuthZoneFnIdentifier::Clear,
                                    )),
                                    ScryptoValue::from_typed(&AuthZoneClearInput {}),
                                )
                                .map_err(InvokeError::Downstream)
                        }
                        ExecutableInstruction::CallFunction {
                            fn_identifier,
                            args,
                        } => {
                            Self::replace_ids(
                                &mut proof_id_mapping,
                                &mut bucket_id_mapping,
                                ScryptoValue::from_slice(args)
                                    .expect("Invalid CALL_FUNCTION arguments"),
                            )
                            .and_then(|call_data| Self::process_expressions(call_data, system_api))
                            .and_then(|call_data| {
                                system_api
                                    .invoke_function(fn_identifier.clone(), call_data)
                                    .map_err(InvokeError::Downstream)
                            })
                            .and_then(|result| {
                                // Auto move into auth_zone
                                for (proof_id, _) in &result.proof_ids {
                                    system_api
                                        .invoke_method(
                                            Receiver::CurrentAuthZone,
                                            FnIdentifier::Native(NativeFnIdentifier::AuthZone(
                                                AuthZoneFnIdentifier::Push,
                                            )),
                                            ScryptoValue::from_typed(&AuthZonePushInput {
                                                proof: scrypto::resource::Proof(*proof_id),
                                            }),
                                        )
                                        .map_err(InvokeError::Downstream)?;
                                }
                                // Auto move into worktop
                                for (bucket_id, _) in &result.bucket_ids {
                                    system_api
                                        .invoke_method(
                                            Receiver::Ref(RENodeId::Worktop),
                                            FnIdentifier::Native(NativeFnIdentifier::Worktop(
                                                WorktopFnIdentifier::Put,
                                            )),
                                            ScryptoValue::from_typed(&WorktopPutInput {
                                                bucket: scrypto::resource::Bucket(*bucket_id),
                                            }),
                                        )
                                        .map_err(InvokeError::Downstream)?;
                                }
                                Ok(result)
                            })
                        }
                        ExecutableInstruction::CallMethod {
                            method_identifier,
                            args,
                        } => {
                            Self::replace_ids(
                                &mut proof_id_mapping,
                                &mut bucket_id_mapping,
                                ScryptoValue::from_slice(args)
                                    .expect("Invalid CALL_METHOD arguments"),
                            )
                            .and_then(|call_data| Self::process_expressions(call_data, system_api))
                            .and_then(|call_data| {
                                // TODO: Move this into preprocessor step
                                match method_identifier {
                                    MethodIdentifier::Scrypto {
                                        component_address,
                                        ident,
                                    } => system_api
                                        .substate_read(SubstateId::ComponentInfo(
                                            *component_address,
                                        ))
                                        .map_err(InvokeError::Downstream)
                                        .and_then(|s| {
                                            let (package_address, blueprint_name): (
                                                PackageAddress,
                                                String,
                                            ) = scrypto_decode(&s.raw)
                                                .expect("Failed to decode ComponentInfo substate");

                                            system_api
                                                .invoke_method(
                                                    Receiver::Ref(RENodeId::Component(
                                                        *component_address,
                                                    )),
                                                    FnIdentifier::Scrypto {
                                                        ident: ident.to_string(),
                                                        package_address,
                                                        blueprint_name,
                                                    },
                                                    call_data,
                                                )
                                                .map_err(InvokeError::Downstream)
                                        }),
                                    MethodIdentifier::Native {
                                        receiver,
                                        native_fn_identifier,
                                    } => Self::replace_receiver(
                                        receiver.clone(),
                                        &mut proof_id_mapping,
                                        &mut bucket_id_mapping,
                                    )
                                    .and_then(|receiver| {
                                        system_api
                                            .invoke_method(
                                                receiver,
                                                FnIdentifier::Native(native_fn_identifier.clone()),
                                                call_data,
                                            )
                                            .map_err(InvokeError::Downstream)
                                    }),
                                }
                            })
                            .and_then(|result| {
                                // Auto move into auth_zone
                                for (proof_id, _) in &result.proof_ids {
                                    system_api
                                        .invoke_method(
                                            Receiver::CurrentAuthZone,
                                            FnIdentifier::Native(NativeFnIdentifier::AuthZone(
                                                AuthZoneFnIdentifier::Push,
                                            )),
                                            ScryptoValue::from_typed(&AuthZonePushInput {
                                                proof: scrypto::resource::Proof(*proof_id),
                                            }),
                                        )
                                        .map_err(InvokeError::Downstream)?;
                                }
                                // Auto move into worktop
                                for (bucket_id, _) in &result.bucket_ids {
                                    system_api
                                        .invoke_method(
                                            Receiver::Ref(RENodeId::Worktop),
                                            FnIdentifier::Native(NativeFnIdentifier::Worktop(
                                                WorktopFnIdentifier::Put,
                                            )),
                                            ScryptoValue::from_typed(&WorktopPutInput {
                                                bucket: scrypto::resource::Bucket(*bucket_id),
                                            }),
                                        )
                                        .map_err(InvokeError::downstream)?;
                                }
                                Ok(result)
                            })
                        }
                        ExecutableInstruction::PublishPackage { code, abi } => system_api
                            .invoke_function(
                                FnIdentifier::Native(NativeFnIdentifier::Package(
                                    PackageFnIdentifier::Publish,
                                )),
                                ScryptoValue::from_typed(&PackagePublishInput {
                                    code: code.clone(),
                                    abi: abi.clone(),
                                }),
                            )
                            .map_err(InvokeError::Downstream),
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
