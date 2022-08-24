use crate::address;
use crate::component::{ComponentAddress, PackageAddress};
use crate::resource::*;

/// The address of the sys-faucet package.
pub const SYS_FAUCET_PACKAGE: PackageAddress = address!(EntityType::Package, 1u8);

/// The address of the sys-utils package.
pub const SYS_UTILS_PACKAGE: PackageAddress = address!(EntityType::Package, 2u8);

/// The address of the account package.
pub const ACCOUNT_PACKAGE: PackageAddress = address!(EntityType::Package, 3u8);

/// The address of the SysFaucet component
pub const SYS_FAUCET_COMPONENT: ComponentAddress = address!(EntityType::SystemComponent, 1u8);
// TODO Add other system components

/// The system token which allows access to system resources (e.g. setting epoch)
pub const SYSTEM_TOKEN: ResourceAddress = address!(EntityType::Resource, 1u8);

/// The ECDSA virtual resource address.
pub const ECDSA_TOKEN: ResourceAddress = address!(EntityType::Resource, 2u8);

/// The ED25519 virtual resource address.
pub const ED25519_TOKEN: ResourceAddress = address!(EntityType::Resource, 3u8);

/// The XRD resource address.
pub const RADIX_TOKEN: ResourceAddress = address!(EntityType::Resource, 4u8);
