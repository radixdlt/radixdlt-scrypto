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
    237,
    145,
    0,
    85,
    29,
    127,
    174,
    145,
    234,
    244,
    19,
    229,
    10,
    60,
    90,
    89,
    248,
    185,
    106,
    249,
    241,
    41,
    120,
    144,
    168,
    244
);

/// The system token which allows access to system resources (e.g. setting epoch)
pub const SYSTEM_TOKEN: ResourceAddress = construct_address!(
    EntityType::Resource,
    8,
    176,
    243,
    225,
    122,
    72,
    73,
    246,
    165,
    39,
    153,
    157,
    162,
    22,
    192,
    89,
    71,
    248,
    164,
    205,
    128,
    162,
    219,
    72,
    86,
    194
);

/// The XRD resource address.
pub const RADIX_TOKEN: ResourceAddress = construct_address!(
    EntityType::Resource,
    141,
    129,
    247,
    20,
    46,
    8,
    166,
    23,
    225,
    192,
    118,
    147,
    168,
    25,
    252,
    113,
    41,
    42,
    140,
    141,
    169,
    183,
    148,
    102,
    224,
    208
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

pub const CLOCK: SystemAddress = construct_address!(
    EntityType::Clock,
    198,
    192,
    61,
    210,
    4,
    230,
    44,
    57,
    219,
    60,
    174,
    35,
    57,
    88,
    91,
    98,
    186,
    244,
    0,
    251,
    251,
    77,
    116,
    187,
    229,
    39
);

/// The ED25519 virtual resource address.
pub const EDDSA_ED25519_TOKEN: ResourceAddress = construct_address!(
    EntityType::Resource,
    24,
    67,
    46,
    131,
    28,
    174,
    236,
    45,
    222,
    176,
    209,
    180,
    88,
    119,
    97,
    212,
    46,
    119,
    120,
    5,
    129,
    234,
    46,
    214,
    27,
    145
);

/// The ED25519 virtual resource address.
pub const ENTITY_OWNER_TOKEN: ResourceAddress = construct_address!(
    EntityType::Resource,
    16,
    85,
    143,
    172,
    193,
    3,
    28,
    187,
    113,
    212,
    124,
    223,
    59,
    119,
    210,
    87,
    6,
    22,
    103,
    39,
    165,
    138,
    45,
    144,
    144,
    32
);

pub const EPOCH_MANAGER_BLUEPRINT: &str = "EpochManager";
pub const CLOCK_BLUEPRINT: &str = "Clock";
pub const RESOURCE_MANAGER_BLUEPRINT: &str = "ResourceManager";
pub const PACKAGE_BLUEPRINT: &str = "Package";
pub const TRANSACTION_PROCESSOR_BLUEPRINT: &str = "TransactionProcessor";
