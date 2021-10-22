use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::kernel::*;
use crate::rust::borrow::ToOwned;
use crate::rust::vec;
use crate::types::*;

/// A collection of blueprints, compiles and published as a single unit.
#[derive(Debug, Clone, PartialEq, Eq)]
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
        let output: PublishPackageOutput = call_kernel(PUBLISH_PACKAGE, input);

        output.package.into()
    }

    pub fn address(&self) -> Address {
        self.address
    }
}

//========
// SBOR
//========

impl TypeId for Package {
    fn type_id() -> u8 {
        Address::type_id()
    }
}

impl Encode for Package {
    fn encode_value(&self, encoder: &mut Encoder) {
        self.address.encode_value(encoder);
    }
}

impl Decode for Package {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        Address::decode_value(decoder).map(Into::into)
    }
}

impl Describe for Package {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_PACKAGE.to_owned(),
            generics: vec![],
        }
    }
}
