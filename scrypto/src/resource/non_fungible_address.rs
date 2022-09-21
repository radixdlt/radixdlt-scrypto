use sbor::rust::borrow::ToOwned;
use sbor::rust::fmt;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::abi::*;
use crate::constants::{ECDSA_TOKEN, ED25519_TOKEN};
use crate::crypto::PublicKey;
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

    pub fn from_public_key<P: Into<PublicKey> + Clone>(public_key: &P) -> Self {
        let public_key: PublicKey = public_key.clone().into();
        match public_key {
            PublicKey::EcdsaSecp256k1(public_key) => {
                NonFungibleAddress::new(ECDSA_TOKEN, NonFungibleId::from_bytes(public_key.to_vec()))
            }
            PublicKey::EddsaEd25519(public_key) => NonFungibleAddress::new(
                ED25519_TOKEN,
                NonFungibleId::from_bytes(public_key.to_vec()),
            ),
        }
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

//=======
// error
//=======

/// Represents an error when parsing non-fungible address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseNonFungibleAddressError {
    InvalidLength(usize),
    InvalidResourceDefId,
    InvalidNonFungibleId,
    InvalidHex(String),
    InvalidPrefix,
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseNonFungibleAddressError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseNonFungibleAddressError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

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
        Self::try_from(bytes.as_ref())
    }
}

impl fmt::Display for NonFungibleAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        // Note that if the non-fungible ID is empty, the non-fungible address won't be distinguishable from resource address.
        // TODO: figure out what's best for the users
        write!(f, "{}", hex::encode(&self.to_vec()))
    }
}

impl fmt::Debug for NonFungibleAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}

//======
// test
//======

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sbor::rust::string::ToString;

    #[test]
    pub fn non_fungible_address_from_and_to_string_succeeds() {
        // Arrange
        let resource_address = ResourceAddress::from_str(
            "resource_sim1qzntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrs520k2p",
        )
        .expect("Resource address from str failed.");
        let non_fungible_id = NonFungibleId(
            hex::decode("30071000000071dba5dd36e30de857049805fd1553cd")
                .expect("Invalid NonFungibleId hex"),
        );
        let non_fungible_address = NonFungibleAddress::new(resource_address, non_fungible_id);

        // Act
        let converted_non_fungible_address =
            NonFungibleAddress::from_str(&non_fungible_address.to_string());

        // Assert
        assert_eq!(converted_non_fungible_address, Ok(non_fungible_address));
    }
}
