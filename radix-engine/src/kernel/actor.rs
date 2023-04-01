use crate::types::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum Actor {
    Method {
        global_address: Option<GlobalAddress>,
        node_id: NodeId,
        module_id: TypedModuleId,
        blueprint: Blueprint,
        ident: String,
    },
    Function {
        blueprint: Blueprint,
        ident: String,
    },
    VirtualLazyLoad {
        blueprint: Blueprint,
        ident: u8,
    },
}

impl Actor {
    pub fn blueprint(&self) -> &Blueprint {
        match self {
            Actor::Method { blueprint, .. }
            | Actor::Function { blueprint, .. }
            | Actor::VirtualLazyLoad { blueprint, .. } => blueprint,
        }
    }

    pub fn package_address(&self) -> &PackageAddress {
        let blueprint = match &self {
            Actor::Method { blueprint, .. } => blueprint,
            Actor::Function { blueprint, .. } => blueprint,
            Actor::VirtualLazyLoad { blueprint, .. } => blueprint,
        };

        &blueprint.package_address
    }

    pub fn blueprint_name(&self) -> &str {
        match &self {
            Actor::Method { blueprint, .. }
            | Actor::Function { blueprint, .. }
            | Actor::VirtualLazyLoad { blueprint, .. } => blueprint.blueprint_name.as_str(),
        }
    }

    pub fn method(
        global_address: Option<GlobalAddress>,
        method: MethodIdentifier,
        blueprint: Blueprint,
    ) -> Self {
        Self::Method {
            global_address,
            node_id: method.0,
            module_id: method.1,
            blueprint,
            ident: method.2,
        }
    }

    pub fn function(ident: FunctionIdentifier) -> Self {
        Self::Function {
            blueprint: ident.0,
            ident: ident.1,
        }
    }

    pub fn virtual_lazy_load(blueprint: Blueprint, ident: u8) -> Self {
        Self::VirtualLazyLoad { blueprint, ident }
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
