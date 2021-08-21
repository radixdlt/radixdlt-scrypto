use sbor::*;
use scrypto::types::rust::vec::Vec;

/// A package is a piece of code published on-chain.
#[derive(Debug, Clone, Describe, Encode, Decode)]
pub struct Package {
    code: Vec<u8>,
}

impl Package {
    pub fn new(code: Vec<u8>) -> Self {
        Self { code }
    }

    pub fn code(&self) -> &[u8] {
        &self.code
    }
}
