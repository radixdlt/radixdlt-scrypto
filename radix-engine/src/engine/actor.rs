use crate::types::*;

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub enum FullyQualifiedMethod {
    Scrypto {
        package_address: PackageAddress,
        blueprint_name: String,
        ident: String,
    },
    Native(NativeMethod),
}

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub struct FullyQualifiedReceiverMethod {
    pub receiver: Receiver,
    pub method: FullyQualifiedMethod,
}

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub enum REActor {
    Function(FunctionIdent),
    Method(FullyQualifiedReceiverMethod),
}

impl REActor {
    pub fn is_substate_readable(&self, substate_id: &SubstateId) -> bool {
        match self {
            REActor::Function(FunctionIdent::Native(..))
            | REActor::Method(FullyQualifiedReceiverMethod {
                method: FullyQualifiedMethod::Native(..),
                ..
            }) => true,
            REActor::Function(FunctionIdent::Scrypto { .. }) => match substate_id {
                SubstateId::KeyValueStoreEntry(..) => true,
                SubstateId::ComponentInfo(..) => true,
                _ => false,
            },
            REActor::Method(FullyQualifiedReceiverMethod {
                receiver: Receiver::Ref(RENodeId::Component(component_address)),
                method: FullyQualifiedMethod::Scrypto { .. },
            }) => match substate_id {
                SubstateId::KeyValueStoreEntry(..) => true,
                SubstateId::ComponentInfo(..) => true,
                SubstateId::ComponentState(addr) => addr.eq(component_address),
                _ => false,
            },
            _ => false,
        }
    }

    pub fn is_substate_writeable(&self, substate_id: &SubstateId) -> bool {
        match self {
            REActor::Function(FunctionIdent::Native(..))
            | REActor::Method(FullyQualifiedReceiverMethod {
                method: FullyQualifiedMethod::Native(..),
                ..
            }) => true,
            REActor::Function(FunctionIdent::Scrypto { .. }) => match substate_id {
                SubstateId::KeyValueStoreEntry(..) => true,
                _ => false,
            },
            REActor::Method(FullyQualifiedReceiverMethod {
                receiver: Receiver::Ref(RENodeId::Component(component_address)),
                method: FullyQualifiedMethod::Scrypto { .. },
            }) => match substate_id {
                SubstateId::KeyValueStoreEntry(..) => true,
                SubstateId::ComponentState(addr) => addr.eq(component_address),
                _ => false,
            },
            _ => false,
        }
    }
}
