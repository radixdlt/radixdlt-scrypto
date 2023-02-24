use crate::types::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ResolvedActor {
    pub identifier: FnIdentifier,
    pub receiver: Option<MethodReceiver>,
}

impl ResolvedActor {
    pub fn method<I: Into<FnIdentifier>>(identifier: I, receiver: MethodReceiver) -> Self {
        Self {
            identifier: identifier.into(),
            receiver: Some(receiver),
        }
    }

    pub fn function<I: Into<FnIdentifier>>(identifier: I) -> Self {
        Self {
            identifier: identifier.into(),
            receiver: None,
        }
    }
}

/// Execution mode
#[derive(Debug, Copy, Clone, Eq, PartialEq, Sbor)]
pub enum ExecutionMode {
    Kernel,
    Resolver,
    DropNode,

    /* Kernel modules */
    KernelModule,

    /* Clients, e.g. blueprints and node modules */
    Client,
}
