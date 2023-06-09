#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("Either feature `std` or `alloc` must be enabled for this crate.");
#[cfg(all(feature = "std", feature = "alloc"))]
compile_error!("Feature `std` and `alloc` can't be enabled at the same time.");

pub mod accounts;
pub mod runners;
pub mod scenario;
pub mod scenarios;

pub mod prelude {}

// Extra things which this crate wants which upstream crates likely don't
pub(crate) mod internal_prelude {
    pub use crate::prelude::*;

    pub use crate::accounts::*;
    pub use crate::scenario::*;
    pub use radix_engine::transaction::*;
    pub use radix_engine_interface::prelude::*;
    pub use transaction::prelude::*;
}
