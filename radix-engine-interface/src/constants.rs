use crate::*;
use lazy_static::lazy_static;
use radix_engine_common::types::*;
use sbor::rust::prelude::*;

pub use radix_engine_common::native_addresses::*;

// Currently, functions and methods can reference these well-known nodes without declaring
// the dependency in the package info.
//
// To avoid creating references from various places, a list of well-known nodes is crafted
// and added to every call frame, as a temporary solution.
//
// FIXME: to remove it, we will have to:
// - Add Scrypto support for declaring dependencies
// - Split bootstrapping into state flushing and transaction execution (the "chicken-and-egg" problem)
//
lazy_static! {
    pub static ref ALWAYS_VISIBLE_GLOBAL_NODES: BTreeSet<NodeId> = {
        btreeset![
            // resource managers
            RADIX_TOKEN.into(),
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
            ACCESS_RULES_MODULE_PACKAGE.into(),
            GENESIS_HELPER_PACKAGE.into(),
            FAUCET_PACKAGE.into(),
            POOL_PACKAGE.into(),
            TRANSACTION_TRACKER_PACKAGE.into(),
            // components
            CONSENSUS_MANAGER.into(),
        ]
    };
}
