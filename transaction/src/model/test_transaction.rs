use sbor::rust::vec::Vec;
use scrypto::buffer::scrypto_encode;
use scrypto::constants::{ECDSA_TOKEN, ED25519_TOKEN};
use scrypto::core::NetworkDefinition;
use scrypto::crypto::*;
use scrypto::resource::{NonFungibleAddress, NonFungibleId};

use crate::builder::TransactionBuilder;
use crate::model::*;

/// Represents a test transaction, for testing/simulation purpose only.
pub struct TestTransaction {
    pub transaction: NotarizedTransaction,
    pub executable_instructions: Vec<ExecutableInstruction>,
    pub signer_public_keys: Vec<PublicKey>,
}

impl TestTransaction {
    pub fn new(
        manifest: TransactionManifest,
        nonce: u64,
        signer_public_keys: Vec<PublicKey>,
    ) -> Self {
        let transaction = TransactionBuilder::new()
            .header(TransactionHeader {
                version: TRANSACTION_VERSION_V1,
                network_id: NetworkDefinition::simulator().id,
                start_epoch_inclusive: 0,
                end_epoch_exclusive: 100,
                nonce,
                notary_public_key: EcdsaSecp256k1PublicKey([0u8; 33]).into(),
                notary_as_signatory: false,
                cost_unit_limit: 10_000_000,
                tip_percentage: 5,
            })
            .manifest(manifest)
            .signer_signatures(
                signer_public_keys
                    .iter()
                    .cloned()
                    .map(|pk| match pk {
                        PublicKey::EcdsaSecp256k1(_) => EcdsaSecp256k1Signature([0u8; 65]).into(),
                        PublicKey::EddsaEd25519(pk) => {
                            (pk, EddsaEd25519Signature([0u8; 64])).into()
                        }
                    })
                    .collect(),
            )
            .notary_signature(EcdsaSecp256k1Signature([0u8; 65]).into())
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
                    fn_identifier,
                    args,
                } => ExecutableInstruction::CallFunction {
                    fn_identifier,
                    args,
                },
                Instruction::CallMethod {
                    method_identifier,
                    args,
                } => ExecutableInstruction::CallMethod {
                    method_identifier,
                    args,
                },

                Instruction::PublishPackage { code, abi } => {
                    ExecutableInstruction::PublishPackage { code, abi }
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

    fn manifest_instructions_size(&self) -> u32 {
        scrypto_encode(&self.transaction.signed_intent.intent.manifest.instructions).len() as u32
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

    fn initial_proofs(&self) -> Vec<NonFungibleAddress> {
        self.signer_public_keys
            .iter()
            .map(|k| match k {
                PublicKey::EddsaEd25519(pk) => {
                    NonFungibleAddress::new(ED25519_TOKEN, NonFungibleId::from_bytes(pk.to_vec()))
                }
                PublicKey::EcdsaSecp256k1(pk) => {
                    NonFungibleAddress::new(ECDSA_TOKEN, NonFungibleId::from_bytes(pk.to_vec()))
                }
            })
            .collect()
    }

    fn blobs(&self) -> &[Vec<u8>] {
        &self.transaction.signed_intent.intent.manifest.blobs
    }
}
