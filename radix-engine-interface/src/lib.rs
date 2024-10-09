#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"] // Enables certain tests of deep typed SBOR to function

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("Either feature `std` or `alloc` must be enabled for this crate.");
#[cfg(all(feature = "std", feature = "alloc"))]
compile_error!("Feature `std` and `alloc` can't be enabled at the same time.");

// TODO: eventually only `api` (System API) should stay in this crate.

pub mod api;
pub mod blueprints;
pub mod macros;
pub mod object_modules;
pub mod types;

// extern crate self as X; in lib.rs allows ::X and X to resolve to this crate inside this crate.
// This enables procedural macros which output code involving paths to this crate, to work inside
// this crate. See this link for details:
// https://users.rust-lang.org/t/how-can-i-use-my-derive-macro-from-the-crate-that-declares-the-trait/60502
//
// IMPORTANT:
// This should never be pub, else `X::X::X::X::...` becomes a valid path in downstream crates,
// which we've discovered can cause really bad autocomplete times (when combined with other
// specific imports, generic traits, resolution paths which likely trigger edge cases in
// Rust Analyzer which get stuck on these infinite possible paths)
extern crate self as radix_engine_interface;

/// Each module should have its own prelude, which:
/// * Adds preludes of upstream crates
/// * Exports types with specific-enough names which mean they can safely be used downstream.
///
/// The idea is that we can just include the current crate's prelude and avoid messing around with tons of includes.
/// This makes refactors easier, and makes integration into the node less painful.
pub mod prelude {
    pub use radix_common_derive::{dec, pdec};

    // Exports from this crate
    pub use crate::api::actor_api::*;
    pub use crate::api::field_api::*;
    pub use crate::api::key_value_entry_api::*;
    pub use crate::api::key_value_store_api::*;
    pub use crate::api::*;
    pub use crate::blueprints::consensus_manager::*;
    pub use crate::blueprints::locker::*;
    pub use crate::blueprints::resource::*;
    pub use crate::blueprints::utils::*;
    pub use crate::object_modules::metadata::*;
    pub use crate::object_modules::role_assignment::*;
    pub use crate::object_modules::royalty::*;
    pub use crate::object_modules::ModuleConfig;
    pub use crate::types::*;
    pub use crate::{
        access_and_or, burn_roles, composite_requirement, deposit_roles, freeze_roles,
        internal_roles, metadata, metadata_init, metadata_init_set_entry, metadata_roles,
        mint_roles, non_fungible_data_update_roles, recall_roles, role_entry, roles2, rule,
        withdraw_roles,
    };
}

pub(crate) mod internal_prelude {
    pub use crate::prelude::*;
    pub use radix_common::prelude::*;
}
