use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::core::*;
use crate::misc::*;
use crate::rust::fmt;
use crate::rust::str::FromStr;
use crate::rust::vec::Vec;
use crate::types::*;

/// A collection of blueprints, compiled and published as a single unit.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PackageId(pub [u8; 26]);

impl PackageId {
    /// Invokes a function on this package.
    pub fn call<T: Decode, S: AsRef<str>>(
        &self,
        blueprint_name: S,
        function: S,
        args: Vec<Vec<u8>>,
    ) -> T {
        let output = Context::call_function(*self, blueprint_name, function, args);

        scrypto_decode(&output).unwrap()
    }
}

//========
// error
//========

#[derive(Debug, Clone)]
pub enum ParsePackageIdError {
    InvalidHex(hex::FromHexError),
    InvalidLength(usize),
    InvalidPrefix,
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParsePackageIdError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParsePackageIdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for PackageId {
    type Error = ParsePackageIdError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            26 => Ok(Self(copy_u8_array(slice))),
            _ => Err(ParsePackageIdError::InvalidLength(slice.len())),
        }
    }
}

impl PackageId {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

custom_type!(PackageId, CustomType::PackageId, Vec::new());

//======
// text
//======

// Before Bech32, we use a fixed prefix for text representation.

impl FromStr for PackageId {
    type Err = ParsePackageIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(ParsePackageIdError::InvalidHex)?;
        if bytes.get(0) != Some(&1u8) {
            return Err(ParsePackageIdError::InvalidPrefix);
        }
        Self::try_from(&bytes[1..])
    }
}

impl fmt::Display for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(combine(1, &self.0)))
    }
}

impl fmt::Debug for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}
