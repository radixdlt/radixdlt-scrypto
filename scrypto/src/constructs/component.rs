use sbor::{Decode, Encode};

use crate::buffer::*;
use crate::constructs::*;
use crate::kernel::*;
use crate::types::rust::string::ToString;
use crate::types::rust::vec::Vec;
use crate::types::*;

/// A self-executing program that holds resources and exposed actions to other entities.
#[derive(Debug)]
pub struct Component {
    address: Address,
}

impl From<Address> for Component {
    fn from(address: Address) -> Self {
        Self { address }
    }
}

impl Into<Address> for Component {
    fn into(self) -> Address {
        self.address
    }
}

impl Component {
    pub fn new<T: Encode>(name: &str, state: T) -> Self {
        let input = CreateComponentInput {
            name: name.to_string(),
            state: scrypto_encode(&state),
        };
        let output: CreateComponentOutput = call_kernel(CREATE_COMPONENT, input);

        output.component.into()
    }

    pub fn invoke<T: Decode>(&self, method: &str, args: Vec<Vec<u8>>) -> T {
        let input = CallComponentInput {
            component: self.address,
            method: method.to_string(),
            args,
        };
        let output: CallComponentOutput = call_kernel(CALL_COMPONENT, input);

        scrypto_decode(&output.rtn).unwrap()
    }

    pub fn get_info(&self) -> ComponentInfo {
        let input = GetComponentInfoInput {
            component: self.address,
        };
        let output: GetComponentInfoOutput = call_kernel(GET_COMPONENT_INFO, input);

        output.result.unwrap()
    }

    pub fn get_blueprint(&self) -> Blueprint {
        let info = self.get_info();
        Blueprint::from(info.package, info.name.as_str())
    }

    pub fn get_state<T: Decode>(&self) -> T {
        let input = GetComponentStateInput {
            component: self.address,
        };
        let output: GetComponentStateOutput = call_kernel(GET_COMPONENT_STATE, input);

        scrypto_decode(&output.state).unwrap()
    }

    pub fn put_state<T: Encode>(&self, state: T) {
        let input = PutComponentStateInput {
            component: self.address,
            state: scrypto_encode(&state),
        };
        let _: PutComponentStateOutput = call_kernel(PUT_COMPONENT_STATE, input);
    }

    pub fn address(&self) -> Address {
        self.address
    }
}
