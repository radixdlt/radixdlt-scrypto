use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::core::*;
use crate::engine::{api::*, call_engine, types::ComponentId};
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
    fn instantiate(self) -> ComponentRef;
}

/// An instance of a blueprint, which lives in the ledger state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentRef(pub ComponentId);

impl ComponentRef {
    /// Instantiates a new component.
    pub fn new<T: ComponentState>(state: T) -> Self {
        // TODO: more thoughts are needed for this interface
        let input = CreateComponentInput {
            blueprint_id: (Context::package().0, T::blueprint_name().to_owned()),
            state: scrypto_encode(&state),
        };
        let output: CreateComponentOutput = call_engine(CREATE_COMPONENT, input);

        ComponentRef(output.component_id)
    }

    /// Invokes a method on this component.
    pub fn call<T: Decode>(&self, method: &str, args: Vec<Vec<u8>>) -> T {
        let output = Context::call_method(*self, method, args);

        scrypto_decode(&output).unwrap()
    }

    /// Returns the state of this component.
    pub fn get_state<T: ComponentState>(&self) -> T {
        let input = GetComponentStateInput {};
        let output: GetComponentStateOutput = call_engine(GET_COMPONENT_STATE, input);

        scrypto_decode(&output.state).unwrap()
    }

    /// Updates the state of this component.
    pub fn put_state<T: ComponentState>(&self, state: T) {
        let input = PutComponentStateInput {
            state: scrypto_encode(&state),
        };
        let _: PutComponentStateOutput = call_engine(PUT_COMPONENT_STATE, input);
    }

    /// Returns the blueprint that this component is instantiated from.
    pub fn blueprint(&self) -> (PackageRef, String) {
        let input = GetComponentInfoInput {
            component_id: self.0,
        };
        let output: GetComponentInfoOutput = call_engine(GET_COMPONENT_INFO, input);
        (PackageRef(output.blueprint_id.0), output.blueprint_id.1)
    }
}

//========
// error
//========

#[derive(Debug, Clone)]
pub enum ParseComponentRefError {
    InvalidHex(hex::FromHexError),
    InvalidLength(usize),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseComponentRefError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseComponentRefError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for ComponentRef {
    type Error = ParseComponentRefError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            26 => Ok(Self(copy_u8_array(slice))),
            _ => Err(ParseComponentRefError::InvalidLength(slice.len())),
        }
    }
}

impl ComponentRef {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

custom_type!(ComponentRef, CustomType::ComponentRef, Vec::new());

//======
// text
//======

// Before Bech32, we use a fixed prefix for text representation.

impl FromStr for ComponentRef {
    type Err = ParseComponentRefError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(ParseComponentRefError::InvalidHex)?;
        Self::try_from(&bytes[1..])
    }
}

impl ToString for ComponentRef {
    fn to_string(&self) -> String {
        hex::encode(combine(2, &self.0))
    }
}
