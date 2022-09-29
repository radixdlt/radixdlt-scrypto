use crate::types::*;
use scrypto::core::{FnIdent, MethodFnIdent, MethodIdent};

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct REActor {
    pub function_identifier: FnIdent,
}

impl REActor {
    pub fn is_substate_readable(&self, substate_id: &SubstateId) -> bool {
        match &self.function_identifier {
            FnIdent::Function(FunctionIdent::Native(..))
            | FnIdent::Method(MethodIdent {
                fn_ident: MethodFnIdent::Native(..),
                ..
            }) => true,
            FnIdent::Function(FunctionIdent::Scrypto { .. }) => match substate_id {
                SubstateId::KeyValueStoreEntry(..) => true,
                SubstateId::ComponentInfo(..) => true,
                _ => false,
            },
            FnIdent::Method(MethodIdent {
                receiver: Receiver::Ref(RENodeId::Component(component_address)),
                fn_ident: MethodFnIdent::Scrypto { .. },
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
        match &self.function_identifier {
            FnIdent::Function(FunctionIdent::Native(..))
            | FnIdent::Method(MethodIdent {
                fn_ident: MethodFnIdent::Native(..),
                ..
            }) => true,
            FnIdent::Function(FunctionIdent::Scrypto { .. }) => match substate_id {
                SubstateId::KeyValueStoreEntry(..) => true,
                _ => false,
            },
            FnIdent::Method(MethodIdent {
                receiver: Receiver::Ref(RENodeId::Component(component_address)),
                fn_ident: MethodFnIdent::Scrypto { .. },
            }) => match substate_id {
                SubstateId::KeyValueStoreEntry(..) => true,
                SubstateId::ComponentState(addr) => addr.eq(component_address),
                _ => false,
            },
            _ => false,
        }
    }
}
