use scrypto::core::{FnIdent, MethodFnIdent, MethodIdent, NativeFunctionFnIdent};
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
    pub instructions: Vec<Instruction>,
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
                        .invoke(
                            FnIdent::Method(MethodIdent {
                                receiver: Receiver::Ref(RENodeId::Worktop),
                                fn_ident: MethodFnIdent::Native(NativeMethodFnIdent::Worktop(
                                    WorktopMethodFnIdent::Drain,
                                )),
                            }),
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
        transaction_processor_fn: TransactionProcessorFunctionFnIdent,
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
            TransactionProcessorFunctionFnIdent::Run => {
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
                        Instruction::TakeFromWorktop { resource_address } => id_allocator
                            .new_bucket_id()
                            .map_err(|e| {
                                InvokeError::Error(TransactionProcessorError::IdAllocationError(e))
                            })
                            .and_then(|new_id| {
                                system_api
                                    .invoke(
                                        FnIdent::Method(MethodIdent {
                                            receiver: Receiver::Ref(RENodeId::Worktop),
                                            fn_ident: MethodFnIdent::Native(
                                                NativeMethodFnIdent::Worktop(
                                                    WorktopMethodFnIdent::TakeAll,
                                                ),
                                            ),
                                        }),
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
                                    .invoke(
                                        FnIdent::Method(MethodIdent {
                                            receiver: Receiver::Ref(RENodeId::Worktop),
                                            fn_ident: MethodFnIdent::Native(
                                                NativeMethodFnIdent::Worktop(
                                                    WorktopMethodFnIdent::TakeAmount,
                                                ),
                                            ),
                                        }),
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
                                    .invoke(
                                        FnIdent::Method(MethodIdent {
                                            receiver: Receiver::Ref(RENodeId::Worktop),
                                            fn_ident: MethodFnIdent::Native(
                                                NativeMethodFnIdent::Worktop(
                                                    WorktopMethodFnIdent::TakeNonFungibles,
                                                ),
                                            ),
                                        }),
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
                        Instruction::ReturnToWorktop { bucket_id } => bucket_id_mapping
                            .remove(bucket_id)
                            .map(|real_id| {
                                system_api
                                    .invoke(
                                        FnIdent::Method(MethodIdent {
                                            receiver: Receiver::Ref(RENodeId::Worktop),
                                            fn_ident: MethodFnIdent::Native(
                                                NativeMethodFnIdent::Worktop(
                                                    WorktopMethodFnIdent::Put,
                                                ),
                                            ),
                                        }),
                                        ScryptoValue::from_typed(&WorktopPutInput {
                                            bucket: scrypto::resource::Bucket(real_id),
                                        }),
                                    )
                                    .map_err(InvokeError::Downstream)
                            })
                            .unwrap_or(Err(InvokeError::Error(
                                TransactionProcessorError::BucketNotFound(*bucket_id),
                            ))),
                        Instruction::AssertWorktopContains { resource_address } => system_api
                            .invoke(
                                FnIdent::Method(MethodIdent {
                                    receiver: Receiver::Ref(RENodeId::Worktop),
                                    fn_ident: MethodFnIdent::Native(NativeMethodFnIdent::Worktop(
                                        WorktopMethodFnIdent::AssertContains,
                                    )),
                                }),
                                ScryptoValue::from_typed(&WorktopAssertContainsInput {
                                    resource_address: *resource_address,
                                }),
                            )
                            .map_err(InvokeError::Downstream),
                        Instruction::AssertWorktopContainsByAmount {
                            amount,
                            resource_address,
                        } => system_api
                            .invoke(
                                FnIdent::Method(MethodIdent {
                                    receiver: Receiver::Ref(RENodeId::Worktop),
                                    fn_ident: MethodFnIdent::Native(NativeMethodFnIdent::Worktop(
                                        WorktopMethodFnIdent::AssertContainsAmount,
                                    )),
                                }),
                                ScryptoValue::from_typed(&WorktopAssertContainsAmountInput {
                                    amount: *amount,
                                    resource_address: *resource_address,
                                }),
                            )
                            .map_err(InvokeError::Downstream),
                        Instruction::AssertWorktopContainsByIds {
                            ids,
                            resource_address,
                        } => system_api
                            .invoke(
                                FnIdent::Method(MethodIdent {
                                    receiver: Receiver::Ref(RENodeId::Worktop),
                                    fn_ident: MethodFnIdent::Native(NativeMethodFnIdent::Worktop(
                                        WorktopMethodFnIdent::AssertContainsNonFungibles,
                                    )),
                                }),
                                ScryptoValue::from_typed(&WorktopAssertContainsNonFungiblesInput {
                                    ids: ids.clone(),
                                    resource_address: *resource_address,
                                }),
                            )
                            .map_err(InvokeError::Downstream),

                        Instruction::PopFromAuthZone {} => id_allocator
                            .new_proof_id()
                            .map_err(|e| {
                                InvokeError::Error(TransactionProcessorError::IdAllocationError(e))
                            })
                            .and_then(|new_id| {
                                system_api
                                    .invoke(
                                        FnIdent::Method(MethodIdent {
                                            receiver: Receiver::CurrentAuthZone,
                                            fn_ident: MethodFnIdent::Native(
                                                NativeMethodFnIdent::AuthZone(
                                                    AuthZoneMethodFnIdent::Pop,
                                                ),
                                            ),
                                        }),
                                        ScryptoValue::from_typed(&AuthZonePopInput {}),
                                    )
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
                                .invoke(
                                    FnIdent::Method(MethodIdent {
                                        receiver: Receiver::CurrentAuthZone,
                                        fn_ident: MethodFnIdent::Native(
                                            NativeMethodFnIdent::AuthZone(
                                                AuthZoneMethodFnIdent::Clear,
                                            ),
                                        ),
                                    }),
                                    ScryptoValue::from_typed(&AuthZoneClearInput {}),
                                )
                                .map_err(InvokeError::Downstream)
                        }
                        Instruction::PushToAuthZone { proof_id } => proof_id_mapping
                            .remove(proof_id)
                            .ok_or(InvokeError::Error(
                                TransactionProcessorError::ProofNotFound(*proof_id),
                            ))
                            .and_then(|real_id| {
                                system_api
                                    .invoke(
                                        FnIdent::Method(MethodIdent {
                                            receiver: Receiver::CurrentAuthZone,
                                            fn_ident: MethodFnIdent::Native(
                                                NativeMethodFnIdent::AuthZone(
                                                    AuthZoneMethodFnIdent::Push,
                                                ),
                                            ),
                                        }),
                                        ScryptoValue::from_typed(&AuthZonePushInput {
                                            proof: scrypto::resource::Proof(real_id),
                                        }),
                                    )
                                    .map_err(InvokeError::Downstream)
                            }),
                        Instruction::CreateProofFromAuthZone { resource_address } => id_allocator
                            .new_proof_id()
                            .map_err(|e| {
                                InvokeError::Error(TransactionProcessorError::IdAllocationError(e))
                            })
                            .and_then(|new_id| {
                                system_api
                                    .invoke(
                                        FnIdent::Method(MethodIdent {
                                            receiver: Receiver::CurrentAuthZone,
                                            fn_ident: MethodFnIdent::Native(
                                                NativeMethodFnIdent::AuthZone(
                                                    AuthZoneMethodFnIdent::CreateProof,
                                                ),
                                            ),
                                        }),
                                        ScryptoValue::from_typed(&AuthZoneCreateProofInput {
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
                                    .invoke(
                                        FnIdent::Method(MethodIdent {
                                            receiver: Receiver::CurrentAuthZone,
                                            fn_ident: MethodFnIdent::Native(
                                                NativeMethodFnIdent::AuthZone(
                                                    AuthZoneMethodFnIdent::CreateProofByAmount,
                                                ),
                                            ),
                                        }),
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
                                    .invoke(
                                        FnIdent::Method(MethodIdent {
                                            receiver: Receiver::CurrentAuthZone,
                                            fn_ident: MethodFnIdent::Native(
                                                NativeMethodFnIdent::AuthZone(
                                                    AuthZoneMethodFnIdent::CreateProofByIds,
                                                ),
                                            ),
                                        }),
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
                                    .invoke(
                                        FnIdent::Method(MethodIdent {
                                            receiver: Receiver::Ref(RENodeId::Bucket(
                                                real_bucket_id,
                                            )),
                                            fn_ident: MethodFnIdent::Native(
                                                NativeMethodFnIdent::Bucket(
                                                    BucketMethodFnIdent::CreateProof,
                                                ),
                                            ),
                                        }),
                                        ScryptoValue::from_typed(&BucketCreateProofInput {}),
                                    )
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
                                            .invoke(
                                                FnIdent::Method(MethodIdent {
                                                    receiver: Receiver::Ref(RENodeId::Proof(
                                                        real_id,
                                                    )),
                                                    fn_ident: MethodFnIdent::Native(
                                                        NativeMethodFnIdent::Proof(
                                                            ProofMethodFnIdent::Clone,
                                                        ),
                                                    ),
                                                }),
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
                        Instruction::DropProof { proof_id } => proof_id_mapping
                            .remove(proof_id)
                            .map(|real_id| {
                                system_api
                                    .invoke(
                                        FnIdent::Method(MethodIdent {
                                            receiver: Receiver::Consumed(RENodeId::Proof(real_id)),
                                            fn_ident: MethodFnIdent::Native(
                                                NativeMethodFnIdent::Proof(
                                                    ProofMethodFnIdent::Drop,
                                                ),
                                            ),
                                        }),
                                        ScryptoValue::from_typed(&ConsumingProofDropInput {}),
                                    )
                                    .map_err(InvokeError::Downstream)
                            })
                            .unwrap_or(Err(InvokeError::Error(
                                TransactionProcessorError::ProofNotFound(*proof_id),
                            ))),
                        Instruction::DropAllProofs => {
                            for (_, real_id) in proof_id_mapping.drain() {
                                system_api
                                    .invoke(
                                        FnIdent::Method(MethodIdent {
                                            receiver: Receiver::Consumed(RENodeId::Proof(real_id)),
                                            fn_ident: MethodFnIdent::Native(
                                                NativeMethodFnIdent::Proof(
                                                    ProofMethodFnIdent::Drop,
                                                ),
                                            ),
                                        }),
                                        ScryptoValue::from_typed(&ConsumingProofDropInput {}),
                                    )
                                    .map_err(InvokeError::Downstream)?;
                            }
                            system_api
                                .invoke(
                                    FnIdent::Method(MethodIdent {
                                        receiver: Receiver::CurrentAuthZone,
                                        fn_ident: MethodFnIdent::Native(
                                            NativeMethodFnIdent::AuthZone(
                                                AuthZoneMethodFnIdent::Clear,
                                            ),
                                        ),
                                    }),
                                    ScryptoValue::from_typed(&AuthZoneClearInput {}),
                                )
                                .map_err(InvokeError::Downstream)
                        }
                        Instruction::CallFunction {
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
                                    .invoke(FnIdent::Function(fn_identifier.clone()), call_data)
                                    .map_err(InvokeError::Downstream)
                            })
                            .and_then(|result| {
                                // Auto move into auth_zone
                                for (proof_id, _) in &result.proof_ids {
                                    system_api
                                        .invoke(
                                            FnIdent::Method(MethodIdent {
                                                receiver: Receiver::CurrentAuthZone,
                                                fn_ident: MethodFnIdent::Native(
                                                    NativeMethodFnIdent::AuthZone(
                                                        AuthZoneMethodFnIdent::Push,
                                                    ),
                                                ),
                                            }),
                                            ScryptoValue::from_typed(&AuthZonePushInput {
                                                proof: scrypto::resource::Proof(*proof_id),
                                            }),
                                        )
                                        .map_err(InvokeError::Downstream)?;
                                }
                                // Auto move into worktop
                                for (bucket_id, _) in &result.bucket_ids {
                                    system_api
                                        .invoke(
                                            FnIdent::Method(MethodIdent {
                                                receiver: Receiver::Ref(RENodeId::Worktop),
                                                fn_ident: MethodFnIdent::Native(
                                                    NativeMethodFnIdent::Worktop(
                                                        WorktopMethodFnIdent::Put,
                                                    ),
                                                ),
                                            }),
                                            ScryptoValue::from_typed(&WorktopPutInput {
                                                bucket: scrypto::resource::Bucket(*bucket_id),
                                            }),
                                        )
                                        .map_err(InvokeError::Downstream)?;
                                }
                                Ok(result)
                            })
                        }
                        Instruction::CallMethod {
                            method_ident: MethodIdent { receiver, fn_ident },
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
                                system_api
                                    .invoke(
                                        FnIdent::Method(MethodIdent {
                                            receiver: Self::replace_receiver(
                                                receiver.clone(),
                                                &mut proof_id_mapping,
                                                &mut bucket_id_mapping,
                                            )?,
                                            fn_ident: fn_ident.clone(),
                                        }),
                                        call_data,
                                    )
                                    .map_err(InvokeError::Downstream)
                            })
                            .and_then(|result| {
                                // Auto move into auth_zone
                                for (proof_id, _) in &result.proof_ids {
                                    system_api
                                        .invoke(
                                            FnIdent::Method(MethodIdent {
                                                receiver: Receiver::CurrentAuthZone,
                                                fn_ident: MethodFnIdent::Native(
                                                    NativeMethodFnIdent::AuthZone(
                                                        AuthZoneMethodFnIdent::Push,
                                                    ),
                                                ),
                                            }),
                                            ScryptoValue::from_typed(&AuthZonePushInput {
                                                proof: scrypto::resource::Proof(*proof_id),
                                            }),
                                        )
                                        .map_err(InvokeError::Downstream)?;
                                }
                                // Auto move into worktop
                                for (bucket_id, _) in &result.bucket_ids {
                                    system_api
                                        .invoke(
                                            FnIdent::Method(MethodIdent {
                                                receiver: Receiver::Ref(RENodeId::Worktop),
                                                fn_ident: MethodFnIdent::Native(
                                                    NativeMethodFnIdent::Worktop(
                                                        WorktopMethodFnIdent::Put,
                                                    ),
                                                ),
                                            }),
                                            ScryptoValue::from_typed(&WorktopPutInput {
                                                bucket: scrypto::resource::Bucket(*bucket_id),
                                            }),
                                        )
                                        .map_err(InvokeError::downstream)?;
                                }
                                Ok(result)
                            })
                        }
                        Instruction::PublishPackage { code, abi } => system_api
                            .invoke(
                                FnIdent::Function(FunctionIdent::Native(
                                    NativeFunctionFnIdent::Package(PackageFunctionFnIdent::Publish),
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
