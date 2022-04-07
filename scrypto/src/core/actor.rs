use sbor::*;

use crate::component::*;
use crate::core::actor::ActorType::Blueprint;
use crate::rust::string::String;

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum ActorType {
    Blueprint,
    Component(ComponentAddress),
}

/// Represents the running entity.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct ScryptoActor {
    package_address: PackageAddress,
    blueprint_name: String,
    actor_type: ActorType,
    export_name: String,
}

impl ScryptoActor {
    pub fn blueprint(package_address: PackageAddress, blueprint_name: String, export_name: String) -> Self {
        Self {
            package_address,
            blueprint_name,
            actor_type: Blueprint,
            export_name,
        }
    }

    pub fn component(package_address: PackageAddress, blueprint_name: String, export_name: String, component_address: ComponentAddress) -> Self {
        Self {
            package_address,
            blueprint_name,
            actor_type: ActorType::Component(component_address),
            export_name,
        }
    }

    pub fn export_name(&self) -> &str {
        &self.export_name
    }

    pub fn actor_type(&self) -> &ActorType {
        &self.actor_type
    }

    pub fn package_address(&self) -> &PackageAddress {
        &self.package_address
    }

    pub fn blueprint_name(&self) -> &str {
        &self.blueprint_name
    }

    pub fn to_package_address(self) -> PackageAddress {
        self.package_address
    }
}