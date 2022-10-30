use sbor::rust::string::String;
use sbor::*;

use crate::component::*;
use crate::engine::types::ComponentId;

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum ScryptoActor {
    Blueprint(PackageAddress, String),
    Component(ComponentId, PackageAddress, String),
}

impl ScryptoActor {
    pub fn blueprint(package_address: PackageAddress, blueprint_name: String) -> Self {
        Self::Blueprint(package_address, blueprint_name)
    }

    pub fn component(
        component_id: ComponentId,
        package_address: PackageAddress,
        blueprint_name: String,
    ) -> Self {
        Self::Component(component_id, package_address, blueprint_name)
    }

    pub fn package_address(&self) -> &PackageAddress {
        match self {
            ScryptoActor::Blueprint(package, _blueprint) => package,
            ScryptoActor::Component(_address, package, _blueprint) => package,
        }
    }

    pub fn blueprint_name(&self) -> &String {
        match self {
            ScryptoActor::Blueprint(_package, blueprint) => blueprint,
            ScryptoActor::Component(_address, _package, blueprint) => blueprint,
        }
    }

    pub fn as_blueprint(&self) -> (PackageAddress, String) {
        match self {
            Self::Blueprint(package_address, blueprint_name) => {
                (*package_address, blueprint_name.clone())
            }
            _ => panic!("Not a blueprint"),
        }
    }

    pub fn as_component(&self) -> (ComponentId, PackageAddress, String) {
        match self {
            Self::Component(component_id, package_address, blueprint) => {
                (*component_id, *package_address, blueprint.clone())
            }
            _ => panic!("Not a component"),
        }
    }
}
