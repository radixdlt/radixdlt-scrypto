use sbor::*;

use crate::buffer::*;
use crate::core::*;
use crate::misc::*;
use crate::rust::borrow::ToOwned;
use crate::rust::fmt;
use crate::rust::str::FromStr;
use crate::rust::string::String;
use crate::rust::vec::Vec;
use crate::types::*;

/// A collection of blueprints, compiled and published as a single unit.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PackageAddress(pub [u8; 26]);

impl PackageAddress {}

#[derive(Debug)]
pub struct Package(pub(crate) PackageAddress);

impl Package {
    /// Invokes a function on this package.
    pub fn call<T: Decode, S: AsRef<str>>(
        &self,
        blueprint_name: S,
        function: S,
        args: Vec<Vec<u8>>,
    ) -> T {
        let output = Runtime::call_function(self.0, blueprint_name, function, args);

        scrypto_decode(&output).unwrap()
    }
}

//========
// error
//========

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsePackageAddressError {
    InvalidHex(String),
    InvalidLength(usize),
    InvalidPrefix,
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParsePackageAddressError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParsePackageAddressError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for PackageAddress {
    type Error = ParsePackageAddressError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            26 => Ok(Self(copy_u8_array(slice))),
            _ => Err(ParsePackageAddressError::InvalidLength(slice.len())),
        }
    }
}

impl PackageAddress {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

custom_type!(PackageAddress, CustomType::PackageAddress, Vec::new());

//======
// text
//======

// Before Bech32, we use a fixed prefix for text representation.

impl FromStr for PackageAddress {
    type Err = ParsePackageAddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes =
            hex::decode(s).map_err(|_| ParsePackageAddressError::InvalidHex(s.to_owned()))?;
        if bytes.get(0) != Some(&1u8) {
            return Err(ParsePackageAddressError::InvalidPrefix);
        }
        Self::try_from(&bytes[1..])
    }
}

impl fmt::Display for PackageAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(combine(1, &self.0)))
    }
}

impl fmt::Debug for PackageAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}
