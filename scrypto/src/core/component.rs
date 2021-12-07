use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::core::*;
use crate::kernel::*;
use crate::rust::borrow::ToOwned;
use crate::rust::vec;
use crate::rust::vec::Vec;
use crate::types::*;
use crate::utils::*;

/// Represents the state of a component.
pub trait ComponentState: sbor::Encode + sbor::Decode {
    fn blueprint_name() -> &'static str;

    fn instantiate(self) -> Component;
}

/// An instance of a blueprint, which lives in the ledger state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    address: Address,
}

impl From<Address> for Component {
    fn from(address: Address) -> Self {
        if !address.is_component() {
            panic!("{} is not a component address", address);
        }

        Self { address }
    }
}

impl From<Component> for Address {
    fn from(a: Component) -> Address {
        a.address
    }
}

impl Component {
    pub fn new<T: ComponentState>(state: T) -> Self {
        let input = CreateComponentInput {
            blueprint_name: T::blueprint_name().to_owned(),
            state: scrypto_encode(&state),
        };
        let output: CreateComponentOutput = call_kernel(CREATE_COMPONENT, input);

        output.component_address.into()
    }

    pub fn call<T: Decode>(&self, method: &str, args: Vec<Vec<u8>>) -> T {
        let output = call_method(self.address, method, args);

        scrypto_unwrap(scrypto_decode(&output))
    }

    pub fn get_state<T: ComponentState>(&self) -> T {
        let input = GetComponentStateInput {
            component_address: self.address,
        };
        let output: GetComponentStateOutput = call_kernel(GET_COMPONENT_STATE, input);

        scrypto_unwrap(scrypto_decode(&output.state))
    }

    pub fn put_state<T: ComponentState>(&self, state: T) {
        let input = PutComponentStateInput {
            component_address: self.address,
            state: scrypto_encode(&state),
        };
        let _: PutComponentStateOutput = call_kernel(PUT_COMPONENT_STATE, input);
    }

    pub fn blueprint(&self) -> Blueprint {
        let input = GetComponentInfoInput {
            component_address: self.address,
        };
        let output: GetComponentInfoOutput = call_kernel(GET_COMPONENT_INFO, input);

        Blueprint::from((output.package_address, output.blueprint_name))
    }

    pub fn address(&self) -> Address {
        self.address
    }
}

//========
// SBOR
//========

impl TypeId for Component {
    fn type_id() -> u8 {
        Address::type_id()
    }
}

impl Encode for Component {
    fn encode_value(&self, encoder: &mut Encoder) {
        self.address.encode_value(encoder);
    }
}

impl Decode for Component {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        Address::decode_value(decoder).map(Into::into)
    }
}

impl Describe for Component {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_COMPONENT.to_owned(),
            generics: vec![],
        }
    }
}
