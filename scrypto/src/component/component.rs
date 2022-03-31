use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::component::*;
use crate::core::*;
use crate::engine::{api::*, call_engine};
use crate::misc::*;
use crate::resource::{ComponentAuthorization, ProofRule};
use crate::rust::borrow::ToOwned;
use crate::rust::fmt;
use crate::rust::str::FromStr;
use crate::rust::string::String;
use crate::rust::vec::Vec;
use crate::types::*;

pub struct ComponentStateWithAuth {
    blueprint_name: String,
    state: Vec<u8>,
    authorization: ComponentAuthorization,
}

impl ComponentStateWithAuth {
    pub fn new(blueprint_name: String, state: Vec<u8>) -> Self {
        Self {
            blueprint_name,
            state,
            authorization: ComponentAuthorization::new(),
        }
    }

    pub fn auth(&mut self, method_name: &str, proof_rule: ProofRule) -> &Self {
        self.authorization.insert(method_name, proof_rule);
        self
    }

    pub fn globalize(self) -> ComponentId {
        let input = CreateComponentInput {
            blueprint_name: self.blueprint_name,
            state: self.state,
            authorization: self.authorization,
        };
        let output: CreateComponentOutput = call_engine(CREATE_COMPONENT, input);
        output.component_id
    }
}

/// Represents the state of a component.
pub trait ComponentState: Encode + Decode {
    /// Instantiates a component from this data structure.
    fn globalize_noauth(self) -> ComponentId;

    /// Instantiates a component from this data structure along with authorization rules
    fn globalize_auth(self, authorization: ComponentAuthorization) -> ComponentId;

    fn to_component(self) -> ComponentStateWithAuth;
}

/// An instance of a blueprint, which lives in the ledger state.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentId(pub [u8; 26]);

impl ComponentId {}

#[derive(Debug)]
pub struct Component(pub(crate) ComponentId);

impl Component {
    /// Invokes a method on this component.
    pub fn call<T: Decode>(&self, method: &str, args: Vec<Vec<u8>>) -> T {
        let output = Process::call_method(self.0, method, args);

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

    /// Returns the package ID of this component.
    pub fn package_id(&self) -> PackageId {
        let input = GetComponentInfoInput {
            component_id: self.0,
        };
        let output: GetComponentInfoOutput = call_engine(GET_COMPONENT_INFO, input);
        output.package_id
    }

    /// Returns the blueprint name of this component.
    pub fn blueprint_name(&self) -> String {
        let input = GetComponentInfoInput {
            component_id: self.0,
        };
        let output: GetComponentInfoOutput = call_engine(GET_COMPONENT_INFO, input);
        output.blueprint_name
    }
}

//========
// error
//========

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseComponentIdError {
    InvalidHex(String),
    InvalidLength(usize),
    InvalidPrefix,
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseComponentIdError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseComponentIdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for ComponentId {
    type Error = ParseComponentIdError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            26 => Ok(Self(copy_u8_array(slice))),
            _ => Err(ParseComponentIdError::InvalidLength(slice.len())),
        }
    }
}

impl ComponentId {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

custom_type!(ComponentId, CustomType::ComponentId, Vec::new());

//======
// text
//======

// Before Bech32, we use a fixed prefix for text representation.

impl FromStr for ComponentId {
    type Err = ParseComponentIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(|_| ParseComponentIdError::InvalidHex(s.to_owned()))?;
        if bytes.get(0) != Some(&2u8) {
            return Err(ParseComponentIdError::InvalidPrefix);
        }
        Self::try_from(&bytes[1..])
    }
}

impl fmt::Display for ComponentId {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(combine(2, &self.0)))
    }
}

impl fmt::Debug for ComponentId {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}
