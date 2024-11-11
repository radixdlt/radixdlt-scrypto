#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("Either feature `std` or `alloc` must be enabled for this crate.");
#[cfg(all(feature = "std", feature = "alloc"))]
compile_error!("Feature `std` and `alloc` can't be enabled at the same time.");

mod contextual_display;
#[cfg(feature = "serde")]
mod contextual_serialize;
mod contextual_try_from_into;
pub mod iterators;
mod macros;
mod resolve;
pub mod rust;
mod slice;
#[cfg(feature = "unicode")]
pub mod unicode;

pub use contextual_display::*;
#[cfg(feature = "serde")]
pub use contextual_serialize::*;
pub use contextual_try_from_into::*;
pub use resolve::*;
pub use slice::*;

/// Each module should have its own prelude, which:
/// * Adds preludes of upstream crates
/// * Exports types with specific-enough names which mean they can safely be used downstream.
///
/// The idea is that we can just include the current crate's prelude and avoid messing around with tons of includes.
/// This makes refactors easier, and makes integration into the node less painful.
pub mod prelude {
    // Add all rust types so that things work in no-std
    pub use crate::rust::prelude::*;

    // Export types and other useful methods
    pub use crate::contextual_display::*;
    #[cfg(feature = "serde")]
    pub use crate::contextual_serialize::*;
    pub use crate::contextual_try_from_into::*;
    pub use crate::resolve::*;
    pub use crate::{
        labelled_resolvable_using_resolvable_impl, labelled_resolvable_with_identity_impl,
        resolvable_with_identity_impl, resolvable_with_try_into_impls,
    };

    pub use crate::iterators::*;
    pub use crate::slice::*;

    pub use crate::assert_matches;
}
