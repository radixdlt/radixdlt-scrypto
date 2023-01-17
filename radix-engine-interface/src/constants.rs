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
pub const EDDSA_ED25519_TOKEN: ResourceAddress = construct_address!(
    EntityType::Resource,
    128, 50, 110, 40, 98, 18, 169, 58, 222, 243, 237, 202, 58, 215, 189, 84, 183, 148, 228, 180, 27, 162, 232, 107, 209, 51
);

/// The system token which allows access to system resources (e.g. setting epoch)
pub const SYSTEM_TOKEN: ResourceAddress = construct_address!(
    EntityType::Resource,
    16, 250, 14, 218, 250, 139, 223, 121, 98, 118, 191, 143, 238, 42, 51, 138, 34, 133, 68, 18, 22, 144, 110, 7, 63, 181
);

pub const PACKAGE_TOKEN: ResourceAddress = construct_address!(
    EntityType::Resource,
    146, 6, 99, 0, 115, 208, 107, 199, 180, 60, 202, 219, 152, 23, 53, 221, 20, 90, 49, 191, 126, 126, 178, 3, 223, 36
);

/// The address of the faucet package.
pub const FAUCET_PACKAGE: PackageAddress = construct_address!(
    EntityType::Package,
    213, 161, 133, 56, 35, 0, 134, 18, 95, 85, 106, 231, 194, 125, 161, 89, 8, 192, 242, 188, 20, 25, 128, 43, 55, 160
);
pub const FAUCET_BLUEPRINT: &str = "Faucet";

/// The address of the account package.
pub const ACCOUNT_PACKAGE: PackageAddress = construct_address!(
    EntityType::Package,
    94, 194, 206, 3, 11, 106, 209, 227, 27, 93, 166, 179, 102, 75, 2, 249, 237, 248, 184, 98, 92, 161, 250, 159, 159, 234
);
pub const ACCOUNT_BLUEPRINT: &str = "Account";

/// The address of the faucet component, test network only.
pub const FAUCET_COMPONENT: ComponentAddress = construct_address!(
    EntityType::NormalComponent,
    176, 10, 0, 75, 205, 239, 112, 228, 194, 134, 77, 97, 49, 158, 52, 119, 229, 181, 108, 197, 4, 27, 45, 0, 196, 52
);

pub const EPOCH_MANAGER: ComponentAddress = construct_address!(
    EntityType::EpochManager,
    234, 179, 239, 19, 250, 229, 160, 188, 178, 152, 196, 66, 133, 204, 37, 144, 243, 45, 76, 85, 249, 85, 205, 212, 31, 23
);

pub const CLOCK: ComponentAddress = construct_address!(
    EntityType::Clock,
    113, 179, 235, 101, 238, 110, 3, 172, 128, 140, 185, 116, 251, 175, 151, 172, 131, 26, 147, 148, 218, 207, 211, 218, 56, 107
);
