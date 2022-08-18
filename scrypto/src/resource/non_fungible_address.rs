use sbor::rust::borrow::ToOwned;
use sbor::rust::fmt;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::abi::*;
use crate::constants::ECDSA_TOKEN;
use crate::crypto::EcdsaPublicKey;
use crate::misc::*;
use crate::resource::*;

/// Identifier for a non-fungible unit.
#[derive(Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct NonFungibleAddress {
    resource_address: ResourceAddress,
    non_fungible_id: NonFungibleId,
}

impl NonFungibleAddress {
    pub fn new(resource_address: ResourceAddress, non_fungible_id: NonFungibleId) -> Self {
        Self {
            resource_address,
            non_fungible_id,
        }
    }

    pub fn from_public_key(public_key: &EcdsaPublicKey) -> Self {
        NonFungibleAddress::new(ECDSA_TOKEN, NonFungibleId::from_bytes(public_key.to_vec()))
    }

    /// Returns the resource address.
    pub fn resource_address(&self) -> ResourceAddress {
        self.resource_address
    }

    /// Returns the non-fungible id.
    pub fn non_fungible_id(&self) -> NonFungibleId {
        self.non_fungible_id.clone()
    }
}

//========
// binary
//========

/// Represents an error when parsing non-fungible address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseNonFungibleAddressError {
    InvalidLength(usize),
    InvalidResourceDefId,
    InvalidNonFungibleId,
    InvalidHex(String),
    InvalidPrefix,
}

impl TryFrom<&[u8]> for NonFungibleAddress {
    type Error = ParseNonFungibleAddressError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() < 27 {
            return Err(ParseNonFungibleAddressError::InvalidLength(slice.len()));
        }

        let (resource_address_slice, non_fungible_id_slice) = slice.split_at(27);
        let resource_address = ResourceAddress::try_from(resource_address_slice)
            .map_err(|_| ParseNonFungibleAddressError::InvalidResourceDefId)?;
        let non_fungible_id = NonFungibleId::try_from(non_fungible_id_slice)
            .map_err(|_| ParseNonFungibleAddressError::InvalidNonFungibleId)?;
        Ok(NonFungibleAddress {
            resource_address,
            non_fungible_id,
        })
    }
}

impl NonFungibleAddress {
    pub fn to_vec(&self) -> Vec<u8> {
        let mut vec = self.resource_address.to_vec();
        let mut other_vec = self.non_fungible_id.to_vec();
        vec.append(&mut other_vec);
        vec
    }
}

scrypto_type!(
    NonFungibleAddress,
    ScryptoType::NonFungibleAddress,
    Vec::new()
);

//======
// text
//======

impl FromStr for NonFungibleAddress {
    type Err = ParseNonFungibleAddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes =
            hex::decode(s).map_err(|_| ParseNonFungibleAddressError::InvalidHex(s.to_owned()))?;
        if bytes.get(0) != Some(&3u8) {
            return Err(ParseNonFungibleAddressError::InvalidPrefix);
        }
        Self::try_from(&bytes[1..])
    }
}

impl fmt::Display for NonFungibleAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        // Note that if the non-fungible ID is empty, the non-fungible address won't be distinguishable from resource address.
        // TODO: figure out what's best for the users
        write!(f, "{}", hex::encode(combine(3, &self.to_vec())))
    }
}

impl fmt::Debug for NonFungibleAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}
