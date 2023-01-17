use crate::address;
use crate::construct_address;
use crate::model::*;

/// The XRD resource address.
pub const RADIX_TOKEN: ResourceAddress = address!(EntityType::Resource, 0);

/// The ECDSA virtual resource address.
pub const ECDSA_SECP256K1_TOKEN: ResourceAddress = address!(EntityType::Resource, 1);

/// The ED25519 virtual resource address.
pub const EDDSA_ED25519_TOKEN: ResourceAddress = address!(EntityType::Resource, 2);

/// The system token which allows access to system resources (e.g. setting epoch)
pub const SYSTEM_TOKEN: ResourceAddress = address!(EntityType::Resource, 3);

pub const PACKAGE_TOKEN: ResourceAddress = address!(EntityType::Resource, 4);

/// The address of the faucet package.
pub const FAUCET_PACKAGE: PackageAddress = address!(EntityType::Package, 0);
pub const FAUCET_BLUEPRINT: &str = "Faucet";

/// The address of the account package.
pub const ACCOUNT_PACKAGE: PackageAddress = address!(EntityType::Package, 1);
pub const ACCOUNT_BLUEPRINT: &str = "Account";

/// The address of the faucet component, test network only.
pub const FAUCET_COMPONENT: ComponentAddress = construct_address!(
    EntityType::NormalComponent,
    57, 78, 78, 153, 179, 175, 144, 54, 157, 76, 67, 70, 161, 216, 233, 38, 187, 32, 234, 127, 84, 0, 140, 48, 198, 110
);

pub const CLOCK: ComponentAddress = address!(EntityType::Clock, 0);
pub const EPOCH_MANAGER: ComponentAddress = address!(EntityType::EpochManager, 0);
