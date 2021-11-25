use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::core::*;
use crate::kernel::*;
use crate::rust::borrow::ToOwned;
use crate::rust::vec;
use crate::rust::vec::Vec;
use crate::types::*;
use crate::utils::*;

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
    pub fn new<T: State>(state: T) -> Self {
        let input = CreateComponentInput {
            name: T::name().to_owned(),
            state: scrypto_encode(&state),
        };
        let output: CreateComponentOutput = call_kernel(CREATE_COMPONENT, input);

        output.component.into()
    }

    pub fn call<T: Decode>(&self, method: &str, args: Vec<Vec<u8>>) -> T {
        let output = call_method(self.address, method, args);

        scrypto_unwrap(scrypto_decode(&output))
    }

    pub fn get_state<T: State>(&self) -> T {
        let input = GetComponentStateInput {
            component: self.address,
        };
        let output: GetComponentStateOutput = call_kernel(GET_COMPONENT_STATE, input);

        scrypto_unwrap(scrypto_decode(&output.state))
    }

    pub fn put_state<T: State>(&self, state: T) {
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
