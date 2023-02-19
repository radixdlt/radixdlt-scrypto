use sbor::rust::convert::TryFrom;
#[cfg(not(feature = "alloc"))]
use sbor::rust::fmt;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::data::*;
use crate::schemaless_scrypto_custom_type;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ManifestProof(pub u32);

//========
// error
//========

/// Represents an error when parsing ManifestProof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseManifestProofError {
    InvalidLength,
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseManifestProofError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseManifestProofError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for ManifestProof {
    type Error = ParseManifestProofError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != 4 {
            return Err(Self::Error::InvalidLength);
        }
        Ok(Self(u32::from_le_bytes(slice.try_into().unwrap())))
    }
}

impl ManifestProof {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }
}

schemaless_scrypto_custom_type!(ManifestProof, ScryptoCustomValueKind::Proof, 4);

// Temporary until ManifestProof is no longer in the ScryptoValue model
impl<C: CustomTypeKind<GlobalTypeId>> Describe<C> for ManifestProof {
    const TYPE_ID: GlobalTypeId = GlobalTypeId::well_known(basic_well_known_types::ANY_ID);
}
