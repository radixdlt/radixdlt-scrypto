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

impl BlueprintAbi {
    pub fn get_function_abi(&self, function_name: &str) -> Option<&Function> {
        for func in &self.functions {
            if func.name.eq(function_name) {
                return Option::Some(func);
            }
        }
        Option::None
    }

    pub fn contains_function(&self, function_name: &str) -> bool {
        self.get_function_abi(function_name).is_some()
    }
}

/// Represents a function.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct Function {
    pub name: String,
    pub mutability: Option<SelfMutability>,
    pub input: Type,
    pub output: Type,
}

/// Whether a method is going to change the component state.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum SelfMutability {
    /// An immutable method requires an immutable reference to component state.
    Immutable,

    /// A mutable method requires a mutable reference to component state.
    Mutable,
}
