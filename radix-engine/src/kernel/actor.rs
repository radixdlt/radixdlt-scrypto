use crate::types::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum Actor {
    Method {
        global_address: Option<Address>,
        node_id: RENodeId,
        module_id: NodeModuleId,
        package_address: PackageAddress,
        blueprint_name: String,
        ident: String,
    },
    Function {
        package_address: PackageAddress,
        blueprint_name: String,
        ident: String,
    },
    VirtualLazyLoad {
        package_address: PackageAddress,
        blueprint_name: String,
        ident: u8,
    },
}

impl Actor {
    pub fn blueprint(&self) -> Blueprint {
        match self {
            Actor::Method {
                package_address,
                blueprint_name,
                ..
            } => Blueprint::new(package_address, blueprint_name.as_str()),
            Actor::Function {
                package_address,
                blueprint_name,
                ..
            } => Blueprint::new(package_address, blueprint_name.as_str()),
            Actor::VirtualLazyLoad {
                package_address,
                blueprint_name,
                ..
            } => Blueprint::new(package_address, blueprint_name.as_str()),
        }
    }

    pub fn fn_identifier(&self) -> FnIdentifier {
        match self {
            Actor::Method {
                package_address,
                blueprint_name,
                ident,
                ..
            } => FnIdentifier::application_ident(
                package_address.clone(),
                blueprint_name.clone(),
                ident.clone(),
            ),
            Actor::Function {
                package_address,
                blueprint_name,
                ident,
            } => FnIdentifier::application_ident(
                package_address.clone(),
                blueprint_name.clone(),
                ident.clone(),
            ),
            Actor::VirtualLazyLoad {
                package_address,
                blueprint_name,
                ident,
            } => FnIdentifier::system_ident(
                package_address.clone(),
                blueprint_name.clone(),
                ident.clone(),
            ),
        }
    }

    pub fn package_address(&self) -> &PackageAddress {
        match &self {
            Actor::Method {
                package_address, ..
            } => package_address,
            Actor::Function {
                package_address, ..
            } => package_address,
            Actor::VirtualLazyLoad {
                package_address, ..
            } => package_address,
        }
    }

    pub fn blueprint_name(&self) -> &str {
        match &self {
            Actor::Method { blueprint_name, .. } => blueprint_name.as_str(),
            Actor::Function { blueprint_name, .. } => blueprint_name.as_str(),
            Actor::VirtualLazyLoad { blueprint_name, .. } => blueprint_name.as_str(),
        }
    }

    pub fn method(
        global_address: Option<Address>,
        method: MethodIdentifier,
        package_address: PackageAddress,
        blueprint_name: String,
    ) -> Self {
        Self::Method {
            global_address,
            node_id: method.0,
            module_id: method.1,
            package_address,
            blueprint_name,
            ident: method.2,
        }
    }

    pub fn function(ident: FunctionIdentifier) -> Self {
        Self::Function {
            package_address: ident.0,
            blueprint_name: ident.1,
            ident: ident.2,
        }
    }

    pub fn virtual_lazy_load(
        package_address: PackageAddress,
        blueprint_name: String,
        ident: u8,
    ) -> Self {
        Self::VirtualLazyLoad {
            package_address,
            blueprint_name,
            ident,
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
