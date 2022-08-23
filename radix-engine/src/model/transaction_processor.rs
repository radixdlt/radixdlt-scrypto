use transaction::errors::IdAllocationError;
use transaction::model::*;
use transaction::validation::*;

use crate::engine::ApplicationError;
use crate::engine::{HeapRENode, RuntimeError, SystemApi};
use crate::fee::FeeReserve;
use crate::model::worktop::{
    WorktopAssertContainsAmountInput, WorktopAssertContainsInput,
    WorktopAssertContainsNonFungiblesInput, WorktopDrainInput, WorktopPutInput,
    WorktopTakeAllInput, WorktopTakeAmountInput, WorktopTakeNonFungiblesInput,
};
use crate::types::*;
use crate::wasm::*;

use super::Worktop;

#[derive(Debug, TypeId, Encode, Decode)]
pub struct TransactionProcessorRunInput {
    pub instructions: Vec<ExecutableInstruction>,
}

#[derive(Debug)]
pub enum TransactionProcessorError {
    RuntimeError(Box<RuntimeError>), // error propagation
    InvalidRequestData(DecodeError),
    InvalidMethod,
    BucketNotFound(BucketId),
    ProofNotFound(ProofId),
    IdAllocationError(IdAllocationError),
    InvalidPackage(DecodeError),
}

impl TransactionProcessorError {
    /// Wraps into a runtime error unless it's already a runtime error.
    ///
    /// TODO: Is this really a good idea?
    pub fn to_runtime_error(self) -> RuntimeError {
        match self {
            TransactionProcessorError::RuntimeError(e) => *e,
            e @ TransactionProcessorError::InvalidRequestData(_)
            | e @ TransactionProcessorError::InvalidMethod
            | e @ TransactionProcessorError::BucketNotFound(_)
            | e @ TransactionProcessorError::ProofNotFound(_)
            | e @ TransactionProcessorError::IdAllocationError(_)
            | e @ TransactionProcessorError::InvalidPackage(_) => {
                RuntimeError::ApplicationError(ApplicationError::TransactionProcessorError(e))
            }
        }
    }
}

pub struct TransactionProcessor {}

impl TransactionProcessor {
    fn replace_ids(
        proof_id_mapping: &mut HashMap<ProofId, ProofId>,
        bucket_id_mapping: &mut HashMap<BucketId, BucketId>,
        mut value: ScryptoValue,
    ) -> Result<ScryptoValue, TransactionProcessorError> {
        value
            .replace_ids(proof_id_mapping, bucket_id_mapping)
            .map_err(|e| match e {
                ScryptoValueReplaceError::BucketIdNotFound(bucket_id) => {
                    TransactionProcessorError::BucketNotFound(bucket_id)
                }
                ScryptoValueReplaceError::ProofIdNotFound(proof_id) => {
                    TransactionProcessorError::ProofNotFound(proof_id)
                }
            })?;
        Ok(value)
    }

    fn process_expressions<'s, Y, W, I, R>(
        args: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, TransactionProcessorError>
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
    {
        let mut value = args.dom;
        for (expression, path) in args.expressions {
            match expression.0.as_str() {
                "ALL_WORKTOP_RESOURCES" => {
                    let buckets = system_api
                        .invoke_method(
                            Receiver::Ref(RENodeId::Worktop),
                            FnIdentifier::Native(NativeFnIdentifier::Worktop(
                                WorktopFnIdentifier::Drain,
                            )),
                            ScryptoValue::from_typed(&WorktopDrainInput {}),
                        )
                        .map_err(|e| TransactionProcessorError::RuntimeError(Box::new(e)))
                        .map(|result| {
                            let mut buckets = Vec::new();
                            for (bucket_id, _) in result.bucket_ids {
                                buckets.push(scrypto::resource::Bucket(bucket_id));
                            }
                            buckets
                        })?;

                    let val = path
                        .get_from_value_mut(&mut value)
                        .expect("Failed to locate a expression using SBOR path");
                    *val =
                        decode_any(&scrypto_encode(&buckets)).expect("Failed to decode Vec<Bucket>")
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
    ) -> Result<ScryptoValue, TransactionProcessorError>
    where
        Y: SystemApi<'s, W, I, R>,
        W: WasmEngine<I>,
        I: WasmInstance,
        R: FeeReserve,
    {
        match transaction_processor_fn {
            TransactionProcessorFnIdentifier::Run => {
                let input: TransactionProcessorRunInput = scrypto_decode(&call_data.raw)
                    .map_err(|e| TransactionProcessorError::InvalidRequestData(e))?;

                let mut proof_id_mapping = HashMap::new();
                let mut bucket_id_mapping = HashMap::new();
                let mut outputs = Vec::new();
                let mut id_allocator = IdAllocator::new(IdSpace::Transaction);

                let _worktop_id = system_api
                    .node_create(HeapRENode::Worktop(Worktop::new()))
                    .map_err(|e| TransactionProcessorError::RuntimeError(Box::new(e)))?;

                for inst in &input.instructions.clone() {
                    let result = match inst {
                        ExecutableInstruction::TakeFromWorktop { resource_address } => id_allocator
                            .new_bucket_id()
                            .map_err(TransactionProcessorError::IdAllocationError)
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
                                    .map_err(|e| {
                                        TransactionProcessorError::RuntimeError(Box::new(e))
                                    })
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
                            .map_err(TransactionProcessorError::IdAllocationError)
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
                                    .map_err(|e| {
                                        TransactionProcessorError::RuntimeError(Box::new(e))
                                    })
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
                            .map_err(TransactionProcessorError::IdAllocationError)
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
                                    .map_err(|e| {
                                        TransactionProcessorError::RuntimeError(Box::new(e))
                                    })
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
                                    .map_err(|e| {
                                        TransactionProcessorError::RuntimeError(Box::new(e))
                                    })
                            })
                            .unwrap_or(Err(TransactionProcessorError::BucketNotFound(*bucket_id))),
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
                                .map_err(|e| TransactionProcessorError::RuntimeError(Box::new(e)))
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
                            .map_err(|e| TransactionProcessorError::RuntimeError(Box::new(e))),
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
                            .map_err(|e| TransactionProcessorError::RuntimeError(Box::new(e))),

                        ExecutableInstruction::PopFromAuthZone {} => id_allocator
                            .new_proof_id()
                            .map_err(TransactionProcessorError::IdAllocationError)
                            .and_then(|new_id| {
                                system_api
                                    .invoke_method(
                                        Receiver::CurrentAuthZone,
                                        FnIdentifier::Native(NativeFnIdentifier::AuthZone(
                                            AuthZoneFnIdentifier::Pop,
                                        )),
                                        ScryptoValue::from_typed(&AuthZonePopInput {}),
                                    )
                                    .map_err(|e| {
                                        TransactionProcessorError::RuntimeError(Box::new(e))
                                    })
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
                                .map_err(|e| TransactionProcessorError::RuntimeError(Box::new(e)))
                        }
                        ExecutableInstruction::PushToAuthZone { proof_id } => proof_id_mapping
                            .remove(proof_id)
                            .ok_or(TransactionProcessorError::ProofNotFound(*proof_id))
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
                                    .map_err(|e| {
                                        TransactionProcessorError::RuntimeError(Box::new(e))
                                    })
                            }),
                        ExecutableInstruction::CreateProofFromAuthZone { resource_address } => {
                            id_allocator
                                .new_proof_id()
                                .map_err(TransactionProcessorError::IdAllocationError)
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
                                        .map_err(|e| {
                                            TransactionProcessorError::RuntimeError(Box::new(e))
                                        })
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
                            .map_err(TransactionProcessorError::IdAllocationError)
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
                                    .map_err(|e| {
                                        TransactionProcessorError::RuntimeError(Box::new(e))
                                    })
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
                            .map_err(TransactionProcessorError::IdAllocationError)
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
                                    .map_err(|e| {
                                        TransactionProcessorError::RuntimeError(Box::new(e))
                                    })
                                    .map(|rtn| {
                                        let proof_id = Self::first_proof(&rtn);
                                        proof_id_mapping.insert(new_id, proof_id);
                                        ScryptoValue::from_typed(&scrypto::resource::Proof(new_id))
                                    })
                            }),
                        ExecutableInstruction::CreateProofFromBucket { bucket_id } => id_allocator
                            .new_proof_id()
                            .map_err(TransactionProcessorError::IdAllocationError)
                            .and_then(|new_id| {
                                bucket_id_mapping
                                    .get(bucket_id)
                                    .cloned()
                                    .map(|real_bucket_id| (new_id, real_bucket_id))
                                    .ok_or(TransactionProcessorError::BucketNotFound(new_id))
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
                                    .map_err(|e| {
                                        TransactionProcessorError::RuntimeError(Box::new(e))
                                    })
                                    .map(|rtn| {
                                        let proof_id = Self::first_proof(&rtn);
                                        proof_id_mapping.insert(new_id, proof_id);
                                        ScryptoValue::from_typed(&scrypto::resource::Proof(new_id))
                                    })
                            }),
                        ExecutableInstruction::CloneProof { proof_id } => id_allocator
                            .new_proof_id()
                            .map_err(TransactionProcessorError::IdAllocationError)
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
                                            .map_err(|e| {
                                                TransactionProcessorError::RuntimeError(Box::new(e))
                                            })
                                            .map(|v| {
                                                let cloned_proof_id = Self::first_proof(&v);
                                                proof_id_mapping.insert(new_id, cloned_proof_id);
                                                ScryptoValue::from_typed(&scrypto::resource::Proof(
                                                    new_id,
                                                ))
                                            })
                                    })
                                    .unwrap_or(Err(TransactionProcessorError::ProofNotFound(
                                        *proof_id,
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
                                    .map_err(|e| {
                                        TransactionProcessorError::RuntimeError(Box::new(e))
                                    })
                            })
                            .unwrap_or(Err(TransactionProcessorError::ProofNotFound(*proof_id))),
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
                                    .map_err(|e| {
                                        TransactionProcessorError::RuntimeError(Box::new(e))
                                    })?;
                            }
                            system_api
                                .invoke_method(
                                    Receiver::CurrentAuthZone,
                                    FnIdentifier::Native(NativeFnIdentifier::AuthZone(
                                        AuthZoneFnIdentifier::Clear,
                                    )),
                                    ScryptoValue::from_typed(&AuthZoneClearInput {}),
                                )
                                .map_err(|e| TransactionProcessorError::RuntimeError(Box::new(e)))
                        }
                        ExecutableInstruction::CallFunction {
                            package_address,
                            blueprint_name,
                            method_name,
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
                                    .invoke_function(
                                        FnIdentifier::Scrypto {
                                            package_address: *package_address,
                                            blueprint_name: blueprint_name.to_string(),
                                            ident: method_name.to_string(),
                                        },
                                        call_data,
                                    )
                                    .map_err(|e| {
                                        TransactionProcessorError::RuntimeError(Box::new(e))
                                    })
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
                                        .map_err(|e| {
                                            TransactionProcessorError::RuntimeError(Box::new(e))
                                        })?;
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
                                        .map_err(|e| {
                                            TransactionProcessorError::RuntimeError(Box::new(e))
                                        })?;
                                }
                                Ok(result)
                            })
                        }
                        ExecutableInstruction::CallMethod {
                            component_address,
                            method_name,
                            args,
                        } => {
                            Self::replace_ids(
                                &mut proof_id_mapping,
                                &mut bucket_id_mapping,
                                ScryptoValue::from_slice(args)
                                    .expect("Invalid CALL_METHOD arguments"),
                            )
                            .and_then(|call_data| {
                                // TODO: Move this into preprocessor step
                                system_api
                                    .substate_read(SubstateId::ComponentInfo(*component_address))
                                    .map_err(|e| {
                                        TransactionProcessorError::RuntimeError(Box::new(e))
                                    })
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
                                                    ident: method_name.to_string(),
                                                    package_address,
                                                    blueprint_name,
                                                },
                                                call_data,
                                            )
                                            .map_err(|e| {
                                                TransactionProcessorError::RuntimeError(Box::new(e))
                                            })
                                    })
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
                                        .map_err(|e| {
                                            TransactionProcessorError::RuntimeError(Box::new(e))
                                        })?;
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
                                        .map_err(|e| {
                                            TransactionProcessorError::RuntimeError(Box::new(e))
                                        })?;
                                }
                                Ok(result)
                            })
                        }
                        ExecutableInstruction::CallMethodWithAllResources {
                            component_address,
                            method,
                        } => system_api
                            .invoke_method(
                                Receiver::Ref(RENodeId::Worktop),
                                FnIdentifier::Native(NativeFnIdentifier::Worktop(
                                    WorktopFnIdentifier::Drain,
                                )),
                                ScryptoValue::from_typed(&WorktopDrainInput {}),
                            )
                            .map_err(|e| TransactionProcessorError::RuntimeError(Box::new(e)))
                            .and_then(|result| {
                                let mut buckets = Vec::new();
                                for (bucket_id, _) in result.bucket_ids {
                                    buckets.push(scrypto::resource::Bucket(bucket_id));
                                }
                                for (_, real_id) in bucket_id_mapping.drain() {
                                    buckets.push(scrypto::resource::Bucket(real_id));
                                }
                                let encoded = args!(buckets);
                                // TODO: Move this into preprocessor step
                                system_api
                                    .substate_read(SubstateId::ComponentInfo(*component_address))
                                    .map_err(|e| {
                                        TransactionProcessorError::RuntimeError(Box::new(e))
                                    })
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
                                                    package_address,
                                                    blueprint_name,
                                                    ident: method.to_string(),
                                                },
                                                ScryptoValue::from_slice(&encoded).expect(
                                                    "Failed to decode ComponentInfo substate",
                                                ),
                                            )
                                            .map_err(|e| {
                                                TransactionProcessorError::RuntimeError(Box::new(e))
                                            })
                                    })
                            }),
                        ExecutableInstruction::PublishPackage { package } => scrypto_decode::<
                            Package,
                        >(
                            package
                        )
                        .map_err(|e| TransactionProcessorError::InvalidPackage(e))
                        .and_then(|package| {
                            system_api
                                .invoke_function(
                                    FnIdentifier::Native(NativeFnIdentifier::Package(
                                        PackageFnIdentifier::Publish,
                                    )),
                                    ScryptoValue::from_typed(&PackagePublishInput { package }),
                                )
                                .map_err(|e| TransactionProcessorError::RuntimeError(Box::new(e)))
                        }),
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
