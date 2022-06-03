use sbor::rust::collections::HashMap;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::buffer::scrypto_decode;
use scrypto::call_data;
use scrypto::component::{Package, PackageFunction};
use scrypto::core::{SNodeRef, ScryptoActor};
use scrypto::engine::types::*;
use scrypto::prelude::{BucketCreateProofInput, ProofCloneInput};
use scrypto::resource::{AuthZoneMethod, ConsumingProofDropInput};
use scrypto::values::*;
use transaction::model::*;
use transaction::validation::*;

use crate::engine::{RuntimeError, RuntimeError::ProofNotFound, SystemApi};
use crate::model::worktop::WorktopMethod;
use crate::wasm::*;

#[derive(Debug, TypeId, Encode, Decode)]
pub enum TransactionProcessorFunction {
    Run(Vec<ExecutableInstruction>),
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
            TransactionProcessorFunction::Run(validated_instructions) => {
                let mut proof_id_mapping = HashMap::new();
                let mut bucket_id_mapping = HashMap::new();
                let mut outputs = Vec::new();
                let mut id_allocator = IdAllocator::new(IdSpace::Transaction);

                for inst in &validated_instructions {
                    let result = match inst {
                        ExecutableInstruction::TakeFromWorktop { resource_address } => id_allocator
                            .new_bucket_id()
                            .map_err(RuntimeError::IdAllocationError)
                            .and_then(|new_id| {
                                system_api
                                    .invoke_snode(
                                        SNodeRef::WorktopRef,
                                        ScryptoValue::from_value(&WorktopMethod::TakeAll(
                                            *resource_address,
                                        )),
                                    )
                                    .map(|rtn| {
                                        let bucket_id = *rtn.bucket_ids.iter().next().unwrap().0;
                                        bucket_id_mapping.insert(new_id, bucket_id);
                                        ScryptoValue::from_value(&scrypto::resource::Bucket(new_id))
                                    })
                            }),
                        ExecutableInstruction::TakeFromWorktopByAmount {
                            amount,
                            resource_address,
                        } => id_allocator
                            .new_bucket_id()
                            .map_err(RuntimeError::IdAllocationError)
                            .and_then(|new_id| {
                                system_api
                                    .invoke_snode(
                                        SNodeRef::WorktopRef,
                                        ScryptoValue::from_value(&WorktopMethod::TakeAmount(
                                            *amount,
                                            *resource_address,
                                        )),
                                    )
                                    .map(|rtn| {
                                        let bucket_id = *rtn.bucket_ids.iter().next().unwrap().0;
                                        bucket_id_mapping.insert(new_id, bucket_id);
                                        ScryptoValue::from_value(&scrypto::resource::Bucket(new_id))
                                    })
                            }),
                        ExecutableInstruction::TakeFromWorktopByIds {
                            ids,
                            resource_address,
                        } => id_allocator
                            .new_bucket_id()
                            .map_err(RuntimeError::IdAllocationError)
                            .and_then(|new_id| {
                                system_api
                                    .invoke_snode(
                                        SNodeRef::WorktopRef,
                                        ScryptoValue::from_value(&WorktopMethod::TakeNonFungibles(
                                            ids.clone(),
                                            *resource_address,
                                        )),
                                    )
                                    .map(|rtn| {
                                        let bucket_id = *rtn.bucket_ids.iter().next().unwrap().0;
                                        bucket_id_mapping.insert(new_id, bucket_id);
                                        ScryptoValue::from_value(&scrypto::resource::Bucket(new_id))
                                    })
                            }),
                        ExecutableInstruction::ReturnToWorktop { bucket_id } => bucket_id_mapping
                            .remove(bucket_id)
                            .map(|real_id| {
                                system_api.invoke_snode(
                                    SNodeRef::WorktopRef,
                                    ScryptoValue::from_value(&WorktopMethod::Put(
                                        scrypto::resource::Bucket(real_id),
                                    )),
                                )
                            })
                            .unwrap_or(Err(RuntimeError::BucketNotFound(*bucket_id))),
                        ExecutableInstruction::AssertWorktopContains { resource_address } => {
                            system_api.invoke_snode(
                                SNodeRef::WorktopRef,
                                ScryptoValue::from_value(&WorktopMethod::AssertContains(
                                    *resource_address,
                                )),
                            )
                        }
                        ExecutableInstruction::AssertWorktopContainsByAmount {
                            amount,
                            resource_address,
                        } => system_api.invoke_snode(
                            SNodeRef::WorktopRef,
                            ScryptoValue::from_value(&WorktopMethod::AssertContainsAmount(
                                *amount,
                                *resource_address,
                            )),
                        ),
                        ExecutableInstruction::AssertWorktopContainsByIds {
                            ids,
                            resource_address,
                        } => system_api.invoke_snode(
                            SNodeRef::WorktopRef,
                            ScryptoValue::from_value(&WorktopMethod::AssertContainsNonFungibles(
                                ids.clone(),
                                *resource_address,
                            )),
                        ),
                        ExecutableInstruction::PopFromAuthZone {} => id_allocator
                            .new_proof_id()
                            .map_err(RuntimeError::IdAllocationError)
                            .and_then(|new_id| {
                                system_api
                                    .invoke_snode(
                                        SNodeRef::AuthZoneRef,
                                        ScryptoValue::from_value(&AuthZoneMethod::Pop()),
                                    )
                                    .map(|rtn| {
                                        let proof_id = *rtn.proof_ids.iter().next().unwrap().0;
                                        proof_id_mapping.insert(new_id, proof_id);
                                        ScryptoValue::from_value(&scrypto::resource::Proof(new_id))
                                    })
                            }),
                        ExecutableInstruction::ClearAuthZone => {
                            proof_id_mapping.clear();
                            system_api.invoke_snode(
                                SNodeRef::AuthZoneRef,
                                ScryptoValue::from_value(&AuthZoneMethod::Clear()),
                            )
                        }
                        ExecutableInstruction::PushToAuthZone { proof_id } => proof_id_mapping
                            .remove(proof_id)
                            .ok_or(RuntimeError::ProofNotFound(*proof_id))
                            .and_then(|real_id| {
                                system_api.invoke_snode(
                                    SNodeRef::AuthZoneRef,
                                    ScryptoValue::from_value(&AuthZoneMethod::Push(
                                        scrypto::resource::Proof(real_id),
                                    )),
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
                                            ScryptoValue::from_value(&AuthZoneMethod::CreateProof(
                                                *resource_address,
                                            )),
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
                                        ScryptoValue::from_value(
                                            &AuthZoneMethod::CreateProofByAmount(
                                                *amount,
                                                *resource_address,
                                            ),
                                        ),
                                    )
                                    .map(|rtn| {
                                        let proof_id = *rtn.proof_ids.iter().next().unwrap().0;
                                        proof_id_mapping.insert(new_id, proof_id);
                                        ScryptoValue::from_value(&scrypto::resource::Proof(new_id))
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
                                        ScryptoValue::from_value(
                                            &AuthZoneMethod::CreateProofByIds(
                                                ids.clone(),
                                                *resource_address,
                                            ),
                                        ),
                                    )
                                    .map(|rtn| {
                                        let proof_id = *rtn.proof_ids.iter().next().unwrap().0;
                                        proof_id_mapping.insert(new_id, proof_id);
                                        ScryptoValue::from_value(&scrypto::resource::Proof(new_id))
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
                        ExecutableInstruction::CloneProof { proof_id } => id_allocator
                            .new_proof_id()
                            .map_err(RuntimeError::IdAllocationError)
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
                        ExecutableInstruction::DropProof { proof_id } => proof_id_mapping
                            .remove(proof_id)
                            .map(|real_id| {
                                system_api.invoke_snode2(
                                    SNodeRef::Proof(real_id),
                                    "drop".to_string(),
                                    ScryptoValue::from_value(&ConsumingProofDropInput {}),
                                )
                            })
                            .unwrap_or(Err(ProofNotFound(*proof_id))),
                        ExecutableInstruction::CallFunction {
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
                                        .invoke_snode(
                                            SNodeRef::AuthZoneRef,
                                            ScryptoValue::from_value(&AuthZoneMethod::Push(
                                                scrypto::resource::Proof(*proof_id),
                                            )),
                                        )
                                        .unwrap(); // TODO: Remove unwrap
                                }
                                // Auto move into worktop
                                for (bucket_id, _) in &result.bucket_ids {
                                    system_api
                                        .invoke_snode(
                                            SNodeRef::WorktopRef,
                                            ScryptoValue::from_value(&WorktopMethod::Put(
                                                scrypto::resource::Bucket(*bucket_id),
                                            )),
                                        )
                                        .unwrap(); // TODO: Remove unwrap
                                }
                                Ok(result)
                            })
                        }
                        ExecutableInstruction::CallMethod {
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
                                        .invoke_snode(
                                            SNodeRef::AuthZoneRef,
                                            ScryptoValue::from_value(&AuthZoneMethod::Push(
                                                scrypto::resource::Proof(*proof_id),
                                            )),
                                        )
                                        .unwrap();
                                }
                                // Auto move into worktop
                                for (bucket_id, _) in &result.bucket_ids {
                                    system_api
                                        .invoke_snode(
                                            SNodeRef::WorktopRef,
                                            ScryptoValue::from_value(&WorktopMethod::Put(
                                                scrypto::resource::Bucket(*bucket_id),
                                            )),
                                        )
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
                                ScryptoValue::from_value(&AuthZoneMethod::Clear()),
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
                                system_api.invoke_snode(
                                    SNodeRef::WorktopRef,
                                    ScryptoValue::from_value(&WorktopMethod::Drain()),
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
                        ExecutableInstruction::PublishPackage { package } => {
                            scrypto_decode::<Package>(package)
                                .map_err(|e| RuntimeError::InvalidPackage(e))
                                .and_then(|package| {
                                    system_api.invoke_snode(
                                        SNodeRef::PackageStatic,
                                        ScryptoValue::from_value(&PackageFunction::Publish(
                                            package,
                                        )),
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
