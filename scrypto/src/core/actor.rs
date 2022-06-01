use sbor::rust::string::String;
use sbor::*;
use scrypto_abi::BlueprintAbi;

use crate::component::*;

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
    blueprint_abi: BlueprintAbi,
    component_address: Option<ComponentAddress>,
}

impl ScryptoActorInfo {
    pub fn blueprint(
        package_address: PackageAddress,
        blueprint_name: String,
        blueprint_abi: BlueprintAbi,
    ) -> Self {
        Self {
            package_address,
            blueprint_name,
            blueprint_abi,
            component_address: None,
        }
    }

    pub fn component(
        package_address: PackageAddress,
        blueprint_name: String,
        blueprint_abi: BlueprintAbi,
        component_address: ComponentAddress,
    ) -> Self {
        Self {
            package_address,
            blueprint_name,
            blueprint_abi,
            component_address: Some(component_address),
        }
    }

    pub fn component_address(&self) -> Option<ComponentAddress> {
        self.component_address
    }

    pub fn blueprint_abi(&self) -> &BlueprintAbi {
        &self.blueprint_abi
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
