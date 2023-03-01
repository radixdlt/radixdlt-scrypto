#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("Either feature `std` or `alloc` must be enabled for this crate.");
#[cfg(all(feature = "std", feature = "alloc"))]
compile_error!("Feature `std` and `alloc` can't be enabled at the same time.");

use sbor::rust::prelude::*;
use sbor::*;

#[derive(Default, Debug, Clone, PartialEq, Eq, Sbor)]
pub struct PackageSchema {
    pub blueprints: HashMap<String, BlueprintSchema>,
}

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub struct BlueprintSchema {
    /// For each offset, there is a [`LocalTypeIndex`]
    pub substate_schemas: BTreeMap<u8, LocalTypeIndex>,
    /// For each function, there is a [`FunctionSchema`]
    pub function_schemas: BTreeMap<String, FunctionSchema>,
}

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub struct FunctionSchema {
    pub receiver: Option<FunctionReceiver>,
    pub input: LocalTypeIndex,
    pub output: LocalTypeIndex,
    pub export_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum FunctionReceiver {
    Immutable,

    Mutable,
}
