use sbor::rust::convert::TryFrom;
#[cfg(not(feature = "alloc"))]
use sbor::rust::fmt;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::data::manifest::*;
use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

manifest_type!(ManifestProof, ManifestCustomValueKind::Proof, 4);
