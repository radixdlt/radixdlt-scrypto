use sbor::rust::collections::{BTreeSet, HashMap};
use sbor::rust::vec::Vec;
use sbor::{Decode, Encode, TypeId};
use scrypto::crypto::*;
use scrypto::resource::{NonFungibleAddress, ResourceAddress};

use crate::model::*;

#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
pub struct AuthZoneParams {
    pub initial_proofs: Vec<NonFungibleAddress>,
    pub virtualizable_proofs_resource_addresses: BTreeSet<ResourceAddress>,
}

#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
pub struct ExecutionContext {
    pub transaction_hash: Hash,
    pub auth_zone_params: AuthZoneParams,
    pub fee_payment: FeePayment,
    pub intent_validation: IntentValidation,
}

#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
pub struct FeePayment {
    pub cost_unit_limit: u32,
    pub tip_percentage: u32,
}

/// Represents a validated transaction
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Executable {
    instructions: Vec<Instruction>,
    blobs: HashMap<Hash, Vec<u8>>,
    context: ExecutionContext,
}

#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
pub enum IntentValidation {
    User {
        intent_hash: Hash,
        start_epoch_inclusive: u64,
        end_epoch_exclusive: u64,
        /// For preview - still do the look ups to give equivalent cost unit spend, but ignore the result
        skip_epoch_assertions: bool,
    },
    None,
}

impl Executable {
    pub fn new(
        instructions: Vec<Instruction>,
        blobs: &[Vec<u8>],
        context: ExecutionContext,
    ) -> Self {
        let blobs = blobs.iter().map(|b| (hash(b), b.clone())).collect();
        Self {
            instructions,
            blobs,
            context,
        }
    }

    pub fn transaction_hash(&self) -> &Hash {
        &self.context.transaction_hash
    }

    pub fn cost_unit_limit(&self) -> u32 {
        self.context.fee_payment.cost_unit_limit
    }

    pub fn tip_percentage(&self) -> u32 {
        self.context.fee_payment.tip_percentage
    }

    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    pub fn auth_zone_params(&self) -> &AuthZoneParams {
        &self.context.auth_zone_params
    }

    pub fn blobs(&self) -> &HashMap<Hash, Vec<u8>> {
        &self.blobs
    }

    pub fn intent_validation(&self) -> &IntentValidation {
        &self.context.intent_validation
    }
}
