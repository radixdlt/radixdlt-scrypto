use crate::model::{AuthModule, TransactionManifest, Validated};
use sbor::*;
use scrypto::crypto::Hash;

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct SystemTransaction {
    // TODO: Add header
    pub manifest: TransactionManifest,
}

impl Into<Validated<SystemTransaction>> for SystemTransaction {
    fn into(self) -> Validated<SystemTransaction> {
        let transaction_hash = Hash([0u8; Hash::LENGTH]);
        let instructions = self.manifest.instructions.clone();
        let blobs = self.manifest.blobs.clone();

        Validated::new(
            self,
            transaction_hash,
            instructions,
            vec![AuthModule::system_role_nf_address()],
            10_000,
            0,
            blobs,
        )
    }
}
