use sbor::rust::vec::Vec;
use scrypto::buffer::scrypto_encode;
use scrypto::crypto::*;
use scrypto::resource::NonFungibleAddress;

use crate::model::*;

/// Represents a validated transaction
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Executable {
    pub transaction_hash: Hash,
    pub instructions: Vec<Instruction>,
    pub initial_proofs: Vec<NonFungibleAddress>,
    pub cost_unit_limit: u32,
    pub tip_percentage: u32,
    pub blobs: Vec<Vec<u8>>,
}

impl Executable {
    pub fn new(
        transaction_hash: Hash,
        instructions: Vec<Instruction>,
        initial_proofs: Vec<NonFungibleAddress>,
        cost_unit_limit: u32,
        tip_percentage: u32,
        blobs: Vec<Vec<u8>>,
    ) -> Self {
        Self {
            transaction_hash,
            instructions,
            initial_proofs,
            cost_unit_limit,
            tip_percentage,
            blobs,
        }
    }

    pub fn transaction_hash(&self) -> Hash {
        self.transaction_hash
    }

    pub fn manifest_instructions_size(&self) -> u32 {
        scrypto_encode(&self.instructions).len() as u32
    }

    pub fn cost_unit_limit(&self) -> u32 {
        self.cost_unit_limit
    }

    pub fn tip_percentage(&self) -> u32 {
        self.tip_percentage
    }

    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    pub fn initial_proofs(&self) -> Vec<NonFungibleAddress> {
        self.initial_proofs.clone()
    }

    pub fn blobs(&self) -> &[Vec<u8>] {
        &self.blobs
    }
}
