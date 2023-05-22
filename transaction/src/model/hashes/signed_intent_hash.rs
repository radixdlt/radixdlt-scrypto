use crate::prelude::*;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Sbor)]
#[sbor(transparent)]
pub struct SignedIntentHash(pub [u8; Self::LENGTH]);

impl SignedIntentHash {
    pub const LENGTH: usize = 32;

    pub fn from_hash(hash: Hash) -> Self {
        Self(hash.0)
    }

    pub fn into_bytes(self) -> [u8; Self::LENGTH] {
        self.0
    }
}

impl AsRef<[u8]> for SignedIntentHash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl IsHash for SignedIntentHash {
    fn into_bytes(self) -> [u8; Hash::LENGTH] {
        self.0
    }
}

impl fmt::Display for SignedIntentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl fmt::Debug for SignedIntentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SignedIntentHash")
            .field(&hex::encode(self.0))
            .finish()
    }
}

pub trait HasSignedIntentHash {
    fn signed_intent_hash(&self) -> SignedIntentHash;
}
