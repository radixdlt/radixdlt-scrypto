use crate::construct_address;
use crate::model::*;

// After changing Radix Engine ID allocation, you will most likely need to update the addresses below.
//
// To obtain the new addresses, uncomment the println code in `id_allocator.rs` and
// run `cd radix-engine && cargo test -- bootstrap_receipt_should_match_constants --nocapture`.
//
// We've arranged the addresses in the order they're created in the genesis transaction.

/// The address of the faucet package.
pub const FAUCET_PACKAGE: PackageAddress = construct_address!(
    EntityType::Package,
    83,
    241,
    195,
    226,
    12,
    194,
    56,
    53,
    94,
    35,
    176,
    29,
    236,
    187,
    0,
    167,
    136,
    92,
    42,
    130,
    100,
    141,
    94,
    133,
    157,
    79
);
pub const FAUCET_BLUEPRINT: &str = "Faucet";

/// The address of the account package.
pub const ACCOUNT_PACKAGE: PackageAddress = construct_address!(
    EntityType::Package,
    206,
    236,
    4,
    30,
    135,
    40,
    9,
    150,
    132,
    187,
    191,
    193,
    135,
    118,
    76,
    60,
    29,
    177,
    19,
    55,
    102,
    201,
    65,
    143,
    168,
    15
);
pub const ACCOUNT_BLUEPRINT: &str = "Account";

/// The ECDSA virtual resource address.
pub const ECDSA_SECP256K1_TOKEN: ResourceAddress = construct_address!(
    EntityType::Resource,
    183,
    5,
    84,
    120,
    29,
    187,
    91,
    52,
    106,
    12,
    202,
    40,
    56,
    242,
    194,
    46,
    214,
    59,
    64,
    82,
    248,
    103,
    140,
    64,
    210,
    19
);

/// The system token which allows access to system resources (e.g. setting epoch)
pub const SYSTEM_TOKEN: ResourceAddress = construct_address!(
    EntityType::Resource,
    251,
    209,
    29,
    182,
    229,
    138,
    124,
    19,
    239,
    132,
    175,
    139,
    211,
    54,
    92,
    87,
    123,
    125,
    29,
    48,
    97,
    12,
    125,
    6,
    131,
    208
);

/// The XRD resource address.
pub const RADIX_TOKEN: ResourceAddress = construct_address!(
    EntityType::Resource,
    236,
    106,
    120,
    159,
    143,
    13,
    221,
    145,
    4,
    37,
    227,
    231,
    245,
    106,
    85,
    104,
    249,
    221,
    70,
    50,
    6,
    109,
    237,
    77,
    32,
    128
);

/// The address of the faucet component, test network only.
pub const FAUCET_COMPONENT: ComponentAddress = construct_address!(
    EntityType::NormalComponent,
    241,
    80,
    14,
    193,
    40,
    120,
    1,
    16,
    50,
    105,
    249,
    218,
    195,
    64,
    201,
    162,
    23,
    173,
    172,
    153,
    29,
    117,
    113,
    45,
    245,
    16
);

pub const EPOCH_MANAGER: SystemAddress = construct_address!(
    EntityType::EpochManager,
    111,
    216,
    129,
    144,
    194,
    100,
    251,
    131,
    110,
    208,
    77,
    97,
    44,
    107,
    166,
    93,
    138,
    28,
    57,
    240,
    45,
    60,
    200,
    116,
    131,
    121
);

pub const CLOCK: SystemAddress = construct_address!(
    EntityType::Clock,
    210,
    4,
    203,
    199,
    253,
    87,
    86,
    55,
    225,
    160,
    209,
    125,
    34,
    246,
    206,
    141,
    224,
    160,
    236,
    54,
    219,
    221,
    233,
    10,
    33,
    79
);

/// The ED25519 virtual resource address.
pub const EDDSA_ED25519_TOKEN: ResourceAddress = construct_address!(
    EntityType::Resource,
    100,
    122,
    90,
    153,
    192,
    230,
    68,
    232,
    52,
    111,
    194,
    67,
    139,
    246,
    24,
    111,
    166,
    139,
    122,
    227,
    235,
    71,
    163,
    178,
    99,
    94
);

pub const EPOCH_MANAGER_BLUEPRINT: &str = "EpochManager";
pub const CLOCK_BLUEPRINT: &str = "Clock";
pub const RESOURCE_MANAGER_BLUEPRINT: &str = "ResourceManager";
pub const PACKAGE_BLUEPRINT: &str = "Package";
pub const TRANSACTION_PROCESSOR_BLUEPRINT: &str = "TransactionProcessor";
