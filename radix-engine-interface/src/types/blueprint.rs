use crate::ScryptoSbor;
use radix_engine_common::types::GlobalAddress;
use radix_engine_common::types::PackageAddress;
use sbor::rust::string::String;
use sbor::rust::string::ToString;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ObjectInfo {
    pub blueprint: Blueprint,
    pub global: bool,
    pub type_parent: Option<GlobalAddress>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct Blueprint {
    pub package_address: PackageAddress,
    pub blueprint_name: String,
}

impl Blueprint {
    pub fn new(package_address: &PackageAddress, blueprint_name: &str) -> Self {
        Blueprint {
            package_address: *package_address,
            blueprint_name: blueprint_name.to_string(),
        }
    }

    pub fn size(&self) -> usize {
        self.package_address.as_ref().len() + self.blueprint_name.len()
    }
}
