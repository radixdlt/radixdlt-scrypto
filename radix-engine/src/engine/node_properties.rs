use crate::types::*;

pub struct SubstateProperties;

impl SubstateProperties {
    pub fn can_own_nodes(offset: &SubstateOffset) -> bool {
        match offset {
            SubstateOffset::Global(..) => true,
            SubstateOffset::AuthZone(..) => false,
            SubstateOffset::Component(ComponentOffset::State) => true,
            SubstateOffset::Component(ComponentOffset::Info) => false,
            SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager) => true,
            SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)) => true,
            SubstateOffset::KeyValueStore(KeyValueStoreOffset::Space) => false,
            SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(..)) => false,
            SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Space) => false,
            SubstateOffset::Vault(..) => false,
            SubstateOffset::Package(..) => false,
            SubstateOffset::System(..) => false,
            SubstateOffset::Bucket(..) => false,
            SubstateOffset::Proof(..) => false,
            SubstateOffset::Worktop(..) => false, // TODO: Fix
        }
    }
}
