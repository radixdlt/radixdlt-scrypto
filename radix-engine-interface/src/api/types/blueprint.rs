use radix_engine_common::data::scrypto::model::PackageAddress;
use crate::ScryptoSbor;
use sbor::rust::string::String;

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
}