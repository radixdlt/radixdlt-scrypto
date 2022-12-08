use super::{ExecutionContext, FeePayment, Instruction, InstructionList};
use crate::model::{AuthModule, AuthZoneParams, Executable};
use radix_engine_interface::crypto::Hash;
use radix_engine_interface::scrypto;
use sbor::*;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct SystemTransaction {
    pub instructions: Vec<Instruction>,
    pub blobs: Vec<Vec<u8>>,
}

impl SystemTransaction {
    pub fn get_executable<'a>(&'a self) -> Executable<'a> {
        let transaction_hash = Hash([0u8; Hash::LENGTH]);

        let auth_zone_params = AuthZoneParams {
            initial_proofs: vec![AuthModule::system_role_non_fungible_address()],
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
            },
        )
    }
}
