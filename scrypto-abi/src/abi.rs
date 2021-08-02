extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use sbor::Type;
use serde::{Deserialize, Serialize};

// TODO: how can we represent component ABI using SBOR?

#[derive(Debug, Serialize, Deserialize)]
pub struct Component {
    pub name: String,
    pub methods: Vec<Method>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Method {
    pub name: String,
    pub mutability: Mutability,
    pub inputs: Vec<Type>,
    pub output: Type,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Mutability {
    /// A stateless method does not require an instantiated component.
    Stateless,

    /// An immutable method only reads component state.
    Immutable,

    /// An mutable method may write into component state.
    Mutable,
}
