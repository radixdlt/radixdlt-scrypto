use crate::component::{ComponentAddress, PackageAddress};
use crate::resource::*;
use crate::{address, construct_address};

// After changing Radix Engine ID allocation, you will most likely need to update the addresses below.
//
// To obtain the new addresses, uncomment the println code in `id_allocator.rs` and
// run `cd radix-engine && cargo test --test metering -- can_withdraw_from_my_account --nocapture`.
//
// We've arranged the addresses in the order they're created in the genesis transaction.

/// The address of the sys-faucet package.
pub const SYS_FAUCET_PACKAGE: PackageAddress = construct_address!(
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

/// The address of the account package.
pub const ACCOUNT_PACKAGE: PackageAddress = construct_address!(
    EntityType::Package,
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

/// The ECDSA virtual resource address.
pub const ECDSA_TOKEN: ResourceAddress = construct_address!(
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

/// The system token which allows access to system resources (e.g. setting epoch)
pub const SYSTEM_TOKEN: ResourceAddress = construct_address!(
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

/// The XRD resource address.
pub const RADIX_TOKEN: ResourceAddress = address!(
    EntityType::Resource,
    241,
    88,
    60,
    234,
    185,
    86,
    59,
    118,
    36,
    26,
    46,
    225,
    245,
    4,
    254,
    227,
    6,
    207,
    47,
    230,
    180,
    123,
    170,
    4,
    214,
    11
);

/// The address of the SysFaucet component
pub const SYS_FAUCET_COMPONENT: ComponentAddress = construct_address!(
    EntityType::NormalComponent,
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

pub const SYS_SYSTEM_COMPONENT: ComponentAddress = construct_address!(
    EntityType::SystemComponent,
    81,
    130,
    179,
    100,
    248,
    151,
    66,
    205,
    2,
    212,
    180,
    65,
    90,
    181,
    166,
    24,
    91,
    47,
    48,
    75,
    176,
    51,
    61,
    100,
    210,
    105
);

/// The ED25519 virtual resource address.
pub const ED25519_TOKEN: ResourceAddress = address!(EntityType::Resource, 3u8);
