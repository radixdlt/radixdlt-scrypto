use crate::api::package::PackageAddress;
use crate::api::types::*;
use crate::*;
use sbor::rust::string::String;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum PackageIdentifier {
    Scrypto(PackageAddress),
    None,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum NativePackage {
    Auth,
    Root,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum InvocationIdentifier {
    Transaction,
    Function(PackageAddress, String, String),
    Method(ScryptoReceiver, NodeModuleId, String),
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum FnIdentifier {
    Scrypto(ScryptoFnIdentifier),
    None,
}

impl FnIdentifier {
    pub fn package_identifier(&self) -> PackageIdentifier {
        match self {
            FnIdentifier::Scrypto(identifier) => {
                PackageIdentifier::Scrypto(identifier.package_address)
            }
            FnIdentifier::None => PackageIdentifier::None,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ScryptoFnIdentifier {
    pub package_address: PackageAddress,
    pub blueprint_name: String,
    pub ident: String,
}

impl ScryptoFnIdentifier {
    pub fn new(package_address: PackageAddress, blueprint_name: String, ident: String) -> Self {
        Self {
            package_address,
            blueprint_name,
            ident,
        }
    }

    pub fn package_address(&self) -> &PackageAddress {
        &self.package_address
    }

    pub fn blueprint_name(&self) -> &String {
        &self.blueprint_name
    }
}