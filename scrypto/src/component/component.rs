use sbor::*;

use crate::buffer::*;
use crate::component::*;
use crate::core::*;
use crate::engine::{api::*, call_engine};
use crate::misc::*;
use crate::resource::AccessRules;
use crate::types::*;
use sbor::rust::borrow::ToOwned;
use sbor::rust::fmt;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;

pub struct LocalComponent {
    blueprint_name: String,
    state: Vec<u8>,
    access_rules_list: Vec<AccessRules>,
}

impl LocalComponent {
    pub fn new(blueprint_name: String, state: Vec<u8>) -> Self {
        Self {
            blueprint_name,
            state,
            access_rules_list: Vec::new(),
        }
    }

    pub fn add_access_check(mut self, authorization: AccessRules) -> Self {
        self.access_rules_list.push(authorization);
        self
    }

    pub fn globalize(self) -> ComponentAddress {
        let input = CreateComponentInput {
            blueprint_name: self.blueprint_name,
            state: self.state,
            access_rules_list: self.access_rules_list,
        };
        let output: CreateComponentOutput = call_engine(CREATE_COMPONENT, input);
        output.component_address
    }
}

/// Represents the state of a component.
pub trait ComponentState: Encode + Decode {
    /// Instantiates a component from this data structure.
    fn instantiate(self) -> LocalComponent;
}

/// An instance of a blueprint, which lives in the ledger state.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentAddress(pub [u8; 26]);

impl ComponentAddress {}

/// Represents an instantiated component.
#[derive(Debug)]
pub struct Component(pub(crate) ComponentAddress);

impl Component {
    /// Invokes a method on this component.
    pub fn call<T: Decode>(&self, method: &str, args: Vec<Vec<u8>>) -> T {
        let output = Runtime::call_method(self.0, method, args);

        scrypto_decode(&output).unwrap()
    }

    /// Returns the state of this component.
    pub fn get_state<T: ComponentState>(&self) -> T {
        let input = GetComponentStateInput {
            component_address: self.0,
        };
        let output: GetComponentStateOutput = call_engine(GET_COMPONENT_STATE, input);

        scrypto_decode(&output.state).unwrap()
    }

    /// Updates the state of this component.
    pub fn put_state<T: ComponentState>(&self, state: T) {
        let input = PutComponentStateInput {
            component_address: self.0,
            state: scrypto_encode(&state),
        };
        let _: PutComponentStateOutput = call_engine(PUT_COMPONENT_STATE, input);
    }

    /// Returns the package ID of this component.
    pub fn package_address(&self) -> PackageAddress {
        let input = GetComponentInfoInput {
            component_address: self.0,
        };
        let output: GetComponentInfoOutput = call_engine(GET_COMPONENT_INFO, input);
        output.package_address
    }

    /// Returns the blueprint name of this component.
    pub fn blueprint_name(&self) -> String {
        let input = GetComponentInfoInput {
            component_address: self.0,
        };
        let output: GetComponentInfoOutput = call_engine(GET_COMPONENT_INFO, input);
        output.blueprint_name
    }
}

//========
// error
//========

/// Represents an error when decoding component address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseComponentAddressError {
    InvalidHex(String),
    InvalidLength(usize),
    InvalidPrefix,
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseComponentAddressError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseComponentAddressError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for ComponentAddress {
    type Error = ParseComponentAddressError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            26 => Ok(Self(copy_u8_array(slice))),
            _ => Err(ParseComponentAddressError::InvalidLength(slice.len())),
        }
    }
}

impl ComponentAddress {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

scrypto_type!(ComponentAddress, ScryptoType::ComponentAddress, Vec::new());

//======
// text
//======

// Before Bech32, we use a fixed prefix for text representation.

impl FromStr for ComponentAddress {
    type Err = ParseComponentAddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes =
            hex::decode(s).map_err(|_| ParseComponentAddressError::InvalidHex(s.to_owned()))?;
        if bytes.get(0) != Some(&2u8) {
            return Err(ParseComponentAddressError::InvalidPrefix);
        }
        Self::try_from(&bytes[1..])
    }
}

impl fmt::Display for ComponentAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(combine(2, &self.0)))
    }
}

impl fmt::Debug for ComponentAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}
