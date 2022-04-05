use sbor::*;

use crate::component::*;
use crate::core::actor::ActorType::Blueprint;
use crate::rust::string::String;

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum ActorType {
    Blueprint,
    Component(ComponentId),
}

/// Represents the running entity.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct Actor {
    package_id: PackageId,
    blueprint_name: String,
    actor_type: ActorType
}

impl Actor {
    pub fn blueprint(package_id: PackageId, blueprint_name: String) -> Self {
        Self {
            package_id,
            blueprint_name,
            actor_type: Blueprint
        }
    }

    pub fn component(package_id: PackageId, blueprint_name: String, component_id: ComponentId) -> Self {
        Self {
            package_id,
            blueprint_name,
            actor_type: ActorType::Component(component_id)
        }
    }

    pub fn actor_type(&self) -> &ActorType {
        &self.actor_type
    }

    pub fn package_id(&self) -> &PackageId {
        &self.package_id
    }

    pub fn blueprint_name(&self) -> &str {
        &self.blueprint_name
    }

    pub fn to_package_id(self) -> PackageId {
        self.package_id
    }
}