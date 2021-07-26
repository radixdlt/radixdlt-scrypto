extern crate alloc;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use crate::kernel::*;
use crate::types::*;

/// A piece of code that defines the structure and methods of components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blueprint {
    address: Address,
}

impl From<Address> for Blueprint {
    fn from(address: Address) -> Self {
        Self { address }
    }
}

impl Blueprint {
    pub fn new(code: &[u8]) -> Self {
        let input = PublishBlueprintInput {
            code: code.to_vec(),
        };
        let output: PublishBlueprintOutput = syscall(PUBLISH_BLUEPRINT, input);

        Self::from(output.blueprint)
    }

    pub fn call(
        &self,
        component: &str,
        method: &str,
        args: Vec<SerializedValue>,
    ) -> SerializedValue {
        let input = CallBlueprintInput {
            blueprint: self.address.clone(),
            component: component.to_string(),
            method: method.to_string(),
            args,
        };
        let output: CallBlueprintOutput = syscall(CALL_BLUEPRINT, input);

        output.rtn
    }

    pub fn address(&self) -> Address {
        self.address.clone()
    }
}
