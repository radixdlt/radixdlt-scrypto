use crate::types::*;
use radix_engine_interface::api::types::RENodeId;

/// Resolved receiver including info whether receiver was derefed
/// or not
#[derive(Debug, Copy, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ResolvedReceiver {
    pub derefed_from: Option<(RENodeId, LockHandle)>,
    pub receiver: RENodeId,
}

impl ResolvedReceiver {
    pub fn derefed(receiver: RENodeId, from: RENodeId, lock_handle: LockHandle) -> Self {
        Self {
            receiver,
            derefed_from: Some((from, lock_handle)),
        }
    }

    pub fn new(receiver: RENodeId) -> Self {
        Self {
            receiver,
            derefed_from: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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
#[derive(Debug, Copy, Clone, Eq, PartialEq, Categorize, Encode, Decode)]
pub enum ExecutionMode {
    Kernel,
    MoveUpstream,
    Deref,
    Globalize,
    Resolver,
    NodeMoveModule,
    AuthModule,
    LoggerModule,
    EntityModule,
    TransactionModule,
    Application,
    DropNode,
}
