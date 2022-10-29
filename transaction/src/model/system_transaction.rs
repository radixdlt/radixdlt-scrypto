use crate::model::{AuthModule, AuthZoneParams, Executable, TransactionManifest};
use sbor::*;
use scrypto::crypto::Hash;
use std::collections::BTreeSet;

use super::{ExecutionContext, FeePayment, IntentValidation, DEFAULT_COST_UNIT_LIMIT};

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct SystemTransaction {
    pub manifest: TransactionManifest,
}

impl SystemTransaction {
    pub fn get_executable<'a>(&'a self) -> Executable<'a> {
        let transaction_hash = Hash([0u8; Hash::LENGTH]);

        let auth_zone_params = AuthZoneParams {
            initial_proofs: vec![AuthModule::system_role_nf_address()],
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
                intent_validation: IntentValidation::None,
            },
        )
    }
}
