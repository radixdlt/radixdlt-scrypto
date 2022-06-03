use sbor::rust::collections::HashMap;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::buffer::scrypto_decode;
use scrypto::call_data;
use scrypto::component::Package;
use scrypto::core::{SNodeRef, ScryptoActor};
use scrypto::engine::types::*;
use scrypto::prelude::{
    AuthZoneClearInput, AuthZoneCreateProofByAmountInput, AuthZoneCreateProofByIdsInput,
    AuthZoneCreateProofInput, AuthZonePushInput, BucketCreateProofInput, PackagePublishInput,
    ProofCloneInput,
};
use scrypto::resource::{AuthZonePopInput, ConsumingProofDropInput};
use scrypto::values::*;
use transaction::model::*;
use transaction::validation::*;

use crate::engine::{RuntimeError, RuntimeError::ProofNotFound, SystemApi};
use crate::model::worktop::{
    WorktopAssertContainsAmountInput, WorktopAssertContainsInput,
    WorktopAssertContainsNonFungiblesInput, WorktopDrainInput, WorktopPutInput,
    WorktopTakeAllInput, WorktopTakeAmountInput, WorktopTakeNonFungiblesInput,
};
use crate::wasm::*;

#[derive(Debug, TypeId, Encode, Decode)]
pub enum TransactionProcessorFunction {
    Run(ValidatedTransaction),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionProcessorError {
    InvalidRequestData(DecodeError),
    RuntimeError(RuntimeError),
}

pub struct TransactionProcessor {}

impl TransactionProcessor {
    fn replace_ids(
        proof_id_mapping: &mut HashMap<ProofId, ProofId>,
        bucket_id_mapping: &mut HashMap<BucketId, BucketId>,
        mut value: ScryptoValue,
    ) -> Result<ScryptoValue, RuntimeError> {
        value
            .replace_ids(proof_id_mapping, bucket_id_mapping)
            .map_err(|e| match e {
                ScryptoValueReplaceError::BucketIdNotFound(bucket_id) => {
                    RuntimeError::BucketNotFound(bucket_id)
                }
                ScryptoValueReplaceError::ProofIdNotFound(proof_id) => {
                    RuntimeError::ProofNotFound(proof_id)
                }
            })?;
        Ok(value)
    }

    pub fn static_main<S: SystemApi<W, I>, W: WasmEngine<I>, I: WasmInstance>(
        call_data: ScryptoValue,
        system_api: &mut S,
    ) -> Result<ScryptoValue, TransactionProcessorError> {
        let function: TransactionProcessorFunction = scrypto_decode(&call_data.raw)
            .map_err(|e| TransactionProcessorError::InvalidRequestData(e))?;

        match function {
            TransactionProcessorFunction::Run(transaction) => {
                let mut proof_id_mapping = HashMap::new();
                let mut bucket_id_mapping = HashMap::new();
                let mut outputs = Vec::new();
                let mut id_allocator = IdAllocator::new(IdSpace::Transaction);

                for inst in &transaction.instructions.clone() {
                    let result = match inst {
                        ValidatedInstruction::TakeFromWorktop { resource_address } => id_allocator
                            .new_bucket_id()
                            .map_err(RuntimeError::IdAllocatorError)
                            .and_then(|new_id| {
                                system_api
                                    .invoke_snode2(
                                        SNodeRef::WorktopRef,
                                        "take_all".to_string(),
                                        ScryptoValue::from_value(&WorktopTakeAllInput {
                                            resource_address: *resource_address,
                                        }),
                                    )
                                    .map(|rtn| {
                                        let bucket_id = *rtn.bucket_ids.iter().next().unwrap().0;
                                        bucket_id_mapping.insert(new_id, bucket_id);
                                        ScryptoValue::from_value(&scrypto::resource::Bucket(new_id))
                                    })
                            }),
                        ValidatedInstruction::TakeFromWorktopByAmount {
                            amount,
                            resource_address,
                        } => id_allocator
                            .new_bucket_id()
                            .map_err(RuntimeError::IdAllocatorError)
                            .and_then(|new_id| {
                                system_api
                                    .invoke_snode2(
                                        SNodeRef::WorktopRef,
                                        "take_amount".to_string(),
                                        ScryptoValue::from_value(&WorktopTakeAmountInput {
                                            amount: *amount,
                                            resource_address: *resource_address,
                                        }),
                                    )
                                    .map(|rtn| {
                                        let bucket_id = *rtn.bucket_ids.iter().next().unwrap().0;
                                        bucket_id_mapping.insert(new_id, bucket_id);
                                        ScryptoValue::from_value(&scrypto::resource::Bucket(new_id))
                                    })
                            }),
                        ValidatedInstruction::TakeFromWorktopByIds {
                            ids,
                            resource_address,
                        } => id_allocator
                            .new_bucket_id()
                            .map_err(RuntimeError::IdAllocatorError)
                            .and_then(|new_id| {
                                system_api
                                    .invoke_snode2(
                                        SNodeRef::WorktopRef,
                                        "take_non_fungibles".to_string(),
                                        ScryptoValue::from_value(&WorktopTakeNonFungiblesInput {
                                            ids: ids.clone(),
                                            resource_address: *resource_address,
                                        }),
                                    )
                                    .map(|rtn| {
                                        let bucket_id = *rtn.bucket_ids.iter().next().unwrap().0;
                                        bucket_id_mapping.insert(new_id, bucket_id);
                                        ScryptoValue::from_value(&scrypto::resource::Bucket(new_id))
                                    })
                            }),
                        ValidatedInstruction::ReturnToWorktop { bucket_id } => bucket_id_mapping
                            .remove(bucket_id)
                            .map(|real_id| {
                                system_api.invoke_snode2(
                                    SNodeRef::WorktopRef,
                                    "put".to_string(),
                                    ScryptoValue::from_value(&WorktopPutInput {
                                        bucket: scrypto::resource::Bucket(real_id),
                                    }),
                                )
                            })
                            .unwrap_or(Err(RuntimeError::BucketNotFound(*bucket_id))),
                        ValidatedInstruction::AssertWorktopContains { resource_address } => {
                            system_api.invoke_snode2(
                                SNodeRef::WorktopRef,
                                "assert_contains".to_string(),
                                ScryptoValue::from_value(&WorktopAssertContainsInput {
                                    resource_address: *resource_address,
                                }),
                            )
                        }
                        ValidatedInstruction::AssertWorktopContainsByAmount {
                            amount,
                            resource_address,
                        } => system_api.invoke_snode2(
                            SNodeRef::WorktopRef,
                            "assert_contains_amount".to_string(),
                            ScryptoValue::from_value(&WorktopAssertContainsAmountInput {
                                amount: *amount,
                                resource_address: *resource_address,
                            }),
                        ),
                        ValidatedInstruction::AssertWorktopContainsByIds {
                            ids,
                            resource_address,
                        } => system_api.invoke_snode2(
                            SNodeRef::WorktopRef,
                            "assert_contains_non_fungibles".to_string(),
                            ScryptoValue::from_value(&WorktopAssertContainsNonFungiblesInput {
                                ids: ids.clone(),
                                resource_address: *resource_address,
                            }),
                        ),
                        ValidatedInstruction::PopFromAuthZone {} => id_allocator
                            .new_proof_id()
                            .map_err(RuntimeError::IdAllocatorError)
                            .and_then(|new_id| {
                                system_api
                                    .invoke_snode2(
                                        SNodeRef::AuthZoneRef,
                                        "pop".to_string(),
                                        ScryptoValue::from_value(&AuthZonePopInput {}),
                                    )
                                    .map(|rtn| {
                                        let proof_id = *rtn.proof_ids.iter().next().unwrap().0;
                                        proof_id_mapping.insert(new_id, proof_id);
                                        ScryptoValue::from_value(&scrypto::resource::Proof(new_id))
                                    })
                            }),
                        ValidatedInstruction::ClearAuthZone => {
                            proof_id_mapping.clear();
                            system_api.invoke_snode2(
                                SNodeRef::AuthZoneRef,
                                "clear".to_string(),
                                ScryptoValue::from_value(&AuthZoneClearInput {}),
                            )
                        }
                        ValidatedInstruction::PushToAuthZone { proof_id } => proof_id_mapping
                            .remove(proof_id)
                            .ok_or(RuntimeError::ProofNotFound(*proof_id))
                            .and_then(|real_id| {
                                system_api.invoke_snode2(
                                    SNodeRef::AuthZoneRef,
                                    "push".to_string(),
                                    ScryptoValue::from_value(&AuthZonePushInput {
                                        proof: scrypto::resource::Proof(real_id),
                                    }),
                                )
                            }),
                        ValidatedInstruction::CreateProofFromAuthZone { resource_address } => {
                            id_allocator
                                .new_proof_id()
                                .map_err(RuntimeError::IdAllocatorError)
                                .and_then(|new_id| {
                                    system_api
                                        .invoke_snode2(
                                            SNodeRef::AuthZoneRef,
                                            "create_proof".to_string(),
                                            ScryptoValue::from_value(&AuthZoneCreateProofInput {
                                                resource_address: *resource_address,
                                            }),
                                        )
                                        .map(|rtn| {
                                            let proof_id = *rtn.proof_ids.iter().next().unwrap().0;
                                            proof_id_mapping.insert(new_id, proof_id);
                                            ScryptoValue::from_value(&scrypto::resource::Proof(
                                                new_id,
                                            ))
                                        })
                                })
                        }
                        ValidatedInstruction::CreateProofFromAuthZoneByAmount {
                            amount,
                            resource_address,
                        } => id_allocator
                            .new_proof_id()
                            .map_err(RuntimeError::IdAllocatorError)
                            .and_then(|new_id| {
                                system_api
                                    .invoke_snode2(
                                        SNodeRef::AuthZoneRef,
                                        "create_proof_by_amount".to_string(),
                                        ScryptoValue::from_value(
                                            &AuthZoneCreateProofByAmountInput {
                                                amount: *amount,
                                                resource_address: *resource_address,
                                            },
                                        ),
                                    )
                                    .map(|rtn| {
                                        let proof_id = *rtn.proof_ids.iter().next().unwrap().0;
                                        proof_id_mapping.insert(new_id, proof_id);
                                        ScryptoValue::from_value(&scrypto::resource::Proof(new_id))
                                    })
                            }),
                        ValidatedInstruction::CreateProofFromAuthZoneByIds {
                            ids,
                            resource_address,
                        } => id_allocator
                            .new_proof_id()
                            .map_err(RuntimeError::IdAllocatorError)
                            .and_then(|new_id| {
                                system_api
                                    .invoke_snode2(
                                        SNodeRef::AuthZoneRef,
                                        "create_proof_by_ids".to_string(),
                                        ScryptoValue::from_value(&AuthZoneCreateProofByIdsInput {
                                            ids: ids.clone(),
                                            resource_address: *resource_address,
                                        }),
                                    )
                                    .map(|rtn| {
                                        let proof_id = *rtn.proof_ids.iter().next().unwrap().0;
                                        proof_id_mapping.insert(new_id, proof_id);
                                        ScryptoValue::from_value(&scrypto::resource::Proof(new_id))
                                    })
                            }),
                        ValidatedInstruction::CreateProofFromBucket { bucket_id } => id_allocator
                            .new_proof_id()
                            .map_err(RuntimeError::IdAllocatorError)
                            .and_then(|new_id| {
                                bucket_id_mapping
                                    .get(bucket_id)
                                    .cloned()
                                    .map(|real_bucket_id| (new_id, real_bucket_id))
                                    .ok_or(RuntimeError::BucketNotFound(new_id))
                            })
                            .and_then(|(new_id, real_bucket_id)| {
                                system_api
                                    .invoke_snode2(
                                        SNodeRef::BucketRef(real_bucket_id),
                                        "create_proof".to_string(),
                                        ScryptoValue::from_value(&BucketCreateProofInput {}),
                                    )
                                    .map(|rtn| {
                                        let proof_id = *rtn.proof_ids.iter().next().unwrap().0;
                                        proof_id_mapping.insert(new_id, proof_id);
                                        ScryptoValue::from_value(&scrypto::resource::Proof(new_id))
                                    })
                            }),
                        ValidatedInstruction::CloneProof { proof_id } => id_allocator
                            .new_proof_id()
                            .map_err(RuntimeError::IdAllocatorError)
                            .and_then(|new_id| {
                                proof_id_mapping
                                    .get(proof_id)
                                    .cloned()
                                    .map(|real_id| {
                                        system_api
                                            .invoke_snode2(
                                                SNodeRef::ProofRef(real_id),
                                                "clone".to_string(),
                                                ScryptoValue::from_value(&ProofCloneInput {}),
                                            )
                                            .map(|v| {
                                                let cloned_proof_id =
                                                    v.proof_ids.iter().next().unwrap().0;
                                                proof_id_mapping.insert(new_id, *cloned_proof_id);
                                                ScryptoValue::from_value(&scrypto::resource::Proof(
                                                    new_id,
                                                ))
                                            })
                                    })
                                    .unwrap_or(Err(RuntimeError::ProofNotFound(*proof_id)))
                            }),
                        ValidatedInstruction::DropProof { proof_id } => proof_id_mapping
                            .remove(proof_id)
                            .map(|real_id| {
                                system_api.invoke_snode2(
                                    SNodeRef::Proof(real_id),
                                    "drop".to_string(),
                                    ScryptoValue::from_value(&ConsumingProofDropInput {}),
                                )
                            })
                            .unwrap_or(Err(ProofNotFound(*proof_id))),
                        ValidatedInstruction::CallFunction {
                            package_address,
                            blueprint_name,
                            call_data,
                        } => {
                            Self::replace_ids(
                                &mut proof_id_mapping,
                                &mut bucket_id_mapping,
                                call_data.clone(),
                            )
                            .and_then(|call_data| {
                                system_api.invoke_snode(
                                    SNodeRef::Scrypto(ScryptoActor::Blueprint(
                                        *package_address,
                                        blueprint_name.to_string(),
                                    )),
                                    call_data,
                                )
                            })
                            .and_then(|result| {
                                // Auto move into auth_zone
                                for (proof_id, _) in &result.proof_ids {
                                    system_api
                                        .invoke_snode2(
                                            SNodeRef::AuthZoneRef,
                                            "push".to_string(),
                                            ScryptoValue::from_value(&AuthZonePushInput {
                                                proof: scrypto::resource::Proof(*proof_id),
                                            }),
                                        )
                                        .unwrap(); // TODO: Remove unwrap
                                }
                                // Auto move into worktop
                                for (bucket_id, _) in &result.bucket_ids {
                                    system_api
                                        .invoke_snode2(
                                            SNodeRef::WorktopRef,
                                            "put".to_string(),
                                            ScryptoValue::from_value(&WorktopPutInput {
                                                bucket: scrypto::resource::Bucket(*bucket_id),
                                            }),
                                        )
                                        .unwrap(); // TODO: Remove unwrap
                                }
                                Ok(result)
                            })
                        }
                        ValidatedInstruction::CallMethod {
                            component_address,
                            call_data,
                        } => {
                            Self::replace_ids(
                                &mut proof_id_mapping,
                                &mut bucket_id_mapping,
                                call_data.clone(),
                            )
                            .and_then(|call_data| {
                                system_api.invoke_snode(
                                    SNodeRef::Scrypto(ScryptoActor::Component(*component_address)),
                                    call_data,
                                )
                            })
                            .and_then(|result| {
                                // Auto move into auth_zone
                                for (proof_id, _) in &result.proof_ids {
                                    system_api
                                        .invoke_snode2(
                                            SNodeRef::AuthZoneRef,
                                            "push".to_string(),
                                            ScryptoValue::from_value(&AuthZonePushInput {
                                                proof: scrypto::resource::Proof(*proof_id),
                                            }),
                                        )
                                        .unwrap();
                                }
                                // Auto move into worktop
                                for (bucket_id, _) in &result.bucket_ids {
                                    system_api
                                        .invoke_snode2(
                                            SNodeRef::WorktopRef,
                                            "put".to_string(),
                                            ScryptoValue::from_value(&WorktopPutInput {
                                                bucket: scrypto::resource::Bucket(*bucket_id),
                                            }),
                                        )
                                        .unwrap(); // TODO: Remove unwrap
                                }
                                Ok(result)
                            })
                        }
                        ValidatedInstruction::CallMethodWithAllResources {
                            component_address,
                            method,
                        } => system_api
                            .invoke_snode2(
                                SNodeRef::AuthZoneRef,
                                "clear".to_string(),
                                ScryptoValue::from_value(&AuthZoneClearInput {}),
                            )
                            .and_then(|_| {
                                for (_, real_id) in proof_id_mapping.drain() {
                                    system_api
                                        .invoke_snode2(
                                            SNodeRef::Proof(real_id),
                                            "drop".to_string(),
                                            ScryptoValue::from_value(&ConsumingProofDropInput {}),
                                        )
                                        .unwrap();
                                }
                                system_api.invoke_snode2(
                                    SNodeRef::WorktopRef,
                                    "drain".to_string(),
                                    ScryptoValue::from_value(&WorktopDrainInput {}),
                                )
                            })
                            .and_then(|result| {
                                let mut buckets = Vec::new();
                                for (bucket_id, _) in result.bucket_ids {
                                    buckets.push(scrypto::resource::Bucket(bucket_id));
                                }
                                for (_, real_id) in bucket_id_mapping.drain() {
                                    buckets.push(scrypto::resource::Bucket(real_id));
                                }
                                let encoded = call_data!(method.to_string(), buckets);
                                system_api.invoke_snode(
                                    SNodeRef::Scrypto(ScryptoActor::Component(*component_address)),
                                    ScryptoValue::from_slice(&encoded).unwrap(),
                                )
                            }),
                        ValidatedInstruction::PublishPackage { package } => {
                            scrypto_decode::<Package>(package)
                                .map_err(|e| RuntimeError::InvalidPackage(e))
                                .and_then(|package| {
                                    system_api.invoke_snode2(
                                        SNodeRef::PackageStatic,
                                        "publish".to_string(),
                                        ScryptoValue::from_value(&PackagePublishInput { package }),
                                    )
                                })
                        }
                    }
                    .map_err(TransactionProcessorError::RuntimeError)?;
                    outputs.push(result);
                }

                Ok(ScryptoValue::from_value(&outputs))
            }
        }
    }
}
