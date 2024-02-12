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

pub mod prelude {
    // Note - radix_engine_common::prelude was previously something like an internal/
    // external prelude, but let's normalize radix-engine to have a prelude
    // like everything else, and add to it where needed
    pub use crate::internal_prelude::*;
}

pub(crate) mod internal_prelude {
    pub use radix_engine_interface::address::{
        AddressBech32DecodeError, AddressBech32Decoder, AddressBech32EncodeError,
        AddressBech32Encoder,
    };
    pub use radix_engine_interface::blueprints::resource::*;
    pub use radix_engine_interface::constants::*;
    pub use radix_engine_interface::prelude::*;
    pub mod blueprints {
        pub use radix_engine_interface::blueprints::*;
    }
    pub use crate::blueprints::internal_prelude::*;
    pub use crate::errors::*;
    pub use crate::system::system_substates::*;
    pub use crate::{event_schema, method_auth_template, roles_template};
    pub use blueprint_schema_init::*;
    pub use sbor::rust::num::NonZeroU32;
    pub use sbor::rust::num::NonZeroUsize;
    pub use sbor::rust::ops::AddAssign;
    pub use sbor::rust::ops::SubAssign;
    #[cfg(feature = "std")]
    pub use std::alloc;
}
