use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::constructs::*;
use crate::kernel::*;
use crate::rust::borrow::ToOwned;
use crate::rust::string::String;
use crate::rust::string::ToString;
use crate::rust::vec::Vec;
use crate::types::*;
use crate::utils::*;

/// A template that describes the common behavior and state structure of its instances.
#[derive(Debug, PartialEq, Eq, TypeId, Encode, Decode)]
pub struct Blueprint {
    package: Address,
    name: String,
}

impl<T: AsRef<str>> From<(Address, T)> for Blueprint {
    fn from(address: (Address, T)) -> Self {
        Self {
            package: address.0,
            name: address.1.as_ref().to_owned(),
        }
    }
}

impl From<Blueprint> for (Address, String) {
    fn from(a: Blueprint) -> Self {
        (a.package, a.name)
    }
}

impl Blueprint {
    pub fn call<T: Decode>(&self, function: &str, args: Vec<Vec<u8>>) -> T {
        let input = CallFunctionInput {
            blueprint: (self.package, self.name.clone()),
            function: function.to_string(),
            args,
        };
        let output: CallFunctionOutput = call_kernel(CALL_FUNCTION, input);

        unwrap_light(scrypto_decode(&output.rtn))
    }

    pub fn package(&self) -> Package {
        self.package.into()
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Describe for Blueprint {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_BLUEPRINT.to_owned(),
        }
    }
}
