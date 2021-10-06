use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::core::*;
use crate::kernel::*;
use crate::rust::borrow::ToOwned;
use crate::rust::string::ToString;
use crate::types::*;
use crate::utils::*;

/// An instance of a blueprint, which lives in the ledger state.
#[derive(Debug, PartialEq, Eq, TypeId, Encode, Decode)]
pub struct Component {
    address: Address,
}

impl From<Address> for Component {
    fn from(address: Address) -> Self {
        Self { address }
    }
}

impl From<Component> for Address {
    fn from(a: Component) -> Address {
        a.address
    }
}

impl Component {
    pub fn new<T: Encode + crate::traits::Blueprint>(state: T) -> Self {
        let input = CreateComponentInput {
            name: T::name().to_string(),
            state: scrypto_encode(&state),
        };
        let output: CreateComponentOutput = call_kernel(CREATE_COMPONENT, input);

        output.component.into()
    }

    pub fn get_state<T: Decode>(&self) -> T {
        let input = GetComponentStateInput {
            component: self.address,
        };
        let output: GetComponentStateOutput = call_kernel(GET_COMPONENT_STATE, input);

        unwrap_light(scrypto_decode(&output.state))
    }

    pub fn put_state<T: Encode>(&self, state: T) {
        let input = PutComponentStateInput {
            component: self.address,
            state: scrypto_encode(&state),
        };
        let _: PutComponentStateOutput = call_kernel(PUT_COMPONENT_STATE, input);
    }

    pub fn blueprint(&self) -> Blueprint {
        let input = GetComponentBlueprintInput {
            component: self.address,
        };
        let output: GetComponentBlueprintOutput = call_kernel(GET_COMPONENT_BLUEPRINT, input);

        Blueprint::from((output.package, output.name))
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
