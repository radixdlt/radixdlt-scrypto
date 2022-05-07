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

#[derive(Debug, TypeId, Encode, Decode)]
pub enum PackageFunction {
    Publish(Package),
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct Package {
    code: Vec<u8>,
}

impl Package {
    pub fn new(code: Vec<u8>) -> Self {
        Package { code }
    }

    pub fn code(&self) -> &[u8] {
        &self.code
    }
}

/// A collection of blueprints, compiled and published as a single unit.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PackageAddress(pub [u8; 26]);

impl PackageAddress {}

/// Represents a published package.
#[derive(Debug)]
pub struct BorrowedPackage(pub(crate) PackageAddress);

impl BorrowedPackage {
    /// Invokes a function on this package.
    pub fn call<T: Decode>(&self, blueprint_name: &str, function: &str, args: Vec<Vec<u8>>) -> T {
        let output = Runtime::call_function(self.0, blueprint_name, function, args);

        scrypto_decode(&output).unwrap()
    }
}

//========
// error
//========

/// Represents an error when decoding package address.
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

scrypto_type!(PackageAddress, ScryptoType::PackageAddress, Vec::new());

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
