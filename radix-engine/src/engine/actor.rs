use scrypto::core::ScryptoActor;
use scrypto::engine::types::*;

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum REActor {
    Native,
    Scrypto(ScryptoActor),
}

impl REActor {
    pub fn is_substate_readable(&self, substate_id: &SubstateId) -> bool {
        match &self {
            REActor::Native => true,
            REActor::Scrypto(ScryptoActor::Blueprint(..)) => match substate_id {
                SubstateId::KeyValueStoreEntry(..) => true,
                SubstateId::ComponentInfo(..) => true,
                _ => false,
            },
            REActor::Scrypto(ScryptoActor::Component(component_address, ..)) => match substate_id {
                SubstateId::KeyValueStoreEntry(..) => true,
                SubstateId::ComponentInfo(..) => true,
                SubstateId::ComponentState(addr) => addr.eq(component_address),
                _ => false,
            },
        }
    }

    pub fn is_substate_writeable(&self, substate_id: &SubstateId) -> bool {
        match &self {
            REActor::Native => true,
            REActor::Scrypto(ScryptoActor::Blueprint(..)) => match substate_id {
                SubstateId::KeyValueStoreEntry(..) => true,
                _ => false,
            },
            REActor::Scrypto(ScryptoActor::Component(component_address, ..)) => match substate_id {
                SubstateId::KeyValueStoreEntry(..) => true,
                SubstateId::ComponentState(addr) => addr.eq(component_address),
                _ => false,
            },
        }
    }
}
