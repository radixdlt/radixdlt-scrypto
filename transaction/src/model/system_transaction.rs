use crate::model::{AuthModule, AuthZoneParams, Executable, TransactionManifest};
use radix_engine_lib::crypto::Hash;
use scrypto::scrypto;
use std::collections::BTreeSet;

use super::{ExecutionContext, FeePayment, DEFAULT_COST_UNIT_LIMIT};

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct SystemTransaction {
    pub manifest: TransactionManifest,
}

impl SystemTransaction {
    pub fn get_executable<'a>(&'a self) -> Executable<'a> {
        let transaction_hash = Hash([0u8; Hash::LENGTH]);

        let auth_zone_params = AuthZoneParams {
            initial_proofs: vec![AuthModule::system_role_non_fungible_address()],
            virtualizable_proofs_resource_addresses: BTreeSet::new(),
        };

        Executable::new(
            &self.manifest.instructions,
            &self.manifest.blobs,
            ExecutionContext {
                transaction_hash,
                auth_zone_params,
                fee_payment: FeePayment {
                    cost_unit_limit: DEFAULT_COST_UNIT_LIMIT,
                    tip_percentage: 0,
                },
                runtime_validations: vec![],
            },
        )
    }
}
