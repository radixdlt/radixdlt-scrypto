use crate::data::manifest::ManifestCustomValueKind;
use crate::data::scrypto::*;
use crate::types::NodeId;
use crate::*;
use sbor::rust::fmt;
use sbor::rust::vec::Vec;
use sbor::*;
use utils::copy_u8_array;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Reference(pub NodeId);

impl Reference {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn as_node_id(&self) -> &NodeId {
        &self.0
    }
}

impl TryFrom<&[u8]> for Reference {
    type Error = ParseReferenceError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            NodeId::LENGTH => Ok(Self(NodeId(copy_u8_array(slice)))),
            _ => Err(ParseReferenceError::InvalidLength(slice.len())),
        }
    }
}

//========
// error
//========

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseReferenceError {
    InvalidLength(usize),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseReferenceError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseReferenceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

well_known_scrypto_custom_type!(
    Reference,
    ScryptoCustomValueKind::Reference,
    Type::Reference,
    NodeId::LENGTH,
    REFERENCE_ID
);

//==================
// binary (manifest)
//==================

manifest_type!(Reference, ManifestCustomValueKind::Address, NodeId::LENGTH);

//======
// text
//======

impl fmt::Debug for Reference {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Reference({})", hex::encode(&self.0))
    }
}
