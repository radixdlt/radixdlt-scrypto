use crate::data::scrypto::model::*;
use crate::*;

pub const RADIX_TOKEN: ResourceAddress = vanity_address!(EntityType::FungibleResource, 0);

pub const ECDSA_SECP256K1_TOKEN: ResourceAddress =
    vanity_address!(EntityType::NonFungibleResource, 0);
pub const EDDSA_ED25519_TOKEN: ResourceAddress =
    vanity_address!(EntityType::NonFungibleResource, 1);
pub const SYSTEM_TOKEN: ResourceAddress = vanity_address!(EntityType::NonFungibleResource, 2);
pub const PACKAGE_TOKEN: ResourceAddress = vanity_address!(EntityType::NonFungibleResource, 3);
pub const GLOBAL_OBJECT_TOKEN: ResourceAddress =
    vanity_address!(EntityType::NonFungibleResource, 4);
pub const VALIDATOR_OWNER_TOKEN: ResourceAddress =
    vanity_address!(EntityType::NonFungibleResource, 5);

/// The address of the faucet package.
pub const PACKAGE_PACKAGE: PackageAddress = vanity_address!(EntityType::Package, 0);
pub const RESOURCE_MANAGER_PACKAGE: PackageAddress = vanity_address!(EntityType::Package, 1);
pub const IDENTITY_PACKAGE: PackageAddress = vanity_address!(EntityType::Package, 2);
pub const EPOCH_MANAGER_PACKAGE: PackageAddress = vanity_address!(EntityType::Package, 3);
pub const CLOCK_PACKAGE: PackageAddress = vanity_address!(EntityType::Package, 4);
pub const ACCOUNT_PACKAGE: PackageAddress = vanity_address!(EntityType::Package, 5);
pub const ACCESS_CONTROLLER_PACKAGE: PackageAddress = vanity_address!(EntityType::Package, 6);
pub const TRANSACTION_PROCESSOR_PACKAGE: PackageAddress = vanity_address!(EntityType::Package, 7);
pub const METADATA_PACKAGE: PackageAddress = vanity_address!(EntityType::Package, 10);
pub const ROYALTY_PACKAGE: PackageAddress = vanity_address!(EntityType::Package, 11);
pub const ACCESS_RULES_PACKAGE: PackageAddress = vanity_address!(EntityType::Package, 12);

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

pub const FAUCET_PACKAGE: PackageAddress = vanity_address!(EntityType::Package, 64);
pub const FAUCET_BLUEPRINT: &str = "Faucet";

/// The address of the faucet component, test network only.
pub const FAUCET_COMPONENT: ComponentAddress = construct_address!(
    EntityType::NormalComponent,
    236,
    50,
    10,
    144,
    199,
    2,
    90,
    211,
    144,
    180,
    74,
    9,
    97,
    68,
    149,
    245,
    250,
    10,
    4,
    229,
    206,
    191,
    50,
    129,
    179,
    215
);

pub const CLOCK: ComponentAddress = vanity_address!(EntityType::Clock, 0);
pub const EPOCH_MANAGER: ComponentAddress = vanity_address!(EntityType::EpochManager, 0);
