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
    fn_identifier: FnIdentifier,
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
        &self.fn_identifier.package_address
    }

    pub fn blueprint_name(&self) -> &str {
        self.fn_identifier.blueprint_name.as_str()
    }

    pub fn method<I: Into<FnIdentifier>>(
        global_address: Option<Address>,
        identifier: I,
        method: MethodIdentifier,
    ) -> Self {
        let fn_identifier = identifier.into();
        let package_address = fn_identifier.package_address;
        let blueprint_name = fn_identifier.blueprint_name.clone();
        Self {
            fn_identifier,
            info: AdditionalActorInfo::Method(global_address, method.0, method.1, package_address, blueprint_name, method.2),
        }
    }

    pub fn function<I: Into<FnIdentifier>>(identifier: I, ident: FunctionIdentifier) -> Self {
        let fn_identifier = identifier.into();
        Self {
            fn_identifier,
            info: AdditionalActorInfo::Function(ident.0, ident.1, ident.2),
        }
    }

    pub fn virtual_lazy_load(package_address: PackageAddress, blueprint_name: String, ident: u8) -> Self {
        Self {
            fn_identifier: FnIdentifier {
                package_address: package_address.clone(),
                blueprint_name: blueprint_name.clone(),
                ident: FnIdent::System(ident),
            },
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
