use radix_engine_interface::blueprints::resource::NonFungibleGlobalId;
use radix_engine_interface::blueprints::transaction_processor::RuntimeValidationRequest;
use radix_engine_interface::crypto::Hash;
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use sbor::rust::collections::BTreeSet;
use sbor::rust::vec::Vec;

use crate::model::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct AuthZoneParams {
    pub initial_proofs: BTreeSet<NonFungibleGlobalId>,
    pub virtual_resources: BTreeSet<ResourceAddress>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ExecutionContext {
    pub transaction_hash: Hash,
    pub pre_allocated_ids: BTreeSet<NodeId>,
    pub payload_size: usize,
    pub auth_zone_params: AuthZoneParams,
    pub fee_payment: FeePayment,
    pub runtime_validations: Vec<RuntimeValidationRequest>,
}

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum FeePayment {
    User {
        cost_unit_limit: u32,
        tip_percentage: u16,
    },
    NoFee,
}

#[derive(Debug)]
pub struct Executable<'a> {
    instructions: Vec<Instruction>,
    blobs: &'a [Vec<u8>],
    pub context: ExecutionContext,
}

impl<'a> Executable<'a> {
    pub fn new(
        instructions: Vec<Instruction>,
        blobs: &'a [Vec<u8>],
        context: ExecutionContext,
    ) -> Self {
        Self {
            instructions,
            blobs,
            context,
        }
    }

    pub fn new_no_blobs(instructions: Vec<Instruction>, context: ExecutionContext) -> Self {
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

    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    pub fn auth_zone_params(&self) -> &AuthZoneParams {
        &self.context.auth_zone_params
    }

    pub fn pre_allocated_ids(&self) -> &BTreeSet<NodeId> {
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
