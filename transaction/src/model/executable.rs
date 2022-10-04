use sbor::rust::vec::Vec;
use scrypto::buffer::scrypto_encode;
use scrypto::crypto::*;
use scrypto::resource::{NonFungibleAddress, ResourceAddress};
pub use std::collections::BTreeSet;

use crate::model::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutableProofs {
    pub initial_proofs: Vec<NonFungibleAddress>,
    pub virtualizable_proofs_resource_addresses: BTreeSet<ResourceAddress>,
}

/// Represents a validated transaction
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Executable {
    pub transaction_hash: Hash,
    pub instructions: Vec<Instruction>,
    pub proofs: ExecutableProofs,
    pub cost_unit_limit: u32,
    pub tip_percentage: u32,
    pub blobs: Vec<Vec<u8>>,
}

impl Executable {
    pub fn new(
        transaction_hash: Hash,
        instructions: Vec<Instruction>,
        proofs: ExecutableProofs,
        cost_unit_limit: u32,
        tip_percentage: u32,
        blobs: Vec<Vec<u8>>,
    ) -> Self {
        Self {
            transaction_hash,
            instructions,
            proofs,
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

    pub fn proofs(&self) -> ExecutableProofs {
        self.proofs.clone()
    }

    pub fn blobs(&self) -> &[Vec<u8>] {
        &self.blobs
    }
}
