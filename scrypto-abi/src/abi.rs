#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::string::ToString;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use sbor::describe::*;
use sbor::{Decode, Encode, TypeId};

/// Represents a blueprint.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct Blueprint {
    pub package_address: String,
    pub blueprint_name: String,
    pub abi: BlueprintAbi,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct BlueprintAbi {
    pub value: Type,
    pub functions: Vec<Function>,
}

/// Represents a function.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct Function {
    pub name: String,
    pub mutability: Option<Mutability>,
    pub input: Type,
    pub output: Type,
}

/// Whether a method is going to change the component state.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum Mutability {
    /// An immutable method requires an immutable reference to component state.
    Immutable,

    /// A mutable method requires a mutable reference to component state.
    Mutable,
}
