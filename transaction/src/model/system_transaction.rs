use super::{ExecutionContext, FeePayment, Instruction, InstructionList};
use crate::model::{AuthZoneParams, Executable};
use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::crypto::hash;
use radix_engine_interface::model::NonFungibleGlobalId;
use radix_engine_interface::*;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct SystemTransaction {
    pub instructions: Vec<Instruction>,
    pub pre_allocated_ids: BTreeSet<RENodeId>,
    pub blobs: Vec<Vec<u8>>,
    pub nonce: u64,
}

impl SystemTransaction {
    pub fn get_executable<'a>(
        &'a self,
        initial_proofs: Vec<NonFungibleGlobalId>,
    ) -> Executable<'a> {
        // Fake transaction hash
        let transaction_hash = hash(self.nonce.to_le_bytes());

        let auth_zone_params = AuthZoneParams {
            initial_proofs,
            virtualizable_proofs_resource_addresses: BTreeSet::new(),
        };

        Executable::new(
            InstructionList::Any(&self.instructions),
            &self.blobs,
            ExecutionContext {
                transaction_hash,
                payload_size: 0,
                auth_zone_params,
                fee_payment: FeePayment::NoFee,
                runtime_validations: vec![],
                pre_allocated_ids: self.pre_allocated_ids.clone(),
            },
        )
    }
}
