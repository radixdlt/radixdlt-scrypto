use crate::types::*;

/// Resolved receiver including info whether receiver was derefed
/// or not
#[derive(Debug, Copy, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResolvedReceiver {
    pub receiver: MethodReceiver,
    // TODO: Add receiver type
}

impl ResolvedReceiver {
    pub fn new(receiver: MethodReceiver) -> Self {
        Self {
            receiver,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ResolvedActor {
    pub identifier: FnIdentifier,
    pub receiver: Option<ResolvedReceiver>,
}

impl ResolvedActor {
    pub fn method<I: Into<FnIdentifier>>(identifier: I, receiver: ResolvedReceiver) -> Self {
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
