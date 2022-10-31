use sbor::rust::vec::Vec;
use sbor::{Decode, Encode, TypeId};
use scrypto::buffer::scrypto_encode;
use scrypto::crypto::*;
use scrypto::resource::{NonFungibleAddress, ResourceAddress};
pub use std::collections::BTreeSet;

use crate::model::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthZoneParams {
    pub initial_proofs: Vec<NonFungibleAddress>,
    pub virtualizable_proofs_resource_addresses: BTreeSet<ResourceAddress>,
}

/// Represents a validated transaction
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Executable {
    transaction_hash: Hash,
    instructions: Vec<Instruction>,
    auth_zone_params: AuthZoneParams,
    cost_unit_limit: u32,
    tip_percentage: u32,
    blobs: Vec<Vec<u8>>,
    runtime_validations: Vec<RuntimeValidationRequest>,
}

#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
pub struct RuntimeValidationRequest {
    /// The validation to perform
    pub validation: RuntimeValidation,
    /// This option is intended for preview uses cases
    /// In these cases, we still want to do the look ups to give equivalent cost unit spend, but may wish to ignore the result
    pub skip_assertion: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
pub enum RuntimeValidation {
    /// To ensure we don't commit a duplicate intent hash
    IntentHashUniqueness { intent_hash: Hash },
    /// For preview - still do the look-ups to give equivalent cost unit spend, but ignore the result
    WithinEpochRange {
        start_epoch_inclusive: u64,
        end_epoch_exclusive: u64,
    },
}

impl RuntimeValidation {
    pub fn enforced(self) -> RuntimeValidationRequest {
        RuntimeValidationRequest {
            validation: self,
            skip_assertion: true,
        }
    }

    pub fn with_skipped_assertion_if(self, skip_assertion: bool) -> RuntimeValidationRequest {
        RuntimeValidationRequest {
            validation: self,
            skip_assertion,
        }
    }
}

impl Executable {
    pub fn new(
        transaction_hash: Hash,
        instructions: Vec<Instruction>,
        auth_zone_params: AuthZoneParams,
        cost_unit_limit: u32,
        tip_percentage: u32,
        blobs: Vec<Vec<u8>>,
        runtime_validations: Vec<RuntimeValidationRequest>,
    ) -> Self {
        Self {
            transaction_hash,
            instructions,
            auth_zone_params,
            cost_unit_limit,
            tip_percentage,
            blobs,
            runtime_validations,
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

    pub fn auth_zone_params(&self) -> &AuthZoneParams {
        &self.auth_zone_params
    }

    pub fn blobs(&self) -> &[Vec<u8>] {
        &self.blobs
    }

    pub fn runtime_validations(&self) -> &[RuntimeValidationRequest] {
        &self.runtime_validations
    }
}
