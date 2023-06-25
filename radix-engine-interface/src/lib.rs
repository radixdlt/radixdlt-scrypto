#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"] // Enables certain tests of deep typed SBOR to function

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("Either feature `std` or `alloc` must be enabled for this crate.");
#[cfg(all(feature = "std", feature = "alloc"))]
compile_error!("Feature `std` and `alloc` can't be enabled at the same time.");

pub mod api;
pub mod blueprints;
pub mod constants;
pub mod traits;
pub mod types;

mod macros;
pub use macros::*;

// Re-export scrypto schema
pub mod schema {
    pub use scrypto_schema::*;
}

// Re-export radix engine common.
pub extern crate radix_engine_common;
pub use radix_engine_common::*;

// Re-export SBOR derive.
pub extern crate sbor;
pub use sbor::{Categorize, Decode, Encode, Sbor};

// This is to make derives work within this crate.
// See: https://users.rust-lang.org/t/how-can-i-use-my-derive-macro-from-the-crate-that-declares-the-trait/60502
pub extern crate self as radix_engine_interface;

/// Each module should have its own prelude, which:
/// * Adds preludes of upstream crates
/// * Exports types with specific-enough names which mean they can safely be used downstream.
///
/// The idea is that we can just include the current crate's prelude and avoid messing around with tons of includes.
/// This makes refactors easier, and makes integration into the node less painful.
pub mod prelude {
    // Extern crates for the purposes of EG being visible from
    // scrypto macros
    pub extern crate radix_engine_common;

    // Exports from upstream crates
    pub use radix_engine_common::prelude::*;

    // Exports from this crate
    pub use crate::blueprints::resource::NonFungibleGlobalId;
    pub use crate::macros::*;
    pub use crate::schema::*;
    pub use crate::traits::*;
    pub use crate::types::*;
    pub use crate::{access_and_or, access_rule_node, role_entry, roles2, rule};
}
