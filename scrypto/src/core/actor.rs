use sbor::rust::string::String;
use sbor::*;

use crate::component::*;

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum ScryptoActor {
    Blueprint(PackageAddress, String),
    Component(ComponentAddress, bool),
}

impl ScryptoActor {
    pub fn blueprint(package_address: PackageAddress, blueprint_name: String) -> Self {
        Self::Blueprint(package_address, blueprint_name)
    }

    pub fn component(component_address: ComponentAddress, is_global: bool) -> Self {
        Self::Component(component_address, is_global)
    }

    pub fn as_blueprint(&self) -> (PackageAddress, String) {
        match self {
            Self::Blueprint(package_address, blueprint_name) => {
                (*package_address, blueprint_name.clone())
            }
            _ => panic!("Not a blueprint"),
        }
    }

    pub fn as_component(&self) -> (ComponentAddress, bool) {
        match self {
            Self::Component(component_address, is_global) => (*component_address, *is_global),
            _ => panic!("Not a component"),
        }
    }
}
