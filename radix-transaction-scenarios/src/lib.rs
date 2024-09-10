#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("Either feature `std` or `alloc` must be enabled for this crate.");
#[cfg(all(feature = "std", feature = "alloc"))]
compile_error!("Feature `std` and `alloc` can't be enabled at the same time.");

//====================================================================================
// TRANSACTION SCENARIOS
// The purpose of scenarios is to add "interesting, pre-determined state"
// to test network ledgers.
//
// These scenarios can be run on a test network immediately after genesis.
// (Therefore please make sure they are all deterministic!).
//
// The intention is for these scenarios to fulfill a number of purposes:
// - Covering all the substate types for exercising Core API and Gateway ingestion
// - Create interesting test data for dashboard/wallet to explore
// - To provide many valid example manifests for reference by integrators
// - To allow for testing of multi-transaction journeys and the typed substate mappings
//   in this repository
//====================================================================================

pub mod accounts;
pub mod executor;
#[allow(unused)] // Some things are only used in std build
pub mod runners;
pub mod scenario;
pub mod scenarios;
pub mod utils;

pub mod prelude {}

// Extra things which this crate wants which upstream crates likely don't
pub(crate) mod internal_prelude {
    pub use crate::accounts::*;
    pub use crate::scenario::*;
    pub use radix_common::prelude::*;
    pub use radix_engine::errors::*;
    pub use radix_engine::transaction::*;
    pub use radix_engine::updates::*;
    pub use radix_engine_interface::prelude::*;
    pub use radix_transactions::errors::*;
    pub use radix_transactions::manifest::*;
    pub use radix_transactions::prelude::*;
    pub use radix_transactions::validation::*;
}
