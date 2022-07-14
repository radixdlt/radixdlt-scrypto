use crate::address::*;
use crate::engine::types::{ComponentAddress, PackageAddress};
use crate::resource::*;
use crate::{address, entity_type_id_from_entity_type, type_from_entity_type};

/// The package of the system blueprint.
pub const SYSTEM_PACKAGE: PackageAddress = address!(EntityType::Package, 1u8);

/// The system component
pub const SYSTEM_COMPONENT: ComponentAddress = address!(EntityType::Component, 2u8);

/// The package of the account blueprint.
pub const ACCOUNT_PACKAGE: PackageAddress = address!(EntityType::Package, 3u8);

/// The XRD resource address.
pub const RADIX_TOKEN: ResourceAddress = address!(EntityType::Resource, 4u8);

/// The ECDSA virtual resource address.
pub const ECDSA_TOKEN: ResourceAddress = address!(EntityType::Resource, 5u8);

/// The system token which allows access to system resources (e.g. setting epoch)
pub const SYSTEM_TOKEN: ResourceAddress = address!(EntityType::Resource, 6u8);
