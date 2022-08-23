use sbor::rust::vec::Vec;
use scrypto::core::NetworkDefinition;
use scrypto::crypto::*;

use crate::builder::TransactionBuilder;
use crate::model::*;

/// Represents a test transaction, for testing/simulation purpose only.
pub struct TestTransaction {
    pub transaction: NotarizedTransaction,
    pub executable_instructions: Vec<ExecutableInstruction>,
    pub signer_public_keys: Vec<EcdsaPublicKey>,
}

impl TestTransaction {
    pub fn new(
        manifest: TransactionManifest,
        nonce: u64,
        signer_public_keys: Vec<EcdsaPublicKey>,
    ) -> Self {
        let transaction = TransactionBuilder::new()
            .header(TransactionHeader {
                version: TRANSACTION_VERSION_V1,
                network_id: NetworkDefinition::local_simulator().id,
                start_epoch_inclusive: 0,
                end_epoch_exclusive: 100,
                nonce,
                notary_public_key: EcdsaPublicKey([0u8; 33]),
                notary_as_signatory: false,
                cost_unit_limit: u32::MAX, // TODO: Temporary fix to be able to publish large packages
                tip_percentage: 5,
            })
            .manifest(manifest)
            .signer_signatures(
                signer_public_keys
                    .iter()
                    .cloned()
                    .map(|pk| (pk, EcdsaSignature([0u8; 64])))
                    .collect(),
            )
            .notary_signature(EcdsaSignature([0u8; 64]))
            .build();

        let executable_instructions = transaction
            .signed_intent
            .intent
            .manifest
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
                Instruction::DropAllProofs => ExecutableInstruction::DropAllProofs,
                Instruction::CallFunction {
                    package_address,
                    blueprint_name,
                    method_name,
                    args,
                } => ExecutableInstruction::CallFunction {
                    package_address,
                    blueprint_name,
                    method_name,
                    args,
                },
                Instruction::CallMethod {
                    component_address,
                    method_name,
                    args,
                } => ExecutableInstruction::CallMethod {
                    component_address,
                    method_name,
                    args,
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
            transaction,
            executable_instructions,
            signer_public_keys,
        }
    }
}

impl ExecutableTransaction for TestTransaction {
    fn transaction_hash(&self) -> Hash {
        self.transaction.hash()
    }

    fn transaction_payload_size(&self) -> u32 {
        self.transaction.to_bytes().len() as u32
    }

    fn cost_unit_limit(&self) -> u32 {
        self.transaction.signed_intent.intent.header.cost_unit_limit
    }

    fn tip_percentage(&self) -> u32 {
        self.transaction.signed_intent.intent.header.tip_percentage
    }

    fn instructions(&self) -> &[ExecutableInstruction] {
        &self.executable_instructions
    }

    fn signer_public_keys(&self) -> &[EcdsaPublicKey] {
        &self.signer_public_keys
    }
}
