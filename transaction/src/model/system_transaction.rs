use crate::model::{AuthModule, Executable, TransactionManifest};
use sbor::*;
use scrypto::crypto::Hash;

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

        Executable::new(
            transaction_hash,
            instructions,
            vec![AuthModule::system_role_nf_address()],
            10_000,
            0,
            blobs,
        )
    }
}
