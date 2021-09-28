use sbor::{describe::Type, *};

use crate::constants::*;
use crate::constructs::*;
use crate::kernel::*;
use crate::rust::borrow::ToOwned;
use crate::types::*;

/// A collection of blueprints, compiles and published as a single unit.
#[derive(Debug, TypeId, Encode, Decode)]
pub struct Package {
    address: Address,
}

impl From<Address> for Package {
    fn from(address: Address) -> Self {
        Self { address }
    }
}

impl From<Package> for Address {
    fn from(a: Package) -> Address {
        a.address
    }
}

impl Package {
    pub fn new(code: &[u8]) -> Self {
        let input = PublishPackageInput {
            code: code.to_vec(),
        };
        let output: PublishPackageOutput = call_kernel(PUBLISH, input);

        output.package.into()
    }

    pub fn blueprint(&self, name: &str) -> Blueprint {
        Blueprint::from((self.address, name))
    }

    pub fn address(&self) -> Address {
        self.address
    }
}

impl Describe for Package {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_PACKAGE.to_owned(),
        }
    }
}
