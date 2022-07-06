use sbor::rust::collections::HashMap;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::buffer::scrypto_decode;
use scrypto::component::Package;
use scrypto::core::{SNodeRef, ScryptoActor};
use scrypto::engine::types::*;
use scrypto::prelude::{
    AuthZoneClearInput, AuthZoneCreateProofByAmountInput, AuthZoneCreateProofByIdsInput,
    AuthZoneCreateProofInput, AuthZonePushInput, BucketCreateProofInput, PackagePublishInput,
    ProofCloneInput,
};
use scrypto::resource::{AuthZonePopInput, ConsumingProofDropInput};
use scrypto::to_struct;
use scrypto::values::*;
use transaction::model::*;
use transaction::validation::*;

use crate::engine::{RuntimeError, RuntimeError::ProofNotFound, SystemApi};
use crate::ledger::ReadableSubstateStore;
use crate::model::worktop::{
    WorktopAssertContainsAmountInput, WorktopAssertContainsInput,
    WorktopAssertContainsNonFungiblesInput, WorktopDrainInput, WorktopPutInput,
    WorktopTakeAllInput, WorktopTakeAmountInput, WorktopTakeNonFungiblesInput,
};
use crate::model::TransactionProcessorError::InvalidMethod;
use crate::wasm::*;

use super::Worktop;

#[derive(Debug, TypeId, Encode, Decode)]
pub struct TransactionProcessorRunInput {
    pub instructions: Vec<ExecutableInstruction>,
}

#[derive(Debug)]
pub enum TransactionProcessorError {
    InvalidRequestData(DecodeError),
    RuntimeError(RuntimeError),
    InvalidMethod,
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

    pub fn static_main<
        'p,
        's,
        Y: SystemApi<'p, 's, W, I, S>,
        W: WasmEngine<I>,
        I: WasmInstance,
        S: 's + ReadableSubstateStore,
    >(
        function_name: &str,
        call_data: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, TransactionProcessorError> {
        match function_name {
            "run" => {
                let input: TransactionProcessorRunInput = scrypto_decode(&call_data.raw)
                    .map_err(|e| TransactionProcessorError::InvalidRequestData(e))?;
                let mut proof_id_mapping = HashMap::new();
                let mut bucket_id_mapping = HashMap::new();
                let mut outputs = Vec::new();
                let mut id_allocator = IdAllocator::new(IdSpace::Transaction);
                let mut worktop = Worktop::new();

                for inst in &input.instructions.clone() {
                    let result = match inst {
                        ExecutableInstruction::TakeFromWorktop { resource_address } => id_allocator
                            .new_bucket_id()
                            .map_err(RuntimeError::IdAllocationError)
                            .and_then(|new_id| {
                                worktop
                                    .main(
                                        "take_all",
                                        ScryptoValue::from_typed(&WorktopTakeAllInput {
                                            resource_address: *resource_address,
                                        }),
                                        system_api,
                                    )
                                    .map_err(RuntimeError::WorktopError)
                                    .map(|rtn| {
                                        let bucket_id = *rtn.bucket_ids.iter().next().unwrap().0;
                                        bucket_id_mapping.insert(new_id, bucket_id);
                                        ScryptoValue::from_typed(&scrypto::resource::Bucket(new_id))
                                    })
                            }),
                        ExecutableInstruction::TakeFromWorktopByAmount {
                            amount,
                            resource_address,
                        } => id_allocator
                            .new_bucket_id()
                            .map_err(RuntimeError::IdAllocationError)
                            .and_then(|new_id| {
                                worktop
                                    .main(
                                        "take_amount",
                                        ScryptoValue::from_typed(&WorktopTakeAmountInput {
                                            amount: *amount,
                                            resource_address: *resource_address,
                                        }),
                                        system_api,
                                    )
                                    .map_err(RuntimeError::WorktopError)
                                    .map(|rtn| {
                                        let bucket_id = *rtn.bucket_ids.iter().next().unwrap().0;
                                        bucket_id_mapping.insert(new_id, bucket_id);
                                        ScryptoValue::from_typed(&scrypto::resource::Bucket(new_id))
                                    })
                            }),
                        ExecutableInstruction::TakeFromWorktopByIds {
                            ids,
                            resource_address,
                        } => id_allocator
                            .new_bucket_id()
                            .map_err(RuntimeError::IdAllocationError)
                            .and_then(|new_id| {
                                worktop
                                    .main(
                                        "take_non_fungibles",
                                        ScryptoValue::from_typed(&WorktopTakeNonFungiblesInput {
                                            ids: ids.clone(),
                                            resource_address: *resource_address,
                                        }),
                                        system_api,
                                    )
                                    .map_err(RuntimeError::WorktopError)
                                    .map(|rtn| {
                                        let bucket_id = *rtn.bucket_ids.iter().next().unwrap().0;
                                        bucket_id_mapping.insert(new_id, bucket_id);
                                        ScryptoValue::from_typed(&scrypto::resource::Bucket(new_id))
                                    })
                            }),
                        ExecutableInstruction::ReturnToWorktop { bucket_id } => bucket_id_mapping
                            .remove(bucket_id)
                            .map(|real_id| {
                                worktop
                                    .main(
                                        "put",
                                        ScryptoValue::from_typed(&WorktopPutInput {
                                            bucket: scrypto::resource::Bucket(real_id),
                                        }),
                                        system_api,
                                    )
                                    .map_err(RuntimeError::WorktopError)
                            })
                            .unwrap_or(Err(RuntimeError::BucketNotFound(*bucket_id))),
                        ExecutableInstruction::AssertWorktopContains { resource_address } => {
                            worktop
                                .main(
                                    "assert_contains",
                                    ScryptoValue::from_typed(&WorktopAssertContainsInput {
                                        resource_address: *resource_address,
                                    }),
                                    system_api,
                                )
                                .map_err(RuntimeError::WorktopError)
                        }
                        ExecutableInstruction::AssertWorktopContainsByAmount {
                            amount,
                            resource_address,
                        } => worktop
                            .main(
                                "assert_contains_amount",
                                ScryptoValue::from_typed(&WorktopAssertContainsAmountInput {
                                    amount: *amount,
                                    resource_address: *resource_address,
                                }),
                                system_api,
                            )
                            .map_err(RuntimeError::WorktopError),
                        ExecutableInstruction::AssertWorktopContainsByIds {
                            ids,
                            resource_address,
                        } => worktop
                            .main(
                                "assert_contains_non_fungibles",
                                ScryptoValue::from_typed(&WorktopAssertContainsNonFungiblesInput {
                                    ids: ids.clone(),
                                    resource_address: *resource_address,
                                }),
                                system_api,
                            )
                            .map_err(RuntimeError::WorktopError),

                        ExecutableInstruction::PopFromAuthZone {} => id_allocator
                            .new_proof_id()
                            .map_err(RuntimeError::IdAllocationError)
                            .and_then(|new_id| {
                                system_api
                                    .invoke_snode(
                                        SNodeRef::AuthZoneRef,
                                        "pop".to_string(),
                                        ScryptoValue::from_typed(&AuthZonePopInput {}),
                                    )
                                    .map(|rtn| {
                                        let proof_id = *rtn.proof_ids.iter().next().unwrap().0;
                                        proof_id_mapping.insert(new_id, proof_id);
                                        ScryptoValue::from_typed(&scrypto::resource::Proof(new_id))
                                    })
                            }),
                        ExecutableInstruction::ClearAuthZone => {
                            proof_id_mapping.clear();
                            system_api.invoke_snode(
                                SNodeRef::AuthZoneRef,
                                "clear".to_string(),
                                ScryptoValue::from_typed(&AuthZoneClearInput {}),
                            )
                        }
                        ExecutableInstruction::PushToAuthZone { proof_id } => proof_id_mapping
                            .remove(proof_id)
                            .ok_or(RuntimeError::ProofNotFound(*proof_id))
                            .and_then(|real_id| {
                                system_api.invoke_snode(
                                    SNodeRef::AuthZoneRef,
                                    "push".to_string(),
                                    ScryptoValue::from_typed(&AuthZonePushInput {
                                        proof: scrypto::resource::Proof(real_id),
                                    }),
                                )
                            }),
                        ExecutableInstruction::CreateProofFromAuthZone { resource_address } => {
                            id_allocator
                                .new_proof_id()
                                .map_err(RuntimeError::IdAllocationError)
                                .and_then(|new_id| {
                                    system_api
                                        .invoke_snode(
                                            SNodeRef::AuthZoneRef,
                                            "create_proof".to_string(),
                                            ScryptoValue::from_typed(&AuthZoneCreateProofInput {
                                                resource_address: *resource_address,
                                            }),
                                        )
                                        .map(|rtn| {
                                            let proof_id = *rtn.proof_ids.iter().next().unwrap().0;
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
                            .map_err(RuntimeError::IdAllocationError)
                            .and_then(|new_id| {
                                system_api
                                    .invoke_snode(
                                        SNodeRef::AuthZoneRef,
                                        "create_proof_by_amount".to_string(),
                                        ScryptoValue::from_typed(
                                            &AuthZoneCreateProofByAmountInput {
                                                amount: *amount,
                                                resource_address: *resource_address,
                                            },
                                        ),
                                    )
                                    .map(|rtn| {
                                        let proof_id = *rtn.proof_ids.iter().next().unwrap().0;
                                        proof_id_mapping.insert(new_id, proof_id);
                                        ScryptoValue::from_typed(&scrypto::resource::Proof(new_id))
                                    })
                            }),
                        ExecutableInstruction::CreateProofFromAuthZoneByIds {
                            ids,
                            resource_address,
                        } => id_allocator
                            .new_proof_id()
                            .map_err(RuntimeError::IdAllocationError)
                            .and_then(|new_id| {
                                system_api
                                    .invoke_snode(
                                        SNodeRef::AuthZoneRef,
                                        "create_proof_by_ids".to_string(),
                                        ScryptoValue::from_typed(&AuthZoneCreateProofByIdsInput {
                                            ids: ids.clone(),
                                            resource_address: *resource_address,
                                        }),
                                    )
                                    .map(|rtn| {
                                        let proof_id = *rtn.proof_ids.iter().next().unwrap().0;
                                        proof_id_mapping.insert(new_id, proof_id);
                                        ScryptoValue::from_typed(&scrypto::resource::Proof(new_id))
                                    })
                            }),
                        ExecutableInstruction::CreateProofFromBucket { bucket_id } => id_allocator
                            .new_proof_id()
                            .map_err(RuntimeError::IdAllocationError)
                            .and_then(|new_id| {
                                bucket_id_mapping
                                    .get(bucket_id)
                                    .cloned()
                                    .map(|real_bucket_id| (new_id, real_bucket_id))
                                    .ok_or(RuntimeError::BucketNotFound(new_id))
                            })
                            .and_then(|(new_id, real_bucket_id)| {
                                system_api
                                    .invoke_snode(
                                        SNodeRef::BucketRef(real_bucket_id),
                                        "create_proof".to_string(),
                                        ScryptoValue::from_typed(&BucketCreateProofInput {}),
                                    )
                                    .map(|rtn| {
                                        let proof_id = *rtn.proof_ids.iter().next().unwrap().0;
                                        proof_id_mapping.insert(new_id, proof_id);
                                        ScryptoValue::from_typed(&scrypto::resource::Proof(new_id))
                                    })
                            }),
                        ExecutableInstruction::CloneProof { proof_id } => id_allocator
                            .new_proof_id()
                            .map_err(RuntimeError::IdAllocationError)
                            .and_then(|new_id| {
                                proof_id_mapping
                                    .get(proof_id)
                                    .cloned()
                                    .map(|real_id| {
                                        system_api
                                            .invoke_snode(
                                                SNodeRef::ProofRef(real_id),
                                                "clone".to_string(),
                                                ScryptoValue::from_typed(&ProofCloneInput {}),
                                            )
                                            .map(|v| {
                                                let cloned_proof_id =
                                                    v.proof_ids.iter().next().unwrap().0;
                                                proof_id_mapping.insert(new_id, *cloned_proof_id);
                                                ScryptoValue::from_typed(&scrypto::resource::Proof(
                                                    new_id,
                                                ))
                                            })
                                    })
                                    .unwrap_or(Err(RuntimeError::ProofNotFound(*proof_id)))
                            }),
                        ExecutableInstruction::DropProof { proof_id } => proof_id_mapping
                            .remove(proof_id)
                            .map(|real_id| {
                                system_api.invoke_snode(
                                    SNodeRef::Consumed(ValueId::Transient(
                                        TransientValueId::Proof(real_id),
                                    )),
                                    "drop".to_string(),
                                    ScryptoValue::from_typed(&ConsumingProofDropInput {}),
                                )
                            })
                            .unwrap_or(Err(ProofNotFound(*proof_id))),
                        ExecutableInstruction::CallFunction {
                            package_address,
                            blueprint_name,
                            method_name,
                            arg,
                        } => {
                            Self::replace_ids(
                                &mut proof_id_mapping,
                                &mut bucket_id_mapping,
                                ScryptoValue::from_slice(arg).expect("Should be valid arg"),
                            )
                            .and_then(|call_data| {
                                system_api.invoke_snode(
                                    SNodeRef::Scrypto(ScryptoActor::Blueprint(
                                        *package_address,
                                        blueprint_name.to_string(),
                                    )),
                                    method_name.to_string(),
                                    call_data,
                                )
                            })
                            .and_then(|result| {
                                // Auto move into auth_zone
                                for (proof_id, _) in &result.proof_ids {
                                    system_api
                                        .invoke_snode(
                                            SNodeRef::AuthZoneRef,
                                            "push".to_string(),
                                            ScryptoValue::from_typed(&AuthZonePushInput {
                                                proof: scrypto::resource::Proof(*proof_id),
                                            }),
                                        )
                                        .unwrap(); // TODO: Remove unwrap
                                }
                                // Auto move into worktop
                                for (bucket_id, _) in &result.bucket_ids {
                                    worktop
                                        .main(
                                            "put",
                                            ScryptoValue::from_typed(&WorktopPutInput {
                                                bucket: scrypto::resource::Bucket(*bucket_id),
                                            }),
                                            system_api,
                                        )
                                        .map_err(RuntimeError::WorktopError)
                                        .unwrap(); // TODO: Remove unwrap
                                }
                                Ok(result)
                            })
                        }
                        ExecutableInstruction::CallMethod {
                            component_address,
                            method_name,
                            arg,
                        } => {
                            Self::replace_ids(
                                &mut proof_id_mapping,
                                &mut bucket_id_mapping,
                                ScryptoValue::from_slice(arg).expect("Should be valid arg"),
                            )
                            .and_then(|call_data| {
                                system_api.invoke_snode(
                                    SNodeRef::Scrypto(ScryptoActor::Component(*component_address)),
                                    method_name.to_string(),
                                    call_data,
                                )
                            })
                            .and_then(|result| {
                                // Auto move into auth_zone
                                for (proof_id, _) in &result.proof_ids {
                                    system_api
                                        .invoke_snode(
                                            SNodeRef::AuthZoneRef,
                                            "push".to_string(),
                                            ScryptoValue::from_typed(&AuthZonePushInput {
                                                proof: scrypto::resource::Proof(*proof_id),
                                            }),
                                        )
                                        .unwrap();
                                }
                                // Auto move into worktop
                                for (bucket_id, _) in &result.bucket_ids {
                                    worktop
                                        .main(
                                            "put",
                                            ScryptoValue::from_typed(&WorktopPutInput {
                                                bucket: scrypto::resource::Bucket(*bucket_id),
                                            }),
                                            system_api,
                                        )
                                        .map_err(RuntimeError::WorktopError)
                                        .unwrap(); // TODO: Remove unwrap
                                }
                                Ok(result)
                            })
                        }
                        ExecutableInstruction::CallMethodWithAllResources {
                            component_address,
                            method,
                        } => system_api
                            .invoke_snode(
                                SNodeRef::AuthZoneRef,
                                "clear".to_string(),
                                ScryptoValue::from_typed(&AuthZoneClearInput {}),
                            )
                            .and_then(|_| {
                                for (_, real_id) in proof_id_mapping.drain() {
                                    system_api
                                        .invoke_snode(
                                            SNodeRef::Consumed(ValueId::Transient(
                                                TransientValueId::Proof(real_id),
                                            )),
                                            "drop".to_string(),
                                            ScryptoValue::from_typed(&ConsumingProofDropInput {}),
                                        )
                                        .unwrap();
                                }
                                worktop
                                    .main(
                                        "drain",
                                        ScryptoValue::from_typed(&WorktopDrainInput {}),
                                        system_api,
                                    )
                                    .map_err(RuntimeError::WorktopError)
                            })
                            .and_then(|result| {
                                let mut buckets = Vec::new();
                                for (bucket_id, _) in result.bucket_ids {
                                    buckets.push(scrypto::resource::Bucket(bucket_id));
                                }
                                for (_, real_id) in bucket_id_mapping.drain() {
                                    buckets.push(scrypto::resource::Bucket(real_id));
                                }
                                let encoded = to_struct!(buckets);
                                system_api.invoke_snode(
                                    SNodeRef::Scrypto(ScryptoActor::Component(*component_address)),
                                    method.to_string(),
                                    ScryptoValue::from_slice(&encoded).unwrap(),
                                )
                            }),
                        ExecutableInstruction::PublishPackage { package } => {
                            scrypto_decode::<Package>(package)
                                .map_err(|e| RuntimeError::InvalidPackage(e))
                                .and_then(|package| {
                                    system_api.invoke_snode(
                                        SNodeRef::PackageStatic,
                                        "publish".to_string(),
                                        ScryptoValue::from_typed(&PackagePublishInput { package }),
                                    )
                                })
                        }
                    }
                    .map_err(TransactionProcessorError::RuntimeError)?;
                    outputs.push(result);
                }

                // This creates frame-owned buckets for all non-zero balances, which triggers a
                // value drop failure when the frame exits.
                //
                // TODO: refactor worktop to be `HashMap<ResourceAddress, BucketId>`
                // TODO: remove this drain invocation by enforcing no non-empty bucket in worktop
                worktop
                    .main(
                        "drain",
                        ScryptoValue::from_typed(&WorktopDrainInput {}),
                        system_api,
                    )
                    .map_err(RuntimeError::WorktopError)
                    .map_err(TransactionProcessorError::RuntimeError)?;

                Ok(ScryptoValue::from_typed(
                    &outputs
                        .into_iter()
                        .map(|sv| sv.raw)
                        .collect::<Vec<Vec<u8>>>(),
                ))
            }
            _ => Err(InvalidMethod),
        }
    }
}
