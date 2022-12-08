use sbor::rust::string::String;

use crate::model::*;
use crate::scrypto;

#[derive(Debug, Clone)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ScryptoActor {
    pub package_address: PackageAddress,
    pub blueprint_name: String,
}

impl ScryptoActor {
    pub fn new(package_address: PackageAddress, blueprint_name: String) -> Self {
        Self {
            package_address,
            blueprint_name,
        }
    }

    pub fn package_address(&self) -> &PackageAddress {
        &self.package_address
    }

    pub fn blueprint_name(&self) -> &String {
        &self.blueprint_name
    }
}
