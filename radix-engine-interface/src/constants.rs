use lazy_static::lazy_static;
pub use radix_engine_common::constants::*;
use radix_engine_common::types::ActorStateHandle;
use radix_engine_common::types::*;
use sbor::rust::prelude::*;

pub const ACTOR_STATE_SELF: ActorStateHandle = 0u32;
pub const ACTOR_STATE_OUTER_OBJECT: ActorStateHandle = 1u32;

pub const ACTOR_REF_SELF: ActorRefHandle = 0u32;
pub const ACTOR_REF_OUTER: ActorRefHandle = 1u32;
pub const ACTOR_REF_GLOBAL: ActorRefHandle = 2u32;
pub const ACTOR_REF_AUTH_ZONE: ActorRefHandle = 8u32;

// Currently, functions and methods can reference these well-known nodes without declaring
// the dependency in the package info.
//
// To avoid creating references from various places, a list of well-known nodes is crafted
// and added to every call frame, as a temporary solution.
//
// To remove it, we will have to:
// - Add Scrypto support for declaring dependencies
// - Split bootstrapping into state flushing and transaction execution (the "chicken-and-egg" problem)
//
lazy_static! {
    pub static ref ALWAYS_VISIBLE_GLOBAL_NODES: IndexSet<NodeId> = {
        indexset!(
            // resource managers
            XRD.into(),
            SECP256K1_SIGNATURE_VIRTUAL_BADGE.into(),
            ED25519_SIGNATURE_VIRTUAL_BADGE.into(),
            SYSTEM_TRANSACTION_BADGE.into(),
            PACKAGE_OF_DIRECT_CALLER_VIRTUAL_BADGE.into(),
            GLOBAL_CALLER_VIRTUAL_BADGE.into(),
            PACKAGE_OWNER_BADGE.into(),
            VALIDATOR_OWNER_BADGE.into(),
            IDENTITY_OWNER_BADGE.into(),
            ACCOUNT_OWNER_BADGE.into(),
            // packages
            PACKAGE_PACKAGE.into(),
            RESOURCE_PACKAGE.into(),
            IDENTITY_PACKAGE.into(),
            CONSENSUS_MANAGER_PACKAGE.into(),
            ACCOUNT_PACKAGE.into(),
            ACCESS_CONTROLLER_PACKAGE.into(),
            TRANSACTION_PROCESSOR_PACKAGE.into(),
            METADATA_MODULE_PACKAGE.into(),
            ROYALTY_MODULE_PACKAGE.into(),
            ROLE_ASSIGNMENT_MODULE_PACKAGE.into(),
            GENESIS_HELPER_PACKAGE.into(),
            FAUCET_PACKAGE.into(),
            POOL_PACKAGE.into(),
            TRANSACTION_TRACKER_PACKAGE.into(),
            // components
            CONSENSUS_MANAGER.into(),
            TRANSACTION_TRACKER.into(),
        )
    };
}
