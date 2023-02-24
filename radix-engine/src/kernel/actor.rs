use crate::types::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ResolvedActor {
    pub fn_identifier: FnIdentifier,
    pub method: Option<MethodIdentifier>,
}

impl ResolvedActor {
    pub fn method<I: Into<FnIdentifier>>(identifier: I, method: MethodIdentifier) -> Self {
        Self {
            fn_identifier: identifier.into(),
            method: Some(method),
        }
    }

    pub fn function<I: Into<FnIdentifier>>(identifier: I) -> Self {
        Self {
            fn_identifier: identifier.into(),
            method: None,
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

    /* Kernel modules */
    KernelModule,

    /* Clients, e.g. blueprints and node modules */
    Client,
}
