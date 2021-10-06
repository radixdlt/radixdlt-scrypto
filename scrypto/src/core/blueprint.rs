use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::core::*;
use crate::rust::borrow::ToOwned;
use crate::rust::string::String;
use crate::types::*;

/// A template that describes shared structure and behavior.
#[derive(Debug, PartialEq, Eq)]
pub struct Blueprint {
    package: Address,
    name: String,
}

impl<A: Into<Address>, S: AsRef<str>> From<(A, S)> for Blueprint {
    fn from(a: (A, S)) -> Self {
        Self {
            package: a.0.into(),
            name: a.1.as_ref().to_owned(),
        }
    }
}

impl From<Blueprint> for (Address, String) {
    fn from(blueprint: Blueprint) -> Self {
        (blueprint.package, blueprint.name)
    }
}

impl Blueprint {
    pub fn package(&self) -> Package {
        self.package.into()
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

//========
// SBOR
//========

impl TypeId for Blueprint {
    fn type_id() -> u8 {
        <(Address, String)>::type_id()
    }
}

impl Encode for Blueprint {
    fn encode_value(&self, encoder: &mut Encoder) {
        (self.package, self.name.to_owned()).encode_value(encoder);
    }
}

impl Decode for Blueprint {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        <(Address, String)>::decode_value(decoder).map(Into::into)
    }
}

impl Describe for Blueprint {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_BLUEPRINT.to_owned(),
        }
    }
}
