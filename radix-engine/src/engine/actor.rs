use crate::types::*;
use scrypto::core::NativeFunction;

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub enum ResolvedMethod {
    Scrypto {
        package_address: PackageAddress,
        blueprint_name: String,
        ident: String,
        export_name: String,
    },
    Native(NativeMethod),
}

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub struct ResolvedReceiver {
    pub derefed_from: Option<RENodeId>,
    pub receiver: Receiver,
}

impl ResolvedReceiver {
    pub fn derefed(receiver: Receiver, from: RENodeId) -> Self {
        Self {
            receiver,
            derefed_from: Some(from),
        }
    }

    pub fn new(receiver: Receiver) -> Self {
        Self {
            receiver,
            derefed_from: None,
        }
    }

    pub fn receiver(&self) -> Receiver {
        self.receiver
    }

    pub fn node_id(&self) -> RENodeId {
        self.receiver.node_id()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub struct ResolvedReceiverMethod {
    pub receiver: ResolvedReceiver,
    pub method: ResolvedMethod,
}

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub enum ResolvedFunction {
    Scrypto {
        package_address: PackageAddress,
        blueprint_name: String,
        ident: String,
        export_name: String,
    },
    Native(NativeFunction),
}

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub enum REActor {
    Function(ResolvedFunction),
    Method(ResolvedReceiverMethod),
}

impl REActor {
    pub fn is_scrypto_or_transaction(&self) -> bool {
        matches!(
            self,
            REActor::Method(ResolvedReceiverMethod {
                method: ResolvedMethod::Scrypto { .. },
                ..
            }) | REActor::Function(ResolvedFunction::Scrypto { .. })
                | REActor::Function(ResolvedFunction::Native(
                    NativeFunction::TransactionProcessor(TransactionProcessorFunction::Run)
                ))
        )
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub enum KernelActor {
    Application,
    Deref,
    ScryptoLoader,
    AuthModule,
}
