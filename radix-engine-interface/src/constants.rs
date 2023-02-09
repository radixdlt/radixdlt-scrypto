use crate::address;
use crate::api::component::ComponentAddress;
use crate::api::package::PackageAddress;
use crate::blueprints::resource::ResourceAddress;
use crate::construct_address;

/// The XRD resource address.
pub const RADIX_TOKEN: ResourceAddress = address!(EntityType::Resource, 0);

/// The ECDSA virtual resource address.
pub const ECDSA_SECP256K1_TOKEN: ResourceAddress = address!(EntityType::Resource, 1);

/// The ED25519 virtual resource address.
pub const EDDSA_ED25519_TOKEN: ResourceAddress = address!(EntityType::Resource, 2);

/// The system token which allows access to system resources (e.g. setting epoch)
pub const SYSTEM_TOKEN: ResourceAddress = address!(EntityType::Resource, 3);

pub const PACKAGE_TOKEN: ResourceAddress = address!(EntityType::Resource, 4);

pub const OLYMPIA_VALIDATOR_TOKEN: ResourceAddress = address!(EntityType::Resource, 5);

/// The address of the faucet package.
pub const FAUCET_PACKAGE: PackageAddress = address!(EntityType::Package, 0);
pub const FAUCET_BLUEPRINT: &str = "Faucet";

/// The address of the faucet component, test network only.
pub const FAUCET_COMPONENT: ComponentAddress = construct_address!(
    EntityType::NormalComponent,
    236,
    50,
    10,
    144,
    199,
    2,
    90,
    211,
    144,
    180,
    74,
    9,
    97,
    68,
    149,
    245,
    250,
    10,
    4,
    229,
    206,
    191,
    50,
    129,
    179,
    215
);

pub const CLOCK: ComponentAddress = address!(EntityType::Clock, 0);
pub const EPOCH_MANAGER: ComponentAddress = address!(EntityType::EpochManager, 0);
