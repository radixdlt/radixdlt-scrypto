use crate::construct_address;
use crate::model::*;

// After changing Radix Engine ID allocation, you will most likely need to update the addresses below.
//
// To obtain the new addresses, uncomment the println code in `id_allocator.rs` and
// run `cd radix-engine && cargo test -- bootstrap_receipt_should_match_constants --nocapture`.
//
// We've arranged the addresses in the order they're created in the genesis transaction.

/// The address of the faucet package.
pub const SYS_FAUCET_PACKAGE: PackageAddress = construct_address!(
    EntityType::Package,
    0,
    44,
    100,
    204,
    153,
    17,
    167,
    139,
    223,
    159,
    221,
    222,
    95,
    90,
    157,
    196,
    136,
    236,
    235,
    197,
    213,
    35,
    187,
    15,
    207,
    158
);
pub const FAUCET_BLUEPRINT: &str = "Faucet";

/// The address of the account package.
pub const ACCOUNT_PACKAGE: PackageAddress = construct_address!(
    EntityType::Package,
    43,
    113,
    132,
    253,
    47,
    66,
    111,
    180,
    52,
    199,
    68,
    195,
    33,
    205,
    145,
    223,
    131,
    117,
    181,
    225,
    240,
    27,
    116,
    0,
    157,
    255
);
pub const ACCOUNT_BLUEPRINT: &str = "Account";

/// The ECDSA virtual resource address.
pub const ECDSA_SECP256K1_TOKEN: ResourceAddress = construct_address!(
    EntityType::Resource,
    185,
    23,
    55,
    238,
    138,
    77,
    229,
    157,
    73,
    218,
    212,
    13,
    229,
    86,
    14,
    87,
    84,
    70,
    106,
    200,
    76,
    245,
    67,
    46,
    169,
    93
);

/// The system token which allows access to system resources (e.g. setting epoch)
pub const SYSTEM_TOKEN: ResourceAddress = construct_address!(
    EntityType::Resource,
    146,
    35,
    6,
    166,
    209,
    58,
    246,
    56,
    102,
    182,
    136,
    201,
    16,
    55,
    25,
    208,
    75,
    20,
    192,
    96,
    188,
    72,
    153,
    166,
    19,
    181
);

/// The XRD resource address.
pub const RADIX_TOKEN: ResourceAddress = construct_address!(
    EntityType::Resource,
    173,
    130,
    50,
    141,
    112,
    34,
    61,
    91,
    174,
    38,
    130,
    96,
    179,
    4,
    93,
    204,
    113,
    220,
    243,
    95,
    55,
    167,
    67,
    74,
    9,
    105
);

/// The address of the faucet component, test network only.
pub const FAUCET_COMPONENT: ComponentAddress = construct_address!(
    EntityType::NormalComponent,
    87,
    220,
    4,
    44,
    216,
    203,
    145,
    111,
    54,
    48,
    2,
    10,
    31,
    51,
    124,
    236,
    90,
    84,
    207,
    239,
    164,
    197,
    8,
    79,
    190,
    60
);

pub const EPOCH_MANAGER: SystemAddress = construct_address!(
    EntityType::EpochManager,
    242,
    112,
    114,
    176,
    201,
    24,
    36,
    161,
    165,
    168,
    98,
    35,
    142,
    88,
    111,
    226,
    199,
    205,
    55,
    97,
    235,
    46,
    52,
    60,
    218,
    190
);

/// The ED25519 virtual resource address.
pub const EDDSA_ED25519_TOKEN: ResourceAddress = construct_address!(
    EntityType::Resource,
    172,
    110,
    120,
    193,
    250,
    70,
    187,
    76,
    68,
    171,
    211,
    30,
    43,
    73,
    30,
    13,
    198,
    37,
    110,
    194,
    242,
    109,
    76,
    165,
    200,
    50
);

/// The ED25519 virtual resource address.
pub const ENTITY_OWNER_TOKEN: ResourceAddress = construct_address!(
    EntityType::Resource,
    112,
    80,
    185,
    38,
    180,
    181,
    171,
    151,
    101,
    224,
    68,
    235,
    5,
    132,
    5,
    4,
    142,
    77,
    126,
    195,
    109,
    190,
    183,
    241,
    137,
    99
);

pub const EPOCH_MANAGER_BLUEPRINT: &str = "EpochManager";
pub const RESOURCE_MANAGER_BLUEPRINT: &str = "ResourceManager";
pub const PACKAGE_BLUEPRINT: &str = "Package";
pub const TRANSACTION_PROCESSOR_BLUEPRINT: &str = "TransactionProcessor";
