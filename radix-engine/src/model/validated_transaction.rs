use scrypto::crypto::*;
use scrypto::engine::types::*;
use scrypto::rust::collections::BTreeSet;
use scrypto::rust::string::String;
use scrypto::rust::vec;
use scrypto::rust::vec::Vec;
use scrypto::values::*;
use crate::engine::Process;
use crate::errors::RuntimeError;
use crate::ledger::SubstateStore;

/// Represents a validated transaction
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedTransaction {
    pub raw_hash: Hash,
    pub instructions: Vec<ValidatedInstruction>,
    pub signers: Vec<EcdsaPublicKey>,
}

impl ValidatedTransaction {
    pub fn main<L: SubstateStore>(&self, proc: &mut Process<L>) -> (Vec<ScryptoValue>, Option<RuntimeError>) {
        let mut error: Option<RuntimeError> = None;
        let mut outputs = vec![];

        for inst in &self.instructions {
            let result = match inst {
                ValidatedInstruction::TakeFromWorktop { resource_address } => proc
                    .take_all_from_worktop(*resource_address)
                    .map(|bucket_id| {
                        ScryptoValue::from_value(&scrypto::resource::Bucket(bucket_id))
                    }),
                ValidatedInstruction::TakeFromWorktopByAmount {
                    amount,
                    resource_address,
                } => proc
                    .take_from_worktop(*amount, *resource_address)
                    .map(|bucket_id| {
                        ScryptoValue::from_value(&scrypto::resource::Bucket(bucket_id))
                    }),
                ValidatedInstruction::TakeFromWorktopByIds {
                    ids,
                    resource_address,
                } => proc
                    .take_non_fungibles_from_worktop(ids, *resource_address)
                    .map(|bucket_id| {
                        ScryptoValue::from_value(&scrypto::resource::Bucket(bucket_id))
                    }),
                ValidatedInstruction::ReturnToWorktop { bucket_id } => {
                    proc.return_to_worktop(*bucket_id)
                }
                ValidatedInstruction::AssertWorktopContains { resource_address } => {
                    proc.assert_worktop_contains(*resource_address)
                }
                ValidatedInstruction::AssertWorktopContainsByAmount {
                    amount,
                    resource_address,
                } => proc.assert_worktop_contains_by_amount(*amount, *resource_address),
                ValidatedInstruction::AssertWorktopContainsByIds {
                    ids,
                    resource_address,
                } => proc.assert_worktop_contains_by_ids(&ids, *resource_address),
                ValidatedInstruction::PopFromAuthZone {} => proc
                    .pop_from_auth_zone()
                    .map(|proof_id| ScryptoValue::from_value(&scrypto::resource::Proof(proof_id))),
                ValidatedInstruction::ClearAuthZone => proc
                    .drop_all_auth_zone_proofs()
                    .map(|_| ScryptoValue::from_value(&())),
                ValidatedInstruction::PushToAuthZone { proof_id } => proc
                    .push_to_auth_zone(*proof_id)
                    .map(|_| ScryptoValue::from_value(&())),
                ValidatedInstruction::CreateProofFromAuthZone { resource_address } => proc
                    .create_auth_zone_proof(*resource_address)
                    .map(|proof_id| ScryptoValue::from_value(&scrypto::resource::Proof(proof_id))),
                ValidatedInstruction::CreateProofFromAuthZoneByAmount {
                    amount,
                    resource_address,
                } => proc
                    .create_auth_zone_proof_by_amount(*amount, *resource_address)
                    .map(|proof_id| ScryptoValue::from_value(&scrypto::resource::Proof(proof_id))),
                ValidatedInstruction::CreateProofFromAuthZoneByIds {
                    ids,
                    resource_address,
                } => proc
                    .create_auth_zone_proof_by_ids(ids, *resource_address)
                    .map(|proof_id| ScryptoValue::from_value(&scrypto::resource::Proof(proof_id))),
                ValidatedInstruction::CreateProofFromBucket { bucket_id } => proc
                    .txn_create_bucket_proof(*bucket_id)
                    .map(|proof_id| ScryptoValue::from_value(&scrypto::resource::Proof(proof_id))),
                ValidatedInstruction::CloneProof { proof_id } => proc
                    .clone_proof(*proof_id)
                    .map(|proof_id| ScryptoValue::from_value(&scrypto::resource::Proof(proof_id))),
                ValidatedInstruction::DropProof { proof_id } => proc
                    .drop_proof(*proof_id)
                    .map(|_| ScryptoValue::from_value(&())),
                ValidatedInstruction::CallFunction {
                    package_address,
                    blueprint_name,
                    function,
                    args,
                } => proc.txn_call_function(
                    *package_address,
                    &blueprint_name,
                    &function,
                    args.clone(),
                ),
                ValidatedInstruction::CallMethod {
                    component_address,
                    method,
                    args,
                } => proc.txn_call_method(*component_address, &method, args.clone()),
                ValidatedInstruction::CallMethodWithAllResources {
                    component_address,
                    method,
                } => proc.call_method_with_all_resources(*component_address, &method),
                ValidatedInstruction::PublishPackage { code } => proc
                    .publish_package(code.clone())
                    .map(|package_address| ScryptoValue::from_value(&package_address)),
            };
            match result {
                Ok(data) => {
                    outputs.push(data);
                }
                Err(e) => {
                    error = Some(e);
                    break;
                }
            }
        }

        // drop all dangling proofs
        error = error.or_else(|| match proc.drop_all_proofs() {
            Ok(_) => None,
            Err(e) => Some(e),
        });

        // check resource
        error = error.or_else(|| match proc.check_resource() {
            Ok(_) => None,
            Err(e) => Some(e),
        });

        (outputs, error)
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidatedInstruction {
    TakeFromWorktop {
        resource_address: ResourceAddress,
    },
    TakeFromWorktopByAmount {
        amount: Decimal,
        resource_address: ResourceAddress,
    },
    TakeFromWorktopByIds {
        ids: BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
    },
    ReturnToWorktop {
        bucket_id: BucketId,
    },
    AssertWorktopContains {
        resource_address: ResourceAddress,
    },
    AssertWorktopContainsByAmount {
        amount: Decimal,
        resource_address: ResourceAddress,
    },
    AssertWorktopContainsByIds {
        ids: BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
    },
    PopFromAuthZone,
    PushToAuthZone {
        proof_id: ProofId,
    },
    ClearAuthZone,
    CreateProofFromAuthZone {
        resource_address: ResourceAddress,
    },
    CreateProofFromAuthZoneByAmount {
        amount: Decimal,
        resource_address: ResourceAddress,
    },
    CreateProofFromAuthZoneByIds {
        ids: BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
    },
    CreateProofFromBucket {
        bucket_id: BucketId,
    },
    CloneProof {
        proof_id: ProofId,
    },
    DropProof {
        proof_id: ProofId,
    },
    CallFunction {
        package_address: PackageAddress,
        blueprint_name: String,
        function: String,
        args: Vec<ScryptoValue>,
    },
    CallMethod {
        component_address: ComponentAddress,
        method: String,
        args: Vec<ScryptoValue>,
    },
    CallMethodWithAllResources {
        component_address: ComponentAddress,
        method: String,
    },
    PublishPackage {
        code: Vec<u8>,
    },
}