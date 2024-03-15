// Modules that should appear explicitly under manifest::
pub mod converter;
pub mod model;

// Modules which should appear part of `manifest`
mod custom_extension;
mod custom_formatting;
mod custom_payload_wrappers;
#[cfg(feature = "serde")]
mod custom_serde;
mod custom_traversal;
mod custom_validation;
mod custom_value;
mod custom_value_kind;
mod definitions;
mod display_context;

pub use custom_extension::*;
pub use custom_payload_wrappers::*;
pub use custom_traversal::*;
pub use custom_value::*;
pub use custom_value_kind::*;
pub use definitions::*;
pub use display_context::*;

// Prelude:
// This exposes all the types/traits directly, without exposing the module
// names. These module names can clash with other preludes so get excluded.
pub mod prelude {
    // Public modules to include in prelude
    pub use super::model::*;

    // Private modules to include in prelude
    pub use super::custom_extension::*;
    pub use super::custom_payload_wrappers::*;
    pub use super::custom_traversal::*;
    pub use super::custom_value::*;
    pub use super::custom_value_kind::*;
    pub use super::definitions::*;
    pub use super::display_context::*;
}
