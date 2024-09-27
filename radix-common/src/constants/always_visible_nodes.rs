use crate::*;
use lazy_static::lazy_static;
use radix_common::types::*;
use sbor::rust::prelude::*;

pub use radix_common::constants::*;

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
    static ref ALWAYS_VISIBLE_GLOBAL_NODES_V1: IndexSet<NodeId> = {
        indexset!(
            // resource managers
            XRD.into(),
            SECP256K1_SIGNATURE_RESOURCE.into(),
            ED25519_SIGNATURE_RESOURCE.into(),
            SYSTEM_EXECUTION_RESOURCE.into(),
            PACKAGE_OF_DIRECT_CALLER_RESOURCE.into(),
            GLOBAL_CALLER_RESOURCE.into(),
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

    static ref ALWAYS_VISIBLE_GLOBAL_NODES_V2: IndexSet<NodeId> = {
        indexset!(
            // resource managers
            XRD.into(),
            SECP256K1_SIGNATURE_RESOURCE.into(),
            ED25519_SIGNATURE_RESOURCE.into(),
            SYSTEM_EXECUTION_RESOURCE.into(),
            PACKAGE_OF_DIRECT_CALLER_RESOURCE.into(),
            GLOBAL_CALLER_RESOURCE.into(),
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
            LOCKER_PACKAGE.into(),
            // components
            CONSENSUS_MANAGER.into(),
            TRANSACTION_TRACKER.into(),
        )
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Sbor)]
pub enum AlwaysVisibleGlobalNodesVersion {
    V1,
    V2,
}

impl AlwaysVisibleGlobalNodesVersion {
    pub const fn latest() -> Self {
        Self::V2
    }
}

pub fn always_visible_global_nodes(
    version: AlwaysVisibleGlobalNodesVersion,
) -> &'static IndexSet<NodeId> {
    match version {
        AlwaysVisibleGlobalNodesVersion::V1 => &ALWAYS_VISIBLE_GLOBAL_NODES_V1,
        AlwaysVisibleGlobalNodesVersion::V2 => &ALWAYS_VISIBLE_GLOBAL_NODES_V2,
    }
}
