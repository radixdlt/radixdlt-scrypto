use crate::*;
use lazy_static::lazy_static;
use radix_engine_common::types::*;
use sbor::rust::prelude::*;

//====================
// Resource Managers
//====================

pub const RADIX_TOKEN: ResourceAddress = resource_address(EntityType::GlobalFungibleResource, 0);
pub const ECDSA_SECP256K1_TOKEN: ResourceAddress =
    resource_address(EntityType::GlobalNonFungibleResource, 0);
pub const EDDSA_ED25519_TOKEN: ResourceAddress =
    resource_address(EntityType::GlobalNonFungibleResource, 1);
pub const SYSTEM_TOKEN: ResourceAddress =
    resource_address(EntityType::GlobalNonFungibleResource, 2);
pub const PACKAGE_TOKEN: ResourceAddress =
    resource_address(EntityType::GlobalNonFungibleResource, 3);
pub const GLOBAL_OBJECT_TOKEN: ResourceAddress =
    resource_address(EntityType::GlobalNonFungibleResource, 4);
pub const PACKAGE_OWNER_TOKEN: ResourceAddress =
    resource_address(EntityType::GlobalNonFungibleResource, 5);
pub const VALIDATOR_OWNER_TOKEN: ResourceAddress =
    resource_address(EntityType::GlobalNonFungibleResource, 6);
pub const IDENTITY_OWNER_TOKEN: ResourceAddress =
    resource_address(EntityType::GlobalNonFungibleResource, 7);
pub const ACCOUNT_OWNER_TOKEN: ResourceAddress =
    resource_address(EntityType::GlobalNonFungibleResource, 8);

//====================
// Packages
//====================

pub const PACKAGE_PACKAGE: PackageAddress = package_address(EntityType::GlobalPackage, 0);
pub const RESOURCE_MANAGER_PACKAGE: PackageAddress = package_address(EntityType::GlobalPackage, 1);
pub const IDENTITY_PACKAGE: PackageAddress = package_address(EntityType::GlobalPackage, 2);
pub const EPOCH_MANAGER_PACKAGE: PackageAddress = package_address(EntityType::GlobalPackage, 3);
pub const CLOCK_PACKAGE: PackageAddress = package_address(EntityType::GlobalPackage, 4);
pub const ACCOUNT_PACKAGE: PackageAddress = package_address(EntityType::GlobalPackage, 5);
pub const ACCESS_CONTROLLER_PACKAGE: PackageAddress = package_address(EntityType::GlobalPackage, 6);
pub const TRANSACTION_PROCESSOR_PACKAGE: PackageAddress =
    package_address(EntityType::GlobalPackage, 7);
pub const METADATA_PACKAGE: PackageAddress = package_address(EntityType::GlobalPackage, 10);
pub const ROYALTY_PACKAGE: PackageAddress = package_address(EntityType::GlobalPackage, 11);
pub const ACCESS_RULES_PACKAGE: PackageAddress = package_address(EntityType::GlobalPackage, 12);
pub const GENESIS_HELPER_PACKAGE: PackageAddress = package_address(EntityType::GlobalPackage, 13);
pub const FAUCET_PACKAGE: PackageAddress = package_address(EntityType::GlobalPackage, 64);

//====================
// Components
//====================

pub const CLOCK: ComponentAddress = component_address(EntityType::GlobalClock, 0);
pub const EPOCH_MANAGER: ComponentAddress = component_address(EntityType::GlobalEpochManager, 0);

//====================
// Blueprint Names
//====================

pub const GENESIS_HELPER_BLUEPRINT: &str = "GenesisHelper";
pub const FAUCET_BLUEPRINT: &str = "Faucet";

// Currently, functions and methods can reference these well-known nodes without declaring
// the dependency in the package info.
//
// To avoid creating references from various places, a list of well-known nodes is crafted
// and added to every call frame, as a temporary solution.
//
// TODO: to remove it, we will have to:
// - Add Scrypto support for declaring dependencies
// - Split bootstrapping into state flushing and transaction execution (the "chicken-and-egg" problem)
//
lazy_static! {
    pub static ref ALWAYS_VISIBLE_GLOBAL_NODES: BTreeSet<NodeId> = {
        btreeset![
            // resource managers
            RADIX_TOKEN.into(),
            ECDSA_SECP256K1_TOKEN.into(),
            EDDSA_ED25519_TOKEN.into(),
            SYSTEM_TOKEN.into(),
            PACKAGE_TOKEN.into(),
            GLOBAL_OBJECT_TOKEN.into(),
            PACKAGE_OWNER_TOKEN.into(),
            VALIDATOR_OWNER_TOKEN.into(),
            IDENTITY_OWNER_TOKEN.into(),
            ACCOUNT_OWNER_TOKEN.into(),
            // packages
            PACKAGE_PACKAGE.into(),
            RESOURCE_MANAGER_PACKAGE.into(),
            IDENTITY_PACKAGE.into(),
            EPOCH_MANAGER_PACKAGE.into(),
            CLOCK_PACKAGE.into(),
            ACCOUNT_PACKAGE.into(),
            ACCESS_CONTROLLER_PACKAGE.into(),
            TRANSACTION_PROCESSOR_PACKAGE.into(),
            METADATA_PACKAGE.into(),
            ROYALTY_PACKAGE.into(),
            ACCESS_RULES_PACKAGE.into(),
            GENESIS_HELPER_PACKAGE.into(),
            FAUCET_PACKAGE.into(),
            // components
            CLOCK.into(),
            EPOCH_MANAGER.into(),
        ]
    };
}
