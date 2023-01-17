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
pub const FAUCET_PACKAGE: PackageAddress = construct_address!(
    EntityType::Package,
    89, 100, 160, 56, 236, 141, 48, 10, 204, 4, 212, 116, 206, 163, 82, 50, 91, 137, 94, 1, 191, 236, 15, 125, 121, 66
);
pub const FAUCET_BLUEPRINT: &str = "Faucet";

/// The address of the account package.
pub const ACCOUNT_PACKAGE: PackageAddress = construct_address!(
    EntityType::Package,
    213, 161, 133, 56, 35, 0, 134, 18, 95, 85, 106, 231, 194, 125, 161, 89, 8, 192, 242, 188, 20, 25, 128, 43, 55, 160
);
pub const ACCOUNT_BLUEPRINT: &str = "Account";

/// The address of the faucet component, test network only.
pub const FAUCET_COMPONENT: ComponentAddress = construct_address!(
    EntityType::NormalComponent,
    68, 85, 49, 134, 195, 96, 197, 209, 94, 115, 234, 233, 202, 98, 101, 192, 200, 249, 60, 15, 10, 240, 94, 121, 160, 14
);

pub const EPOCH_MANAGER: ComponentAddress = construct_address!(
    EntityType::EpochManager,
    44, 201, 216, 26, 12, 130, 244, 40, 246, 101, 231, 56, 157, 150, 155, 85, 99, 116, 124, 101, 220, 228, 216, 136, 204, 65
);

pub const CLOCK: ComponentAddress = construct_address!(
    EntityType::Clock,
    227, 163, 118, 15, 127, 189, 22, 95, 36, 41, 56, 156, 128, 25, 19, 116, 9, 130, 179, 131, 64, 103, 79, 48, 161, 181
);
