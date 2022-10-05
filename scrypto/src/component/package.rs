use sbor::rust::fmt;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::abi::*;
use crate::address::{AddressError, Bech32Encoder, EntityType};
use crate::core::*;
use crate::misc::*;

#[derive(Debug, TypeId, Encode, Decode)]
pub struct PackagePublishInput {
    pub code: Blob,
    pub abi: Blob,
}

/// A collection of blueprints, compiled and published as a single unit.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PackageAddress {
    Normal([u8; 26]),
}

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
            27 => match EntityType::try_from(slice[0])
                .map_err(|_| AddressError::InvalidEntityTypeId(slice[0]))?
            {
                EntityType::Package => Ok(Self::Normal(copy_u8_array(&slice[1..]))),
                _ => Err(AddressError::InvalidEntityTypeId(slice[0])),
            },
            _ => Err(AddressError::InvalidLength(slice.len())),
        }
    }
}

impl PackageAddress {
    pub fn to_vec(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(EntityType::package(self).id());
        match self {
            Self::Normal(v) => buf.extend(v),
        }
        buf
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.to_vec())
    }

    pub fn try_from_hex(hex_str: &str) -> Result<Self, AddressError> {
        let bytes = hex::decode(hex_str).map_err(|_| AddressError::HexDecodingError)?;

        Self::try_from(bytes.as_ref())
    }

    pub fn displayable<'a, T: Into<Option<&'a Bech32Encoder>>>(
        &'a self,
        bech32_encoder: T,
    ) -> DisplayablePackageAddress<'a> {
        DisplayablePackageAddress(self, bech32_encoder.into())
    }
}

scrypto_type!(PackageAddress, ScryptoType::PackageAddress, Vec::new());

//======
// text
//======

impl fmt::Debug for PackageAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.displayable(None))
    }
}

pub struct DisplayablePackageAddress<'a>(&'a PackageAddress, Option<&'a Bech32Encoder>);

impl<'a> fmt::Display for DisplayablePackageAddress<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        if let Some(bech32_encoder) = self.1 {
            return write!(f, "{}", bech32_encoder.encode_package_address(self.0));
        }
        match self.0 {
            PackageAddress::Normal(_) => {
                write!(f, "NormalPackage[{}]", self.0.to_hex())
            }
        }
    }
}
