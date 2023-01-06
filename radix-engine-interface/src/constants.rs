use crate::construct_address;
use crate::model::*;

// After changing Radix Engine ID allocation, you will most likely need to update the addresses below.
//
// To obtain the new addresses, uncomment the println code in `id_allocator.rs` and
// run `cd radix-engine && cargo test -- bootstrap_receipt_should_match_constants --nocapture`.
//
// We've arranged the addresses in the order they're created in the genesis transaction.

/// The XRD resource address.
pub const RADIX_TOKEN: ResourceAddress = construct_address!(
    EntityType::Resource,
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

/// The ED25519 virtual resource address.
pub const EDDSA_ED25519_TOKEN: ResourceAddress = construct_address!(
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

/// The system token which allows access to system resources (e.g. setting epoch)
pub const SYSTEM_TOKEN: ResourceAddress = construct_address!(
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

/// The address of the faucet package.
pub const FAUCET_PACKAGE: PackageAddress = construct_address!(
    EntityType::Package,
    41,
    141,
    243,
    71,
    21,
    23,
    189,
    70,
    143,
    131,
    179,
    73,
    250,
    68,
    140,
    250,
    231,
    21,
    73,
    177,
    234,
    232,
    148,
    74,
    98,
    197
);
pub const FAUCET_BLUEPRINT: &str = "Faucet";

/// The address of the account package.
pub const ACCOUNT_PACKAGE: PackageAddress = construct_address!(
    EntityType::Package,
    139,
    102,
    112,
    90,
    86,
    241,
    123,
    106,
    194,
    118,
    77,
    122,
    228,
    192,
    200,
    254,
    97,
    228,
    48,
    125,
    233,
    170,
    107,
    105,
    87,
    105
);
pub const ACCOUNT_BLUEPRINT: &str = "Account";

/// The address of the faucet component, test network only.
pub const FAUCET_COMPONENT: ComponentAddress = construct_address!(
    EntityType::NormalComponent,
    168,
    222,
    159,
    163,
    116,
    25,
    115,
    128,
    191,
    83,
    43,
    211,
    134,
    178,
    13,
    170,
    221,
    74,
    166,
    159,
    237,
    25,
    224,
    234,
    93,
    16
);

pub const EPOCH_MANAGER: SystemAddress = construct_address!(
    EntityType::EpochManager,
    68,
    160,
    7,
    98,
    178,
    140,
    86,
    219,
    59,
    214,
    120,
    133,
    86,
    56,
    212,
    115,
    198,
    8,
    151,
    39,
    203,
    125,
    28,
    210,
    215,
    13
);

pub const CLOCK: SystemAddress = construct_address!(
    EntityType::Clock,
    10,
    252,
    122,
    98,
    202,
    201,
    171,
    182,
    129,
    235,
    221,
    160,
    28,
    137,
    17,
    63,
    185,
    222,
    48,
    85,
    176,
    59,
    255,
    172,
    127,
    30
);

pub const EPOCH_MANAGER_BLUEPRINT: &str = "EpochManager";
pub const CLOCK_BLUEPRINT: &str = "Clock";
pub const RESOURCE_MANAGER_BLUEPRINT: &str = "ResourceManager";
pub const PACKAGE_BLUEPRINT: &str = "Package";
pub const TRANSACTION_PROCESSOR_BLUEPRINT: &str = "TransactionProcessor";
