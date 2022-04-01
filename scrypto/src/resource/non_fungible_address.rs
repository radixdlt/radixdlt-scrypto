use sbor::*;

use crate::resource::*;
use crate::rust::fmt;
use crate::rust::str::FromStr;
use crate::types::*;

use crate::rust::vec::Vec;

/// Identifier for a non-fungible unit.
#[derive(Clone, PartialEq, Eq, Hash)]
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

    /// Returns the resource definition.
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseNonFungibleAddressError {
    Invalid,
}

impl TryFrom<&[u8]> for NonFungibleAddress {
    type Error = ParseNonFungibleAddressError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() < 26 {
            return Err(ParseNonFungibleAddressError::Invalid);
        }

        let (resource_address_slice, non_fungible_id_slice) = slice.split_at(26);
        let resource_address = ResourceAddress::try_from(resource_address_slice)
            .map_err(|_| ParseNonFungibleAddressError::Invalid)?;
        let non_fungible_id = NonFungibleId::try_from(non_fungible_id_slice)
            .map_err(|_| ParseNonFungibleAddressError::Invalid)?;
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

custom_type!(
    NonFungibleAddress,
    CustomType::NonFungibleAddress,
    Vec::new()
);

//======
// text
//======

impl FromStr for NonFungibleAddress {
    type Err = ParseNonFungibleAddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(|_| ParseNonFungibleAddressError::Invalid)?;
        Self::try_from(bytes.as_slice())
    }
}

impl fmt::Display for NonFungibleAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}{}", self.resource_address, self.non_fungible_id)
    }
}

impl fmt::Debug for NonFungibleAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}
