pub mod builder;
pub mod data;
pub mod errors;
pub mod manifest;
#[cfg(feature = "message_encryption")]
pub mod message;
pub mod model;
pub mod signing;
pub mod validation;

/// Each module should have its own prelude, which:
/// * Adds preludes of upstream crates
/// * Exports types with specific-enough names which mean they can safely be used downstream.
///
/// The idea is that we can just include the current crate's prelude and avoid messing around with tons of includes.
/// This makes refactors easier, and makes integration into the node less painful.
pub mod prelude {
    // Exports from upstream crates
    pub use radix_engine_constants::*;
    pub use radix_engine_interface::prelude::*;

    // Exports from this crate
    pub use crate::builder::*;
    pub use crate::model::*;
    pub use crate::signing::ed25519::*;
    pub use crate::signing::secp256k1::*;
    pub use crate::signing::{PrivateKey, Signer};
}

// Extra things which this crate wants which upstream crates likely don't
pub(crate) mod internal_prelude {
    pub use crate::prelude::*;

    pub use crate::define_raw_transaction_payload;
    pub use crate::errors::*;
    pub use crate::manifest::*;
    pub use crate::validation::*;
}
