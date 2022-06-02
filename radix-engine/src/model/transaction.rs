use sbor::rust::collections::BTreeSet;
use sbor::rust::string::String;
use sbor::rust::vec;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::buffer::scrypto_encode;
use scrypto::crypto::*;
use scrypto::engine::types::*;
use scrypto::values::*;

use crate::engine::*;
use crate::model::{ValidatedInstruction, ValidatedTransaction};

/// Represents an unsigned transaction
#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct Transaction {
    pub instructions: Vec<Instruction>,
}

/// Represents a signed transaction
pub struct SignedTransaction {
    /// The unsigned transaction
    pub transaction: Transaction,
    /// The signatures. Public keys are for signature algorithm that doesn't support public key recovery, e.g. ed25519.
    pub signatures: Vec<(EcdsaPublicKey, EcdsaSignature)>,
}

/// Represents an instruction
#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub enum Instruction {
    /// Takes resource from worktop.
    TakeFromWorktop { resource_address: ResourceAddress },

    /// Takes resource from worktop by the given amount.
    TakeFromWorktopByAmount {
        amount: Decimal,
        resource_address: ResourceAddress,
    },

    /// Takes resource from worktop by the given non-fungible IDs.
    TakeFromWorktopByIds {
        ids: BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
    },

    /// Returns a bucket of resource to worktop.
    ReturnToWorktop { bucket_id: BucketId },

    /// Asserts worktop contains resource.
    AssertWorktopContains { resource_address: ResourceAddress },

    /// Asserts worktop contains resource by at least the given amount.
    AssertWorktopContainsByAmount {
        amount: Decimal,
        resource_address: ResourceAddress,
    },

    /// Asserts worktop contains resource by at least the given non-fungible IDs.
    AssertWorktopContainsByIds {
        ids: BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
    },

    /// Creates a new AuthZone making the old one the previous AuthZone
    StartAuthZone,
    /// Removes the current AuthZone and replaces it with the previous AuthZone
    EndAuthZone,

    /// Takes the last proof from the auth zone.
    PopFromAuthZone,

    /// Adds a proof to the auth zone.
    PushToAuthZone { proof_id: ProofId },

    /// Drops all proofs in the auth zone
    ClearAuthZone,

    // TODO: do we need `CreateProofFromWorktop`, to avoid taking resource out and then creating proof?
    /// Creates a proof from the auth zone
    CreateProofFromAuthZone { resource_address: ResourceAddress },

    /// Creates a proof from the auth zone, by the given amount
    CreateProofFromAuthZoneByAmount {
        amount: Decimal,
        resource_address: ResourceAddress,
    },

    /// Creates a proof from the auth zone, by the given non-fungible IDs.
    CreateProofFromAuthZoneByIds {
        ids: BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
    },

    /// Creates a proof from a bucket.
    CreateProofFromBucket { bucket_id: BucketId },

    /// Clones a proof.
    CloneProof { proof_id: ProofId },

    /// Drops a proof.
    DropProof { proof_id: ProofId },

    /// Calls a blueprint function.
    ///
    /// Buckets and proofs in arguments moves from transaction context to the callee.
    CallFunction {
        package_address: PackageAddress,
        blueprint_name: String,
        call_data: Vec<u8>,
    },

    /// Calls a component method.
    ///
    /// Buckets and proofs in arguments moves from transaction context to the callee.
    CallMethod {
        component_address: ComponentAddress,
        call_data: Vec<u8>,
    },

    /// Calls a component method with all resources owned by the transaction.
    CallMethodWithAllResources {
        component_address: ComponentAddress,
        method: String,
    },

    /// Publishes a package.
    PublishPackage { package: Vec<u8> },

    /// Specifies transaction nonce
    Nonce {
        nonce: u64, // TODO: may be replaced with substate id for entropy
    },
}

impl Transaction {
    pub fn to_vec(&self) -> Vec<u8> {
        scrypto_encode(self)
    }

    pub fn raw_hash(&self) -> Hash {
        hash(self.to_vec())
    }

    pub fn add_nonce(&mut self, nonce: u64) {
        self.instructions.push(Instruction::Nonce { nonce });
    }

    // TODO: introduce a `Signer` trait
    pub fn sign<'a, T: AsRef<[&'a EcdsaPrivateKey]>>(self, sks: T) -> SignedTransaction {
        let msg = self.to_vec();
        let signatures = sks
            .as_ref()
            .iter()
            .map(|sk| (sk.public_key(), sk.sign(&msg)))
            .collect();

        SignedTransaction {
            transaction: self,
            signatures: signatures,
        }
    }
}

impl SignedTransaction {
    pub fn validate(&self) -> Result<ValidatedTransaction, TransactionValidationError> {
        let mut instructions = vec![];
        let mut signers = vec![];

        // verify signature (may defer to runtime)
        let msg = self.transaction.to_vec();
        for (pk, sig) in &self.signatures {
            if !EcdsaVerifier::verify(&msg, pk, sig) {
                return Err(TransactionValidationError::InvalidSignature);
            }
            signers.push(pk.clone());
        }

        // semantic analysis
        let mut id_validator = IdValidator::new();
        for inst in &self.transaction.instructions {
            match inst.clone() {
                Instruction::TakeFromWorktop { resource_address } => {
                    id_validator
                        .new_bucket()
                        .map_err(TransactionValidationError::IdValidatorError)?;
                    instructions.push(ValidatedInstruction::TakeFromWorktop { resource_address });
                }
                Instruction::TakeFromWorktopByAmount {
                    amount,
                    resource_address,
                } => {
                    id_validator
                        .new_bucket()
                        .map_err(TransactionValidationError::IdValidatorError)?;
                    instructions.push(ValidatedInstruction::TakeFromWorktopByAmount {
                        amount,
                        resource_address,
                    });
                }
                Instruction::TakeFromWorktopByIds {
                    ids,
                    resource_address,
                } => {
                    id_validator
                        .new_bucket()
                        .map_err(TransactionValidationError::IdValidatorError)?;
                    instructions.push(ValidatedInstruction::TakeFromWorktopByIds {
                        ids,
                        resource_address,
                    });
                }
                Instruction::ReturnToWorktop { bucket_id } => {
                    id_validator
                        .drop_bucket(bucket_id)
                        .map_err(TransactionValidationError::IdValidatorError)?;
                    instructions.push(ValidatedInstruction::ReturnToWorktop { bucket_id });
                }
                Instruction::AssertWorktopContains { resource_address } => {
                    instructions
                        .push(ValidatedInstruction::AssertWorktopContains { resource_address });
                }
                Instruction::AssertWorktopContainsByAmount {
                    amount,
                    resource_address,
                } => {
                    instructions.push(ValidatedInstruction::AssertWorktopContainsByAmount {
                        amount,
                        resource_address,
                    });
                }
                Instruction::AssertWorktopContainsByIds {
                    ids,
                    resource_address,
                } => {
                    instructions.push(ValidatedInstruction::AssertWorktopContainsByIds {
                        ids,
                        resource_address,
                    });
                }
                Instruction::StartAuthZone => {
                    instructions.push(ValidatedInstruction::StartAuthZone);
                }
                Instruction::EndAuthZone => {
                    instructions.push(ValidatedInstruction::EndAuthZone);
                }
                Instruction::PopFromAuthZone => {
                    id_validator
                        .new_proof(ProofKind::AuthZoneProof)
                        .map_err(TransactionValidationError::IdValidatorError)?;
                    instructions.push(ValidatedInstruction::PopFromAuthZone);
                }
                Instruction::PushToAuthZone { proof_id } => {
                    id_validator
                        .drop_proof(proof_id)
                        .map_err(TransactionValidationError::IdValidatorError)?;
                    instructions.push(ValidatedInstruction::PushToAuthZone { proof_id });
                }
                Instruction::ClearAuthZone => {
                    instructions.push(ValidatedInstruction::ClearAuthZone);
                }
                Instruction::CreateProofFromAuthZone { resource_address } => {
                    id_validator
                        .new_proof(ProofKind::AuthZoneProof)
                        .map_err(TransactionValidationError::IdValidatorError)?;
                    instructions
                        .push(ValidatedInstruction::CreateProofFromAuthZone { resource_address });
                }
                Instruction::CreateProofFromAuthZoneByAmount {
                    amount,
                    resource_address,
                } => {
                    id_validator
                        .new_proof(ProofKind::AuthZoneProof)
                        .map_err(TransactionValidationError::IdValidatorError)?;
                    instructions.push(ValidatedInstruction::CreateProofFromAuthZoneByAmount {
                        amount,
                        resource_address,
                    });
                }
                Instruction::CreateProofFromAuthZoneByIds {
                    ids,
                    resource_address,
                } => {
                    id_validator
                        .new_proof(ProofKind::AuthZoneProof)
                        .map_err(TransactionValidationError::IdValidatorError)?;
                    instructions.push(ValidatedInstruction::CreateProofFromAuthZoneByIds {
                        ids,
                        resource_address,
                    });
                }
                Instruction::CreateProofFromBucket { bucket_id } => {
                    id_validator
                        .new_proof(ProofKind::BucketProof(bucket_id))
                        .map_err(TransactionValidationError::IdValidatorError)?;
                    instructions.push(ValidatedInstruction::CreateProofFromBucket { bucket_id });
                }
                Instruction::CloneProof { proof_id } => {
                    id_validator
                        .clone_proof(proof_id)
                        .map_err(TransactionValidationError::IdValidatorError)?;
                    instructions.push(ValidatedInstruction::CloneProof { proof_id });
                }
                Instruction::DropProof { proof_id } => {
                    id_validator
                        .drop_proof(proof_id)
                        .map_err(TransactionValidationError::IdValidatorError)?;
                    instructions.push(ValidatedInstruction::DropProof { proof_id });
                }
                Instruction::CallFunction {
                    package_address,
                    blueprint_name,
                    call_data,
                } => {
                    instructions.push(ValidatedInstruction::CallFunction {
                        package_address,
                        blueprint_name,
                        call_data: Self::validate_call_data(call_data, &mut id_validator)?,
                    });
                }
                Instruction::CallMethod {
                    component_address,
                    call_data,
                } => {
                    instructions.push(ValidatedInstruction::CallMethod {
                        component_address,
                        call_data: Self::validate_call_data(call_data, &mut id_validator)?,
                    });
                }
                Instruction::CallMethodWithAllResources {
                    component_address,
                    method,
                } => {
                    id_validator
                        .move_all_resources()
                        .map_err(TransactionValidationError::IdValidatorError)?;
                    instructions.push(ValidatedInstruction::CallMethodWithAllResources {
                        component_address,
                        method,
                    });
                }
                Instruction::PublishPackage { package } => {
                    instructions.push(ValidatedInstruction::PublishPackage { package });
                }
                Instruction::Nonce { .. } => {
                    // TODO: validate nonce
                }
            }
        }

        Ok(ValidatedTransaction {
            raw_hash: self.transaction.raw_hash(),
            instructions,
            signers,
        })
    }

    fn validate_call_data(
        call_data: Vec<u8>,
        id_validator: &mut IdValidator,
    ) -> Result<ScryptoValue, TransactionValidationError> {
        let value = ScryptoValue::from_slice(&call_data)
            .map_err(TransactionValidationError::ParseScryptoValueError)?;
        id_validator
            .move_resources(&value)
            .map_err(TransactionValidationError::IdValidatorError)?;
        if let Some(vault_id) = value.vault_ids.iter().nth(0) {
            return Err(TransactionValidationError::VaultNotAllowed(
                vault_id.clone(),
            ));
        }
        if let Some(lazy_map_id) = value.lazy_map_ids.iter().nth(0) {
            return Err(TransactionValidationError::LazyMapNotAllowed(
                lazy_map_id.clone(),
            ));
        }
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use sbor::rust::marker::PhantomData;
    use scrypto::engine::types::ComponentAddress;

    #[test]
    fn should_reject_transaction_passing_vault() {
        assert_eq!(
            SignedTransaction {
                transaction: Transaction {
                    instructions: vec![Instruction::CallMethod {
                        component_address: ComponentAddress([1u8; 26]),
                        call_data: scrypto_encode(&scrypto::resource::Vault((Hash([2u8; 32]), 0,))),
                    }],
                },
                signatures: Vec::new(),
            }
            .validate(),
            Err(TransactionValidationError::VaultNotAllowed((
                Hash([2u8; 32]),
                0,
            ))),
        );
    }

    #[test]
    fn should_reject_transaction_passing_lazy_map() {
        assert_eq!(
            SignedTransaction {
                transaction: Transaction {
                    instructions: vec![Instruction::CallMethod {
                        component_address: ComponentAddress([1u8; 26]),
                        call_data: scrypto_encode(&scrypto::component::LazyMap::<(), ()> {
                            id: (Hash([2u8; 32]), 0,),
                            key: PhantomData,
                            value: PhantomData,
                        }),
                    }],
                },
                signatures: Vec::new()
            }
            .validate(),
            Err(TransactionValidationError::LazyMapNotAllowed((
                Hash([2u8; 32]),
                0,
            ))),
        );
    }
}
