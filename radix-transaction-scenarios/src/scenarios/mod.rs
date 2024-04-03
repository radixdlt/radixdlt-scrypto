use crate::internal_prelude::*;

pub mod account_authorized_depositors;
mod all_scenarios;
pub mod fungible_resource;
pub mod global_n_owned;
pub mod kv_store_with_remote_type;
pub mod max_transaction;
pub mod maya_router;
pub mod metadata;
pub mod non_fungible_resource;
pub mod non_fungible_resource_with_remote_type;
pub mod radiswap;
pub mod royalties;
pub mod transfer_xrd;

pub use all_scenarios::*;
