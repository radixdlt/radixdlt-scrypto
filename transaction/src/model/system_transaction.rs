use crate::model::{AuthModule, Executable, AuthZoneParams, TransactionManifest};
use sbor::*;
use scrypto::crypto::Hash;
use std::collections::BTreeSet;

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct SystemTransaction {
    // TODO: Add header
    pub manifest: TransactionManifest,
}

impl Into<Executable> for SystemTransaction {
    fn into(self) -> Executable {
        let transaction_hash = Hash([0u8; Hash::LENGTH]);
        let instructions = self.manifest.instructions;
        let blobs = self.manifest.blobs;

        let auth_zone_params = AuthZoneParams {
            initial_proofs: vec![AuthModule::system_role_nf_address()],
            virtualizable_proofs_resource_addresses: BTreeSet::new(),
        };

        Executable::new(transaction_hash, instructions, auth_zone_params, 10_000, 0, blobs)
    }
}
