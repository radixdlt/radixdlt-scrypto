#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
pub use alloc::string::String;
#[cfg(feature = "alloc")]
pub use alloc::vec::Vec;

use sbor::types::*;
use sbor::{Decode, Encode};
#[cfg(any(feature = "json_std", feature = "json_alloc"))]
use serde::{Deserialize, Serialize};

/// Represents the ABI of a component.
#[cfg_attr(
    any(feature = "json_std", feature = "json_alloc"),
    derive(Serialize, Deserialize)
)]
#[derive(Debug, Clone, Encode, Decode)]
pub struct Component {
    pub name: String,
    pub methods: Vec<Method>,
}

/// Represents a method of a component.
#[cfg_attr(
    any(feature = "json_std", feature = "json_alloc"),
    derive(Serialize, Deserialize)
)]
#[derive(Debug, Clone, Encode, Decode)]
pub struct Method {
    pub name: String,
    pub mutability: Mutability,
    pub inputs: Vec<Type>,
    pub output: Type,
}

/// Represents method state mutability.
#[cfg_attr(
    any(feature = "json_std", feature = "json_alloc"),
    derive(Serialize, Deserialize)
)]
#[derive(Debug, Clone, Encode, Decode)]
pub enum Mutability {
    /// A stateless method does not require an instantiated component.
    Stateless,

    /// An immutable method only reads component state.
    Immutable,

    /// An mutable method may write into component state.
    Mutable,
}
