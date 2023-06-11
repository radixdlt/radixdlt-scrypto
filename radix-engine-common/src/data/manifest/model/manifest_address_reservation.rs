#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;
use sbor::rust::convert::TryFrom;
#[cfg(not(feature = "alloc"))]
use sbor::rust::fmt;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::data::manifest::*;
use crate::*;

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ManifestAddressReservation(pub u32);

//========
// error
//========

/// Represents an error when parsing ManifestAddressReservation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseManifestAddressReservationError {
    InvalidLength,
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseManifestAddressReservationError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseManifestAddressReservationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for ManifestAddressReservation {
    type Error = ParseManifestAddressReservationError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != 4 {
            return Err(Self::Error::InvalidLength);
        }
        Ok(Self(u32::from_le_bytes(slice.try_into().unwrap())))
    }
}

impl ManifestAddressReservation {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }
}

manifest_type!(
    ManifestAddressReservation,
    ManifestCustomValueKind::AddressReservation,
    4
);
