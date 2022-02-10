use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::core::*;
use crate::engine::*;
use crate::misc::*;
use crate::rust::borrow::ToOwned;
use crate::rust::fmt;
use crate::rust::str::FromStr;
use crate::rust::string::ToString;
use crate::rust::vec::Vec;
use crate::types::*;

/// Represents the state of a component.
pub trait ComponentState: Encode + Decode {
    /// Returns the blueprint name.
    fn blueprint_name() -> &'static str;

    /// Instantiates a component from this data structure.
    fn instantiate(self) -> Component;
}

/// An instance of a blueprint, which lives in the ledger state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Component(pub [u8; 26]);

impl Component {
    fn this(&self) -> Self {
        Self(self.0)
    }

    fn this_package() -> Package {
        match Context::actor() {
            Actor::Blueprint(package, _) => package,
            Actor::Component(component) => component.blueprint().0,
        }
    }

    /// Instantiates a new component.
    pub fn new<T: ComponentState>(state: T) -> Self {
        // TODO: more thoughts are needed for this interface
        let input = CreateComponentInput {
            blueprint: (Self::this_package(), T::blueprint_name().to_owned()),
            state: scrypto_encode(&state),
        };
        let output: CreateComponentOutput = call_engine(CREATE_COMPONENT, input);

        output.component
    }

    /// Invokes a method on this component.
    pub fn call<T: Decode>(&self, method: &str, args: Vec<Vec<u8>>) -> T {
        let output = Context::call_method(self.this(), method, args);

        scrypto_decode(&output).unwrap()
    }

    /// Returns the state of this component.
    pub fn get_state<T: ComponentState>(&self) -> T {
        let input = GetComponentStateInput {
            component: self.this(),
        };
        let output: GetComponentStateOutput = call_engine(GET_COMPONENT_STATE, input);

        scrypto_decode(&output.state).unwrap()
    }

    /// Updates the state of this component.
    pub fn put_state<T: ComponentState>(&self, state: T) {
        let input = PutComponentStateInput {
            component: self.this(),
            state: scrypto_encode(&state),
        };
        let _: PutComponentStateOutput = call_engine(PUT_COMPONENT_STATE, input);
    }

    /// Returns the blueprint that this component is instantiated from.
    pub fn blueprint(&self) -> (Package, String) {
        let input = GetComponentInfoInput {
            component: self.this(),
        };
        let output: GetComponentInfoOutput = call_engine(GET_COMPONENT_INFO, input);
        output.blueprint
    }
}

//========
// error
//========

#[derive(Debug, Clone)]
pub enum ParseComponentError {
    InvalidHex(hex::FromHexError),
    InvalidLength(usize),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseComponentError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseComponentError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for Component {
    type Error = ParseComponentError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            26 => Ok(Self(copy_u8_array(slice))),
            _ => Err(ParseComponentError::InvalidLength(slice.len())),
        }
    }
}

impl Component {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

custom_type!(Component, CustomType::Component, Vec::new());

//======
// text
//======

impl FromStr for Component {
    type Err = ParseComponentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(ParseComponentError::InvalidHex)?;
        Self::try_from(bytes.as_slice())
    }
}

impl ToString for Component {
    fn to_string(&self) -> String {
        hex::encode(self.0)
    }
}
