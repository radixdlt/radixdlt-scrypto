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

use crate::types::{EntityType, NodeId};

pub const fn component_address(entity_type: EntityType, last_byte: u8) -> ComponentAddress {
    let mut node_id = [0u8; NodeId::LENGTH];
    node_id[0] = entity_type as u8;
    node_id[NodeId::LENGTH - 1] = last_byte;
    ComponentAddress::new_unchecked(node_id)
}

pub const fn resource_address(entity_type: EntityType, last_byte: u8) -> ResourceAddress {
    let mut node_id = [0u8; NodeId::LENGTH];
    node_id[0] = entity_type as u8;
    node_id[NodeId::LENGTH - 1] = last_byte;
    ResourceAddress::new_unchecked(node_id)
}

pub const fn package_address(entity_type: EntityType, last_byte: u8) -> PackageAddress {
    let mut node_id = [0u8; NodeId::LENGTH];
    node_id[0] = entity_type as u8;
    node_id[NodeId::LENGTH - 1] = last_byte;
    PackageAddress::new_unchecked(node_id)
}

pub const fn local_address(entity_type: EntityType, last_byte: u8) -> LocalAddress {
    let mut node_id = [0u8; NodeId::LENGTH];
    node_id[0] = entity_type as u8;
    node_id[NodeId::LENGTH - 1] = last_byte;
    LocalAddress::new_unchecked(node_id)
}
