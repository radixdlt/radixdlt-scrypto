use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::core::*;
use crate::engine::{api::*, call_engine};
use crate::misc::*;
use crate::rust::borrow::ToOwned;
use crate::rust::fmt;
use crate::rust::str::FromStr;
use crate::rust::string::String;
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
pub struct ComponentRef(pub [u8; 26]);

impl ComponentRef {
    /// Instantiates a new component.
    pub fn new<T: ComponentState>(state: T) -> Self {
        // TODO: more thoughts are needed for this interface
        let input = CreateComponentInput {
            package_ref: Context::package(),
            blueprint_name: T::blueprint_name().to_owned(),
            state: scrypto_encode(&state),
        };
        let output: CreateComponentOutput = call_engine(CREATE_COMPONENT, input);

        output.component_ref
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
            component_ref: *self,
        };
        let output: GetComponentInfoOutput = call_engine(GET_COMPONENT_INFO, input);
        (output.package_ref, output.blueprint_name)
    }
}

//========
// error
//========

#[derive(Debug, Clone)]
pub enum ParseComponentRefError {
    InvalidHex(hex::FromHexError),
    InvalidLength(usize),
    InvalidPrefix,
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
        if bytes.get(0) != Some(&2u8) {
            return Err(ParseComponentRefError::InvalidPrefix);
        }
        Self::try_from(&bytes[1..])
    }
}

impl fmt::Display for ComponentRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(combine(2, &self.0)))
    }
}
