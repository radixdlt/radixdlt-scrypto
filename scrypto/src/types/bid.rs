extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use sbor::{Decode, Describe, Encode};

use crate::types::Hash;
use crate::utils::hex_encode;

/// Resource bucket id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Describe, Encode, Decode)]
pub enum BID {
    Transient(u32),

    Persisted(Hash, u32),
}

impl BID {
    pub fn is_transient(&self) -> bool {
        match self {
            Self::Transient(_) => true,
            _ => false,
        }
    }

    pub fn is_persisted(&self) -> bool {
        !self.is_transient()
    }
}

impl ToString for BID {
    fn to_string(&self) -> String {
        let mut buf = Vec::new();
        match self {
            Self::Transient(index) => {
                buf.push(1u8);
                buf.extend(index.to_le_bytes());
            }
            Self::Persisted(hash, index) => {
                buf.push(2u8);
                buf.extend(hash.slice());
                buf.extend(index.to_le_bytes());
            }
        }
        hex_encode(buf)
    }
}
