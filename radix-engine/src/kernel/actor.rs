use crate::types::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum AdditionalActorInfo {
    Method(Option<Address>, RENodeId, NodeModuleId, PackageAddress, String, String),
    Function(PackageAddress, String, String),
    VirtualLazyLoad(PackageAddress, String, u8),
}

// TODO: This structure along with ActorIdentifier needs to be cleaned up
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct Actor {
    pub info: AdditionalActorInfo,
}

impl Actor {
    pub fn fn_identifier(&self) -> FnIdentifier {
        match &self.info {
            AdditionalActorInfo::Method(_, _, _, package_address, blueprint_name, ident) => {
                FnIdentifier::application_ident(package_address.clone(), blueprint_name.clone(), ident.clone())
            }
            AdditionalActorInfo::Function(package_address, blueprint_name, ident) => {
                FnIdentifier::application_ident(package_address.clone(), blueprint_name.clone(), ident.clone())
            }
            AdditionalActorInfo::VirtualLazyLoad(package_address, blueprint_name, ident) => {
                FnIdentifier::system_ident(package_address.clone(), blueprint_name.clone(), ident.clone())
            }
        }
    }

    pub fn package_address(&self) -> &PackageAddress {
        match &self.info {
            AdditionalActorInfo::Method(_, _, _, package_address, ..) => {
                package_address
            }
            AdditionalActorInfo::Function(package_address, ..) => {
                package_address
            }
            AdditionalActorInfo::VirtualLazyLoad(package_address, ..) => {
                package_address
            }
        }
    }

    pub fn blueprint_name(&self) -> &str {
        match &self.info {
            AdditionalActorInfo::Method(_, _, _, _, blueprint_name, ..) => {
                blueprint_name.as_str()
            }
            AdditionalActorInfo::Function(_, blueprint_name, ..) => {
                blueprint_name.as_str()
            }
            AdditionalActorInfo::VirtualLazyLoad(_, blueprint_name, ..) => {
                blueprint_name.as_str()
            }
        }
    }

    pub fn method(
        global_address: Option<Address>,
        method: MethodIdentifier,
        package_address: PackageAddress,
        blueprint_name: String,
    ) -> Self {
        Self {
            info: AdditionalActorInfo::Method(global_address, method.0, method.1, package_address, blueprint_name, method.2),
        }
    }

    pub fn function(ident: FunctionIdentifier) -> Self {
        Self {
            info: AdditionalActorInfo::Function(ident.0, ident.1, ident.2),
        }
    }

    pub fn virtual_lazy_load(package_address: PackageAddress, blueprint_name: String, ident: u8) -> Self {
        Self {
            info: AdditionalActorInfo::VirtualLazyLoad(package_address, blueprint_name, ident),
        }
    }
}

/// Execution mode
#[derive(Debug, Copy, Clone, Eq, PartialEq, Sbor)]
pub enum ExecutionMode {
    Kernel,
    Resolver,
    DropNode,
    AutoDrop,

    /* System */
    System,

    /* Kernel modules */
    KernelModule,

    /* Clients, e.g. blueprints and node modules */
    Client,
}
