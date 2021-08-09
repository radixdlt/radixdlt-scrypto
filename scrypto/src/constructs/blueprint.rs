extern crate alloc;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::kernel::*;
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

impl Blueprint {
    pub fn new(code: &[u8]) -> Address {
        let input = PublishBlueprintInput {
            code: code.to_vec(),
        };
        let output: PublishBlueprintOutput = call_kernel(PUBLISH_BLUEPRINT, input);

        output.blueprint
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
