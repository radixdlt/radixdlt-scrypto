use radix_engine_interface::crypto::hash;
use radix_engine_interface::data::scrypto_encode;
use radix_engine_interface::model::*;
use radix_engine_interface::*;
use sbor::rust::vec::Vec;
use std::collections::BTreeSet;

use crate::model::*;

#[derive(ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct TestTransaction {
    nonce: u64,
    cost_unit_limit: u32,
    manifest: TransactionManifest,
}

impl TestTransaction {
    pub fn new(manifest: TransactionManifest, nonce: u64, cost_unit_limit: u32) -> Self {
        Self {
            nonce,
            cost_unit_limit,
            manifest,
        }
    }

    pub fn get_executable<'a>(
        &'a self,
        initial_proofs: Vec<NonFungibleGlobalId>,
    ) -> Executable<'a> {
        let payload = scrypto_encode(self).unwrap();
        let payload_size = payload.len();
        let transaction_hash = hash(payload);

        Executable::new(
            InstructionList::Basic(&self.manifest.instructions),
            &self.manifest.blobs,
            ExecutionContext {
                transaction_hash,
                payload_size,
                auth_zone_params: AuthZoneParams {
                    initial_proofs,
                    virtualizable_proofs_resource_addresses: BTreeSet::new(),
                },
                fee_payment: FeePayment::User {
                    cost_unit_limit: self.cost_unit_limit,
                    tip_percentage: 0,
                },
                runtime_validations: vec![],
                pre_allocated_ids: BTreeSet::new(),
            },
        )
    }
}
