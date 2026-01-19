#![allow(
    // This lint is allowed since in the implementation of the native blueprints we usually get the
    // return from the invoked function and then encode it without checking what the type of it is
    // as a general coding-style. Following this lint actually hurts us instead of helping us, thus
    // we permit it in the blueprints module.
    clippy::let_unit_value
)]

pub mod metadata;
pub mod role_assignment;
pub mod royalty;
