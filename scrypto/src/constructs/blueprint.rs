use crate::kernel::*;
use crate::types::rust::string::ToString;
use crate::types::rust::vec::Vec;
use crate::types::*;

/// A piece of code that defines the structure and methods of components.
#[derive(Debug)]
pub struct Blueprint {
    address: Address,
}

impl From<Address> for Blueprint {
    fn from(address: Address) -> Self {
        Self { address }
    }
}

impl Into<Address> for Blueprint {
    fn into(self) -> Address {
        self.address
    }
}

impl Blueprint {
    pub fn new(code: &[u8]) -> Self {
        let input = PublishBlueprintInput {
            code: code.to_vec(),
        };
        let output: PublishBlueprintOutput = call_kernel(PUBLISH_BLUEPRINT, input);

        output.blueprint.into()
    }

    pub fn call(&self, component: &str, method: &str, args: Vec<Vec<u8>>) -> Vec<u8> {
        let input = CallBlueprintInput {
            blueprint: self.address,
            component: component.to_string(),
            method: method.to_string(),
            args,
        };
        let output: CallBlueprintOutput = call_kernel(CALL_BLUEPRINT, input);

        output.rtn
    }

    pub fn address(&self) -> Address {
        self.address
    }
}
