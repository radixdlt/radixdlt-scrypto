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
pub const ECDSA_SECP256K1_TOKEN: ResourceAddress = address!(EntityType::Resource,
    226, 20, 49, 54, 10, 211, 33, 197, 167, 120, 0, 138, 190, 178, 244, 113, 150, 183, 105, 78, 68, 36, 222, 121, 148, 233
);

/// The ED25519 virtual resource address.
pub const EDDSA_ED25519_TOKEN: ResourceAddress = construct_address!(
    EntityType::Resource,
    225, 129, 131, 180, 138, 36, 112, 43, 200, 17, 98, 3, 254, 48, 235, 0, 250, 44, 30, 232, 206, 16, 250, 109, 171, 172
);

/// The system token which allows access to system resources (e.g. setting epoch)
pub const SYSTEM_TOKEN: ResourceAddress = construct_address!(
    EntityType::Resource,
    94, 243, 82, 38, 121, 134, 34, 59, 99, 8, 244, 59, 24, 113, 244, 1, 9, 180, 42, 187, 103, 88, 187, 89, 25, 242
);

pub const PACKAGE_TOKEN: ResourceAddress = construct_address!(
    EntityType::Resource,
    92, 79, 54, 236, 171, 112, 227, 86, 75, 146, 40, 60, 170, 136, 15, 136, 203, 76, 237, 63, 25, 68, 145, 30, 190, 244
);

/// The address of the faucet package.
pub const FAUCET_PACKAGE: PackageAddress = construct_address!(
    EntityType::Package,
    91, 209, 29, 27, 30, 180, 28, 133, 167, 1, 118, 177, 96, 131, 149, 140, 172, 94, 195, 71, 227, 13, 108, 137, 236, 242
);
pub const FAUCET_BLUEPRINT: &str = "Faucet";

/// The address of the account package.
pub const ACCOUNT_PACKAGE: PackageAddress = construct_address!(
    EntityType::Package,
    57, 78, 78, 153, 179, 175, 144, 54, 157, 76, 67, 70, 161, 216, 233, 38, 187, 32, 234, 127, 84, 0, 140, 48, 198, 110
);
pub const ACCOUNT_BLUEPRINT: &str = "Account";

/// The address of the faucet component, test network only.
pub const FAUCET_COMPONENT: ComponentAddress = construct_address!(
    EntityType::NormalComponent,
    44, 201, 216, 26, 12, 130, 244, 40, 246, 101, 231, 56, 157, 150, 155, 85, 99, 116, 124, 101, 220, 228, 216, 136, 204, 65
);

pub const EPOCH_MANAGER: ComponentAddress = construct_address!(
    EntityType::EpochManager,
    138, 74, 99, 90, 238, 68, 188, 90, 51, 2, 91, 141, 166, 61, 140, 75, 226, 206, 4, 77, 132, 149, 103, 82, 83, 171
);

pub const CLOCK: ComponentAddress = construct_address!(
    EntityType::Clock,
    58, 117, 34, 241, 164, 99, 21, 98, 149, 118, 85, 226, 89, 114, 5, 179, 129, 206, 228, 174, 209, 252, 96, 28, 102, 178
);
