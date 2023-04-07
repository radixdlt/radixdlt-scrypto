mod component_address;
mod global_address;
mod local_address;
mod package_address;
mod resource_address;

pub use component_address::*;
pub use global_address::*;
pub use local_address::*;
pub use package_address::*;
pub use resource_address::*;

use crate::types::EntityType;

pub const fn component_address(entity_type: EntityType, last_byte: u8) -> ComponentAddress {
    assert!(entity_type.is_global_component());
    ComponentAddress::new_unchecked([
        entity_type as u8,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        last_byte,
    ])
}

pub const fn resource_address(entity_type: EntityType, last_byte: u8) -> ResourceAddress {
    assert!(entity_type.is_global_resource());
    ResourceAddress::new_unchecked([
        entity_type as u8,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        last_byte,
    ])
}

pub const fn package_address(entity_type: EntityType, last_byte: u8) -> PackageAddress {
    assert!(entity_type.is_global_package());
    PackageAddress::new_unchecked([
        entity_type as u8,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        last_byte,
    ])
}

pub const fn local_address(entity_type: EntityType, last_byte: u8) -> LocalAddress {
    assert!(entity_type.is_local());
    LocalAddress::new_unchecked([
        entity_type as u8,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        last_byte,
    ])
}
