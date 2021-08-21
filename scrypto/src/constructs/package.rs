use sbor::model::*;
use sbor::{Decode, Describe, Encode};

use crate::constructs::*;
use crate::kernel::*;
use crate::types::rust::borrow::ToOwned;
use crate::types::*;

/// A package consists of blueprints.
#[derive(Debug, Encode, Decode)]
pub struct Package {
    address: Address,
}

impl Describe for Package {
    fn describe() -> Type {
        Type::SystemType {
            name: "::scrypto::constructs::Package".to_owned(),
        }
    }
}

impl From<Address> for Package {
    fn from(address: Address) -> Self {
        Self { address }
    }
}

impl Into<Address> for Package {
    fn into(self) -> Address {
        self.address
    }
}

impl Package {
    pub fn new(code: &[u8]) -> Self {
        let input = PublishPackageInput {
            code: code.to_vec(),
        };
        let output: PublishPackageOutput = call_kernel(PUBLISH_PACKAGE, input);

        output.package.into()
    }

    pub fn blueprint(&self, name: &str) -> Blueprint {
        Blueprint::from(self.address, name)
    }

    pub fn address(&self) -> Address {
        self.address
    }
}
