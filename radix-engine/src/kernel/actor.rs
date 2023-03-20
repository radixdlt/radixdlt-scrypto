use crate::types::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ActorIdentifier {
    Method(MethodIdentifier),
    Function(FnIdentifier),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct Actor {
    pub fn_identifier: FnIdentifier,
    pub identifier: ActorIdentifier,
}

impl Actor {
    pub fn method<I: Into<FnIdentifier>>(identifier: I, method: MethodIdentifier) -> Self {
        Self {
            fn_identifier: identifier.into(),
            identifier: ActorIdentifier::Method(method),
        }
    }

    pub fn function<I: Into<FnIdentifier>>(identifier: I) -> Self {
        let fn_identifier = identifier.into();
        Self {
            fn_identifier: fn_identifier.clone(),
            identifier: ActorIdentifier::Function(fn_identifier),
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
