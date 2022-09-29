use crate::types::*;

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub enum FullyQualifiedMethodFn {
    Scrypto {
        package_address: PackageAddress,
        blueprint_name: String,
        ident: String,
    },
    Native(NativeMethodFnIdent),
}

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub struct FullyQualifiedMethod {
    pub receiver: Receiver,
    pub fn_ident: FullyQualifiedMethodFn,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum REActor {
    Function(FunctionIdent),
    Method(FullyQualifiedMethod)
}

impl REActor {
    pub fn is_substate_readable(&self, substate_id: &SubstateId) -> bool {
        match self {
            REActor::Function(FunctionIdent::Native(..))
            | REActor::Method(FullyQualifiedMethod {
                fn_ident: FullyQualifiedMethodFn::Native(..),
                ..
            }) => true,
            REActor::Function(FunctionIdent::Scrypto { .. }) => match substate_id {
                SubstateId::KeyValueStoreEntry(..) => true,
                SubstateId::ComponentInfo(..) => true,
                _ => false,
            },
            REActor::Method(FullyQualifiedMethod {
                receiver: Receiver::Ref(RENodeId::Component(component_address)),
                fn_ident: FullyQualifiedMethodFn::Scrypto { .. },
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
            | REActor::Method(FullyQualifiedMethod {
                fn_ident: FullyQualifiedMethodFn::Native(..),
                ..
            }) => true,
            REActor::Function(FunctionIdent::Scrypto { .. }) => match substate_id {
                SubstateId::KeyValueStoreEntry(..) => true,
                _ => false,
            },
            REActor::Method(FullyQualifiedMethod {
                receiver: Receiver::Ref(RENodeId::Component(component_address)),
                fn_ident: FullyQualifiedMethodFn::Scrypto { .. },
            }) => match substate_id {
                SubstateId::KeyValueStoreEntry(..) => true,
                SubstateId::ComponentState(addr) => addr.eq(component_address),
                _ => false,
            },
            _ => false,
        }
    }
}
