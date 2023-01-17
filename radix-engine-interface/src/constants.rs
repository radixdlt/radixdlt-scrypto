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
pub const ACCOUNT_PACKAGE: PackageAddress = address!(EntityType::Package, 1);
pub const ACCOUNT_BLUEPRINT: &str = "Account";

/// The address of the faucet component, test network only.
pub const FAUCET_COMPONENT: ComponentAddress = construct_address!(
    EntityType::NormalComponent,
    57, 78, 78, 153, 179, 175, 144, 54, 157, 76, 67, 70, 161, 216, 233, 38, 187, 32, 234, 127, 84, 0, 140, 48, 198, 110
);

pub const EPOCH_MANAGER: ComponentAddress = construct_address!(
    EntityType::EpochManager,
    227, 163, 118, 15, 127, 189, 22, 95, 36, 41, 56, 156, 128, 25, 19, 116, 9, 130, 179, 131, 64, 103, 79, 48, 161, 181
);

pub const CLOCK: ComponentAddress = construct_address!(
    EntityType::Clock,
    68, 85, 49, 134, 195, 96, 197, 209, 94, 115, 234, 233, 202, 98, 101, 192, 200, 249, 60, 15, 10, 240, 94, 121, 160, 14
);
