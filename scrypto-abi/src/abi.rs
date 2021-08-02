extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use sbor::*;
#[cfg(feature = "json")]
use serde::{Deserialize, Serialize};

// TODO: how can we represent component ABI using SBOR?

#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[derive(Debug, Encode, Decode)]
pub struct Component {
    pub name: String,
    pub methods: Vec<Method>,
}

#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[derive(Debug, Encode, Decode)]
pub struct Method {
    pub name: String,
    pub mutability: Mutability,
    pub inputs: Vec<Type>,
    pub output: Type,
}

#[cfg_attr(feature = "json", derive(Serialize, Deserialize))]
#[derive(Debug, Encode, Decode)]
pub enum Mutability {
    /// A stateless method does not require an instantiated component.
    Stateless,

    /// An immutable method only reads component state.
    Immutable,

    /// An mutable method may write into component state.
    Mutable,
}
