use sbor::rust::collections::HashMap;
use sbor::rust::fmt;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto_abi::BlueprintAbi;

use crate::abi::*;
use crate::address::{AddressError, BECH32_DECODER, BECH32_ENCODER};
use crate::core::*;
use crate::misc::*;

#[derive(Debug, TypeId, Encode, Decode)]
pub struct PackagePublishInput {
    pub package: Package,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct Package {
    pub code: Vec<u8>,
    pub blueprints: HashMap<String, BlueprintAbi>,
}

/// A collection of blueprints, compiled and published as a single unit.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PackageAddress(pub [u8; 27]);

impl PackageAddress {}

/// Represents a published package.
#[derive(Debug)]
pub struct BorrowedPackage(pub(crate) PackageAddress);

impl BorrowedPackage {
    /// Invokes a function on this package.
    pub fn call<T: Decode>(&self, blueprint_name: &str, function: &str, args: Vec<u8>) -> T {
        Runtime::call_function(self.0, blueprint_name, function, args)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for PackageAddress {
    type Error = AddressError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            27 => Ok(Self(copy_u8_array(slice))),
            _ => Err(AddressError::InvalidLength(slice.len())),
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

impl FromStr for PackageAddress {
    type Err = AddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        BECH32_DECODER.validate_and_decode_package_address(s)
    }
}

impl fmt::Display for PackageAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            f,
            "{}",
            BECH32_ENCODER.encode_package_address(self).unwrap()
        )
    }
}

impl fmt::Debug for PackageAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}
