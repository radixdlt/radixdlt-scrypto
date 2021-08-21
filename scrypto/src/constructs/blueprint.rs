use crate::buffer::*;
use crate::kernel::*;
use crate::types::rust::borrow::ToOwned;
use crate::types::rust::string::String;
use crate::types::rust::string::ToString;
use crate::types::rust::vec::Vec;
use crate::types::*;
use sbor::*;

/// A piece of code that defines the structure and methods of components.
#[derive(Debug)]
pub struct Blueprint {
    package: Address,
    name: String,
}

impl Blueprint {
    pub fn from(package: Address, name: &str) -> Self {
        Self {
            package,
            name: name.to_owned(),
        }
    }

    pub fn invoke<T: Decode>(&self, function: &str, args: Vec<Vec<u8>>) -> T {
        let input = CallBlueprintInput {
            package: self.package,
            name: self.name.clone(),
            function: function.to_string(),
            args,
        };
        let output: CallBlueprintOutput = call_kernel(CALL_BLUEPRINT, input);

        scrypto_decode(&output.rtn).unwrap()
    }

    pub fn package(&self) -> Address {
        self.package
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
