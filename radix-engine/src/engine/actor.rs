use crate::types::*;

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct REActor {
    pub fn_identifier: FnIdentifier,
    pub receiver: Option<Receiver>,
}

impl REActor {
    pub fn is_substate_readable(&self, substate_id: &SubstateId) -> bool {
        match &self.fn_identifier {
            FnIdentifier::Native(..) => true,
            FnIdentifier::Scrypto { .. } => match self.receiver {
                None => match substate_id {
                    SubstateId::KeyValueStoreEntry(..) => true,
                    SubstateId::ComponentInfo(..) => true,
                    _ => false,
                },
                Some(Receiver::Ref(RENodeId::Component(ref component_address))) => {
                    match substate_id {
                        SubstateId::KeyValueStoreEntry(..) => true,
                        SubstateId::ComponentInfo(..) => true,
                        SubstateId::ComponentState(addr) => addr.eq(component_address),
                        _ => false,
                    }
                }
                _ => false,
            },
        }
    }

    pub fn is_substate_writeable(&self, substate_id: &SubstateId) -> bool {
        match &self.fn_identifier {
            FnIdentifier::Native(..) => true,
            FnIdentifier::Scrypto { .. } => match self.receiver {
                None => match substate_id {
                    SubstateId::KeyValueStoreEntry(..) => true,
                    _ => false,
                },
                Some(Receiver::Ref(RENodeId::Component(ref component_address))) => {
                    match substate_id {
                        SubstateId::KeyValueStoreEntry(..) => true,
                        SubstateId::ComponentState(addr) => addr.eq(component_address),
                        _ => false,
                    }
                }
                _ => false,
            },
        }
    }
}
