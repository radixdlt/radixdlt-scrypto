use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::constants::*;
use crate::constructs::*;
use crate::kernel::*;
use crate::rust::borrow::ToOwned;
use crate::rust::string::ToString;
use crate::rust::vec::Vec;
use crate::types::*;
use crate::utils::*;

/// A self-executing program that holds resources and exposed actions to other entities.
#[derive(Debug, Encode, Decode)]
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
    pub fn new<T: Encode + crate::traits::Blueprint>(state: T) -> Self {
        let input = CreateComponentInput {
            blueprint: T::name().to_string(),
            state: scrypto_encode(&state),
        };
        let output: CreateComponentOutput = call_kernel(CREATE_COMPONENT, input);

        output.component.into()
    }

    pub fn call<T: Decode>(&self, method: &str, args: Vec<Vec<u8>>) -> T {
        let input = CallMethodInput {
            component: self.address,
            method: method.to_string(),
            args,
        };
        let output: CallMethodOutput = call_kernel(CALL_METHOD, input);

        unwrap_or_panic(scrypto_decode(&output.rtn))
    }

    pub fn get_info(&self) -> ComponentInfo {
        let input = GetComponentInfoInput {
            component: self.address,
        };
        let output: GetComponentInfoOutput = call_kernel(GET_COMPONENT_INFO, input);

        ComponentInfo {
            package: output.package,
            blueprint: output.blueprint,
        }
    }

    pub fn get_blueprint(&self) -> Blueprint {
        let info = self.get_info();
        Blueprint::from(info.package, info.blueprint.as_str())
    }

    pub fn get_state<T: Decode>(&self) -> T {
        let input = GetComponentStateInput {
            component: self.address,
        };
        let output: GetComponentStateOutput = call_kernel(GET_COMPONENT_STATE, input);

        unwrap_or_panic(scrypto_decode(&output.state))
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

impl Describe for Component {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_COMPONENT.to_owned(),
        }
    }
}
