use crate::address;
use crate::construct_address;
use crate::model::*;

// After changing Radix Engine ID allocation, you will most likely need to update the addresses below.
//
// To obtain the new addresses, uncomment the println code in `id_allocator.rs` and
// run `cd radix-engine && cargo test -- bootstrap_receipt_should_match_constants --nocapture`.
//
// We've arranged the addresses in the order they're created in the genesis transaction.

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
pub const ACCOUNT_PACKAGE: PackageAddress = construct_address!(
    EntityType::Package,
    207, 236, 86, 95, 3, 195, 152, 29, 79, 149, 88, 154, 46, 145, 227, 3, 124, 205, 101, 35, 246, 126, 64, 75, 176, 175
);
pub const ACCOUNT_BLUEPRINT: &str = "Account";

/// The address of the faucet component, test network only.
pub const FAUCET_COMPONENT: ComponentAddress = construct_address!(
    EntityType::NormalComponent,
    185, 97, 245, 214, 187, 182, 109, 5, 245, 67, 89, 29, 143, 159, 167, 177, 194, 186, 135, 12, 32, 227, 114, 156, 61, 150
);

pub const EPOCH_MANAGER: ComponentAddress = construct_address!(
    EntityType::EpochManager,
    176, 10, 0, 75, 205, 239, 112, 228, 194, 134, 77, 97, 49, 158, 52, 119, 229, 181, 108, 197, 4, 27, 45, 0, 196, 52
);

pub const CLOCK: ComponentAddress = construct_address!(
    EntityType::Clock,
    30, 175, 126, 217, 126, 105, 54, 141, 50, 230, 89, 92, 255, 39, 106, 43, 186, 185, 13, 165, 15, 193, 113, 158, 82, 4
);
