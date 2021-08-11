use sbor::*;
use scrypto::kernel::*;

/// Represents a resource created.
#[derive(Debug, Clone, Encode, Decode)]
pub struct Resource {
    info: ResourceInfo,
}

impl Resource {
    pub fn new(info: ResourceInfo) -> Self {
        Self { info }
    }

    pub fn info(&self) -> &ResourceInfo {
        &self.info
    }
}
