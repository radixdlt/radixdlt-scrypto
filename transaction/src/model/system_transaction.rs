use super::{ExecutionContext, FeePayment, Instruction};
use crate::model::{AuthZoneParams, Executable};
use radix_engine_interface::blueprints::resource::NonFungibleGlobalId;
use radix_engine_interface::crypto::hash;
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct SystemTransaction {
    pub instructions: Vec<Instruction>,
    pub pre_allocated_ids: BTreeSet<NodeId>,
    pub blobs: Vec<Vec<u8>>,
    pub nonce: u64,
}

impl SystemTransaction {
    pub fn get_executable<'a>(
        &'a self,
        initial_proofs: BTreeSet<NonFungibleGlobalId>,
    ) -> Executable<'a> {
        // Fake transaction hash
        let transaction_hash = hash(self.nonce.to_le_bytes());

        let auth_zone_params = AuthZoneParams {
            initial_proofs,
            virtual_resources: BTreeSet::new(),
        };

        Executable::new(
            self.instructions.clone(),
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
