#![allow(
    // This lint is allowed since in the implementation of the native blueprints we usually get the
    // return from the invoked function and then encode it without checking what the type of it is
    // as a general coding-style. Following this lint actually hurts us instead of helping us, thus
    // we permit it in the blueprints module.
    clippy::let_unit_value
)]

pub mod access_controller;
pub mod account;
pub mod consensus_manager;
pub mod identity;
pub mod locker;
pub mod models;
pub mod native_schema;
pub mod package;
pub mod pool;
pub mod resource;
pub mod test_utils;
pub mod transaction_processor;
pub mod transaction_tracker;
pub mod util;

pub(crate) mod internal_prelude {
    pub use super::models::*;
    pub use super::package::*;
}
