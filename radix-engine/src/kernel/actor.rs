use crate::types::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum AdditionalActorInfo {
    Method(Option<Address>, RENodeId, NodeModuleId),
    Function,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct Actor {
    pub fn_identifier: FnIdentifier,
    pub info: AdditionalActorInfo,
}

impl Actor {
    pub fn ident(&self) -> &str {
        self.fn_identifier.ident.as_str()
    }

    pub fn method<I: Into<FnIdentifier>>(
        global_address: Option<Address>,
        identifier: I,
        method: MethodIdentifier,
    ) -> Self {
        Self {
            fn_identifier: identifier.into(),
            info: AdditionalActorInfo::Method(global_address, method.0, method.1),
        }
    }

    pub fn function<I: Into<FnIdentifier>>(identifier: I) -> Self {
        let fn_identifier = identifier.into();
        Self {
            fn_identifier,
            info: AdditionalActorInfo::Function,
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
