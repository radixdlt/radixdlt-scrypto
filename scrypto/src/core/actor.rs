use sbor::*;

use crate::component::*;
use crate::rust::string::String;

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum ScryptoActor {
    Blueprint(PackageAddress, String),
    Component(ComponentAddress),
}

/// Represents the running entity.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct ScryptoActorInfo {
    package_address: PackageAddress,
    blueprint_name: String,
    component_address: Option<ComponentAddress>,
    export_name: String,
}

impl ScryptoActorInfo {
    pub fn blueprint(
        package_address: PackageAddress,
        blueprint_name: String,
        export_name: String,
    ) -> Self {
        Self {
            package_address,
            blueprint_name,
            component_address: None,
            export_name,
        }
    }

    pub fn component(
        package_address: PackageAddress,
        blueprint_name: String,
        export_name: String,
        component_address: ComponentAddress,
    ) -> Self {
        Self {
            package_address,
            blueprint_name,
            component_address: Some(component_address),
            export_name,
        }
    }

    pub fn component_address(&self) -> Option<ComponentAddress> {
        self.component_address
    }

    pub fn export_name(&self) -> &str {
        &self.export_name
    }

    pub fn actor(&self) -> ScryptoActor {
        if let Some(addr) = self.component_address {
            ScryptoActor::Component(addr)
        } else {
            ScryptoActor::Blueprint(self.package_address.clone(), self.blueprint_name.clone())
        }
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
