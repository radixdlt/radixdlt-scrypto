use radix_engine_interface::crypto::{hash, Hash, PublicKey};
use radix_engine_interface::data::scrypto_encode;
use radix_engine_interface::*;
use sbor::*;

use crate::model::TransactionIntent;

#[derive(Debug, Clone, Categorize, Encode, Decode, PartialEq, Eq)]
pub struct PreviewFlags {
    pub unlimited_loan: bool,
    pub assume_all_signature_proofs: bool,
    pub permit_duplicate_intent_hash: bool,
    pub permit_invalid_header_epoch: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct PreviewIntent {
    pub intent: TransactionIntent,
    pub signer_public_keys: Vec<PublicKey>,
    pub flags: PreviewFlags,
}

impl PreviewIntent {
    pub fn hash(&self) -> Result<Hash, EncodeError> {
        Ok(hash(self.to_bytes()?))
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, EncodeError> {
        scrypto_encode(self)
    }
}
