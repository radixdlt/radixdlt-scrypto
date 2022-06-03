use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::crypto::*;
use scrypto::values::*;

use crate::model::*;

/// Represents a test transaction, for testing/simulation purpose only.
pub struct TestTransaction {
    pub manifest: TransactionManifest,
    pub instructions: Vec<ExecutableInstruction>,
    pub nonce: u64,
    pub signers: Vec<EcdsaPublicKey>,
}

impl TestTransaction {
    pub fn new(manifest: TransactionManifest, nonce: u64, signers: Vec<EcdsaPublicKey>) -> Self {
        let instructions = manifest
            .instructions
            .iter()
            .map(|i| match i.clone() {
                Instruction::TakeFromWorktop { resource_address } => {
                    ExecutableInstruction::TakeFromWorktop { resource_address }
                }
                Instruction::TakeFromWorktopByAmount {
                    amount,
                    resource_address,
                } => ExecutableInstruction::TakeFromWorktopByAmount {
                    amount,
                    resource_address,
                },
                Instruction::TakeFromWorktopByIds {
                    ids,
                    resource_address,
                } => ExecutableInstruction::TakeFromWorktopByIds {
                    ids,
                    resource_address,
                },
                Instruction::ReturnToWorktop { bucket_id } => {
                    ExecutableInstruction::ReturnToWorktop { bucket_id }
                }
                Instruction::AssertWorktopContains { resource_address } => {
                    ExecutableInstruction::AssertWorktopContains { resource_address }
                }
                Instruction::AssertWorktopContainsByAmount {
                    amount,
                    resource_address,
                } => ExecutableInstruction::AssertWorktopContainsByAmount {
                    amount,
                    resource_address,
                },
                Instruction::AssertWorktopContainsByIds {
                    ids,
                    resource_address,
                } => ExecutableInstruction::AssertWorktopContainsByIds {
                    ids,
                    resource_address,
                },
                Instruction::PopFromAuthZone => ExecutableInstruction::PopFromAuthZone,
                Instruction::PushToAuthZone { proof_id } => {
                    ExecutableInstruction::PushToAuthZone { proof_id }
                }
                Instruction::ClearAuthZone => ExecutableInstruction::ClearAuthZone,
                Instruction::CreateProofFromAuthZone { resource_address } => {
                    ExecutableInstruction::CreateProofFromAuthZone { resource_address }
                }
                Instruction::CreateProofFromAuthZoneByAmount {
                    amount,
                    resource_address,
                } => ExecutableInstruction::CreateProofFromAuthZoneByAmount {
                    amount,
                    resource_address,
                },
                Instruction::CreateProofFromAuthZoneByIds {
                    ids,
                    resource_address,
                } => ExecutableInstruction::CreateProofFromAuthZoneByIds {
                    ids,
                    resource_address,
                },
                Instruction::CreateProofFromBucket { bucket_id } => {
                    ExecutableInstruction::CreateProofFromBucket { bucket_id }
                }
                Instruction::CloneProof { proof_id } => {
                    ExecutableInstruction::CloneProof { proof_id }
                }
                Instruction::DropProof { proof_id } => {
                    ExecutableInstruction::DropProof { proof_id }
                }
                Instruction::CallFunction {
                    package_address,
                    blueprint_name,
                    call_data,
                } => ExecutableInstruction::CallFunction {
                    package_address,
                    blueprint_name,
                    call_data: ScryptoValue::from_slice(&call_data).unwrap(),
                },
                Instruction::CallMethod {
                    component_address,
                    call_data,
                } => ExecutableInstruction::CallMethod {
                    component_address,
                    call_data: ScryptoValue::from_slice(&call_data).unwrap(),
                },
                Instruction::CallMethodWithAllResources {
                    component_address,
                    method,
                } => ExecutableInstruction::CallMethodWithAllResources {
                    component_address,
                    method,
                },
                Instruction::PublishPackage { package } => {
                    ExecutableInstruction::PublishPackage { package }
                }
            })
            .collect();

        Self {
            manifest,
            instructions,
            nonce,
            signers,
        }
    }
}

impl ExecutableTransaction for TestTransaction {
    fn transaction_hash(&self) -> Hash {
        hash(self.nonce.to_string())
    }

    fn instructions(&self) -> &[ExecutableInstruction] {
        &self.instructions
    }

    fn signers(&self) -> &[EcdsaPublicKey] {
        &self.signers
    }
}
