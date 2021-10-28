use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::core::*;
use crate::rust::borrow::ToOwned;
use crate::rust::string::String;
use crate::rust::vec;
use crate::rust::vec::Vec;
use crate::types::*;
use crate::utils::*;

/// A template that describes shared structure and behavior.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Blueprint {
    package: Package,
    name: String,
}

impl<A: Into<Package>, S: AsRef<str>> From<(A, S)> for Blueprint {
    fn from(a: (A, S)) -> Self {
        Self {
            package: a.0.into(),
            name: a.1.as_ref().to_owned(),
        }
    }
}

impl From<Blueprint> for (Address, String) {
    fn from(blueprint: Blueprint) -> Self {
        (blueprint.package.address(), blueprint.name)
    }
}

impl Blueprint {
    pub fn package(&self) -> Package {
        self.package.clone()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn call<T: Decode>(&self, function: &str, args: Vec<Vec<u8>>) -> T {
        let output = call_function(self.package.address(), self.name(), function, args);

        scrypto_unwrap(scrypto_decode(&output))
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
        (self.package.address(), self.name.to_owned()).encode_value(encoder);
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
            generics: vec![],
        }
    }
}
