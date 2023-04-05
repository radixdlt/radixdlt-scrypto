use crate::*;
use radix_engine_common::types::*;

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

// There should be no need of this function, but many of our configurations are depending on it.
// Having it in a single place to avoid out-of-sync.
pub fn is_native_package(address: PackageAddress) -> bool {
    match address {
        PACKAGE_PACKAGE
        | RESOURCE_MANAGER_PACKAGE
        | IDENTITY_PACKAGE
        | EPOCH_MANAGER_PACKAGE
        | CLOCK_PACKAGE
        | ACCOUNT_PACKAGE
        | ACCESS_CONTROLLER_PACKAGE
        | TRANSACTION_PROCESSOR_PACKAGE
        | METADATA_PACKAGE
        | ROYALTY_PACKAGE
        | ACCESS_RULES_PACKAGE => true,
        _ => false,
    }
}

pub const FAUCET_PACKAGE: PackageAddress = package_address(EntityType::GlobalPackage, 64);
pub const FAUCET_BLUEPRINT: &str = "Faucet";

pub const CLOCK: ComponentAddress = component_address(EntityType::GlobalClock, 0);
pub const EPOCH_MANAGER: ComponentAddress = component_address(EntityType::GlobalEpochManager, 0);
