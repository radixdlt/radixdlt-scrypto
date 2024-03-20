// Modules that should appear explicitly under the scrypto module
pub mod model;

// Modules which should appear part of the scrypto module

/// Defines the full Scrypto extension.
mod custom_extension;
mod custom_formatting;
mod custom_payload_wrappers;
/// Defines the custom Scrypto schema types.
mod custom_schema;
/// Defines custom serialization of the types.
#[cfg(feature = "serde")]
mod custom_serde;
/// Defines how to traverse scrypto custom types.
mod custom_traversal;
mod custom_validation;
/// Defines the model of Scrypto custom values.
mod custom_value;
/// Defines the custom value kind model that scrypto uses.
mod custom_value_kind;
/// Defines the scrypto custom well known types.
mod custom_well_known_types;
/// Defines the core traits and methods for scrypto SBOR encoding
mod definitions;

pub use custom_extension::*;
pub use custom_formatting::*;
pub use custom_payload_wrappers::*;
pub use custom_schema::*;
pub use custom_traversal::*;
pub use custom_value::*;
pub use custom_value_kind::*;
pub use custom_well_known_types::*;
pub use definitions::*;

// Prelude:
// This exposes all the types/traits directly, without exposing the module
// names. These module names can clash with other preludes so get excluded.
pub mod prelude {
    // Public modules to include in prelude
    pub use super::model::*;

    // Private modules to include in prelude
    pub use super::custom_extension::*;
    pub use super::custom_formatting::*;
    pub use super::custom_payload_wrappers::*;
    pub use super::custom_schema::*;
    pub use super::custom_traversal::*;
    pub use super::custom_value::*;
    pub use super::custom_value_kind::*;
    pub use super::custom_well_known_types::well_known_scrypto_custom_types::*;
    pub use super::custom_well_known_types::*;
    pub use super::definitions::*;
}
