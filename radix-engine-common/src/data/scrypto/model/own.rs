use crate::data::scrypto::ScryptoCustomValueKind;
use crate::types::NodeId;
use crate::*;
#[cfg(not(feature = "alloc"))]
use sbor::rust::fmt;
use sbor::rust::prelude::*;
use sbor::*;
use utils::copy_u8_array;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Own(pub NodeId);

impl Own {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn as_node_id(&self) -> &NodeId {
        &self.0
    }
}

impl TryFrom<&[u8]> for Own {
    type Error = ParseOwnError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            NodeId::LENGTH => Ok(Self(NodeId(copy_u8_array(slice)))),
            _ => Err(ParseOwnError::InvalidLength(slice.len())),
        }
    }
}

//========
// error
//========

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseOwnError {
    InvalidLength(usize),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseOwnError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseOwnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

well_known_scrypto_custom_type!(
    Own,
    ScryptoCustomValueKind::Own,
    Type::Own,
    NodeId::LENGTH,
    OWN_ID
);

//======
// text
//======

impl fmt::Debug for Own {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Own({})", hex::encode(&self.0))
    }
}
