use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::crypto::Hash;
use radix_engine_interface::model::*;
use radix_engine_interface::*;
use sbor::rust::collections::BTreeSet;
use sbor::rust::vec::Vec;
use sbor::{Categorize, Decode, Encode};

use crate::model::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AuthZoneParams {
    pub initial_proofs: Vec<NonFungibleGlobalId>,
    pub virtualizable_proofs_resource_addresses: BTreeSet<ResourceAddress>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ExecutionContext {
    pub transaction_hash: Hash,
    pub pre_allocated_ids: BTreeSet<RENodeId>,
    pub payload_size: usize,
    pub auth_zone_params: AuthZoneParams,
    pub fee_payment: FeePayment,
    pub runtime_validations: Vec<RuntimeValidationRequest>,
}

#[derive(Debug, Clone, PartialEq, Eq, Categorize, Encode, Decode)]
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
    AnyOwned(Vec<Instruction>),
}

#[derive(Debug)]
pub struct Executable<'a> {
    instructions: InstructionList<'a>,
    blobs: &'a [Vec<u8>],
    context: ExecutionContext,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct RuntimeValidationRequest {
    /// The validation to perform
    pub validation: RuntimeValidation,
    /// This option is intended for preview uses cases
    /// In these cases, we still want to do the look ups to give equivalent cost unit spend, but may wish to ignore the result
    pub skip_assertion: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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
        Self {
            instructions,
            blobs,
            context,
        }
    }

    pub fn new_no_blobs(instructions: InstructionList<'a>, context: ExecutionContext) -> Self {
        Self {
            instructions,
            blobs: &[],
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

    pub fn pre_allocated_ids(&self) -> &BTreeSet<RENodeId> {
        &self.context.pre_allocated_ids
    }

    pub fn blobs(&self) -> &[Vec<u8>] {
        &self.blobs
    }

    pub fn payload_size(&self) -> usize {
        self.context.payload_size
    }

    pub fn runtime_validations(&self) -> &[RuntimeValidationRequest] {
        &self.context.runtime_validations
    }
}
