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
    117,
    149,
    161,
    192,
    155,
    192,
    68,
    56,
    79,
    186,
    128,
    155,
    199,
    188,
    92,
    59,
    83,
    241,
    146,
    178,
    126,
    213,
    55,
    167,
    164,
    201
);
pub const FAUCET_BLUEPRINT: &str = "Faucet";

/// The address of the account package.
pub const ACCOUNT_PACKAGE: PackageAddress = construct_address!(
    EntityType::Package,
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
pub const ACCOUNT_BLUEPRINT: &str = "Account";

/// The ECDSA virtual resource address.
pub const ECDSA_SECP256K1_TOKEN: ResourceAddress = construct_address!(
    EntityType::Resource,
    143,
    46,
    234,
    87,
    25,
    53,
    120,
    228,
    5,
    237,
    56,
    58,
    19,
    153,
    205,
    168,
    37,
    196,
    182,
    161,
    162,
    189,
    144,
    106,
    252,
    99
);

/// The system token which allows access to system resources (e.g. setting epoch)
pub const SYSTEM_TOKEN: ResourceAddress = construct_address!(
    EntityType::Resource,
    159,
    148,
    169,
    154,
    227,
    78,
    75,
    52,
    72,
    3,
    114,
    131,
    232,
    41,
    172,
    176,
    75,
    148,
    70,
    164,
    177,
    26,
    121,
    68,
    254,
    162
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
    115,
    9,
    63,
    87,
    114,
    161,
    225,
    209,
    191,
    174,
    22,
    244,
    105,
    12,
    88,
    40,
    227,
    50,
    217,
    76,
    172,
    184,
    235,
    208,
    222,
    10
);

pub const EPOCH_MANAGER: SystemAddress = construct_address!(
    EntityType::EpochManager,
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

pub const CLOCK: SystemAddress = construct_address!(
    EntityType::Clock,
    15,
    142,
    146,
    10,
    167,
    159,
    83,
    52,
    157,
    10,
    153,
    116,
    110,
    23,
    181,
    146,
    65,
    189,
    81,
    225,
    154,
    187,
    80,
    173,
    107,
    106
);

/// The ED25519 virtual resource address.
pub const EDDSA_ED25519_TOKEN: ResourceAddress = construct_address!(
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
pub const CLOCK_BLUEPRINT: &str = "Clock";
pub const RESOURCE_MANAGER_BLUEPRINT: &str = "ResourceManager";
pub const PACKAGE_BLUEPRINT: &str = "Package";
pub const TRANSACTION_PROCESSOR_BLUEPRINT: &str = "TransactionProcessor";
