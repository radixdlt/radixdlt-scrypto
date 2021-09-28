use sbor::*;
use scrypto::rust::vec::Vec;

/// A collection of blueprints, compiled and published as a single unit.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
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
