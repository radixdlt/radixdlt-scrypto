#![cfg_attr(not(feature = "std"), no_std)]

extern crate core;
#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("Either feature `std` or `alloc` must be enabled for this crate.");
#[cfg(all(feature = "std", feature = "alloc"))]
compile_error!("Feature `std` and `alloc` can't be enabled at the same time.");

#[cfg(not(any(feature = "moka", feature = "lru")))]
compile_error!("Either feature `moka` or `lru` must be enabled for this crate.");
#[cfg(all(feature = "moka", feature = "lru"))]
compile_error!("Feature `moka` and `lru` can't be enabled at the same time.");

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

pub mod init;

pub(crate) mod internal_prelude {
    pub use crate::blueprints::internal_prelude::*;
    pub use crate::errors::*;
    pub use crate::init::*;
    pub use crate::kernel::kernel_api::*;
    pub use crate::system::system_substates::*;
    pub use crate::vm::*;
    pub use crate::{
        dispatch, event_schema, function_schema, method_auth_template, roles_template,
    };
    pub use radix_blueprint_schema_init::*;
    pub use radix_common::prelude::*;
    pub use radix_engine_interface::api::*;
    pub use radix_engine_interface::blueprints::component::*;
    pub use radix_engine_interface::prelude::*;
    pub use radix_native_sdk::resource::*;
    pub use radix_native_sdk::runtime::*;
    pub use radix_substate_store_interface::interface::*;
    pub use sbor::rust::ops::AddAssign;
    pub use sbor::rust::ops::SubAssign;
}
