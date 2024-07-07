#![cfg_attr(not(feature = "std"), no_std)]
extern crate core;

#[macro_export]
macro_rules! assert_unique_feature {
    () => {};
    ($first:tt $(,$rest:tt)*) => {
        $(
            #[cfg(all(feature = $first, feature = $rest))]
            compile_error!(concat!("features \"", $first, "\" and \"", $rest, "\" cannot be used together"));
        )*
        assert_unique_feature!($($rest),*);
    }
}

assert_unique_feature!("wasmi", "wasmer");
assert_unique_feature!("moka", "lru");
assert_unique_feature!("std", "alloc");

/// Radix Engine kernel, defining state, ownership and (low-level) invocation semantics.
pub mod kernel;
/// Radix Engine system, defining packages (a.k.a. classes), components (a.k.a. objects) and invocation semantics.
pub mod system;
/// Radix Engine transaction interface.
pub mod transaction;

/// Native blueprints (to be moved to individual crates)
pub mod blueprints;

/// Object module blueprints (to be moved to individual crates)
pub mod object_modules;

pub mod track;

pub mod errors;

pub mod utils;

pub mod vm;

/// Protocol updates
pub mod updates;

pub(crate) mod internal_prelude {
    pub use crate::blueprints::internal_prelude::*;
    pub use crate::errors::*;
    pub use crate::system::system_substates::*;
    pub use crate::{
        dispatch, event_schema, function_schema, method_auth_template, roles_template,
    };
    pub use radix_blueprint_schema_init::*;
    pub use radix_common::prelude::*;
    pub use radix_engine_interface::blueprints::component::*;
    pub use radix_engine_interface::prelude::*;
    pub use radix_substate_store_interface::interface::*;
    pub use sbor::rust::ops::AddAssign;
    pub use sbor::rust::ops::SubAssign;
}
