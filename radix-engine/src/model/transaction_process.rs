use scrypto::core::SNodeRef;
use scrypto::engine::types::*;
use scrypto::prelude::ScryptoActor;
use scrypto::rust::collections::{HashMap};
use scrypto::rust::string::ToString;
use scrypto::rust::vec;
use scrypto::rust::vec::Vec;
use scrypto::values::*;
use crate::engine::{IdAllocator, IdSpace, Process};
use crate::errors::RuntimeError::{ProofNotFound};
use crate::errors::RuntimeError;
use crate::ledger::SubstateStore;
use crate::model::{ValidatedInstruction, ValidatedTransaction};

pub struct TransactionProcess {
    transaction: ValidatedTransaction,
    proof_id_mapping: HashMap<ProofId, ProofId>,
    bucket_id_mapping: HashMap<BucketId, BucketId>,
    id_allocator: IdAllocator,
}

pub enum TransactionError {
    Error {
        error: RuntimeError,
        outputs: Vec<Vec<u8>>,
    }
}

impl TransactionProcess {
    pub fn new(transaction: ValidatedTransaction) -> Self {
        Self {
            transaction,
            proof_id_mapping: HashMap::new(),
            bucket_id_mapping: HashMap::new(),
            id_allocator: IdAllocator::new(IdSpace::Transaction),
        }
    }

    fn replace_ids(
        &mut self,
        mut values: Vec<ScryptoValue>,
    ) -> Result<Vec<ScryptoValue>, RuntimeError> {
        for value in values.iter_mut() {
            value.replace_ids(&mut self.proof_id_mapping, &mut self.bucket_id_mapping)
                .map_err(|e| match e {
                    ScryptoValueReplaceError::BucketIdNotFound(bucket_id) => RuntimeError::BucketNotFound(bucket_id),
                    ScryptoValueReplaceError::ProofIdNotFound(proof_id) => RuntimeError::ProofNotFound(proof_id),
                })?;
        }
        Ok(values)
    }

    pub fn main<L: SubstateStore>(mut self, proc: &mut Process<L>) -> Result<Vec<Vec<u8>>, TransactionError> {

        let mut outputs = vec![];

        for inst in &self.transaction.instructions.clone() {
            let result = match inst {
                ValidatedInstruction::TakeFromWorktop { resource_address } => {
                    self.id_allocator.new_bucket_id()
                        .map_err(RuntimeError::IdAllocatorError)
                        .and_then(|new_id| {
                            proc.call(
                                SNodeRef::WorktopRef,
                                "take_all".to_string(),
                                vec![
                                    ScryptoValue::from_value(resource_address),
                                ]
                            ).map(|rtn| {
                                let bucket_id = *rtn.bucket_ids.iter().next().unwrap().0;
                                self.bucket_id_mapping.insert(new_id, bucket_id);
                                ScryptoValue::from_value(&scrypto::resource::Bucket(new_id))
                            })
                        })
                },
                ValidatedInstruction::TakeFromWorktopByAmount {
                    amount,
                    resource_address,
                } =>
                    self.id_allocator
                        .new_bucket_id()
                        .map_err(RuntimeError::IdAllocatorError)
                        .and_then(|new_id| {
                            proc.call(
                                SNodeRef::WorktopRef,
                                "take_amount".to_string(),
                                vec![
                                    ScryptoValue::from_value(amount),
                                    ScryptoValue::from_value(resource_address),
                                ]
                            ).map(|rtn| {
                                let bucket_id = *rtn.bucket_ids.iter().next().unwrap().0;
                                self.bucket_id_mapping.insert(new_id, bucket_id);
                                ScryptoValue::from_value(&scrypto::resource::Bucket(new_id))
                            })
                        }),
                ValidatedInstruction::TakeFromWorktopByIds {
                    ids,
                    resource_address,
                } =>
                    self.id_allocator
                        .new_bucket_id()
                        .map_err(RuntimeError::IdAllocatorError)
                        .and_then(|new_id| {
                            proc.call(
                                SNodeRef::WorktopRef,
                                "take_non_fungibles".to_string(),
                                vec![
                                    ScryptoValue::from_value(ids),
                                    ScryptoValue::from_value(resource_address),
                                ]
                            ).map(|rtn| {
                                let bucket_id = *rtn.bucket_ids.iter().next().unwrap().0;
                                self.bucket_id_mapping.insert(new_id, bucket_id);
                                ScryptoValue::from_value(&scrypto::resource::Bucket(new_id))
                            })
                        }),
                ValidatedInstruction::ReturnToWorktop { bucket_id } => {
                    self.bucket_id_mapping.remove(bucket_id)
                        .map(|real_id| {
                            proc.call(
                                SNodeRef::WorktopRef,
                                "put".to_string(),
                                vec![
                                    ScryptoValue::from_value(&scrypto::resource::Bucket(real_id)),
                                ]
                            )
                        })
                        .unwrap_or(Err(RuntimeError::BucketNotFound(*bucket_id)))
                }
                ValidatedInstruction::AssertWorktopContains { resource_address } => {
                    proc.call(
                        SNodeRef::WorktopRef,
                        "assert_contains".to_string(),
                        vec![
                            ScryptoValue::from_value(resource_address),
                        ]
                    )
                }
                ValidatedInstruction::AssertWorktopContainsByAmount {
                    amount,
                    resource_address,
                } => {
                    proc.call(
                        SNodeRef::WorktopRef,
                        "assert_contains_amount".to_string(),
                        vec![
                            ScryptoValue::from_value(amount),
                            ScryptoValue::from_value(resource_address),
                        ]
                    )
                },
                ValidatedInstruction::AssertWorktopContainsByIds {
                    ids,
                    resource_address,
                } => {
                    proc.call(
                        SNodeRef::WorktopRef,
                        "assert_contains_amount".to_string(),
                        vec![
                            ScryptoValue::from_value(ids),
                            ScryptoValue::from_value(resource_address),
                        ]
                    )
                },
                ValidatedInstruction::PopFromAuthZone {} => {
                    self.id_allocator.new_proof_id()
                        .map_err(RuntimeError::IdAllocatorError)
                        .and_then(|new_id| {
                            proc.call(
                                SNodeRef::AuthZoneRef,
                                "pop".to_string(),
                                vec![]
                            ).map(|rtn| {
                                let proof_id = *rtn.proof_ids.iter().next().unwrap().0;
                                self.proof_id_mapping.insert(new_id, proof_id);
                                ScryptoValue::from_value(&scrypto::resource::Proof(new_id))
                            })
                        })
                },
                ValidatedInstruction::ClearAuthZone => {
                    self.proof_id_mapping.clear();
                    proc.call(SNodeRef::AuthZoneRef, "clear".to_string(), vec![])
                },
                ValidatedInstruction::PushToAuthZone { proof_id } => {
                    self.proof_id_mapping.remove(proof_id)
                        .ok_or(RuntimeError::ProofNotFound(*proof_id))
                        .and_then(|real_id|
                            proc.call(
                                SNodeRef::AuthZoneRef,
                                "push".to_string(),
                                vec![ScryptoValue::from_value(&scrypto::resource::Proof(real_id))]
                            )
                        )
                },
                ValidatedInstruction::CreateProofFromAuthZone { resource_address } =>
                    self.id_allocator.new_proof_id()
                        .map_err(RuntimeError::IdAllocatorError)
                        .and_then(|new_id| {
                            proc.call(
                                SNodeRef::AuthZoneRef,
                                "create_proof".to_string(),
                                vec![ScryptoValue::from_value(resource_address)]
                            ).map(|rtn| {
                                let proof_id = *rtn.proof_ids.iter().next().unwrap().0;
                                self.proof_id_mapping.insert(new_id, proof_id);
                                ScryptoValue::from_value(&scrypto::resource::Proof(new_id))
                            })
                        }),
                ValidatedInstruction::CreateProofFromAuthZoneByAmount {
                    amount,
                    resource_address,
                } =>
                    self.id_allocator.new_proof_id()
                        .map_err(RuntimeError::IdAllocatorError)
                        .and_then(|new_id| {
                            proc.call(
                                SNodeRef::AuthZoneRef,
                                "create_proof_by_amount".to_string(),
                                vec![
                                    ScryptoValue::from_value(amount),
                                    ScryptoValue::from_value(resource_address)
                                ]
                            ).map(|rtn| {
                                let proof_id = *rtn.proof_ids.iter().next().unwrap().0;
                                self.proof_id_mapping.insert(new_id, proof_id);
                                ScryptoValue::from_value(&scrypto::resource::Proof(new_id))
                            })
                        }),
                ValidatedInstruction::CreateProofFromAuthZoneByIds {
                    ids,
                    resource_address,
                } =>
                    self.id_allocator.new_proof_id()
                        .map_err(RuntimeError::IdAllocatorError)
                        .and_then(|new_id| {
                            proc.call(
                                SNodeRef::AuthZoneRef,
                                "create_proof_by_ids".to_string(),
                                vec![
                                    ScryptoValue::from_value(ids),
                                    ScryptoValue::from_value(resource_address)
                                ]
                            ).map(|rtn| {
                                let proof_id = *rtn.proof_ids.iter().next().unwrap().0;
                                self.proof_id_mapping.insert(new_id, proof_id);
                                ScryptoValue::from_value(&scrypto::resource::Proof(new_id))
                            })
                        }),
                ValidatedInstruction::CreateProofFromBucket { bucket_id } => {
                    self.id_allocator.new_proof_id()
                        .map_err(RuntimeError::IdAllocatorError)
                        .and_then(|new_id| {
                            self.bucket_id_mapping.get(bucket_id).cloned()
                                .map(|real_bucket_id| (new_id, real_bucket_id))
                                .ok_or(RuntimeError::BucketNotFound(new_id))
                        })
                        .and_then(|(new_id, real_bucket_id)| {
                            proc.call(
                                SNodeRef::BucketRef(real_bucket_id),
                                "create_bucket_proof".to_string(),
                                vec![],
                            ).map(|rtn| {
                                let proof_id = *rtn.proof_ids.iter().next().unwrap().0;
                                self.proof_id_mapping.insert(new_id, proof_id);
                                ScryptoValue::from_value(&scrypto::resource::Proof(new_id))
                            })
                        })
                },
                ValidatedInstruction::CloneProof { proof_id } =>
                    self.id_allocator
                        .new_proof_id()
                        .map_err(RuntimeError::IdAllocatorError)
                        .and_then(|new_id| {
                            self.proof_id_mapping
                                .get(proof_id)
                                .cloned()
                                .map(|real_id| {
                                    proc.call(SNodeRef::ProofRef(real_id),
                                              "clone".to_string(),
                                              vec![]
                                    ).map(|v| {
                                        let cloned_proof_id = v.proof_ids.iter().next().unwrap().0;
                                        self.proof_id_mapping.insert(new_id, *cloned_proof_id);
                                        ScryptoValue::from_value(&scrypto::resource::Proof(new_id))
                                    })
                                })
                                .unwrap_or(Err(RuntimeError::ProofNotFound(*proof_id)))
                        }),
                ValidatedInstruction::DropProof { proof_id } => {
                    self.proof_id_mapping.remove(proof_id)
                        .map(|real_id| {
                            proc.call(
                                SNodeRef::Proof(real_id),
                                "drop".to_string(),
                                vec![]
                            )
                        })
                        .unwrap_or(Err(ProofNotFound(*proof_id)))
                },
                ValidatedInstruction::CallFunction {
                    package_address,
                    blueprint_name,
                    function,
                    args,
                } => {
                    self.replace_ids(args.clone())
                        .and_then(|args|
                            proc.call(
                                SNodeRef::Scrypto(ScryptoActor::Blueprint(*package_address, blueprint_name.to_string())),
                                function.to_string(),
                                args
                            )
                        )
                        .and_then(|result| {
                            // Auto move into auth_zone
                            for (proof_id, _) in &result.proof_ids {
                                proc.call(
                                    SNodeRef::AuthZoneRef,
                                    "push".to_string(),
                                    vec![ScryptoValue::from_value(&scrypto::resource::Proof(*proof_id))]
                                ).unwrap(); // TODO: Remove unwrap
                            }
                            // Auto move into worktop
                            for (bucket_id, _) in &result.bucket_ids {
                                proc.call(
                                    SNodeRef::WorktopRef,
                                    "put".to_string(),
                                    vec![ScryptoValue::from_value(&scrypto::resource::Bucket(*bucket_id))]
                                ).unwrap(); // TODO: Remove unwrap
                            }
                            Ok(result)
                        })
                },
                ValidatedInstruction::CallMethod {
                    component_address,
                    method,
                    args,
                } => {
                    self.replace_ids(args.clone())
                        .and_then(|args|
                            proc.call(
                                SNodeRef::Scrypto(ScryptoActor::Component(*component_address)),
                                method.to_string(),
                                args
                            )
                        )
                        .and_then(|result| {
                            // Auto move into auth_zone
                            for (proof_id, _) in &result.proof_ids {
                                proc.call(
                                    SNodeRef::AuthZoneRef,
                                    "push".to_string(),
                                    vec![ScryptoValue::from_value(&scrypto::resource::Proof(*proof_id))]
                                ).unwrap();
                            }
                            // Auto move into worktop
                            for (bucket_id, _) in &result.bucket_ids {
                                proc.call(
                                    SNodeRef::WorktopRef,
                                    "put".to_string(),
                                    vec![ScryptoValue::from_value(&scrypto::resource::Bucket(*bucket_id))]
                                ).unwrap(); // TODO: Remove unwrap
                            }
                            Ok(result)
                        })
                },
                ValidatedInstruction::CallMethodWithAllResources {
                    component_address,
                    method,
                } => {
                    proc.call(SNodeRef::AuthZoneRef, "clear".to_string(), vec![])
                        .and_then(|_| {
                            for (_, real_id) in self.proof_id_mapping.drain() {
                                proc.call(
                                    SNodeRef::Proof(real_id),
                                    "drop".to_string(),
                                    vec![]
                                ).unwrap();
                            }
                            proc.call(SNodeRef::WorktopRef, "drain".to_string(), vec![])
                        })
                        .and_then(|result| {
                            let mut buckets = Vec::new();
                            for (bucket_id, _) in result.bucket_ids {
                                buckets.push(scrypto::resource::Bucket(bucket_id));
                            }
                            for (_, real_id) in self.bucket_id_mapping.drain() {
                                buckets.push(scrypto::resource::Bucket(real_id));
                            }
                            proc.call(
                                SNodeRef::Scrypto(ScryptoActor::Component(*component_address)),
                                method.to_string(),
                                vec![ScryptoValue::from_value(&buckets)],
                            )
                        })
                },
                ValidatedInstruction::PublishPackage { code } => {
                    proc.call(
                        SNodeRef::PackageStatic,
                        "publish".to_string(),
                        vec![ScryptoValue::from_value(code)],
                    )
                },
            };
            match result {
                Ok(data) => {
                    outputs.push(data.raw);
                }
                Err(e) => {
                    return Err(TransactionError::Error { error: e, outputs });
                }
            }
        }

        proc.drop_all_proofs().map_err(|e| TransactionError::Error { error: e, outputs: outputs.clone() })?;
        proc.check_resource().map_err(|e| TransactionError::Error { error: e, outputs: outputs.clone() })?;

        Ok(outputs)
    }
}