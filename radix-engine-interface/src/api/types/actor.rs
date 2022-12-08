use radix_engine_interface::api::types::{NativeFunction, NativeMethod};
use sbor::rust::string::String;
use crate::api::types::TransactionProcessorFunction;

use crate::model::*;
use crate::scrypto;

#[derive(Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub enum FnIdentifier {
    Scrypto(ScryptoFnIdentifier),
    NativeFunction(NativeFunction),
    NativeMethod(NativeMethod),
}

impl FnIdentifier {
    pub fn is_scrypto_or_transaction(&self) -> bool {
        matches!(
            self,
            FnIdentifier::Scrypto(..)
                | FnIdentifier::NativeFunction(NativeFunction::TransactionProcessor(TransactionProcessorFunction::Run))
        )
    }
}


#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
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
