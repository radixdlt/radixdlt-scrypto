use crate::core::*;
use crate::resource::*;

/// The package which defines the `System` blueprint.
pub const SYSTEM_PACKAGE: PackageRef = PackageRef::SYSTEM;

/// The system component
pub const SYSTEM_COMPONENT: ComponentRef = ComponentRef::SYSTEM;

/// The package that defines the `Account` blueprint.
pub const ACCOUNT_PACKAGE: PackageRef = PackageRef::ACCOUNT;

/// The XRD resource definition.
pub const RADIX_TOKEN: ResourceDefRef = ResourceDefRef::RADIX_TOKEN;

/// The ECDSA virtual resource definition.
pub const ECDSA_TOKEN: ResourceDefRef = ResourceDefRef::ECDSA_TOKEN;
