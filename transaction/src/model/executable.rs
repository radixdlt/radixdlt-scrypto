use radix_engine_interface::crypto::{hash, Hash};
use radix_engine_interface::model::*;
use radix_engine_interface::scrypto;
use sbor::rust::collections::{BTreeSet, HashMap};
use sbor::rust::vec::Vec;
use sbor::{Decode, Encode, TypeId};

use crate::model::*;

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AuthZoneParams {
    pub initial_proofs: Vec<NonFungibleAddress>,
    pub virtualizable_proofs_resource_addresses: BTreeSet<ResourceAddress>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ExecutionContext {
    pub transaction_hash: Hash,
    pub payload_size: usize,
    pub auth_zone_params: AuthZoneParams,
    pub fee_payment: FeePayment,
    pub runtime_validations: Vec<RuntimeValidationRequest>,
}

#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
pub enum FeePayment {
    User {
        cost_unit_limit: u32,
        tip_percentage: u16,
    },
    NoFee,
}

#[derive(Debug)]
pub enum InstructionList<'a> {
    Basic(&'a [BasicInstruction]),
    Any(&'a [Instruction]),
}

#[derive(Debug)]
pub struct Executable<'a> {
    instructions: InstructionList<'a>,
    blobs: HashMap<Hash, &'a [u8]>,
    context: ExecutionContext,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct RuntimeValidationRequest {
    /// The validation to perform
    pub validation: RuntimeValidation,
    /// This option is intended for preview uses cases
    /// In these cases, we still want to do the look ups to give equivalent cost unit spend, but may wish to ignore the result
    pub skip_assertion: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
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
            skip_assertion: false,
        }
    }

    pub fn with_skipped_assertion_if(self, skip_assertion: bool) -> RuntimeValidationRequest {
        RuntimeValidationRequest {
            validation: self,
            skip_assertion,
        }
    }
}

impl<'a> Executable<'a> {
    pub fn new(
        instructions: InstructionList<'a>,
        blobs: &'a [Vec<u8>],
        context: ExecutionContext,
    ) -> Self {
        let blobs = blobs.iter().map(|b| (hash(b), b.as_slice())).collect();
        Self {
            instructions,
            blobs,
            context,
        }
    }

    pub fn transaction_hash(&self) -> &Hash {
        &self.context.transaction_hash
    }

    pub fn fee_payment(&self) -> &FeePayment {
        &self.context.fee_payment
    }

    pub fn instructions(&self) -> &InstructionList {
        &self.instructions
    }

    pub fn auth_zone_params(&self) -> &AuthZoneParams {
        &self.context.auth_zone_params
    }

    pub fn blobs(&self) -> &HashMap<Hash, &[u8]> {
        &self.blobs
    }

    pub fn payload_size(&self) -> usize {
        self.context.payload_size
    }

    pub fn runtime_validations(&self) -> &[RuntimeValidationRequest] {
        &self.context.runtime_validations
    }
}
