use sbor::rust::vec::Vec;
use scrypto::buffer::scrypto_encode;
use scrypto::crypto::*;
use scrypto::resource::NonFungibleAddress;

use crate::model::*;

/// Represents a validated transaction
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Validated<T> {
    transaction: T,
    transaction_hash: Hash,
    instructions: Vec<Instruction>,
    initial_proofs: Vec<NonFungibleAddress>,
    cost_unit_limit: u32,
    tip_percentage: u32,
    blobs: Vec<Vec<u8>>,
}

impl Into<Validated<Transaction>> for Validated<NotarizedTransaction> {
    fn into(self) -> Validated<Transaction> {
        Validated::<Transaction> {
            transaction: self.transaction.into(),
            transaction_hash: self.transaction_hash,
            instructions: self.instructions,
            initial_proofs: self.initial_proofs,
            cost_unit_limit: self.cost_unit_limit,
            tip_percentage: self.tip_percentage,
            blobs: self.blobs,
        }
    }
}

impl<T> Validated<T> {
    pub fn new(
        transaction: T,
        transaction_hash: Hash,
        instructions: Vec<Instruction>,
        initial_proofs: Vec<NonFungibleAddress>,
        cost_unit_limit: u32,
        tip_percentage: u32,
        blobs: Vec<Vec<u8>>,
    ) -> Self {
        Self {
            transaction,
            transaction_hash,
            instructions,
            initial_proofs,
            cost_unit_limit,
            tip_percentage,
            blobs,
        }
    }
}

impl<T> ExecutableTransaction for Validated<T> {
    fn transaction_hash(&self) -> Hash {
        self.transaction_hash
    }

    fn manifest_instructions_size(&self) -> u32 {
        scrypto_encode(&self.instructions).len() as u32
    }

    fn cost_unit_limit(&self) -> u32 {
        self.cost_unit_limit
    }

    fn tip_percentage(&self) -> u32 {
        self.tip_percentage
    }

    fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    fn initial_proofs(&self) -> Vec<NonFungibleAddress> {
        self.initial_proofs.clone()
    }

    fn blobs(&self) -> &[Vec<u8>] {
        &self.blobs
    }
}
