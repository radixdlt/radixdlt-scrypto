use crate::address::{AddressBech32EncodeError, AddressDisplayContext};
use crate::internal_prelude::*;
use crate::types::*;
use crate::*;
#[cfg(feature = "fuzzing")]
use arbitrary::Arbitrary;

//=========================================================================
// Please update REP-60 after updating types/configs defined in this file!
//=========================================================================

pub const BOOT_LOADER_RESERVED_NODE_ID_FIRST_BYTE: u8 = 0u8;

/// The unique identifier of a (stored) node.
#[cfg_attr(
    feature = "fuzzing",
    derive(Arbitrary, serde::Serialize, serde::Deserialize)
)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Sbor)]
#[sbor(transparent)]
pub struct NodeId(pub [u8; Self::LENGTH]);

impl NodeId {
    pub const ENTITY_ID_LENGTH: usize = 1;
    pub const RID_LENGTH: usize = 29;
    pub const LENGTH: usize = Self::ENTITY_ID_LENGTH + Self::RID_LENGTH;

    pub fn new(entity_byte: u8, random_bytes: &[u8; Self::RID_LENGTH]) -> Self {
        let mut buf = [0u8; Self::LENGTH];
        buf[0] = entity_byte;
        buf[1..random_bytes.len() + 1].copy_from_slice(random_bytes);
        Self(buf)
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn to_hex(&self) -> String {
        hex::encode(&self.0)
    }

    pub fn try_from_hex(hex: &str) -> Option<Self> {
        let bytes = hex::decode(hex).ok()?;
        Some(Self(bytes.try_into().ok()?))
    }

    // TODO: gradually remove dependency on the following entity-type related methods

    pub const fn entity_type(&self) -> Option<EntityType> {
        EntityType::from_repr(self.0[0])
    }

    /// `Global` means root nodes in the store
    pub const fn is_global(&self) -> bool {
        matches!(self.entity_type(), Some(t) if t.is_global())
    }

    /// `Internal` means non-global per current implementation.
    /// It includes both non-root nodes in the store and any nodes in the heap.
    pub const fn is_internal(&self) -> bool {
        matches!(self.entity_type(), Some(t) if t.is_internal())
    }

    pub const fn is_global_component(&self) -> bool {
        matches!(self.entity_type(), Some(t) if t.is_global_component())
    }

    pub const fn is_global_package(&self) -> bool {
        matches!(self.entity_type(), Some(t) if t.is_global_package())
    }

    pub const fn is_global_consensus_manager(&self) -> bool {
        matches!(self.entity_type(), Some(t) if t.is_global_consensus_manager())
    }

    pub const fn is_global_validator(&self) -> bool {
        matches!(self.entity_type(), Some(t) if t.is_global_validator())
    }

    pub const fn is_global_resource_manager(&self) -> bool {
        matches!(self.entity_type(), Some(t) if t.is_global_resource_manager())
    }

    pub const fn is_global_fungible_resource_manager(&self) -> bool {
        matches!(self.entity_type(), Some(t) if t.is_global_fungible_resource_manager())
    }

    pub const fn is_global_non_fungible_resource_manager(&self) -> bool {
        matches!(self.entity_type(), Some(t) if t.is_global_non_fungible_resource_manager())
    }

    pub const fn is_global_virtual(&self) -> bool {
        matches!(self.entity_type(), Some(t) if t.is_global_virtual())
    }

    pub const fn is_internal_kv_store(&self) -> bool {
        matches!(self.entity_type(), Some(t) if t.is_internal_kv_store())
    }

    pub const fn is_internal_fungible_vault(&self) -> bool {
        matches!(self.entity_type(), Some(t) if t.is_internal_fungible_vault())
    }

    pub const fn is_internal_non_fungible_vault(&self) -> bool {
        matches!(self.entity_type(), Some(t) if t.is_internal_non_fungible_vault())
    }

    pub const fn is_internal_vault(&self) -> bool {
        matches!(self.entity_type(), Some(t) if t.is_internal_vault())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl AsRef<NodeId> for NodeId {
    fn as_ref(&self) -> &NodeId {
        self
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

impl From<InternalAddress> for NodeId {
    fn from(value: InternalAddress) -> Self {
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

impl<'a> ContextualDisplay<AddressDisplayContext<'a>> for NodeId {
    type Error = AddressBech32EncodeError;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &AddressDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        if let Some(encoder) = context.encoder {
            let result = encoder.encode_to_fmt(f, self.as_ref());
            match result {
                Ok(_)
                | Err(AddressBech32EncodeError::FormatError(_))
                | Err(AddressBech32EncodeError::Bech32mEncodingError(_))
                | Err(AddressBech32EncodeError::MissingEntityTypeByte) => return result,
                // Only persistable NodeIds are guaranteed to have an address - so
                // fall through to using hex if necessary.
                Err(AddressBech32EncodeError::InvalidEntityTypeId(_)) => {}
            }
        }

        // This could be made more performant by streaming the hex into the formatter
        write!(f, "NodeId({})", hex::encode(&self.0)).map_err(AddressBech32EncodeError::FormatError)
    }
}

/// The unique identifier of a node module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Sbor)]
#[sbor(transparent)]
pub struct PartitionNumber(pub u8);

impl PartitionNumber {
    pub const fn at_offset(self, offset: PartitionOffset) -> Option<Self> {
        match self.0.checked_add(offset.0) {
            Some(n) => Some(Self(n)),
            None => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Sbor)]
pub struct PartitionOffset(pub u8);

/// The unique identifier of a substate within a node module.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Sbor)]
pub enum SubstateKey {
    Field(FieldKey),
    Map(MapKey),
    Sorted(SortedKey),
}

// Keep SubstateKey like the Babylon format to avoid execution traces changing
impl Debug for SubstateKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Field(x) => f.debug_tuple("Field").field(x).finish(),
            Self::Map(x) => f.debug_tuple("Map").field(&x.as_slice()).finish(),
            Self::Sorted(x) => {
                let content = (x.0, x.1.as_slice());
                f.debug_tuple("Sorted").field(&content).finish()
            }
        }
    }
}

impl SubstateKey {
    pub fn for_field(&self) -> Option<&FieldKey> {
        match self {
            SubstateKey::Field(key) => Some(key),
            _ => None,
        }
    }

    pub fn for_map(&self) -> Option<&MapKey> {
        match self {
            SubstateKey::Map(key) => Some(key),
            _ => None,
        }
    }

    pub fn into_map(self) -> MapKey {
        match self {
            SubstateKey::Map(key) => key,
            _ => panic!("Not a Map Key"),
        }
    }

    pub fn for_sorted(&self) -> Option<&SortedKey> {
        match self {
            SubstateKey::Sorted(key) => Some(key),
            _ => None,
        }
    }
}

pub type FieldKey = u8;
pub type MapKey = ScryptoOwnedRawPayload;
pub type SortedKey = ([u8; 2], ScryptoOwnedRawPayload);

pub type UnvalidatedMapKey<'a> = ScryptoUnvalidatedRawPayload<'a>;
pub type UnvalidatedSortedKey<'a> = ([u8; 2], ScryptoUnvalidatedRawPayload<'a>);
