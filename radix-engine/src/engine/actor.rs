use crate::types::*;
use radix_engine_interface::api::types::{
    NativeFunction, NativeMethod, RENodeId, TransactionProcessorFunction,
};

/// Resolved receiver including info whether receiver was derefed
/// or not
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
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

#[derive(Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub enum ResolvedFunction {
    Scrypto(ScryptoFnIdent),
    Native(NativeFunction),
}

#[derive(Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub enum ResolvedMethod {
    Scrypto(ScryptoFnIdent),
    Native(NativeMethod),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub enum ResolvedActor {
    Function(ResolvedFunction),
    Method(ResolvedMethod, ResolvedReceiver),
}

impl ResolvedActor {
    pub fn is_scrypto_or_transaction(&self) -> bool {
        matches!(
            self,
            ResolvedActor::Method(ResolvedMethod::Scrypto { .. }, ..)
                | ResolvedActor::Function(ResolvedFunction::Scrypto { .. })
                | ResolvedActor::Function(ResolvedFunction::Native(
                    NativeFunction::TransactionProcessor(TransactionProcessorFunction::Run)
                ))
        )
    }
}

impl fmt::Debug for ResolvedFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Scrypto(ScryptoFnIdent {
                package_address,
                blueprint_name,
                ident,
            }) => f
                .debug_struct("Scrypto")
                .field("package_address", package_address)
                .field("blueprint_name", blueprint_name)
                .field("ident", ident)
                .finish(),
            Self::Native(arg0) => f.debug_tuple("Native").field(arg0).finish(),
        }
    }
}

impl fmt::Debug for ResolvedMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Scrypto(ScryptoFnIdent {
                package_address,
                blueprint_name,
                ident,
            }) => f
                .debug_struct("Scrypto")
                .field("package_address", package_address)
                .field("blueprint_name", blueprint_name)
                .field("ident", ident)
                .finish(),
            Self::Native(arg0) => f.debug_tuple("Native").field(arg0).finish(),
        }
    }
}

/// Execution mode
#[derive(Debug, Copy, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub enum ExecutionMode {
    Kernel,
    Globalize,
    MoveUpstream,
    Deref,
    ScryptoInterpreter,
    NodeMoveModule,
    AuthModule,
    EntityModule,
    TransactionModule,
    Application,
    DropNode,
}
