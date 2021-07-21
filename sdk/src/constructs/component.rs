extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::abi::*;
use crate::buffer::*;
use crate::constructs::*;
use crate::types::*;
use crate::*;

/// A self-executing program that holds resources and exposed actions to other entities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    address: Address,
}

impl From<Address> for Component {
    fn from(address: Address) -> Self {
        Self { address }
    }
}

impl Component {
    pub fn new<T: Serialize>(kind: &str, state: T) -> Self {
        let input = CreateComponentInput {
            kind: kind.to_string(),
            state: radix_encode(&state),
        };
        let output: CreateComponentOutput = call_kernel!(CREATE_COMPONENT, input);

        Self::from(output.component)
    }

    pub fn call(&self, method: &str, args: Vec<SerializedValue>) -> SerializedValue {
        let data = self.get_info();

        let mut args_buf = Vec::new();
        args_buf.push(radix_encode(&self));
        args_buf.extend(args);

        let input = CallBlueprintInput {
            blueprint: data.blueprint,
            component: data.kind,
            method: method.to_string(),
            args: args_buf,
        };
        let output: CallBlueprintOutput = call_kernel!(CALL_BLUEPRINT, input);

        output.rtn
    }

    pub fn get_info(&self) -> ComponentInfo {
        let input = GetComponentInfoInput {
            component: self.address.clone(),
        };
        let output: GetComponentInfoOutput = call_kernel!(GET_COMPONENT_INFO, input);

        output.result.unwrap()
    }

    pub fn get_blueprint(&self) -> Blueprint {
        self.get_info().blueprint.into()
    }

    pub fn get_kind(&self) -> String {
        self.get_info().kind
    }

    pub fn get_state<T: DeserializeOwned>(&self) -> T {
        let input = GetComponentStateInput {
            component: self.address.clone(),
        };
        let output: GetComponentStateOutput = call_kernel!(GET_COMPONENT_STATE, input);

        radix_decode(&output.state)
    }

    pub fn put_state<T: Serialize>(&self, state: T) {
        let input = PutComponentStateInput {
            component: self.address.clone(),
            state: radix_encode(&state),
        };
        let _: PutComponentStateOutput = call_kernel!(PUT_COMPONENT_STATE, input);
    }

    pub fn address(&self) -> Address {
        self.address.clone()
    }
}
