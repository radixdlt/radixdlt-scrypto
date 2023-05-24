use crate::prelude::*;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Sbor)]
#[sbor(transparent)]
pub struct SystemTransactionHash(pub [u8; Self::LENGTH]);

impl SystemTransactionHash {
    pub const LENGTH: usize = 32;

    pub fn from_hash(hash: Hash) -> Self {
        Self(hash.0)
    }

    pub fn into_bytes(self) -> [u8; Self::LENGTH] {
        self.0
    }
}

impl AsRef<[u8]> for SystemTransactionHash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl IsHash for SystemTransactionHash {
    fn into_bytes(self) -> [u8; Hash::LENGTH] {
        self.0
    }
}

impl fmt::Display for SystemTransactionHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl fmt::Debug for SystemTransactionHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SystemTransactionHash")
            .field(&hex::encode(self.0))
            .finish()
    }
}

pub trait HasSystemTransactionHash {
    fn system_transaction_hash(&self) -> SystemTransactionHash;
}
