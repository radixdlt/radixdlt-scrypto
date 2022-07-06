use sbor::rust::collections::HashMap;
use sbor::rust::fmt;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto_abi::BlueprintAbi;

use crate::abi::*;
use crate::address::Bech32Addressable;
use crate::address::EntityType;
use crate::address::ParseAddressError;
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
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PackageAddress(pub [u8; 27]);

impl PackageAddress {}

/// Represents a published package.
#[derive(Debug)]
pub struct BorrowedPackage(pub(crate) PackageAddress);

impl BorrowedPackage {
    /// Invokes a function on this package.
    pub fn call<T: Decode>(&self, blueprint_name: &str, function: &str, args: Vec<Vec<u8>>) -> T {
        Runtime::call_function(self.0, blueprint_name, function, args)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for PackageAddress {
    type Error = ParseAddressError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            27 => Ok(Self(copy_u8_array(slice))),
            _ => Err(ParseAddressError::InvalidLength(slice.len())),
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

impl Bech32Addressable for PackageAddress {
    fn data(&self) -> &[u8] {
        &self.0
    }

    fn allowed_entity_types() -> &'static [EntityType] {
        &[EntityType::Package]
    }
}

impl FromStr for PackageAddress {
    type Err = ParseAddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_bech32_string(s, &CURRENT_NETWORK)
    }
}

impl fmt::Display for PackageAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.to_bech32_string(&CURRENT_NETWORK).unwrap())
    }
}

impl fmt::Debug for PackageAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}
