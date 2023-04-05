use crate::data::scrypto::model::*;
use crate::types::*;
use crate::*;
use sbor::rust::prelude::*;

//=========================================================================
// Please update REP-60 after updating types/configs defined in this file!
//=========================================================================

/// The unique identifier of a (stored) node.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Sbor)]
#[sbor(transparent)]
pub struct NodeId(pub [u8; Self::LENGTH]);

impl NodeId {
    pub const LENGTH: usize = 27;

    pub fn new(entity_byte: u8, random_bytes: &[u8; Self::LENGTH - 1]) -> Self {
        let mut buf = [0u8; Self::LENGTH];
        buf[0] = entity_byte;
        buf[1..random_bytes.len() + 1].copy_from_slice(random_bytes);
        Self(buf)
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn to_hex(&self) -> String {
        hex::encode(&self.0)
    }

    // TODO: gradually remove dependency on the following entity-type related methods

    pub fn entity_type(&self) -> Option<EntityType> {
        EntityType::from_repr(self.0[0])
    }

    pub fn is_global_fungible_resource(&self) -> bool {
        match self.entity_type() {
            Some(t) => matches!(t, EntityType::GlobalFungibleResource),
            None => false,
        }
    }

    pub fn is_internal_fungible_vault(&self) -> bool {
        match self.entity_type() {
            Some(t) => matches!(t, EntityType::InternalFungibleVault),
            None => false,
        }
    }

    pub fn is_global(&self) -> bool {
        match self.entity_type() {
            Some(t) => t.is_global(),
            None => false,
        }
    }

    pub fn is_global_component(&self) -> bool {
        match self.entity_type() {
            Some(t) => t.is_global_component(),
            None => false,
        }
    }

    pub fn is_global_resource(&self) -> bool {
        match self.entity_type() {
            Some(t) => t.is_global_resource(),
            None => false,
        }
    }

    pub fn is_global_package(&self) -> bool {
        match self.entity_type() {
            Some(t) => t.is_global_package(),
            None => false,
        }
    }

    pub fn is_global_virtual(&self) -> bool {
        match self.entity_type() {
            Some(t) => t.is_global_virtual(),
            None => false,
        }
    }
    pub fn is_local(&self) -> bool {
        match self.entity_type() {
            Some(t) => t.is_local(),
            None => false,
        }
    }

    pub fn is_internal_kv_store(&self) -> bool {
        match self.entity_type() {
            Some(t) => t.is_internal_kv_store(),
            None => false,
        }
    }

    pub fn is_internal_vault(&self) -> bool {
        match self.entity_type() {
            Some(t) => t.is_internal_vault(),
            None => false,
        }
    }
}

impl AsRef<[u8]> for NodeId {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Into<[u8; NodeId::LENGTH]> for NodeId {
    fn into(self) -> [u8; NodeId::LENGTH] {
        self.0
    }
}

impl From<[u8; NodeId::LENGTH]> for NodeId {
    fn from(value: [u8; NodeId::LENGTH]) -> Self {
        Self(value)
    }
}

impl From<GlobalAddress> for NodeId {
    fn from(value: GlobalAddress) -> Self {
        Self(value.into())
    }
}

impl From<LocalAddress> for NodeId {
    fn from(value: LocalAddress) -> Self {
        Self(value.into())
    }
}

impl From<ComponentAddress> for NodeId {
    fn from(value: ComponentAddress) -> Self {
        Self(value.into())
    }
}

impl From<ResourceAddress> for NodeId {
    fn from(value: ResourceAddress) -> Self {
        Self(value.into())
    }
}

impl From<PackageAddress> for NodeId {
    fn from(value: PackageAddress) -> Self {
        Self(value.into())
    }
}

impl From<Own> for NodeId {
    fn from(value: Own) -> Self {
        Self(value.0.into())
    }
}

impl From<Reference> for NodeId {
    fn from(value: Reference) -> Self {
        Self(value.0.into())
    }
}

impl Debug for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("NodeId")
            .field(&hex::encode(&self.0))
            .finish()
    }
}

/// The unique identifier of a node module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Sbor)]
#[sbor(transparent)]
pub struct ModuleId(pub u8);

/// The unique identifier of a substate within a node module.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Sbor)]
#[sbor(transparent)]
pub struct SubstateKey(Vec<u8>);

impl SubstateKey {
    pub const MIN_LENGTH: usize = 1;
    pub const MAX_LENGTH: usize = 128;

    pub fn min() -> Self {
        Self(vec![u8::MIN; Self::MIN_LENGTH])
    }

    pub fn max() -> Self {
        Self(vec![u8::MAX; Self::MAX_LENGTH])
    }

    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        Self::from_vec(slice.to_vec())
    }

    pub fn from_vec(bytes: Vec<u8>) -> Option<Self> {
        if bytes.len() < Self::MIN_LENGTH || bytes.len() > Self::MAX_LENGTH {
            None
        } else {
            Some(Self(bytes))
        }
    }

    pub fn to_hex(&self) -> String {
        hex::encode(&self.0)
    }
}

impl Debug for SubstateKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SubstateKey")
            .field(&hex::encode(&self.0))
            .finish()
    }
}

impl AsRef<[u8]> for SubstateKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Into<Vec<u8>> for SubstateKey {
    fn into(self) -> Vec<u8> {
        self.0
    }
}
